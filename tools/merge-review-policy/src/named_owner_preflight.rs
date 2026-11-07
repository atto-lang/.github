//! Read-only, conservative candidate checks for the .github review-policy pilot.
//!
//! These snapshots MUST come from an authenticated administrator at the pinned
//! base branch head. A successful candidate is NOT proof of live GitHub merge
//! eligibility, required check identity, or enforcement across other repos.
//! This module NEVER authorizes PATCH, and production --apply stays disabled.
use serde_json::Value;

pub const PILOT_REPO: &str = ".github";
pub const PILOT_BRANCH: &str = "main";
pub const EXPECTED_OWNERS: &str = "* @oresoftware @the1mills";

/// All values must be read from the same independently pinned base-branch
/// snapshot. Any missing, malformed, stale or inaccessible input is a failure.
pub struct PilotEvidence<'a> {
    pub repo: &'a str,
    pub branch: &'a str,
    pub base_sha: &'a str,
    pub codeowners_ref_sha: &'a str,
    pub codeowners: &'a str,
    pub codeowners_errors: &'a Value,
    pub protection: &'a Value,
    pub rulesets: &'a Value,
    pub oresoftware_access: &'a Value,
    pub the1mills_access: &'a Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadOnlyCandidate {
    pub base_sha: String,
}

fn sha40(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|c| c.is_ascii_hexdigit())
}

fn one_owner_rule(text: &str) -> bool {
    if text.is_empty() || text.contains('\r') { return false; }
    let mut rules = text.lines().map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'));
    rules.next() == Some(EXPECTED_OWNERS) && rules.next().is_none()
}

fn has_write(value: &Value) -> bool {
    matches!(value["permission"].as_str(), Some("write" | "maintain" | "admin"))
}

fn no_bypass(value: &Value) -> bool {
    if value.is_null() { return true; }
    let Some(object) = value.as_object() else { return false; };
    for (name, v) in object {
        if !matches!(name.as_str(), "users" | "teams" | "apps") {
            return false;
        }
        if !v.as_array().is_some_and(Vec::is_empty) {
            return false;
        }
    }
    true
}

fn has_required_status_checks(protection: &Value) -> bool {
    let checks = &protection["required_status_checks"];
    if !checks.is_object() || checks["strict"].as_bool() != Some(true) {
        return false;
    }
    // This verifies only that some status checks are required. The pilot must
    // separately verify exact check names, trusted publishers and behavior.
    let contexts = checks["contexts"].as_array().is_some_and(|items| {
        !items.is_empty() && items.iter().all(|item| item.as_str().is_some_and(|s| !s.is_empty()))
    });
    let structured = checks["checks"].as_array().is_some_and(|items| {
        !items.is_empty() && items.iter().all(|item| {
            item["context"].as_str().is_some_and(|s| !s.is_empty())
                && item["app_id"].as_i64().is_some_and(|id| id > 0)
        })
    });
    contexts || structured
}

/// Fail closed on every unknown input. Even an Ok result is only permission to
/// investigate a live pilot; it is NOT an authorization to change a threshold.
pub fn assess_pilot(e: &PilotEvidence<'_>) -> Result<ReadOnlyCandidate, String> {
    if e.repo != PILOT_REPO || e.branch != PILOT_BRANCH {
        return Err("pilot evidence is scoped only to .github/main".into());
    }
    if !sha40(e.base_sha) || !sha40(e.codeowners_ref_sha)
        || !e.base_sha.eq_ignore_ascii_case(e.codeowners_ref_sha)
    {
        return Err("CODEOWNERS evidence is not pinned to the exact base SHA".into());
    }
    if !one_owner_rule(e.codeowners) {
        return Err("CODEOWNERS must contain exactly one full-coverage two-owner rule".into());
    }
    match e.codeowners_errors.get("errors").and_then(Value::as_array) {
        Some(errors) if errors.is_empty() => {}
        _ => return Err("GitHub CODEOWNERS error readback missing or nonempty".into()),
    }
    if !has_write(e.oresoftware_access) || !has_write(e.the1mills_access) {
        return Err("both designated reviewers must have write/maintain/admin access".into());
    }
    let rules = e.rulesets.as_array()
        .ok_or("ruleset inventory missing or not an array")?;
    for rule in rules {
        match rule["enforcement"].as_str() {
            Some("disabled" | "evaluate") if rule["id"].as_u64().is_some() => {}
            Some("active") => {
                return Err("active local/inherited ruleset requires separate effective-rule audit".into());
            }
            _ => return Err("unknown or incomplete ruleset inventory".into()),
        }
    }
    let p = e.protection;
    let reviews = &p["required_pull_request_reviews"];
    if reviews["require_code_owner_reviews"].as_bool() != Some(true) {
        return Err("GitHub does not require code-owner review".into());
    }
    if reviews["required_approving_review_count"].as_u64() != Some(1) {
        return Err("approval threshold is not exactly one".into());
    }
    if reviews["dismiss_stale_reviews"].as_bool() != Some(true) {
        return Err("stale approval dismissal is not verified".into());
    }
    if reviews["require_last_push_approval"].as_bool() != Some(true) {
        return Err("last-push independent review is not verified".into());
    }
    if !no_bypass(&reviews["bypass_pull_request_allowances"]) {
        return Err("review bypass permissions are present or ambiguous".into());
    }
    if p.pointer("/enforce_admins/enabled").and_then(Value::as_bool) != Some(true) {
        return Err("admin enforcement is missing or unverified".into());
    }
    if !has_required_status_checks(p) {
        return Err("required CI/status checks missing or malformed".into());
    }
    Ok(ReadOnlyCandidate { base_sha: e.base_sha.to_owned() })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    const HEAD: &str = "fedcba9876543210fedcba9876543210fedcba98";

    struct Fixture {
        codeowners: String,
        errors: Value,
        protection: Value,
        rulesets: Value,
        software: Value,
        mills: Value,
        base: String,
        co_base: String,
        repo: String,
        branch: String,
    }
    impl Fixture {
        fn new() -> Self {
            Self {
                codeowners: format!("# proposal\n{EXPECTED_OWNERS}\n"),
                errors: json!({"errors":[]}),
                protection: json!({
                    "enforce_admins":{"enabled":true},
                    "required_status_checks":{"strict":true,"checks":[
                        {"context":"trusted-ci","app_id":15368}]},
                    "required_pull_request_reviews":{
                        "required_approving_review_count":1,
                        "require_code_owner_reviews":true,
                        "dismiss_stale_reviews":true,
                        "require_last_push_approval":true,
                        "bypass_pull_request_allowances":{"users":[],"teams":[],"apps":[]}
                    }
                }),
                rulesets: json!([]),
                software: json!({"permission":"write"}),
                mills: json!({"permission":"admin"}),
                base: HEAD.into(), co_base: HEAD.into(),
                repo: PILOT_REPO.into(), branch: PILOT_BRANCH.into(),
            }
        }
        fn check(&self) -> Result<ReadOnlyCandidate,String> {
            assess_pilot(&PilotEvidence {
                repo:&self.repo,branch:&self.branch,base_sha:&self.base,
                codeowners_ref_sha:&self.co_base,codeowners:&self.codeowners,
                codeowners_errors:&self.errors,protection:&self.protection,
                rulesets:&self.rulesets,oresoftware_access:&self.software,
                the1mills_access:&self.mills
            })
        }
    }

    #[test]
    fn complete_exact_head_candidate_is_read_only() {
        assert_eq!(Fixture::new().check(), Ok(ReadOnlyCandidate { base_sha: HEAD.into() }));
    }
    #[test]
    fn wrong_repo_is_not_a_global_codeowners_proof() {
        let mut f=Fixture::new();f.repo="oreslang-source.java".into();assert!(f.check().is_err());
    }
    #[test]
    fn wrong_default_branch_rejected() {
        let mut f=Fixture::new();f.branch="development".into();assert!(f.check().is_err());
    }
    #[test]
    fn unpinned_or_stale_codeowners_rejected() {
        let mut f=Fixture::new();f.co_base="0000000000000000000000000000000000000000".into();
        assert!(f.check().is_err());
        f.co_base="main".into();assert!(f.check().is_err());
    }
    #[test]
    fn malformed_head_rejected() {
        let mut f=Fixture::new();f.base="HEAD".into();assert!(f.check().is_err());
    }
    #[test]
    fn unrelated_wildcard_owner_rejected() {
        let mut f=Fixture::new();f.codeowners="* @stranger\n".into();assert!(f.check().is_err());
    }
    #[test]
    fn later_owner_override_rejected() {
        let mut f=Fixture::new();f.codeowners.push_str("/src/ @someone\n");assert!(f.check().is_err());
    }
    #[test]
    fn missing_codeowner_errors_receipt_rejected() {
        let mut f=Fixture::new();f.errors=json!({});assert!(f.check().is_err());
    }
    #[test]
    fn codeowner_parse_error_rejected() {
        let mut f=Fixture::new();f.errors=json!({"errors":[{"line":1}]});assert!(f.check().is_err());
    }
    #[test]
    fn missing_reviewer_write_permission_rejected() {
        let mut f=Fixture::new();f.software=json!({"permission":"read"});assert!(f.check().is_err());
    }
    #[test]
    fn malformed_second_reviewer_access_rejected() {
        let mut f=Fixture::new();f.mills=json!({});assert!(f.check().is_err());
    }
    #[test]
    fn unknown_ruleset_inventory_rejected() {
        let mut f=Fixture::new();f.rulesets=json!({});assert!(f.check().is_err());
    }
    #[test]
    fn active_ruleset_requires_manual_review() {
        let mut f=Fixture::new();f.rulesets=json!([{"id":8,"enforcement":"active"}]);
        assert!(f.check().is_err());
    }
    #[test]
    fn unknown_ruleset_enforcement_rejected() {
        let mut f=Fixture::new();f.rulesets=json!([{"id":8,"enforcement":"future"}]);
        assert!(f.check().is_err());
    }
    #[test]
    fn missing_required_codeowner_protection_rejected() {
        let mut f=Fixture::new();f.protection["required_pull_request_reviews"]["require_code_owner_reviews"]=json!(false);
        assert!(f.check().is_err());
    }
    #[test]
    fn count_above_one_is_not_candidate() {
        let mut f=Fixture::new();f.protection["required_pull_request_reviews"]["required_approving_review_count"]=json!(2);
        assert!(f.check().is_err());
    }
    #[test]
    fn stale_reviews_must_be_dismissed() {
        let mut f=Fixture::new();f.protection["required_pull_request_reviews"]["dismiss_stale_reviews"]=json!(false);
        assert!(f.check().is_err());
    }
    #[test]
    fn last_push_must_have_independent_review() {
        let mut f=Fixture::new();f.protection["required_pull_request_reviews"]["require_last_push_approval"]=json!(false);
        assert!(f.check().is_err());
    }
    #[test]
    fn bypass_permission_rejected() {
        let mut f=Fixture::new();f.protection["required_pull_request_reviews"]["bypass_pull_request_allowances"]["users"]=json!(["bypass"]);
        assert!(f.check().is_err());
    }
    #[test]
    fn admin_enforcement_must_be_present() {
        let mut f=Fixture::new();f.protection["enforce_admins"]["enabled"]=json!(false);
        assert!(f.check().is_err());
    }
    #[test]
    fn no_required_status_checks_rejected() {
        let mut f=Fixture::new();f.protection["required_status_checks"]=Value::Null;
        assert!(f.check().is_err());
    }
    #[test]
    fn status_checks_without_strict_base_freshness_rejected() {
        let mut f=Fixture::new();
        f.protection["required_status_checks"]["strict"]=json!(false);
        assert!(f.check().is_err());
        f.protection["required_status_checks"]["strict"]=Value::Null;
        assert!(f.check().is_err());
    }
    #[test]
    fn unknown_status_publisher_rejected() {
        let mut f=Fixture::new();f.protection["required_status_checks"]["checks"][0]["app_id"]=json!(-1);
        assert!(f.check().is_err());
    }
    #[test]
    fn alternate_valid_status_contexts_accepted() {
        let mut f=Fixture::new();f.protection["required_status_checks"]=json!({"strict":true,"contexts":["trusted-ci"]});
        assert!(f.check().is_ok());
    }
    #[test]
    fn codeowner_rule_with_crlf_rejected() {
        let mut f=Fixture::new();f.codeowners=EXPECTED_OWNERS.to_owned()+"\r\n";
        assert!(f.check().is_err());
    }
}
