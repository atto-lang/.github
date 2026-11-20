//! Read-only GitHub Actions metadata collector; Rust/std only.
//! This is NOT a release attestation: executable native tests, immutable
//! checkout provenance, effective review/ruleset checks are separate gates.
//! Run only in a protected, trusted, source-local workflow. Never with PR code.
#[cfg(test)]
use std::{collections::BTreeMap, io::Write, process::Stdio};
#[cfg(not(test))]
use std::env;
use std::process::Command;

trait ReadOnlyApi {
    fn get_tsv(&self, path: &str, query: &str) -> Result<String, String>;
}

// Never resolve an authenticated gh executable through PATH. A build step
// that prepends a writable directory to PATH must not redirect the token
// into a sibling executable. Use the system-owned location on Ubuntu 24.04.
// Missing binary is a hard failure, not an unpinned fallback.
const PINNED_GH_EXECUTABLE: &str = "/usr/bin/gh";

fn get_tsv_via_gh(binary: &str, path: &str, query: &str) -> Result<String, String> {
    let result = Command::new(binary)
        .args(["api", "--method", "GET", "-H",
               "Accept: application/vnd.github+json", path, "--jq", query])
        .output()
        .map_err(|_| "pinned GitHub CLI unavailable".to_string())?;
    if !result.status.success() {
        // Never emit stderr: it may include upstream or private data.
        return Err("authenticated read-only GitHub API request failed".into());
    }
    let raw = String::from_utf8(result.stdout)
        .map_err(|_| "API result is not valid UTF-8".to_string())?;
    if raw.len() > 8192 || raw.contains('\0') {
        return Err("API scalar response too large or malformed".into());
    }
    Ok(raw.trim_end_matches('\n').to_string())
}

#[cfg(not(test))]
struct GhApi;
#[cfg(not(test))]
impl ReadOnlyApi for GhApi {
    fn get_tsv(&self, path: &str, query: &str) -> Result<String, String> {
        get_tsv_via_gh(PINNED_GH_EXECUTABLE, path, query)
    }
}

fn repo_slug(slug: &str) -> bool {
    let pieces: Vec<&str> = slug.split('/').collect();
    pieces.len() == 2 && pieces.iter().all(|part| {
        !part.is_empty() && part.len() <= 100
            && part != &"." && part != &".."
            && part.bytes().all(|b| b.is_ascii_alphanumeric()
                || b == b'-' || b == b'_' || b == b'.')
    })
}

fn oid(sha: &str) -> bool {
    sha.len() == 40 && sha.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        && sha.bytes().any(|b| b != b'0')
}

fn fields(raw: &str, len: usize) -> Result<Vec<&str>, String> {
    if raw.contains('\n') || raw.contains('\r') {
        return Err("unexpected multiline API response".into());
    }
    let parts: Vec<_> = raw.split('\t').collect();
    if parts.len() != len || parts.iter().any(|s| s.is_empty()) {
        return Err("wrong number of API fields".into());
    }
    Ok(parts)
}

fn numeric(value: &str) -> Result<u64, String> {
    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
        return Err("invalid numeric API field".into());
    }
    value.parse::<u64>().map_err(|_| "numeric API field overflow".into())
}

fn expect_eq(a: &str, b: &str, reason: &str) -> Result<(), String> {
    if a == b { Ok(()) } else { Err(reason.into()) }
}
fn expect_num(a: &str, b: u64, reason: &str) -> Result<(), String> {
    if numeric(a)? == b && b != 0 { Ok(()) } else { Err(reason.into()) }
}

#[derive(Clone)]
struct Expected<'a> {
    repo: &'a str,
    branch: &'a str,
    commit: &'a str,
    tree: &'a str,
    run_id: u64,
    job_id: u64,
    check_id: u64,
    workflow_id: u64,
    issuer_app_id: u64,
    required_context: &'a str,
}

const REPO_Q: &str = r#"[.id, .private]|@tsv"#;
const BRANCH_Q: &str = r#"[.protected, .commit.sha]|@tsv"#;
// __CONTEXT__ and __APP_ID__ are interpolated ONLY after strict
// [A-Za-z0-9 _./-] validation and decimal u64 encoding.
// No arbitrary user input is interpreted as jq source.
const PROTECTION_Q: &str = r#"[
  (.required_status_checks.strict == true),
  (.enforce_admins.enabled == true),
  (.required_pull_request_reviews.required_approving_review_count // 0),
  (.required_pull_request_reviews.require_code_owner_reviews == true),
  (.required_pull_request_reviews.dismiss_stale_reviews == true),
  (.required_pull_request_reviews.require_last_push_approval == true),
  (.required_conversation_resolution.enabled == true),
  ((.required_status_checks.checks // [])
    | map(select(.context == "__CONTEXT__" and .app_id == __APP_ID__))
    | length),
  (((.required_pull_request_reviews.bypass_pull_request_allowances.users // []) | length)
    + ((.required_pull_request_reviews.bypass_pull_request_allowances.teams // []) | length)
    + ((.required_pull_request_reviews.bypass_pull_request_allowances.apps // []) | length)),
  (.allow_force_pushes.enabled == true),
  (.allow_deletions.enabled == true)
]|@tsv"#;
const RUN_Q: &str = r#"[.id, .repository.id, .head_sha, .workflow_id, .status, .conclusion, .check_suite_id, .event, .head_branch, .run_attempt]|@tsv"#;
const JOB_Q: &str = r#"[.id, .run_id, (.runner_id // 0), .status, .conclusion, ((.steps // [])|length), ((.steps // [])|map(select(.status == "completed" and .conclusion == "success"))|length), .check_run_url]|@tsv"#;
const CHECK_Q: &str = r#"[.id, .app.id, .head_sha, .status, .conclusion, .check_suite.id, .name]|@tsv"#;
const COMMIT_Q: &str = r#"[.sha, .tree.sha]|@tsv"#;

fn audit(api: &impl ReadOnlyApi, e: &Expected<'_>) -> Result<(), String> {
    if !repo_slug(e.repo) || e.branch != "main" || !oid(e.commit) || !oid(e.tree)
        || e.required_context.is_empty() || e.required_context.len() > 150
        || !e.required_context.bytes().all(|b|
            b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.' | b'/' | b' '))
    {
        return Err("invalid or unsafe immutable target".into());
    }
    if [e.run_id,e.job_id,e.check_id,e.workflow_id,e.issuer_app_id]
        .iter().any(|id| *id == 0)
    {
        return Err("zero immutable evidence ID".into());
    }
    let root = format!("repos/{}", e.repo);
    let repo = api.get_tsv(&root, REPO_Q)?;
    let repo = fields(&repo,2)?;
    let repo_id = numeric(repo[0])?;
    if repo_id == 0 || repo[1] != "true" {
        return Err("source repository must be private and identified".into());
    }

    let branch = api.get_tsv(&format!("{root}/branches/{}",e.branch), BRANCH_Q)?;
    let branch = fields(&branch,2)?;
    if branch[0] != "true" || !oid(branch[1]) {
        return Err("branch is unprotected or unverified".into());
    }
    expect_eq(branch[1],e.commit,"expected commit is not current protected main")?;

    // The 'protected' Boolean cannot establish effective review or CI rules.
    // GitHub's protection endpoint requires administration/read permission.
    // Any plan/permission/404 error is a release blocker, never a pass.
    let jq=PROTECTION_Q
        .replace("__CONTEXT__",e.required_context)
        .replace("__APP_ID__",&e.issuer_app_id.to_string());
    let policy=api.get_tsv(&format!("{root}/branches/main/protection"),&jq)?;
    let policy=fields(&policy,11)?;
    if policy[0]!="true" || policy[1]!="true" || numeric(policy[2])?==0
        || policy[3]!="true" || policy[4]!="true" || policy[5]!="true"
        || policy[6]!="true" || numeric(policy[7])?!=1
        || numeric(policy[8])?!=0 || policy[9]!="false" || policy[10]!="false"
    {
        return Err("required protected-branch rules or App-bound CI missing".into());
    }

    let run = api.get_tsv(&format!("{root}/actions/runs/{}",e.run_id),RUN_Q)?;
    let run = fields(&run,10)?;
    expect_num(run[0],e.run_id,"wrong run")?;
    expect_num(run[1],repo_id,"cross-repository workflow run")?;
    expect_eq(run[2],e.commit,"stale run head")?;
    expect_num(run[3],e.workflow_id,"untrusted workflow ID")?;
    if run[4]!="completed" || run[5]!="success" || run[7]!="push"
        || run[8]!=e.branch || numeric(run[9])?==0
    {
        return Err("workflow not successful protected-source push".into());
    }
    let suite_id = numeric(run[6])?;
    if suite_id==0 { return Err("missing check suite".into()); }

    let job = api.get_tsv(&format!("{root}/actions/jobs/{}",e.job_id),JOB_Q)?;
    let job = fields(&job,8)?;
    expect_num(job[0],e.job_id,"wrong job")?;
    expect_num(job[1],e.run_id,"job not linked to run")?;
    if numeric(job[2])?==0 || job[3]!="completed" || job[4]!="success" {
        return Err("job missing assigned runner or success".into());
    }
    let count = numeric(job[5])?;
    if count==0 || numeric(job[6])? != count {
        return Err("absent, skipped or failing executed job steps".into());
    }
    let expected_url = format!(
        "https://api.github.com/repos/{}/check-runs/{}",e.repo,e.check_id
    );
    expect_eq(job[7],&expected_url,"job not linked to expected check")?;

    let check = api.get_tsv(&format!("{root}/check-runs/{}",e.check_id),CHECK_Q)?;
    let check = fields(&check,7)?;
    expect_num(check[0],e.check_id,"wrong check")?;
    expect_num(check[1],e.issuer_app_id,"untrusted check issuer")?;
    expect_eq(check[2],e.commit,"stale check")?;
    if check[3]!="completed" || check[4]!="success" {
        return Err("check incomplete or failed".into());
    }
    expect_num(check[5],suite_id,"check not linked to run check-suite")?;
    expect_eq(check[6],e.required_context,"untrusted required check context")?;

    // The Git object tree is fetched via authenticated API, but this alone
    // does NOT attest which filesystem tree a runner actually built.
    let commit = api.get_tsv(&format!("{root}/git/commits/{}",e.commit),COMMIT_Q)?;
    let commit = fields(&commit,2)?;
    expect_eq(commit[0],e.commit,"wrong Git commit object")?;
    expect_eq(commit[1],e.tree,"wrong Git commit root tree")?;

    // Fail closed if main or its effective protection changed during evidence
    // collection. Neither earlier green snapshots nor stale checks certify HEAD.
    let branch_after = api.get_tsv(&format!("{root}/branches/{}",e.branch),BRANCH_Q)?;
    let branch_after = fields(&branch_after,2)?;
    if branch_after[0]!="true" || branch_after[1]!=e.commit {
        return Err("protected branch changed during evidence collection".into());
    }
    let policy_after = api.get_tsv(&format!("{root}/branches/main/protection"),&jq)?;
    let policy_after = fields(&policy_after,11)?;
    if policy_after!=policy {
        return Err("effective protection changed during evidence collection".into());
    }
    Ok(())
}

// Environment preflight avoids API requests from PRs, detached refs,
// unprotected branches, stale revisions or unexpected GitHub hosts.
// It is a defense-in-depth guard, NOT proof that the caller is trusted.
fn trusted_context(get: &impl Fn(&str) -> Option<String>,
                   repo: &str, sha: &str) -> bool {
    let value = |name: &str, expected: &str| get(name).as_deref() == Some(expected);
    let token_present = get("GH_TOKEN").is_some_and(|v| !v.is_empty())
        || get("GITHUB_TOKEN").is_some_and(|v| !v.is_empty());
    value("GITHUB_ACTIONS", "true")
        && value("GITHUB_REF_PROTECTED", "true")
        && value("GITHUB_EVENT_NAME", "push")
        && value("GITHUB_REF", "refs/heads/main")
        && value("GITHUB_REPOSITORY", repo)
        && value("GITHUB_SHA", sha)
        && value("GITHUB_SERVER_URL", "https://github.com")
        && value("GITHUB_API_URL", "https://api.github.com")
        && get("GH_HOST").is_none_or(|v| v == "github.com")
        && token_present
}

#[cfg(not(test))]
fn main() {
    // Never perform privileged API reads on untrusted PR refs or outside a
    // protected Actions job. GITHUB_TOKEN must be read-only; not printed.
    let args:Vec<_> = env::args().skip(1).collect();
    if args.len()!=10 {
        eprintln!("Usage: collector <owner/repo> main <commit> <tree> <run_id> <job_id> <check_id> <workflow_id> <issuer_app_id> <required_context>");
        std::process::exit(2);
    }
    let parsed = || -> Result<Expected<'_>,String> {
        Ok(Expected {
            repo:&args[0],branch:&args[1],commit:&args[2],tree:&args[3],
            run_id:numeric(&args[4])?,job_id:numeric(&args[5])?,
            check_id:numeric(&args[6])?,workflow_id:numeric(&args[7])?,
            issuer_app_id:numeric(&args[8])?, required_context:&args[9],
        })
    };
    if !trusted_context(&|key| env::var(key).ok(), &args[0], &args[2]) {
        eprintln!("NOT_CERTIFIED: authenticated protected source-local push context required");
        std::process::exit(3);
    }
    let result = parsed().and_then(|p|audit(&GhApi,&p));
    match result {
        Ok(()) => {
            println!("METADATA_VERIFIED_ONLY: checkout attestation, independent review and native test provenance not proven");
            std::process::exit(3); // Intentionally cannot certify.
        }
        Err(err) => {
            eprintln!("NOT_CERTIFIED: {err}");
            std::process::exit(3);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct MockApi(BTreeMap<String,String>);
    impl ReadOnlyApi for MockApi {
        fn get_tsv(&self,path:&str,_jq:&str)->Result<String,String> {
            if path.ends_with("/branches/main/protection")
                && (! _jq.contains(".context == \"test\"")
                    || ! _jq.contains(".app_id == 15368")) {
                return Err("malformed required-check binding query".into());
            }
            self.0.get(path).cloned().ok_or_else(||"API failure".into())
        }
    }
    fn fixture() -> (MockApi,Expected<'static>) {
        let repo="trusted/source";
        let sha="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let tree="bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        let mut m=BTreeMap::new();
        let root=format!("repos/{repo}");
        m.insert(root.clone(),"901\ttrue".to_string());
        m.insert(format!("{root}/branches/main"),format!("true\t{sha}"));
        // Synthetic response to the authenticated Get branch protection endpoint.
        // In production this must come from GitHub, not candidate PR data.
        m.insert(format!("{root}/branches/main/protection"),
          "true\ttrue\t1\ttrue\ttrue\ttrue\ttrue\t1\t0\tfalse\tfalse".into());
        m.insert(format!("{root}/actions/runs/400"),format!("400\t901\t{sha}\t200\tcompleted\tsuccess\t350\tpush\tmain\t1"));
        m.insert(format!("{root}/actions/jobs/500"),
          format!("500\t400\t100\tcompleted\tsuccess\t3\t3\thttps://api.github.com/repos/{repo}/check-runs/600"));
        m.insert(format!("{root}/check-runs/600"),format!("600\t15368\t{sha}\tcompleted\tsuccess\t350\ttest"));
        m.insert(format!("{root}/git/commits/{sha}"),format!("{sha}\t{tree}"));
        (MockApi(m),Expected {
            repo,branch:"main",commit:sha,tree,run_id:400,job_id:500,
            check_id:600,workflow_id:200,issuer_app_id:15368,required_context:"test"
        })
    }
    #[test] fn all_mock_metadata_links_consistent() {
        let (a,e)=fixture();assert!(audit(&a,&e).is_ok());
    }
    #[test] fn rejects_protection_disabled() {
        let (mut a,e)=fixture();a.0.insert("repos/trusted/source/branches/main".into(),
          format!("false\t{}",e.commit));assert!(audit(&a,&e).is_err());
    }
    #[test] fn rejects_changed_main_head() {
        let (mut a,e)=fixture();a.0.insert("repos/trusted/source/branches/main".into(),
          "true\tcccccccccccccccccccccccccccccccccccccccc".into());assert!(audit(&a,&e).is_err());
    }
    #[test] fn rejects_run_wrong_repo() {
        let (mut a,e)=fixture();a.0.insert("repos/trusted/source/actions/runs/400".into(),
          format!("400\t902\t{}\t200\tcompleted\tsuccess\t350\tpush\tmain\t1",e.commit));
        assert!(audit(&a,&e).is_err());
    }
    #[test] fn rejects_stale_run_and_incomplete_run() {
        let (mut a,e)=fixture();
        a.0.insert("repos/trusted/source/actions/runs/400".into(),
          "400\t901\tcccccccccccccccccccccccccccccccccccccccc\t200\tcompleted\tsuccess\t350\tpush\tmain\t1".into());
        assert!(audit(&a,&e).is_err());
        a.0.insert("repos/trusted/source/actions/runs/400".into(),
          format!("400\t901\t{}\t200\tqueued\tsuccess\t350\tpush\tmain\t1",e.commit));
        assert!(audit(&a,&e).is_err());
    }
    #[test] fn rejects_untrusted_workflow_id() {
        let (mut a,e)=fixture();
        a.0.insert("repos/trusted/source/actions/runs/400".into(),
          format!("400\t901\t{}\t201\tcompleted\tsuccess\t350\tpush\tmain\t1",e.commit));
        assert!(audit(&a,&e).is_err());
    }
    #[test] fn rejects_zero_runner_and_steps() {
        let (mut a,e)=fixture();let p="repos/trusted/source/actions/jobs/500";
        a.0.insert(p.into(),"500\t400\t0\tcompleted\tsuccess\t3\t3\thttps://api.github.com/repos/trusted/source/check-runs/600".into());
        assert!(audit(&a,&e).is_err());
        a.0.insert(p.into(),"500\t400\t100\tcompleted\tsuccess\t0\t0\thttps://api.github.com/repos/trusted/source/check-runs/600".into());
        assert!(audit(&a,&e).is_err());
    }
    #[test] fn rejects_skipped_steps_and_wrong_check_link() {
        let (mut a,e)=fixture();let p="repos/trusted/source/actions/jobs/500";
        a.0.insert(p.into(),"500\t400\t100\tcompleted\tsuccess\t3\t2\thttps://api.github.com/repos/trusted/source/check-runs/600".into());
        assert!(audit(&a,&e).is_err());
        a.0.insert(p.into(),"500\t400\t100\tcompleted\tsuccess\t3\t3\thttps://api.github.com/repos/trusted/source/check-runs/601".into());
        assert!(audit(&a,&e).is_err());
    }
    #[test] fn rejects_wrong_app_and_suite() {
        let (mut a,e)=fixture();let p="repos/trusted/source/check-runs/600";
        a.0.insert(p.into(),format!("600\t42\t{}\tcompleted\tsuccess\t350\ttest",e.commit));
        assert!(audit(&a,&e).is_err());
        a.0.insert(p.into(),format!("600\t15368\t{}\tcompleted\tsuccess\t351\ttest",e.commit));
        assert!(audit(&a,&e).is_err());
    }
    #[test] fn rejects_wrong_git_root_tree() {
        let (mut a,e)=fixture();
        a.0.insert(format!("repos/trusted/source/git/commits/{}",e.commit),
          format!("{}\tcccccccccccccccccccccccccccccccccccccccc",e.commit));
        assert!(audit(&a,&e).is_err());
    }
    #[test] fn fail_closed_on_denied_api_or_malformed_fields() {
        let (mut a,e)=fixture();a.0.remove("repos/trusted/source/check-runs/600");
        assert!(audit(&a,&e).is_err());
        let (mut a,e)=fixture();
        a.0.insert("repos/trusted/source".into(),"901\ttrue\textra".into());
        assert!(audit(&a,&e).is_err());
    }
    #[test]
    fn pinned_executable_never_uses_path_lookup() {
        assert_eq!(PINNED_GH_EXECUTABLE,"/usr/bin/gh");
        assert!(std::path::Path::new(PINNED_GH_EXECUTABLE).is_absolute());
    }

    #[test]
    fn command_adapter_executes_only_get_and_suppresses_cli_errors() {
        use std::{fs,os::unix::fs::PermissionsExt};
        let dir=std::env::temp_dir().join(format!("oreslang-gh-command-test-{}",std::process::id()));
        let _=fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let bin=dir.join("fake-gh");
        fs::write(&bin,r#"#!/bin/sh
case "$1 $2 $3" in
  "api --method GET") printf "901\ttrue\n"; exit 0 ;;
  *) echo "secret source path in stderr" >&2; exit 2 ;;
esac
"#).unwrap();
        let mut permissions=fs::metadata(&bin).unwrap().permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&bin,permissions).unwrap();
        let success=get_tsv_via_gh(bin.to_str().unwrap(),"repos/trusted/source",REPO_Q);
        assert_eq!(success,Ok("901\ttrue".into()));
        fs::write(&bin,r#"#!/bin/sh
echo "secret source path in stderr" >&2
exit 2
"#).unwrap();
        let failure=get_tsv_via_gh(bin.to_str().unwrap(),"repos/trusted/source",REPO_Q);
        assert_eq!(failure,Err("authenticated read-only GitHub API request failed".into()));
        let _=fs::remove_dir_all(&dir);
    }

    #[test] fn rejects_unsafe_repo_names_and_ids() {
        let (a,mut e)=fixture();e.repo="trusted/../../other";
        assert!(audit(&a,&e).is_err());
        e.repo="trusted/source";e.job_id=0;assert!(audit(&a,&e).is_err());
    }

    #[test] fn rejects_plan_or_permission_denied_protection_read() {
        let (mut a,e)=fixture();
        a.0.remove("repos/trusted/source/branches/main/protection");
        assert!(audit(&a,&e).is_err());
    }
    #[test] fn rejects_missing_reviews_codeowners_and_stale_dismissal() {
        let path="repos/trusted/source/branches/main/protection";
        for (position,value) in [(2,"0"),(3,"false"),(4,"false"),(5,"false")] {
            let (mut a,e)=fixture();
            let mut cells:Vec<String>=a.0[path].split('\t').map(ToString::to_string).collect();
            cells[position]=value.into();
            a.0.insert(path.into(),cells.join("\t"));
            assert!(audit(&a,&e).is_err(),"field {position}");
        }
    }
    #[test] fn rejects_unbound_ci_admin_bypass_and_missing_conversation_protection() {
        let path="repos/trusted/source/branches/main/protection";
        for (position,value) in [(0,"false"),(1,"false"),(6,"false"),(7,"0"),
                                 (8,"1"),(9,"true"),(10,"true")] {
            let (mut a,e)=fixture();
            let mut cells:Vec<String>=a.0[path].split('\t').map(ToString::to_string).collect();
            cells[position]=value.into();
            a.0.insert(path.into(),cells.join("\t"));
            assert!(audit(&a,&e).is_err(),"field {position}");
        }
    }
    #[test] fn rejects_context_name_spoof_and_unsafe_jq_arguments() {
        let (mut a,e)=fixture();
        a.0.insert("repos/trusted/source/check-runs/600".into(),
          format!("600\t15368\t{}\tcompleted\tsuccess\t350\tother",e.commit));
        assert!(audit(&a,&e).is_err());
        let (a,mut e)=fixture(); e.required_context="test\" or true #";
        assert!(audit(&a,&e).is_err());
        e.required_context="test\tforge";
        assert!(audit(&a,&e).is_err());
        e.required_context="test";
        assert!(audit(&a,&e).is_ok());
    }


    #[test] fn rejects_successful_run_on_wrong_branch_or_zero_attempt() {
        let path="repos/trusted/source/actions/runs/400";
        let (mut a,e)=fixture();
        a.0.insert(path.into(),
          format!("400\t901\t{}\t200\tcompleted\tsuccess\t350\tpush\tfeature/ci\t1",e.commit));
        assert!(audit(&a,&e).is_err());
        a.0.insert(path.into(),
          format!("400\t901\t{}\t200\tcompleted\tsuccess\t350\tpush\tmain\t0",e.commit));
        assert!(audit(&a,&e).is_err());
    }

    struct MutatingReadApi {
        values: MockApi,
        counter: std::cell::Cell<u32>,
        mutate_path: &'static str,
        mutation: String,
    }
    impl ReadOnlyApi for MutatingReadApi {
        fn get_tsv(&self,path:&str,jq:&str)->Result<String,String> {
            if path==self.mutate_path {
                let prior=self.counter.get();
                self.counter.set(prior+1);
                if prior>0 {return Ok(self.mutation.clone());}
            }
            self.values.get_tsv(path,jq)
        }
    }
    #[test] fn rejects_main_head_change_during_collection() {
        let (values,e)=fixture();
        let api=MutatingReadApi {
            values,counter:std::cell::Cell::new(0),
            mutate_path:"repos/trusted/source/branches/main",
            mutation:"true\tcccccccccccccccccccccccccccccccccccccccc".into(),
        };
        assert_eq!(audit(&api,&e).unwrap_err(),
                   "protected branch changed during evidence collection");
        assert_eq!(api.counter.get(),2);
    }
    #[test] fn rejects_changed_protection_during_collection() {
        let (values,e)=fixture();
        let api=MutatingReadApi {
            values,counter:std::cell::Cell::new(0),
            mutate_path:"repos/trusted/source/branches/main/protection",
            mutation:"false\ttrue\t1\ttrue\ttrue\ttrue\ttrue\t1\t0\tfalse\tfalse".into(),
        };
        assert_eq!(audit(&api,&e).unwrap_err(),
                   "effective protection changed during evidence collection");
        assert_eq!(api.counter.get(),2);
    }


    fn synthetic_env() -> BTreeMap<String,String> {
        [
            ("GITHUB_ACTIONS","true"),
            ("GITHUB_REF_PROTECTED","true"),
            ("GITHUB_EVENT_NAME","push"),
            ("GITHUB_REF","refs/heads/main"),
            ("GITHUB_REPOSITORY","trusted/source"),
            ("GITHUB_SHA","aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
            ("GITHUB_SERVER_URL","https://github.com"),
            ("GITHUB_API_URL","https://api.github.com"),
            ("GH_TOKEN","synthetic-non-secret-test-value"),
        ].into_iter().map(|(k,v)|(k.to_string(),v.to_string())).collect()
    }

    #[test] fn protected_source_environment_is_only_a_preflight() {
        let m=synthetic_env();
        assert!(trusted_context(&|key|m.get(key).cloned(),"trusted/source",
                                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"));
    }

    #[test] fn refuses_detached_pr_unprotected_or_foreign_environment() {
        let names=[
            ("GITHUB_ACTIONS","false"),("GITHUB_REF_PROTECTED","false"),
            ("GITHUB_EVENT_NAME","pull_request"),("GITHUB_REF","refs/heads/testing"),
            ("GITHUB_REPOSITORY","other/private"),
            ("GITHUB_SHA","bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
            ("GITHUB_SERVER_URL","https://example.com"),
            ("GITHUB_API_URL","https://not-github.example/api"),
        ];
        for (name,bad) in names {
            let mut m=synthetic_env();m.insert(name.into(),bad.into());
            assert!(!trusted_context(&|key|m.get(key).cloned(),"trusted/source",
                                     "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),"{name}");
        }
        let mut m=synthetic_env();m.insert("GH_HOST".into(),"untrusted.example".into());
        assert!(!trusted_context(&|key|m.get(key).cloned(),"trusted/source",
                                 "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"));
        m=synthetic_env();m.remove("GH_TOKEN");
        assert!(!trusted_context(&|key|m.get(key).cloned(),"trusted/source",
                                 "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"));
    }

    // Exercise the actual jq expressions against realistic-shaped synthetic
    // REST JSON, not only a mock that ignores the query string.
    fn execute_jq(filter: &str, json: &str) -> String {
        let mut child=Command::new("jq").arg("-r").arg(filter)
            .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped())
            .spawn().expect("jq must be installed on source-free CI runner");
        child.stdin.take().unwrap().write_all(json.as_bytes()).unwrap();
        let output=child.wait_with_output().unwrap();
        assert!(output.status.success(),"jq expression failed: {}",
                String::from_utf8_lossy(&output.stderr));
        String::from_utf8(output.stdout).unwrap().trim_end_matches('\n').to_string()
    }

    #[test] fn validates_real_jq_branch_protection_projection() {
        let policy_json=r#"{
          "required_status_checks":{"strict":true,
            "checks":[{"context":"test","app_id":15368}]},
          "enforce_admins":{"enabled":true},
          "required_pull_request_reviews":{
            "required_approving_review_count":2,
            "require_code_owner_reviews":true,
            "dismiss_stale_reviews":true,
            "require_last_push_approval":true,
            "bypass_pull_request_allowances":{"users":[],"teams":[],"apps":[]}
          },
          "required_conversation_resolution":{"enabled":true},
          "allow_force_pushes":{"enabled":false},
          "allow_deletions":{"enabled":false}
        }"#;
        let jq=PROTECTION_Q.replace("__CONTEXT__","test").replace("__APP_ID__","15368");
        assert_eq!(execute_jq(&jq,policy_json),
          "true\ttrue\t2\ttrue\ttrue\ttrue\ttrue\t1\t0\tfalse\tfalse");
        let wrong_app=PROTECTION_Q.replace("__CONTEXT__","test")
            .replace("__APP_ID__","99999");
        let projected=execute_jq(&wrong_app,policy_json);
        assert_eq!(projected.split('\t').nth(7),Some("0"));
    }

    #[test] fn validates_actual_jq_rest_projection_shapes() {
        let sha="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        assert_eq!(execute_jq(REPO_Q,r#"{"id":901,"private":true}"#),
            "901\ttrue");
        assert_eq!(execute_jq(BRANCH_Q,&format!(r#"{{"protected":true,
            "commit":{{"sha":"{sha}"}}}}"#)),format!("true\t{sha}"));
        assert_eq!(execute_jq(RUN_Q,&format!(r#"{{"id":400,
            "repository":{{"id":901}},"head_sha":"{sha}","workflow_id":200,
            "status":"completed","conclusion":"success","check_suite_id":350,
            "event":"push","head_branch":"main","run_attempt":1}}"#)),
            format!("400\t901\t{sha}\t200\tcompleted\tsuccess\t350\tpush\tmain\t1"));
        assert_eq!(execute_jq(JOB_Q,r#"{"id":500,"run_id":400,
            "runner_id":100,"status":"completed","conclusion":"success",
            "steps":[{"status":"completed","conclusion":"success"},
                     {"status":"completed","conclusion":"success"}],
            "check_run_url":"https://api.github.com/repos/trusted/source/check-runs/600"}"#),
            "500\t400\t100\tcompleted\tsuccess\t2\t2\thttps://api.github.com/repos/trusted/source/check-runs/600");
        assert_eq!(execute_jq(CHECK_Q,&format!(r#"{{"id":600,
            "app":{{"id":15368}},"head_sha":"{sha}","status":"completed",
            "conclusion":"success","check_suite":{{"id":350}},"name":"test"}}"#)),
            format!("600\t15368\t{sha}\tcompleted\tsuccess\t350\ttest"));
        assert_eq!(execute_jq(COMMIT_Q,&format!(r#"{{"sha":"{sha}",
            "tree":{{"sha":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}}}}"#)),
            format!("{sha}\tbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"));
    }

}
