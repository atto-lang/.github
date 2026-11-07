// Negative and operational contract fixtures for the two-phase GitHub policy tool.
// These are source tests, not proof that hosted Actions executed them.
use atto_merge_review_policy::{
    apply_planned_repo, audit_repo, count, encode_segment, expected_after,
    parse_options, patch, preflight_all, valid_repo_name, Api, Outcome, RepoPlan, ORG,
};
use serde_json::{json, Value};
use std::collections::VecDeque;

#[derive(Default)]
struct Mock {
    steps: VecDeque<(String, String, Result<Value, String>)>,
    writes: Vec<Value>,
}
impl Mock {
    fn ok(mut self, method: &str, path: &str, value: Value) -> Self {
        self.steps.push_back((method.into(), path.into(), Ok(value)));
        self
    }
    fn err(mut self, method: &str, path: &str) -> Self {
        self.steps.push_back((method.into(), path.into(), Err("permission denied".into())));
        self
    }
    fn done(&self) {
        assert!(self.steps.is_empty(), "unconsumed API calls");
    }
}
impl Api for Mock {
    fn request(&mut self, method: &str, endpoint: &str, body: Option<Value>) -> Result<Value, String> {
        let (expected_method, expected_path, result) = self.steps.pop_front()
            .unwrap_or_else(|| panic!("unexpected {method} {endpoint}"));
        assert_eq!(method, expected_method);
        assert_eq!(endpoint, expected_path);
        if method == "PATCH" {
            self.writes.push(body.expect("missing PATCH body"));
        } else {
            assert!(body.is_none(), "GET must not send a body");
        }
        result
    }
}

fn rpath(repo: &str) -> String {
    format!("repos/{ORG}/{repo}/rulesets?includes_parents=true&per_page=100&page=1")
}
fn ppath(repo: &str) -> String {
    format!("repos/{ORG}/{repo}/branches/main/protection")
}
fn writepath(repo: &str) -> String {
    format!("{}/required_pull_request_reviews", ppath(repo))
}
fn sample(n: u64) -> Value {
    json!({
        "required_pull_request_reviews": {
            "required_approving_review_count": n,
            "require_code_owner_reviews": false,
            "dismiss_stale_reviews": true,
            "require_last_push_approval": true,
            "bypass_pull_request_allowances": {"users": [], "teams": [], "apps": []}
        },
        "required_status_checks": {"strict": true, "contexts": ["security", "ci"]},
        "enforce_admins": {"enabled": true}
    })
}
fn base(repo: &str, n: u64) -> Mock {
    Mock::default().ok("GET", &rpath(repo), json!([]))
        .ok("GET", &ppath(repo), sample(n))
}
fn plan(n: u64) -> RepoPlan {
    RepoPlan { repo: "r".into(), branch: "main".into(), before: sample(n), count: n }
}
fn ready_apply(n: u64) -> Mock {
    base("r",n).ok("GET",&rpath("r"),json!([])).ok("GET",&ppath("r"),sample(n))
}

#[test]
fn reject_apply_without_scope() {
    let args = ["--apply"];
    assert!(parse_options(args.into_iter().map(String::from)).is_err());
}

#[test]
fn reject_bulk_without_confirmation() {
    let args = ["--apply", "--all"];
    assert!(parse_options(args.into_iter().map(String::from)).is_err());
}

#[test]
fn reject_single_apply_without_confirmation() {
    let args = ["--apply", "--repo", "r"];
    assert!(parse_options(args.into_iter().map(String::from)).is_err());
}

#[test]
fn reject_confirmation_without_apply() {
    let args = ["--confirm-org", "ores-truffle-oreslang"];
    assert!(parse_options(args.into_iter().map(String::from)).is_err());
}

#[test]
fn reject_wrong_confirmation() {
    let args = ["--apply", "--repo", "r", "--confirm-org", "other"];
    assert!(parse_options(args.into_iter().map(String::from)).is_err());
}

#[test]
fn reject_all_and_repo_combined() {
    let args = ["--apply", "--all", "--repo", "r", "--confirm-org", "ores-truffle-oreslang"];
    assert!(parse_options(args.into_iter().map(String::from)).is_err());
}

#[test]
fn reject_duplicate_repo() {
    let args = ["--repo", "r", "--repo", "x"];
    assert!(parse_options(args.into_iter().map(String::from)).is_err());
}

#[test]
fn reject_duplicate_apply() {
    let args = ["--apply", "--apply", "--repo", "r", "--confirm-org", "ores-truffle-oreslang"];
    assert!(parse_options(args.into_iter().map(String::from)).is_err());
}

#[test]
fn reject_duplicate_all() {
    let args = ["--all", "--all"];
    assert!(parse_options(args.into_iter().map(String::from)).is_err());
}

#[test]
fn reject_duplicate_confirmation() {
    let args = ["--apply", "--all", "--confirm-org", "ores-truffle-oreslang", "--confirm-org", "ores-truffle-oreslang"];
    assert!(parse_options(args.into_iter().map(String::from)).is_err());
}

#[test]
fn reject_missing_repo_value() {
    let args = ["--repo"];
    assert!(parse_options(args.into_iter().map(String::from)).is_err());
}

#[test]
fn reject_missing_org_value() {
    let args = ["--confirm-org"];
    assert!(parse_options(args.into_iter().map(String::from)).is_err());
}

#[test]
fn reject_unknown_flag() {
    let args = ["--force"];
    assert!(parse_options(args.into_iter().map(String::from)).is_err());
}

#[test]
fn reject_user_supplied_org_swap() {
    let args = ["--org", "other"];
    assert!(parse_options(args.into_iter().map(String::from)).is_err());
}

#[test]
fn reject_slash_repo_injection() {
    let args = ["--repo", "x/y"];
    assert!(parse_options(args.into_iter().map(String::from)).is_err());
}

#[test]
fn reject_dotdot_repo_injection() {
    let args = ["--repo", ".."];
    assert!(parse_options(args.into_iter().map(String::from)).is_err());
}

#[test]
fn reject_query_repo_injection() {
    let args = ["--repo", "x?admin=true"];
    assert!(parse_options(args.into_iter().map(String::from)).is_err());
}

#[test]
fn reject_newline_repo_injection() {
    let args = ["--repo", "x\ny"];
    assert!(parse_options(args.into_iter().map(String::from)).is_err());
}

#[test]
fn reject_unicode_repo_injection() {
    let args = ["--repo", "⚡"];
    assert!(parse_options(args.into_iter().map(String::from)).is_err());
}

#[test]
fn reject_bulk_flag_without_apply() {
    let args = ["--all"];
    assert!(parse_options(args.into_iter().map(String::from)).is_err());
}

#[test]
fn reject_oversized_repository_name() {
    assert!(!valid_repo_name(&"r".repeat(101)));
}

#[test]
fn accept_read_only_default() {
    let o = parse_options(Vec::<String>::new()).unwrap();
    assert!(!o.apply && !o.all && o.repo.is_none());
}

#[test]
fn accept_read_only_single_repo() {
    let o = parse_options(["--repo", "oreslang-source.java"].into_iter().map(String::from)).unwrap();
    assert!(!o.apply && !o.all && o.repo.as_deref() == Some("oreslang-source.java"));
}

#[test]
fn accept_confirmed_one_repo_apply() {
    let o = parse_options(["--repo", "r", "--apply", "--confirm-org", ORG, "--ack-numeric-only"].into_iter().map(String::from)).unwrap();
    assert!(o.apply && !o.all && o.repo.as_deref() == Some("r"));
}

#[test]
fn accept_confirmed_full_org_apply() {
    let o = parse_options(["--apply", "--all", "--confirm-org", ORG, "--expect-repos", "80", "--ack-numeric-only"].into_iter().map(String::from)).unwrap();
    assert!(o.apply && o.all && o.repo.is_none());
}

#[test]
fn allow_dot_and_dash_in_safe_repo() {
    assert!(valid_repo_name("oreslang-source.java"));
}

#[test]
fn encode_branch_slash() {
    assert_eq!(encode_segment("release/2026"), "release%2F2026");
}

#[test]
fn encode_branch_space() {
    assert_eq!(encode_segment("feature q"), "feature%20q");
}

#[test]
fn encode_branch_query() {
    assert_eq!(encode_segment("feature?x"), "feature%3Fx");
}

#[test]
fn encode_branch_fragment() {
    assert_eq!(encode_segment("feature#x"), "feature%23x");
}

#[test]
fn encode_branch_percent() {
    assert_eq!(encode_segment("feature%2F"), "feature%252F");
}

#[test]
fn encode_branch_colon() {
    assert_eq!(encode_segment("feature:patch"), "feature%3Apatch");
}

#[test]
fn encode_branch_unicode() {
    assert_eq!(encode_segment("föö"), "f%C3%B6%C3%B6");
}

#[test]
fn patch_zero_approval_to_one() {
    let p = sample(0); assert_eq!(patch(&p), Some(json!({"required_approving_review_count":1})));
}

#[test]
fn patch_two_approvals_to_one() {
    let p = sample(2); assert_eq!(patch(&p), Some(json!({"required_approving_review_count":1})));
}

#[test]
fn patch_six_approvals_to_one() {
    let p = sample(6); assert_eq!(patch(&p), Some(json!({"required_approving_review_count":1})));
}

#[test]
fn do_not_patch_already_one() {
    assert!(patch(&sample(1)).is_none());
}

#[test]
fn do_not_patch_unsupported_seven() {
    assert!(patch(&sample(7)).is_none());
}

#[test]
fn do_not_patch_missing_review_block() {
    assert!(patch(&json!({})).is_none());
}

#[test]
fn do_not_patch_codeowners_true() {
    let mut p = sample(3); p["required_pull_request_reviews"]["require_code_owner_reviews"] = json!(true); assert!(patch(&p).is_none());
}

#[test]
fn do_not_patch_codeowners_string() {
    let mut p = sample(3); p["required_pull_request_reviews"]["require_code_owner_reviews"] = json!("false"); assert!(patch(&p).is_none());
}

#[test]
fn do_not_patch_codeowners_absent() {
    let mut p = sample(3); p["required_pull_request_reviews"].as_object_mut().unwrap().remove("require_code_owner_reviews"); assert!(patch(&p).is_none());
}

#[test]
fn do_not_patch_bypass_users() {
    let mut p = sample(3); p["required_pull_request_reviews"]["bypass_pull_request_allowances"]["users"] = json!(["u"]); assert!(patch(&p).is_none());
}

#[test]
fn do_not_patch_bypass_teams() {
    let mut p = sample(3); p["required_pull_request_reviews"]["bypass_pull_request_allowances"]["teams"] = json!(["t"]); assert!(patch(&p).is_none());
}

#[test]
fn do_not_patch_bypass_apps() {
    let mut p = sample(3); p["required_pull_request_reviews"]["bypass_pull_request_allowances"]["apps"] = json!(["a"]); assert!(patch(&p).is_none());
}

#[test]
fn do_not_patch_malformed_bypass() {
    let mut p = sample(3); p["required_pull_request_reviews"]["bypass_pull_request_allowances"] = json!("invalid"); assert!(patch(&p).is_none());
}

#[test]
fn do_not_patch_malformed_review_boolean() {
    let mut p = sample(3); p["required_pull_request_reviews"]["dismiss_stale_reviews"] = json!("true"); assert!(patch(&p).is_none());
}

#[test]
fn do_not_patch_malformed_last_push() {
    let mut p = sample(3); p["required_pull_request_reviews"]["require_last_push_approval"] = json!(1); assert!(patch(&p).is_none());
}

#[test]
fn only_change_review_count_snapshot() {
    let p = sample(3); let after = expected_after(p.clone()).unwrap(); assert_eq!(count(&after),Some(1)); assert_eq!(after["required_status_checks"],p["required_status_checks"]); assert_eq!(after["enforce_admins"],p["enforce_admins"]);
}

#[test]
fn preflight_rejects_empty_inventory() {
    let mut api = Mock::default(); assert!(preflight_all(&mut api, &[]).is_err()); assert!(api.writes.is_empty());
}

#[test]
fn preflight_accepts_one_repository() {
    let mut api=base("r",3); let plans=preflight_all(&mut api,&[("r".into(),"main".into())]).unwrap(); assert_eq!(plans.len(),1); assert_eq!(plans[0].count,3); assert_eq!(plans[0].before,sample(3)); api.done();
}

#[test]
fn preflight_accepts_compliant_repository() {
    let mut api=base("r",1); let plans=preflight_all(&mut api,&[("r".into(),"main".into())]).unwrap(); assert_eq!(plans[0].count,1); api.done();
}

#[test]
fn preflight_handles_two_repositories() {
    let mut api=base("r",3).ok("GET",&rpath("s"),json!([])).ok("GET",&ppath("s"),sample(2)); let plans=preflight_all(&mut api,&[("r".into(),"main".into()),("s".into(),"main".into())]).unwrap(); assert_eq!(plans.len(),2); assert!(api.writes.is_empty()); api.done();
}

#[test]
fn preflight_rejects_duplicate_repositories() {
    let mut api=base("r",3); let e=preflight_all(&mut api,&[("r".into(),"main".into()),("r".into(),"main".into())]).unwrap_err(); assert!(e.iter().any(|x|x.contains("duplicate"))); assert!(api.writes.is_empty()); api.done();
}

#[test]
fn preflight_rejects_empty_branch() {
    let mut api=Mock::default(); assert!(preflight_all(&mut api,&[("r".into(),"".into())]).is_err()); api.done();
}

#[test]
fn preflight_rejects_unsafe_repo() {
    let mut api=Mock::default(); assert!(preflight_all(&mut api,&[("../x".into(),"main".into())]).is_err()); api.done();
}

#[test]
fn preflight_rejects_active_ruleset() {
    let mut api=Mock::default().ok("GET",&rpath("r"),json!([{"id":7,"enforcement":"active"}])); assert!(preflight_all(&mut api,&[("r".into(),"main".into())]).is_err()); assert!(api.writes.is_empty()); api.done();
}

#[test]
fn preflight_rejects_ruleset_permission_error() {
    let mut api=Mock::default().err("GET",&rpath("r")); assert!(preflight_all(&mut api,&[("r".into(),"main".into())]).is_err()); api.done();
}

#[test]
fn preflight_rejects_protection_permission_error() {
    let mut api=Mock::default().ok("GET",&rpath("r"),json!([])).err("GET",&ppath("r")); assert!(preflight_all(&mut api,&[("r".into(),"main".into())]).is_err()); api.done();
}

#[test]
fn preflight_reports_all_failed_repositories() {
    let mut api=Mock::default().err("GET",&rpath("r")).err("GET",&rpath("s")); let err=preflight_all(&mut api,&[("r".into(),"main".into()),("s".into(),"main".into())]).unwrap_err(); assert_eq!(err.len(),2); assert!(api.writes.is_empty()); api.done();
}

#[test]
fn apply_plan_rejects_stale_approval_count() {
    let mut api=Mock::default().ok("GET",&rpath("r"),json!([])).ok("GET",&ppath("r"),sample(4)); let outcome=apply_planned_repo(&mut api,&plan(3)); assert!(matches!(outcome,Outcome::Manual(_))); assert!(api.writes.is_empty()); api.done();
}

#[test]
fn apply_plan_rejects_stale_status_checks() {
    let mut altered=sample(3); altered["required_status_checks"]["contexts"]=json!(["ci"]); let mut api=Mock::default().ok("GET",&rpath("r"),json!([])).ok("GET",&ppath("r"),altered); assert!(matches!(apply_planned_repo(&mut api,&plan(3)),Outcome::Manual(_))); assert!(api.writes.is_empty()); api.done();
}

#[test]
fn apply_plan_rejects_new_ruleset() {
    let mut api=Mock::default().ok("GET",&rpath("r"),json!([{"id":9,"enforcement":"active"}])); assert!(matches!(apply_planned_repo(&mut api,&plan(3)),Outcome::Manual(_))); assert!(api.writes.is_empty()); api.done();
}

#[test]
fn apply_plan_rejects_midflight_drift() {
    let mut api=base("r",3).ok("GET",&rpath("r"),json!([])).ok("GET",&ppath("r"),sample(4)); assert!(matches!(apply_planned_repo(&mut api,&plan(3)),Outcome::Manual(_))); assert!(api.writes.is_empty()); api.done();
}

#[test]
fn apply_plan_writes_only_count() {
    let mut api=ready_apply(3).ok("PATCH",&writepath("r"),json!({})).ok("GET",&ppath("r"),expected_after(sample(3)).unwrap()).ok("GET",&rpath("r"),json!([])); assert_eq!(apply_planned_repo(&mut api,&plan(3)),Outcome::Changed(3)); assert_eq!(api.writes,vec![json!({"required_approving_review_count":1})]); api.done();
}

#[test]
fn apply_plan_rejects_patch_error() {
    let mut api=ready_apply(3).err("PATCH",&writepath("r")); assert!(matches!(apply_planned_repo(&mut api,&plan(3)),Outcome::Error(_))); assert_eq!(api.writes.len(),1); api.done();
}

#[test]
fn apply_plan_rejects_modified_readback() {
    let mut after=expected_after(sample(3)).unwrap(); after["required_status_checks"]["strict"]=json!(false); let mut api=ready_apply(3).ok("PATCH",&writepath("r"),json!({})).ok("GET",&ppath("r"),after); assert!(matches!(apply_planned_repo(&mut api,&plan(3)),Outcome::Error(_))); api.done();
}

#[test]
fn apply_plan_rejects_ruleset_after_write() {
    let mut api=ready_apply(3).ok("PATCH",&writepath("r"),json!({})).ok("GET",&ppath("r"),expected_after(sample(3)).unwrap()).ok("GET",&rpath("r"),json!([{"id":77,"enforcement":"active"}])); assert!(matches!(apply_planned_repo(&mut api,&plan(3)),Outcome::Error(_))); api.done();
}

#[test]
fn apply_plan_noop_when_already_compliant() {
    let mut api=base("r",1); assert_eq!(apply_planned_repo(&mut api,&plan(1)),Outcome::Compliant); assert!(api.writes.is_empty()); api.done();
}

#[test]
fn single_repo_dry_run_never_writes() {
    let mut api=base("r",3); assert_eq!(audit_repo(&mut api,"r","main",false),Outcome::WouldChange(3)); assert!(api.writes.is_empty()); api.done();
}
