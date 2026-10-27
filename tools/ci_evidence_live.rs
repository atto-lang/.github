//! Read-only GitHub Actions metadata collector; Rust/std only.
//! This is NOT a release attestation: executable native tests, immutable
//! checkout provenance, effective review/ruleset checks are separate gates.
//! Run only in a protected, trusted, source-local workflow. Never with PR code.
#[cfg(test)]
use std::collections::BTreeMap;
#[cfg(not(test))]
use std::{env, process::Command};

trait ReadOnlyApi {
    fn get_tsv(&self, path: &str, query: &str) -> Result<String, String>;
}

#[cfg(not(test))]
struct GhApi;
#[cfg(not(test))]
impl ReadOnlyApi for GhApi {
    fn get_tsv(&self, path: &str, query: &str) -> Result<String, String> {
        let result = Command::new("gh")
            .args(["api", "--method", "GET", "-H",
                   "Accept: application/vnd.github+json", path, "--jq", query])
            .output()
            .map_err(|_| "github cli not available".to_string())?;
        if !result.status.success() {
            // Never echo stderr: upstream errors might contain private details.
            return Err("authenticated read-only GitHub API request failed".into());
        }
        let raw = String::from_utf8(result.stdout)
            .map_err(|_| "API result is not valid UTF-8".to_string())?;
        if raw.len() > 8192 || raw.contains('\0') {
            return Err("API scalar response too large or malformed".into());
        }
        Ok(raw.trim_end_matches('\n').to_string())
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
}

const REPO_Q: &str = r#"[.id, .private]|@tsv"#;
const BRANCH_Q: &str = r#"[.protected, .commit.sha]|@tsv"#;
const RUN_Q: &str = r#"[.id, .repository.id, .head_sha, .workflow_id, .status, .conclusion, .check_suite_id, .event]|@tsv"#;
const JOB_Q: &str = r#"[.id, .run_id, (.runner_id // 0), .status, .conclusion, ((.steps // [])|length), ((.steps // [])|map(select(.status == "completed" and .conclusion == "success"))|length), .check_run_url]|@tsv"#;
const CHECK_Q: &str = r#"[.id, .app.id, .head_sha, .status, .conclusion, .check_suite.id]|@tsv"#;
const COMMIT_Q: &str = r#"[.sha, .tree.sha]|@tsv"#;

fn audit(api: &impl ReadOnlyApi, e: &Expected<'_>) -> Result<(), String> {
    if !repo_slug(e.repo) || e.branch != "main" || !oid(e.commit) || !oid(e.tree) {
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

    let run = api.get_tsv(&format!("{root}/actions/runs/{}",e.run_id),RUN_Q)?;
    let run = fields(&run,8)?;
    expect_num(run[0],e.run_id,"wrong run")?;
    expect_num(run[1],repo_id,"cross-repository workflow run")?;
    expect_eq(run[2],e.commit,"stale run head")?;
    expect_num(run[3],e.workflow_id,"untrusted workflow ID")?;
    if run[4]!="completed" || run[5]!="success" || run[7]!="push" {
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
    let check = fields(&check,6)?;
    expect_num(check[0],e.check_id,"wrong check")?;
    expect_num(check[1],e.issuer_app_id,"untrusted check issuer")?;
    expect_eq(check[2],e.commit,"stale check")?;
    if check[3]!="completed" || check[4]!="success" {
        return Err("check incomplete or failed".into());
    }
    expect_num(check[5],suite_id,"check not linked to run check-suite")?;

    // The Git object tree is fetched via authenticated API, but this alone
    // does NOT attest which filesystem tree a runner actually built.
    let commit = api.get_tsv(&format!("{root}/git/commits/{}",e.commit),COMMIT_Q)?;
    let commit = fields(&commit,2)?;
    expect_eq(commit[0],e.commit,"wrong Git commit object")?;
    expect_eq(commit[1],e.tree,"wrong Git commit root tree")?;
    Ok(())
}

#[cfg(not(test))]
fn main() {
    // Never perform privileged API reads on untrusted PR refs or outside a
    // protected Actions job. GITHUB_TOKEN must be read-only; not printed.
    if env::var("GITHUB_ACTIONS").as_deref()!=Ok("true")
        || env::var("GITHUB_REF_PROTECTED").as_deref()!=Ok("true")
        || (env::var_os("GH_TOKEN").is_none() && env::var_os("GITHUB_TOKEN").is_none())
    {
        eprintln!("NOT_CERTIFIED: protected authenticated source-local job required");
        std::process::exit(3);
    }
    let args:Vec<_> = env::args().skip(1).collect();
    if args.len()!=9 {
        eprintln!("Usage: collector <owner/repo> main <commit> <tree> <run_id> <job_id> <check_id> <workflow_id> <issuer_app_id>");
        std::process::exit(2);
    }
    let parsed = || -> Result<Expected<'_>,String> {
        Ok(Expected {
            repo:&args[0],branch:&args[1],commit:&args[2],tree:&args[3],
            run_id:numeric(&args[4])?,job_id:numeric(&args[5])?,
            check_id:numeric(&args[6])?,workflow_id:numeric(&args[7])?,
            issuer_app_id:numeric(&args[8])?,
        })
    };
    if env::var("GITHUB_REPOSITORY").ok().as_deref()!=Some(args[0].as_str()) {
        eprintln!("NOT_CERTIFIED: foreign source repository");
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
        m.insert(format!("{root}/actions/runs/400"),format!("400\t901\t{sha}\t200\tcompleted\tsuccess\t350\tpush"));
        m.insert(format!("{root}/actions/jobs/500"),
          format!("500\t400\t100\tcompleted\tsuccess\t3\t3\thttps://api.github.com/repos/{repo}/check-runs/600"));
        m.insert(format!("{root}/check-runs/600"),format!("600\t15368\t{sha}\tcompleted\tsuccess\t350"));
        m.insert(format!("{root}/git/commits/{sha}"),format!("{sha}\t{tree}"));
        (MockApi(m),Expected {
            repo,branch:"main",commit:sha,tree,run_id:400,job_id:500,
            check_id:600,workflow_id:200,issuer_app_id:15368
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
          format!("400\t902\t{}\t200\tcompleted\tsuccess\t350\tpush",e.commit));
        assert!(audit(&a,&e).is_err());
    }
    #[test] fn rejects_stale_run_and_incomplete_run() {
        let (mut a,e)=fixture();
        a.0.insert("repos/trusted/source/actions/runs/400".into(),
          "400\t901\tcccccccccccccccccccccccccccccccccccccccc\t200\tcompleted\tsuccess\t350\tpush".into());
        assert!(audit(&a,&e).is_err());
        a.0.insert("repos/trusted/source/actions/runs/400".into(),
          format!("400\t901\t{}\t200\tqueued\tsuccess\t350\tpush",e.commit));
        assert!(audit(&a,&e).is_err());
    }
    #[test] fn rejects_untrusted_workflow_id() {
        let (mut a,e)=fixture();
        a.0.insert("repos/trusted/source/actions/runs/400".into(),
          format!("400\t901\t{}\t201\tcompleted\tsuccess\t350\tpush",e.commit));
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
        a.0.insert(p.into(),format!("600\t42\t{}\tcompleted\tsuccess\t350",e.commit));
        assert!(audit(&a,&e).is_err());
        a.0.insert(p.into(),format!("600\t15368\t{}\tcompleted\tsuccess\t351",e.commit));
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
    #[test] fn rejects_unsafe_repo_names_and_ids() {
        let (a,mut e)=fixture();e.repo="trusted/../../other";
        assert!(audit(&a,&e).is_err());
        e.repo="trusted/source";e.job_id=0;assert!(audit(&a,&e).is_err());
    }
}
