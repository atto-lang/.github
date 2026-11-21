//! Strict, dependency-free policy core for an opt-in, repository-local
//! AttoLang Draft hold controller. No untrusted PR code is ever executed.
//!
//! This is NOT the four-party approval gate and must NEVER be presented as
//! complete authorization to merge.
use std::fmt;

pub const STATUS_CONTEXT: &str = "atto-governance/draft-hold";

// The GitHub CLI executes this jq expression against an authenticated live
// pull-request API response. Emit ONLY fixed-schema scalar values; label names
// never become shell arguments or delimiters in our parser.
pub const PR_JQ: &str = r#"[
  .base.repo.full_name,
  .number,
  .state,
  .draft,
  .merged,
  .head.sha,
  (.labels | type == "array"),
  (([.labels[]? | .name | ascii_downcase] | index("keep-as-draft")) != null),
  (([.labels[]? | .name | ascii_downcase] | index("keep-draft")) != null),
  (([.labels[]? | .name | ascii_downcase] | index("not-ready")) != null)
] | @tsv"#;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub repository: String,
    pub number: u64,
    pub state: String,
    pub draft: bool,
    pub merged: bool,
    pub head_sha: String,
    pub labels_verified: bool,
    pub keep_as_draft: bool,
    pub keep_draft: bool,
    pub not_ready: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// No passing status may be issued.
    Blocked,
    /// Publish failure BEFORE attempting the Draft conversion.
    RestoreDraft,
    /// Only the narrowly scoped draft-hold check can pass.
    ClearOfDraftHolds,
}

impl fmt::Display for Decision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Blocked => write!(f, "BLOCKED"),
            Self::RestoreDraft => write!(f, "RESTORE_DRAFT"),
            Self::ClearOfDraftHolds => write!(f, "DRAFT_HOLD_CLEAR_ONLY"),
        }
    }
}

pub fn validate_repo(repo: &str) -> Result<(), &'static str> {
    let Some(name) = repo.strip_prefix("atto-lang/") else {
        return Err("not an atto-lang repository");
    };
    if name.is_empty() || name.len() > 100 || name == "." || name == ".." ||
        !name.bytes().all(|c| c.is_ascii_alphanumeric() ||
            matches!(c, b'.' | b'-' | b'_'))
    {
        return Err("invalid repository name");
    }
    Ok(())
}

fn bool_value(s: &str) -> Result<bool, &'static str> {
    match s {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err("non-boolean response field"),
    }
}

pub fn parse_snapshot(output: &str, expected_repo: &str, expected_pr: u64)
    -> Result<Snapshot, &'static str>
{
    validate_repo(expected_repo)?;
    if expected_pr == 0 {
        return Err("invalid PR number");
    }
    let line = output.strip_suffix('\n').unwrap_or(output);
    if line.contains('\n') || line.contains('\r') {
        return Err("multiline or malformed API response");
    }
    let parts: Vec<&str> = line.split('\t').collect();
    if parts.len() != 10 {
        return Err("incomplete API response");
    }
    if parts[0] != expected_repo {
        return Err("PR repository identity mismatch");
    }
    let number = parts[1].parse::<u64>().map_err(|_| "invalid PR number")?;
    if number != expected_pr {
        return Err("PR number mismatch");
    }
    if !matches!(parts[2], "open" | "closed") {
        return Err("invalid PR state");
    }
    let head_sha = parts[5];
    if head_sha.len() != 40 ||
        !head_sha.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("missing or invalid immutable HEAD SHA");
    }
    let result = Snapshot {
        repository: parts[0].to_owned(),
        number,
        state: parts[2].to_owned(),
        draft: bool_value(parts[3])?,
        merged: bool_value(parts[4])?,
        head_sha: head_sha.to_owned(),
        labels_verified: bool_value(parts[6])?,
        keep_as_draft: bool_value(parts[7])?,
        keep_draft: bool_value(parts[8])?,
        not_ready: bool_value(parts[9])?,
    };
    if !result.labels_verified {
        return Err("live labels missing or malformed");
    }
    Ok(result)
}

impl Snapshot {
    pub fn persistent_hold(&self) -> bool {
        self.keep_as_draft || self.keep_draft
    }

    pub fn any_hold(&self) -> bool {
        self.persistent_hold() || self.not_ready
    }

    pub fn decision(&self) -> Decision {
        if !self.labels_verified || self.state != "open" || self.merged {
            return Decision::Blocked;
        }
        if self.persistent_hold() && !self.draft {
            return Decision::RestoreDraft;
        }
        if self.any_hold() || self.draft {
            return Decision::Blocked;
        }
        Decision::ClearOfDraftHolds
    }

    // A non-hold PR can never be auto-promoted by this controller. This
    // function only checks whether it may report the draft-hold check as clear.
    pub fn may_post_success(&self) -> bool {
        self.decision() == Decision::ClearOfDraftHolds
    }

    // Compare the ENTIRE relevant live snapshot before any write. If labels
    // were changed between reads (even with same head), re-evaluate.
    pub fn still_same(&self, newer: &Self) -> bool {
        self == newer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::process::{Command, Stdio};

    const SHA: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn snapshot() -> Snapshot {
        Snapshot {
            repository: "atto-lang/.github".to_owned(), number: 49,
            state: "open".to_owned(), draft: false, merged: false,
            head_sha: SHA.to_owned(), labels_verified: true,
            keep_as_draft: false, keep_draft: false, not_ready: false,
        }
    }

    fn tsv(s: &Snapshot) -> String {
        format!("{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            s.repository, s.number, s.state, s.draft, s.merged, s.head_sha,
            s.labels_verified, s.keep_as_draft, s.keep_draft, s.not_ready)
    }

    #[test]
    fn clean_ready_pr_clears_only_draft_hold_check() {
        let s = snapshot();
        assert_eq!(s.decision(), Decision::ClearOfDraftHolds);
        assert!(s.may_post_success());
    }

    #[test]
    fn held_ready_pr_requires_restoration() {
        let mut s = snapshot();
        s.keep_as_draft = true;
        assert_eq!(s.decision(), Decision::RestoreDraft);
        assert!(!s.may_post_success());
    }

    #[test]
    fn held_draft_remains_draft() {
        let mut s = snapshot();
        s.keep_as_draft = true;
        s.draft = true;
        assert_eq!(s.decision(), Decision::Blocked);
    }

    #[test]
    fn legacy_hold_is_equivalent() {
        let mut s = snapshot();
        s.keep_draft = true;
        assert_eq!(s.decision(), Decision::RestoreDraft);
        s.draft = true;
        assert_eq!(s.decision(), Decision::Blocked);
    }

    #[test]
    fn not_ready_blocks_but_does_not_force_draft() {
        let mut s = snapshot();
        s.not_ready = true;
        assert_eq!(s.decision(), Decision::Blocked);
    }

    #[test]
    fn unlabelled_draft_never_passes() {
        let mut s = snapshot();
        s.draft = true;
        assert_eq!(s.decision(), Decision::Blocked);
    }

    #[test]
    fn a_closed_or_merged_pr_never_passes_or_restores() {
        let mut s = snapshot();
        s.keep_as_draft = true;
        s.state = "closed".to_owned();
        assert_eq!(s.decision(), Decision::Blocked);
        s.state = "open".to_owned();
        s.merged = true;
        assert_eq!(s.decision(), Decision::Blocked);
    }

    #[test]
    fn malformed_labels_block_eligibility() {
        let mut s = snapshot();
        s.labels_verified = false;
        assert_eq!(s.decision(), Decision::Blocked);
    }

    #[test]
    fn rejects_other_organizations_and_invalid_repo_strings() {
        for value in ["other/.github", "atto-lang/", "atto-lang/../foo",
            "atto-lang/test repo", "atto-lang/..", "atto-lang/test\nevil"]
        {
            assert!(validate_repo(value).is_err(), "{value}");
        }
        assert!(validate_repo("atto-lang/.github").is_ok());
        assert!(validate_repo("atto-lang/atto-source.llvm").is_ok());
    }

    #[test]
    fn strict_snapshot_parser_accepts_only_exact_identity() {
        let s = snapshot();
        assert_eq!(parse_snapshot(&tsv(&s), &s.repository, s.number), Ok(s));
    }

    #[test]
    fn strict_snapshot_parser_rejects_missing_extra_or_multiline_fields() {
        let s = snapshot();
        for value in ["", "42", "x\ty", "x\nx", "x\rx",
            &format!("{}\textra", tsv(&s).trim_end())]
        {
            assert!(parse_snapshot(value, "atto-lang/.github", 49).is_err());
        }
    }

    #[test]
    fn strict_snapshot_parser_rejects_replayed_foreign_repo_or_pr() {
        let s = snapshot();
        assert!(parse_snapshot(&tsv(&s), "atto-lang/other", 49).is_err());
        assert!(parse_snapshot(&tsv(&s), "atto-lang/.github", 50).is_err());
        assert!(parse_snapshot(&tsv(&s), "atto-lang/.github", 0).is_err());
    }

    #[test]
    fn malformed_sha_and_state_are_invalid() {
        let mut s = snapshot();
        s.head_sha = "bad".to_string();
        assert!(parse_snapshot(&tsv(&s), &s.repository, s.number).is_err());
        s.head_sha = SHA.to_uppercase();
        assert!(parse_snapshot(&tsv(&s), &s.repository, s.number).is_err());
        s.head_sha = SHA.to_string();
        s.state = "unknown".to_owned();
        assert!(parse_snapshot(&tsv(&s), &s.repository, s.number).is_err());
    }

    #[test]
    fn rejects_invalid_boolean_and_missing_labels() {
        let mut s = snapshot();
        s.labels_verified = false;
        assert!(parse_snapshot(&tsv(&s), &s.repository, s.number).is_err());
        let v = tsv(&snapshot()).replace("\tfalse\ttrue\t", "\tmaybe\ttrue\t");
        assert!(parse_snapshot(&v, "atto-lang/.github", 49).is_err());
    }

    #[test]
    fn hold_add_remove_and_head_change_invalidate_old_decisions() {
        let old = snapshot();
        let mut updated = old.clone();
        updated.keep_as_draft = true;
        assert!(!old.still_same(&updated));
        assert_eq!(updated.decision(), Decision::RestoreDraft);
        updated.keep_as_draft = false;
        updated.head_sha = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_owned();
        assert!(!old.still_same(&updated));
    }

    #[test]
    fn labels_are_casefolded_by_the_jq_live_api_projection() {
        // This also verifies the actual jq query against a hostile arbitrary
        // label name. The test runner's jq is a documented runtime dependency
        // of GitHub CLI --jq; no Python or network access is involved.
        let fixture = format!(
            r#"{{"base":{{"repo":{{"full_name":"atto-lang/.github"}}}},"number":49,"state":"open","draft":false,"merged":false,"head":{{"sha":"{}"}},"labels":[{{"name":"KEEP-AS-DRAFT"}},{{"name":"other\tlabel"}}]}}"#,
            SHA
        );
        let mut child = Command::new("jq")
            .args(["-r", PR_JQ])
            .stdin(Stdio::piped()).stdout(Stdio::piped())
            .spawn().expect("jq must be installed in audited CI environment");
        child.stdin.take().unwrap().write_all(fixture.as_bytes()).unwrap();
        let out = child.wait_with_output().unwrap();
        assert!(out.status.success(), "jq predicate should be valid");
        let parsed = parse_snapshot(&String::from_utf8(out.stdout).unwrap(),
            "atto-lang/.github", 49).unwrap();
        assert!(parsed.keep_as_draft);
        assert_eq!(parsed.decision(), Decision::RestoreDraft);
    }

    #[test]
    fn missing_labels_field_fails_closed_even_when_jq_predicates_are_false() {
        let fixture = format!(
            r#"{{"base":{{"repo":{{"full_name":"atto-lang/.github"}}}},"number":49,"state":"open","draft":false,"merged":false,"head":{{"sha":"{}"}}}}"#,
            SHA
        );
        let mut child = Command::new("jq")
            .args(["-r", PR_JQ])
            .stdin(Stdio::piped()).stdout(Stdio::piped())
            .spawn().unwrap();
        child.stdin.take().unwrap().write_all(fixture.as_bytes()).unwrap();
        let out = child.wait_with_output().unwrap();
        assert!(out.status.success());
        assert!(parse_snapshot(&String::from_utf8(out.stdout).unwrap(),
            "atto-lang/.github", 49).is_err());
    }
}
