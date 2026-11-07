//! Fail-closed, narrow GitHub PR approval-threshold auditor.
//! This code cannot enforce an allowlist of individual GitHub reviewers.
//! Do not interpret a numeric threshold as proof of designated-reviewer enforcement.
pub mod inventory;
pub mod ci_evidence;
pub mod review_evidence;
pub mod named_owner_preflight;

use serde_json::{json, Value};
use std::fmt::Write as _;

pub const ORG: &str = "ores-truffle-oreslang";
const PER_PAGE: usize = 100;
const MAX_PAGES: usize = 100;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    pub repo: Option<String>,
    pub apply: bool,
    pub all: bool,
    /// Independent operator inventory cardinality confirmation for bulk writes.
    pub expected_repos: Option<usize>,
    /// Numeric review count alone cannot enforce either named account.
    pub acknowledge_numeric_only: bool,
}

pub fn valid_repo_name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 100
        && s != "."
        && s != ".."
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

pub fn parse_options<I: IntoIterator<Item = String>>(args: I) -> Result<Options, String> {
    let mut opts = Options { repo: None, apply: false, all: false, expected_repos: None, acknowledge_numeric_only: false };
    let mut confirmed = false;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--repo" if opts.repo.is_none() => {
                let name = args.next().ok_or("--repo requires a name")?;
                if !valid_repo_name(&name) { return Err("invalid repository name".into()); }
                opts.repo = Some(name);
            }
            "--all" if !opts.all => opts.all = true,
            "--ack-numeric-only" if !opts.acknowledge_numeric_only => opts.acknowledge_numeric_only = true,
            "--expect-repos" if opts.expected_repos.is_none() => {
                let n = args.next().ok_or("--expect-repos requires a positive integer")?;
                let count = n.parse::<usize>().map_err(|_| "invalid --expect-repos count")?;
                if count == 0 { return Err("--expect-repos cannot be zero".into()); }
                opts.expected_repos = Some(count);
            },
            "--apply" if !opts.apply => opts.apply = true,
            "--confirm-org" if !confirmed => {
                if args.next().as_deref() != Some(ORG) {
                    return Err(format!("--confirm-org must equal {ORG}"));
                }
                confirmed = true;
            }
            "-h" | "--help" => return Err(usage().into()),
            _ => return Err(format!("unknown or repeated argument: {arg}")),
        }
    }
    if opts.all && opts.repo.is_some() { return Err("--all conflicts with --repo".into()); }
    if opts.all && !opts.apply { return Err("--all only has meaning with --apply".into()); }
    if opts.apply && !(opts.all || opts.repo.is_some()) {
        return Err("write requires explicit --repo NAME or --all".into());
    }
    if opts.apply && !confirmed {
        return Err(format!("write requires --confirm-org {ORG}"));
    }
    if opts.apply && !opts.acknowledge_numeric_only {
        return Err("numeric-only changes need explicit --ack-numeric-only acknowledgement; this does not restrict reviewer identity".into());
    }
    if !opts.apply && opts.acknowledge_numeric_only {
        return Err("--ack-numeric-only only makes sense with --apply".into());
    }
    if opts.all && opts.expected_repos.is_none() {
        return Err("--all writes require --expect-repos COUNT from an independently verified inventory".into());
    }
    if !opts.all && opts.expected_repos.is_some() {
        return Err("--expect-repos is only valid with --all".into());
    }
    if !opts.apply && confirmed {
        return Err("--confirm-org is only permitted during a write".into());
    }
    Ok(opts)
}

pub fn usage() -> &'static str {
    "Usage: cargo run -- [--repo NAME] [--apply --confirm-org ores-truffle-oreslang --ack-numeric-only [--all --expect-repos COUNT|--repo NAME]]\nRead-only by default. --ack-numeric-only explicitly acknowledges that count=1 does NOT enforce either named reviewer. All --apply commands currently fail closed: GitHub-effective named-reviewer enforcement has not been verified. Audit is read-only; future writes additionally require independently verified complete inventory cardinality."
}

/// Encode every unsafe byte of one GitHub REST URL path component, including '/'.
pub fn encode_segment(s: &str) -> String {
    let mut result = String::new();
    for byte in s.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            result.push(byte as char);
        } else {
            let _ = write!(&mut result, "%{byte:02X}");
        }
    }
    result
}

pub trait Api {
    fn request(&mut self, method: &str, endpoint: &str, body: Option<Value>) -> Result<Value, String>;
    fn get(&mut self, endpoint: &str) -> Result<Value, String> {
        self.request("GET", endpoint, None)
    }
}

fn pages<A: Api>(api: &mut A, endpoint_prefix: &str) -> Result<Vec<Value>, String> {
    let mut result = Vec::new();
    for page in 1..=MAX_PAGES {
        let path = format!("{endpoint_prefix}&per_page={PER_PAGE}&page={page}");
        let value = api.get(&path)?;
        let array = value.as_array().ok_or("GitHub pagination response was not an array")?;
        result.extend(array.iter().cloned());
        if array.len() < PER_PAGE { return Ok(result); }
    }
    Err(format!("pagination limit {MAX_PAGES} exceeded: incomplete inventory"))
}

pub fn repositories<A: Api>(api: &mut A) -> Result<Vec<(String, String)>, String> {
    let raw = pages(api, &format!("orgs/{ORG}/repos?type=all"))?;
    if raw.is_empty() { return Err("no accessible repositories; refusing to claim complete audit".into()); }
    let mut result = Vec::new();
    for item in raw {
        let archived = item["archived"].as_bool()
            .ok_or("repository archive status is missing or malformed")?;
        if archived { continue; }
        let name = item["name"].as_str().ok_or("repository missing name")?;
        let expected = format!("{ORG}/{name}");
        if !valid_repo_name(name) || item["full_name"].as_str() != Some(expected.as_str()) {
            return Err(format!("repository owner/name mismatch: {name}"));
        }
        let branch = item["default_branch"].as_str().filter(|s| !s.is_empty())
            .ok_or("repository missing default branch")?;
        result.push((name.to_owned(), branch.to_owned()));
    }
    if result.is_empty() { return Err("all visible repositories archived; nothing to audit".into()); }
    result.sort();
    if result.windows(2).any(|rows| rows[0].0 == rows[1].0) {
        return Err("duplicate repository in paginated inventory; refusing ambiguous targets".into());
    }
    Ok(result)
}

pub fn repository<A: Api>(api: &mut A, repo: &str) -> Result<(String, String), String> {
    if !valid_repo_name(repo) { return Err("invalid repository name".into()); }
    let meta = api.get(&format!("repos/{ORG}/{repo}"))?;
    if meta["full_name"].as_str() != Some(format!("{ORG}/{repo}").as_str()) {
        return Err("wrong repository owner or name returned by GitHub".into());
    }
    match meta["archived"].as_bool() {
        Some(false) => {},
        Some(true) => return Err("repository is archived".into()),
        None => return Err("repository archive status is missing or malformed".into()),
    }
    let branch = meta["default_branch"].as_str().filter(|s| !s.is_empty())
        .ok_or("missing default branch")?;
    Ok((repo.to_owned(), branch.to_owned()))
}

fn ruleset_endpoint(repo: &str) -> String {
    format!("repos/{ORG}/{repo}/rulesets?includes_parents=true")
}

fn active_rule_ids<A: Api>(api: &mut A, repo: &str) -> Result<Vec<String>, String> {
    let rules = pages(api, &ruleset_endpoint(repo))?;
    let mut ids = Vec::new();
    for rule in rules {
        let enforcement = rule["enforcement"].as_str().ok_or("ruleset missing enforcement")?;
        match enforcement {
            "active" => {
                // Even a push/tag-only ruleset must be reviewed before changing
                // governance: only the full ruleset disambiguates target/conditions.
                let id = rule["id"].as_u64().ok_or("active ruleset missing id")?;
                ids.push(id.to_string());
            }
            "disabled" | "evaluate" => {}
            value => return Err(format!("unknown ruleset enforcement: {value}")),
        }
    }
    ids.sort();
    ids.dedup();
    Ok(ids)
}

fn protection_endpoint(repo: &str, branch: &str) -> String {
    format!("repos/{ORG}/{repo}/branches/{}/protection", encode_segment(branch))
}

pub fn count(protection: &Value) -> Option<u64> {
    protection.pointer("/required_pull_request_reviews/required_approving_review_count")
        .and_then(Value::as_u64)
}

fn unsafe_review_configuration(protection: &Value) -> Option<&'static str> {
    let Some(reviews) = protection.get("required_pull_request_reviews") else {
        return Some("no required PR-review configuration");
    };
    if !reviews.is_object() { return Some("invalid required PR-review configuration"); }
    match reviews.get("require_code_owner_reviews").and_then(Value::as_bool) {
        Some(false) => {}
        Some(true) => return Some("code-owner approvals must be reviewed separately"),
        None => return Some("missing or malformed code-owner review boolean"),
    }
    for field in ["dismiss_stale_reviews", "require_last_push_approval"] {
        if let Some(value) = reviews.get(field) {
            if !value.is_boolean() { return Some("malformed required PR-review boolean"); }
        }
    }
    let bypass = &reviews["bypass_pull_request_allowances"];
    if !bypass.is_null() {
        if !bypass.is_object() { return Some("malformed review bypass allowance"); }
        for field in ["users", "teams", "apps"] {
            if let Some(value) = bypass.get(field) {
                let Some(entries) = value.as_array() else {
                    return Some("malformed review bypass allowance list");
                };
                if !entries.is_empty() {
                    return Some("PR review bypass allowances require manual review");
                }
            }
        }
    }
    None
}

pub fn patch(protection: &Value) -> Option<Value> {
    match count(protection) {
        Some(n) if n <= 6 && n != 1 && unsafe_review_configuration(protection).is_none() => {
            Some(json!({"required_approving_review_count": 1}))
        }
        _ => None,
    }
}

pub fn expected_after(mut protection: Value) -> Option<Value> {
    let count_value = protection.pointer_mut("/required_pull_request_reviews/required_approving_review_count")?;
    if !count_value.is_u64() { return None; }
    *count_value = json!(1);
    Some(protection)
}

/// A complete branch-protection snapshot obtained before *any* bulk mutation.
/// An admin must audit this plan; it never authorizes bypassing another rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoPlan {
    pub repo: String,
    pub branch: String,
    pub before: Value,
    pub count: u64,
}

/// Inventory every targeted repository before writing any of them. If *any*
/// repository cannot be fully read, return all failures and prohibit bulk writes.
pub fn preflight_all<A: Api>(
    api: &mut A, repos: &[(String, String)],
) -> Result<Vec<RepoPlan>, Vec<String>> {
    if repos.is_empty() { return Err(vec!["empty repository inventory".into()]); }
    let mut plan = Vec::new();
    let mut errors = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for (repo, branch) in repos {
        if !valid_repo_name(repo) || branch.is_empty() {
            errors.push(format!("{repo}: invalid repo or default branch"));
            continue;
        }
        if !seen.insert(repo.clone()) {
            errors.push(format!("{repo}: duplicate repository in inventory"));
            continue;
        }
        match initial_decision(api, repo, branch) {
            Ok((before, count)) => plan.push(RepoPlan {
                repo: repo.clone(), branch: branch.clone(), before, count,
            }),
            Err(err) => errors.push(format!("{repo}:{branch}: {err}")),
        }
    }
    if errors.is_empty() { Ok(plan) } else { Err(errors) }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Compliant,
    WouldChange(u64),
    Changed(u64),
    Manual(String),
    Error(String),
}

fn initial_decision<A: Api>(
    api: &mut A,
    repo: &str,
    branch: &str,
) -> Result<(Value, u64), String> {
    let ids = active_rule_ids(api, repo)?;
    if !ids.is_empty() {
        return Err(format!("active local/inherited rulesets {ids:?}; inspect effective rules first"));
    }
    let current = api.get(&protection_endpoint(repo, branch))?;
    if let Some(message) = unsafe_review_configuration(&current) {
        return Err(message.into());
    }
    let n = count(&current).ok_or("no numeric PR-review protection; do not create a new policy implicitly")?;
    if n > 6 {
        return Err(format!("unsupported approval count {n}; requires manual inspection"));
    }
    Ok((current, n))
}

fn audit_repo_impl<A: Api>(
    api: &mut A, repo: &str, branch: &str, apply: bool, expected: Option<&Value>,
) -> Outcome {
    if !valid_repo_name(repo) || branch.is_empty() {
        return Outcome::Manual("unsafe repo/branch name".into());
    }
    let (before, n) = match initial_decision(api, repo, branch) {
        Ok(v) => v,
        Err(e) => return Outcome::Manual(e),
    };
    if expected.is_some_and(|original| original != &before) {
        return Outcome::Manual("protection changed since bulk preflight; refusing stale plan".into());
    }
    if n == 1 { return Outcome::Compliant; }
    if !apply { return Outcome::WouldChange(n); }

    // Guard against policy races: check both inherited rulesets and complete
    // branch protection again immediately before the sole write.
    let (fresh, fresh_count) = match initial_decision(api, repo, branch) {
        Ok(v) => v,
        Err(e) => return Outcome::Manual(format!("pre-write recheck: {e}")),
    };
    if fresh != before || fresh_count != n {
        return Outcome::Manual("branch protection changed between audit and write; refusing stale write".into());
    }
    let Some(body) = patch(&fresh) else {
        return Outcome::Manual("review protection not safely patchable".into());
    };
    let endpoint = format!("{}/required_pull_request_reviews", protection_endpoint(repo, branch));
    if let Err(e) = api.request("PATCH", &endpoint, Some(body)) {
        return Outcome::Error(format!("PATCH rejected: {e}"));
    }
    let after = match api.get(&protection_endpoint(repo, branch)) {
        Ok(v) => v,
        Err(e) => return Outcome::Error(format!("post-write readback failed: {e}")),
    };
    // Fail closed on any unintended status-check, CODEOWNERS, bypass, or other
    // field alteration. Do not silently claim compliance after a partial patch.
    if Some(after) != expected_after(before) {
        return Outcome::Error("readback mismatch: unrelated protection fields changed or approval count != 1".into());
    }
    // A ruleset can be added while the write executes; a complete success
    // receipt must verify the effective ruleset surface *after* the write too.
    match active_rule_ids(api, repo) {
        Ok(ids) if ids.is_empty() => Outcome::Changed(n),
        Ok(ids) => Outcome::Error(format!("rulesets appeared during write: {ids:?}; effective policy unverified")),
        Err(e) => Outcome::Error(format!("post-write ruleset readback failed: {e}")),
    }
}

/// Read-only audit, or explicit one-repository apply (with fresh rechecks).
pub fn audit_repo<A: Api>(
    api: &mut A, repo: &str, branch: &str, apply: bool,
) -> Outcome {
    audit_repo_impl(api, repo, branch, apply, None)
}

/// Apply only a repository whose *complete* protection snapshot still matches
/// the all-repository preflight; also re-check it immediately before PATCH.
pub fn apply_planned_repo<A: Api>(api: &mut A, plan: &RepoPlan) -> Outcome {
    audit_repo_impl(api, &plan.repo, &plan.branch, true, Some(&plan.before))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    fn opts(args: &[&str]) -> Result<Options, String> {
        parse_options(args.iter().map(|s| (*s).to_owned()))
    }

    fn protection(n: u64) -> Value {
        json!({
            "required_status_checks": {"strict": true, "contexts": ["security", "ci"]},
            "enforce_admins": {"enabled": true},
            "required_pull_request_reviews": {
                "required_approving_review_count": n,
                "dismiss_stale_reviews": true,
                "require_last_push_approval": true,
                "require_code_owner_reviews": false,
                "bypass_pull_request_allowances": {"users": [], "teams": [], "apps": []},
                "dismissal_restrictions": {"users": [{"login":"security"}], "teams": []}
            }
        })
    }

    struct MockApi {
        queue: VecDeque<(&'static str, String, Result<Value, String>)>,
        writes: Vec<Value>,
    }
    impl MockApi {
        fn new() -> Self { Self { queue: VecDeque::new(), writes: Vec::new() } }
        fn step(mut self, method: &'static str, path: &str, answer: Value) -> Self {
            self.queue.push_back((method, path.into(), Ok(answer)));
            self
        }
        fn error(mut self, method: &'static str, path: &str) -> Self {
            self.queue.push_back((method, path.into(), Err("simulated unavailable".into())));
            self
        }
        fn finished(&self) { assert!(self.queue.is_empty(), "unexpected unconsumed mock API steps"); }
    }
    impl Api for MockApi {
        fn request(&mut self, method: &str, endpoint: &str, body: Option<Value>) -> Result<Value, String> {
            let (expected_method, expected_path, response) =
                self.queue.pop_front().expect("unexpected API request");
            assert_eq!(method, expected_method);
            assert_eq!(endpoint, expected_path);
            if method == "PATCH" {
                self.writes.push(body.expect("PATCH missing body"));
            } else {
                assert!(body.is_none(), "GET must be read-only");
            }
            response
        }
    }

    fn rules_path() -> String {
        format!("repos/{ORG}/example/rulesets?includes_parents=true&per_page=100&page=1")
    }
    fn protect_path() -> String {
        protection_endpoint("example", "main")
    }
    fn base_mock(n: u64) -> MockApi {
        MockApi::new().step("GET", &rules_path(), json!([]))
            .step("GET", &protect_path(), protection(n))
    }

    #[test]
    fn audit_is_default_and_accepts_a_single_repo() {
        assert_eq!(opts(&[]).unwrap(), Options { repo: None, apply: false, all: false, expected_repos: None, acknowledge_numeric_only: false });
        assert_eq!(opts(&["--repo","java"]).unwrap().repo.as_deref(), Some("java"));
    }

    #[test]
    fn writes_require_both_scope_and_org_confirmation() {
        assert!(opts(&["--apply"]).is_err());
        assert!(opts(&["--apply","--all"]).is_err());
        assert!(opts(&["--apply","--repo","a"]).is_err());
        assert!(opts(&["--confirm-org",ORG]).is_err());
        assert!(opts(&["--apply","--all","--confirm-org","other"]).is_err());
        assert!(opts(&["--apply","--all","--expect-repos","80","--confirm-org",ORG,"--ack-numeric-only"]).is_ok());
        assert!(opts(&["--apply","--repo","a","--confirm-org",ORG,"--ack-numeric-only"]).is_ok());
    }

    #[test]
    fn numeric_approval_write_requires_explicit_identity_warning() {
        assert!(opts(&["--apply","--repo","java","--confirm-org",ORG]).is_err());
        assert!(opts(&["--ack-numeric-only"]).is_err());
        assert!(opts(&["--apply","--repo","java","--confirm-org",ORG,"--ack-numeric-only"]).is_ok());
    }

    #[test]
    fn bulk_write_requires_independently_expected_inventory_count() {
        assert!(opts(&["--all","--apply","--confirm-org",ORG,"--ack-numeric-only"]).is_err());
        assert!(opts(&["--all","--apply","--confirm-org",ORG,"--ack-numeric-only","--expect-repos","0"]).is_err());
        assert!(opts(&["--all","--apply","--confirm-org",ORG,"--ack-numeric-only","--expect-repos","-1"]).is_err());
        assert!(opts(&["--all","--apply","--confirm-org",ORG,"--ack-numeric-only","--expect-repos","80"]).is_ok());
    }

    #[test]
    fn count_confirmation_cannot_be_used_for_single_repository() {
        assert!(opts(&["--repo","java","--expect-repos","80"]).is_err());
        assert!(opts(&["--expect-repos","80"]).is_err());
    }

    #[test]
    fn write_confirmation_rejects_duplicates() {
        let args = ["--apply","--repo","a","--confirm-org",ORG,"--ack-numeric-only","--ack-numeric-only"];
        assert!(opts(&args).is_err());
        let args = ["--all","--apply","--confirm-org",ORG,"--ack-numeric-only",
            "--expect-repos","80","--expect-repos","80"];
        assert!(opts(&args).is_err());
    }

    #[test]
    fn rejects_conflicting_repeated_and_injected_args() {
        assert!(opts(&["--all","--apply","--repo","a","--confirm-org",ORG]).is_err());
        assert!(opts(&["--repo","a","--repo","b"]).is_err());
        assert!(opts(&["--apply","--apply","--repo","a","--confirm-org",ORG]).is_err());
        for candidate in ["", ".", "..", "../x", "x/y", "x?y=2", "n\nx", "🚀"] {
            assert!(!valid_repo_name(candidate));
        }
        assert!(valid_repo_name("oreslang-source.java"));
    }

    #[test]
    fn encodes_branch_and_all_path_metacharacters() {
        assert_eq!(encode_segment("release/2026 11#hot?"), "release%2F2026%2011%23hot%3F");
        assert_eq!(encode_segment("main"), "main");
        assert_eq!(encode_segment("x%y"), "x%25y");
        assert_eq!(encode_segment("föö"), "f%C3%B6%C3%B6");
    }

    #[test]
    fn never_replaces_unrelated_review_or_ci_fields() {
        let before = protection(3);
        assert_eq!(patch(&before), Some(json!({"required_approving_review_count":1})));
        let after = expected_after(before.clone()).unwrap();
        assert_eq!(after["required_status_checks"], before["required_status_checks"]);
        assert_eq!(after["required_pull_request_reviews"]["dismiss_stale_reviews"], json!(true));
        assert_eq!(after["required_pull_request_reviews"]["require_last_push_approval"], json!(true));
        assert_eq!(after["required_pull_request_reviews"]["dismissal_restrictions"],
            before["required_pull_request_reviews"]["dismissal_restrictions"]);
    }

    #[test]
    fn missing_or_invalid_count_is_not_patchable() {
        for value in [json!({}),json!({"required_pull_request_reviews":null}),
            json!({"required_pull_request_reviews":{"required_approving_review_count":"2"}}),
            json!({"required_pull_request_reviews":{"required_approving_review_count":99}}),
        ] {
            assert!(patch(&value).is_none());
        }
    }

    #[test]
    fn zero_approvals_is_tightened_to_one_not_disabled() {
        assert_eq!(patch(&protection(0)), Some(json!({"required_approving_review_count":1})));
        assert!(patch(&protection(1)).is_none());
    }

    #[test]
    fn refuse_codeowner_and_bypass_configuration() {
        let mut p = protection(2);
        p["required_pull_request_reviews"]["require_code_owner_reviews"] = json!(true);
        assert!(patch(&p).is_none());
        p["required_pull_request_reviews"]["require_code_owner_reviews"] = json!(false);
        p["required_pull_request_reviews"]["bypass_pull_request_allowances"]["users"] = json!(["exempt"]);
        assert!(patch(&p).is_none());
    }

    #[test]
    fn dry_run_does_not_write() {
        let mut api = base_mock(3);
        assert_eq!(audit_repo(&mut api, "example", "main", false), Outcome::WouldChange(3));
        assert!(api.writes.is_empty());
        api.finished();
    }

    #[test]
    fn compliant_policy_never_writes() {
        let mut api = base_mock(1);
        assert_eq!(audit_repo(&mut api, "example", "main", true), Outcome::Compliant);
        api.finished();
    }

    #[test]
    fn exact_review_patch_and_successful_complete_readback() {
        let initial = protection(3);
        let mut api = base_mock(3)
            .step("GET", &rules_path(), json!([]))
            .step("GET", &protect_path(), initial.clone())
            .step("PATCH", &format!("{}/required_pull_request_reviews",protect_path()), json!({}))
            .step("GET", &protect_path(), expected_after(initial).unwrap())
            .step("GET", &rules_path(), json!([]));
        assert_eq!(audit_repo(&mut api,"example","main",true),Outcome::Changed(3));
        assert_eq!(api.writes, vec![json!({"required_approving_review_count":1})]);
        api.finished();
    }

    #[test]
    fn rejects_review_threshold_race_without_write() {
        let mut api = base_mock(3)
            .step("GET", &rules_path(), json!([]))
            .step("GET", &protect_path(), protection(4));
        assert!(matches!(audit_repo(&mut api,"example","main",true), Outcome::Manual(_)));
        assert!(api.writes.is_empty());
        api.finished();
    }

    #[test]
    fn rejects_new_ruleset_added_between_reads() {
        let mut api = base_mock(3)
            .step("GET", &rules_path(), json!([{"id":7,"enforcement":"active"}]));
        assert!(matches!(audit_repo(&mut api,"example","main",true), Outcome::Manual(_)));
        assert!(api.writes.is_empty());
        api.finished();
    }

    #[test]
    fn rejects_active_rulesets_even_when_malformed() {
        let mut api = MockApi::new().step("GET", &rules_path(), json!([{"enforcement":"active"}]));
        assert!(matches!(audit_repo(&mut api,"example","main",false),Outcome::Manual(_)));
        api.finished();
    }

    #[test]
    fn rejects_unknown_ruleset_enforcement() {
        let mut api = MockApi::new().step("GET", &rules_path(), json!([{"id":8,"enforcement":"maybe"}]));
        assert!(matches!(audit_repo(&mut api,"example","main",false),Outcome::Manual(_)));
        api.finished();
    }

    #[test]
    fn reports_unavailable_ruleset_as_manual() {
        let mut api = MockApi::new().error("GET", &rules_path());
        assert!(matches!(audit_repo(&mut api,"example","main",false), Outcome::Manual(_)));
        api.finished();
    }

    #[test]
    fn refuses_post_write_corruption_even_when_count_is_one() {
        let mut altered = expected_after(protection(3)).unwrap();
        altered["required_status_checks"]["strict"] = json!(false);
        let mut api = base_mock(3)
            .step("GET", &rules_path(), json!([]))
            .step("GET", &protect_path(), protection(3))
            .step("PATCH", &format!("{}/required_pull_request_reviews",protect_path()), json!({}))
            .step("GET", &protect_path(), altered);
        assert!(matches!(audit_repo(&mut api,"example","main",true),Outcome::Error(_)));
        assert_eq!(api.writes.len(), 1);
        api.finished();
    }

    #[test]
    fn rejected_patch_is_not_reported_successful() {
        let mut api = base_mock(3)
            .step("GET", &rules_path(), json!([]))
            .step("GET", &protect_path(), protection(3))
            .error("PATCH", &format!("{}/required_pull_request_reviews",protect_path()));
        assert!(matches!(audit_repo(&mut api,"example","main",true),Outcome::Error(_)));
        api.finished();
    }

    #[test]
    fn bad_postwrite_readback_is_error() {
        let mut api = base_mock(3)
            .step("GET", &rules_path(), json!([]))
            .step("GET", &protect_path(), protection(3))
            .step("PATCH", &format!("{}/required_pull_request_reviews",protect_path()), json!({}))
            .error("GET", &protect_path());
        assert!(matches!(audit_repo(&mut api,"example","main",true),Outcome::Error(_)));
        api.finished();
    }

    #[test]
    fn refuses_wrong_owner_from_repo_metadata() {
        let mut api = MockApi::new().step("GET", &format!("repos/{ORG}/example"),
            json!({"full_name":"other/example","default_branch":"main"}));
        assert!(repository(&mut api,"example").is_err());
        api.finished();
    }

    #[test]
    fn inventory_requires_nonempty_page_and_matching_owner() {
        let mut empty = MockApi::new()
            .step("GET",&format!("orgs/{ORG}/repos?type=all&per_page=100&page=1"),json!([]));
        assert!(repositories(&mut empty).is_err());
        let mut mismatch = MockApi::new()
            .step("GET",&format!("orgs/{ORG}/repos?type=all&per_page=100&page=1"),
                  json!([{"name":"repo","full_name":"other/repo","default_branch":"main"}]));
        assert!(repositories(&mut mismatch).is_err());
    }

    #[test]
    fn archived_repos_skipped_and_sort_is_stable() {
        let mut api = MockApi::new()
            .step("GET",&format!("orgs/{ORG}/repos?type=all&per_page=100&page=1"),
                json!([
                    {"name":"z","full_name":format!("{ORG}/z"),"default_branch":"main","archived":false},
                    {"name":"archived","full_name":format!("{ORG}/archived"),"default_branch":"main","archived":true},
                    {"name":"a","full_name":format!("{ORG}/a"),"default_branch":"dev","archived":false}
                ]));
        assert_eq!(repositories(&mut api).unwrap(),vec![
            ("a".into(),"dev".into()),("z".into(),"main".into())]);
        api.finished();
    }


    #[test]
    fn active_rulesets_on_second_page_are_not_ignored() {
        let first_page: Vec<Value> = (0..100).map(|id| {
            json!({"id":id,"enforcement":"disabled"})
        }).collect();
        let second_path = format!("repos/{ORG}/example/rulesets?includes_parents=true&per_page=100&page=2");
        let mut api = MockApi::new()
            .step("GET", &rules_path(), json!(first_page))
            .step("GET", &second_path, json!([{"id":900,"enforcement":"active"}]));
        assert!(matches!(audit_repo(&mut api,"example","main",false),Outcome::Manual(_)));
        assert!(api.writes.is_empty());
        api.finished();
    }

    #[test]
    fn new_ruleset_during_patch_invalidates_success_receipt() {
        let mut api = base_mock(3)
            .step("GET", &rules_path(), json!([]))
            .step("GET", &protect_path(), protection(3))
            .step("PATCH", &format!("{}/required_pull_request_reviews",protect_path()), json!({}))
            .step("GET", &protect_path(), expected_after(protection(3)).unwrap())
            .step("GET", &rules_path(), json!([{"id":44,"enforcement":"active"}]));
        assert!(matches!(audit_repo(&mut api,"example","main",true),Outcome::Error(_)));
        api.finished();
    }

    #[test]
    fn unavailable_postwrite_rulesets_invalidates_success_receipt() {
        let mut api = base_mock(3)
            .step("GET", &rules_path(), json!([]))
            .step("GET", &protect_path(), protection(3))
            .step("PATCH", &format!("{}/required_pull_request_reviews",protect_path()), json!({}))
            .step("GET", &protect_path(), expected_after(protection(3)).unwrap())
            .error("GET", &rules_path());
        assert!(matches!(audit_repo(&mut api,"example","main",true),Outcome::Error(_)));
        api.finished();
    }

    #[test]
    fn code_owner_requirement_does_not_become_an_automatic_write() {
        let mut protected = protection(3);
        protected["required_pull_request_reviews"]["require_code_owner_reviews"] = json!(true);
        let mut api = MockApi::new()
            .step("GET", &rules_path(), json!([]))
            .step("GET", &protect_path(), protected);
        assert!(matches!(audit_repo(&mut api,"example","main",true),Outcome::Manual(_)));
        assert!(api.writes.is_empty());
        api.finished();
    }

    #[test]
    fn bypass_allowance_is_not_silently_treated_as_review_satisfaction() {
        let mut protected = protection(3);
        protected["required_pull_request_reviews"]["bypass_pull_request_allowances"]["teams"] = json!(["bypass"]);
        let mut api = MockApi::new()
            .step("GET", &rules_path(), json!([]))
            .step("GET", &protect_path(), protected);
        assert!(matches!(audit_repo(&mut api,"example","main",true),Outcome::Manual(_)));
        assert!(api.writes.is_empty());
        api.finished();
    }

    #[test]
    fn review_protection_missing_is_never_created_by_this_tool() {
        let mut protected = protection(3);
        protected["required_pull_request_reviews"] = Value::Null;
        let mut api = MockApi::new()
            .step("GET", &rules_path(), json!([]))
            .step("GET", &protect_path(), protected);
        assert!(matches!(audit_repo(&mut api,"example","main",true),Outcome::Manual(_)));
        assert!(api.writes.is_empty());
        api.finished();
    }

    #[test]
    fn changed_status_checks_between_audit_and_write_block_the_patch() {
        let mut changed = protection(3);
        changed["required_status_checks"]["contexts"] = json!(["ci"]);
        let mut api = base_mock(3)
            .step("GET", &rules_path(), json!([]))
            .step("GET", &protect_path(), changed);
        assert!(matches!(audit_repo(&mut api,"example","main",true),Outcome::Manual(_)));
        assert!(api.writes.is_empty());
        api.finished();
    }

    #[test]
    fn failed_branch_protection_read_is_not_a_success() {
        let mut api = MockApi::new()
            .step("GET", &rules_path(), json!([]))
            .error("GET", &protect_path());
        assert!(matches!(audit_repo(&mut api,"example","main",false),Outcome::Manual(_)));
        api.finished();
    }


    #[test]
    fn bulk_preflight_rejects_empty_inventory_without_api_calls() {
        let mut api = MockApi::new();
        assert!(preflight_all(&mut api, &[]).is_err());
        assert!(api.writes.is_empty());
        api.finished();
    }

    #[test]
    fn bulk_preflight_collects_compliant_and_would_change_without_writes() {
        let list = vec![("example".into(), "main".into()), ("example-alt".into(), "main".into())];
        // Duplicate protection path fixture is sufficient for unit testing
        // the in-order two target preflight behavior; repository names differ.
        // MockApi checks full paths, so use distinct endpoints.
        let mut api = MockApi::new()
            .step("GET", &rules_path(), json!([]))
            .step("GET", &protect_path(), protection(1))
            .step("GET", &format!("repos/{ORG}/example-alt/rulesets?includes_parents=true&per_page=100&page=1"), json!([]))
            .step("GET", &protection_endpoint("example-alt","main"), protection(3));
        let rows = preflight_all(&mut api, &list).unwrap();
        assert_eq!(rows[0].count, 1);
        assert_eq!(rows[1].count, 3);
        assert_eq!(rows[1].repo, "example-alt");
        assert_eq!(rows[1].before, protection(3));
        assert!(api.writes.is_empty());
        api.finished();
    }

    #[test]
    fn bulk_preflight_stops_rollout_when_later_target_is_manual() {
        let mut api = base_mock(3)
            .step("GET", &format!("repos/{ORG}/blocked/rulesets?includes_parents=true&per_page=100&page=1"),
                json!([{"id":9,"enforcement":"active"}]));
        let list = vec![("example".into(),"main".into()),("blocked".into(),"main".into())];
        let rows = preflight_all(&mut api,&list).unwrap_err();
        assert!(rows.iter().any(|reason| reason.contains("blocked") && reason.contains("rulesets")));
        assert!(api.writes.is_empty());
        api.finished();
    }

    #[test]
    fn duplicate_inventory_fails_preflight_without_replaying_a_repo() {
        let mut api = base_mock(1);
        let list = vec![("example".into(),"main".into()),("example".into(),"dev".into())];
        let rows = preflight_all(&mut api,&list).unwrap_err();
        assert!(rows.iter().any(|reason| reason.contains("duplicate")));
        assert!(api.writes.is_empty());
        api.finished();
    }

    #[test]
    fn repo_inventory_rejects_missing_archive_flag() {
        let mut api = MockApi::new()
            .step("GET",&format!("orgs/{ORG}/repos?type=all&per_page=100&page=1"),
                json!([{"name":"one","full_name":format!("{ORG}/one"),"default_branch":"main"}]));
        assert!(repositories(&mut api).is_err());
        api.finished();
    }

    #[test]
    fn repo_inventory_rejects_duplicate_entry_in_same_page() {
        let item = json!({"name":"one","full_name":format!("{ORG}/one"),
            "default_branch":"main","archived":false});
        let mut api = MockApi::new()
            .step("GET",&format!("orgs/{ORG}/repos?type=all&per_page=100&page=1"),
                json!([item.clone(),item]));
        assert!(repositories(&mut api).is_err());
        api.finished();
    }
    #[test]
    fn rejects_wrong_fullname_even_for_single_repo() {
        let mut api = MockApi::new().step("GET",&format!("repos/{ORG}/example"),
            json!({"full_name":"another/example","default_branch":"main"}));
        assert!(repository(&mut api,"example").is_err());
    }
}
