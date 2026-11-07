//! Audit and narrowly update repository branch-review settings.
//! Deliberately does not bypass reviews, status checks, CODEOWNERS, or rulesets.
use serde_json::{json, Value};
use std::env;
use std::io::Write;
use std::process::{Command, Stdio};

const DEFAULT_ORG: &str = "ores-truffle-oreslang";

#[derive(Debug, PartialEq, Eq)]
struct Options {
    org: String,
    repo: Option<String>,
    apply: bool,
}

fn options_from<I: IntoIterator<Item = String>>(args: I) -> Result<Options, String> {
    let mut opts = Options { org: DEFAULT_ORG.into(), repo: None, apply: false };
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--org" => opts.org = args.next().ok_or("--org needs a value")?,
            "--repo" => opts.repo = Some(args.next().ok_or("--repo needs a value")?),
            "--apply" => opts.apply = true,
            "-h" | "--help" => return Err("usage: cargo run -- [--org ores-truffle-oreslang] [--repo REPO] [--apply]\nDry run is the default. An explicit --apply updates only the numeric branch-review approval threshold. Other gates are untouched.".into()),
            _ => return Err(format!("unknown argument: {arg}")),
        }
    }
    let valid = |s: &str| !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.');
    if !valid(&opts.org) || opts.repo.as_deref().is_some_and(|r| !valid(r)) {
        return Err("unsafe --org or --repo value".into());
    }
    Ok(opts)
}

fn gh(method: &str, endpoint: &str, body: Option<&Value>) -> Result<Value, String> {
    let mut cmd = Command::new("gh");
    cmd.arg("api").arg("--method").arg(method)
        .arg("-H").arg("Accept: application/vnd.github+json")
        .arg(endpoint).stdout(Stdio::piped()).stderr(Stdio::piped());
    if body.is_some() { cmd.arg("--input").arg("-").stdin(Stdio::piped()); }
    let mut child = cmd.spawn().map_err(|e| format!("failed to execute gh CLI: {e}"))?;
    if let Some(data) = body {
        child.stdin.take().ok_or("gh stdin unavailable")?
            .write_all(data.to_string().as_bytes()).map_err(|e| e.to_string())?;
    }
    let result = child.wait_with_output().map_err(|e| e.to_string())?;
    if !result.status.success() {
        return Err(String::from_utf8_lossy(&result.stderr).trim().to_string());
    }
    serde_json::from_slice(&result.stdout).map_err(|e| format!("invalid gh response for {endpoint}: {e}"))
}

fn get(endpoint: &str) -> Result<Value, String> { gh("GET", endpoint, None) }

fn list_repos(org: &str) -> Result<Vec<(String, String)>, String> {
    let mut found = Vec::new();
    for page in 1..=100 {
        let path = format!("orgs/{org}/repos?per_page=100&page={page}&type=all");
        let data = get(&path)?;
        let repos = data.as_array().ok_or("repo response was not an array")?;
        for r in repos {
            if r["archived"].as_bool() == Some(true) { continue; }
            let (Some(name), Some(branch)) = (r["name"].as_str(), r["default_branch"].as_str()) else {
                return Err(format!("repo entry has no name/default branch: {r}"));
            };
            found.push((name.into(), branch.into()));
        }
        if repos.len() < 100 { return Ok(found); }
    }
    Err("pagination exceeded 100 pages; refusing incomplete run".into())
}

fn review_count(protection: &Value) -> Option<u64> {
    protection.pointer("/required_pull_request_reviews/required_approving_review_count")
        .and_then(Value::as_u64)
}

fn review_patch(protection: &Value) -> Option<Value> {
    // A missing review rule cannot be safely created using the PATCH endpoint.
    // Changing the whole protection via PUT could erase unrelated protections.
    let old = review_count(protection)?;
    if old == 1 { return None; }
    // PATCH only the numeric threshold: leave stale-review, codeowner and last-push
    // restrictions as they are. Do not transmit bypass grants or dismissal restrictions.
    Some(json!({"required_approving_review_count": 1}))
}

fn repo_rulesets(org: &str, repo: &str) -> Result<Vec<Value>, String> {
    let data = get(&format!("repos/{org}/{repo}/rulesets?includes_parents=true"))?;
    Ok(data.as_array().ok_or("rulesets response was not an array")?.to_vec())
}

fn run(opts: &Options) -> Result<(), String> {
    let repos = if let Some(repo) = &opts.repo {
        let meta = get(&format!("repos/{}/{}", opts.org, repo))?;
        vec![(repo.clone(), meta["default_branch"].as_str().ok_or("missing default branch")?.into())]
    } else { list_repos(&opts.org)? };
    println!("mode={} org={} repos={}", if opts.apply { "APPLY" } else { "AUDIT" }, opts.org, repos.len());
    let mut errors = 0_u32;
    let mut changed = 0_u32;
    let mut manual = 0_u32;
    for (repo, branch) in repos {
        let stem = format!("repos/{}/{}/branches/{}/protection", opts.org, repo, branch);
        let rulesets = match repo_rulesets(&opts.org, &repo) {
            Ok(r) => r,
            Err(e) => { eprintln!("MANUAL {repo}: could not audit rulesets: {e}"); manual += 1; continue; }
        };
        let relevant: Vec<_> = rulesets.iter()
            .filter(|r| r["enforcement"].as_str() == Some("active"))
            .filter_map(|r| r["id"].as_u64())
            .collect();
        if !relevant.is_empty() {
            println!("MANUAL {repo}: active inherited/local ruleset ids={relevant:?}; inspect their pull_request rule before assuming one approval is sufficient");
            manual += 1;
            continue;
        }
        let current = match get(&stem) {
            Ok(p) => p,
            Err(e) => { println!("MANUAL {repo}: protection unavailable/unsupported (plan, permissions, or unprotected branch): {e}"); manual += 1; continue; }
        };
        if current.pointer("/required_pull_request_reviews/require_code_owner_reviews")
            .and_then(Value::as_bool) == Some(true) {
            println!("MANUAL {repo}: CODEOWNERS approval is additionally required; inspect owners before changing the numeric threshold");
            manual += 1;
            continue;
        }
        if review_count(&current).is_none() {
            println!("MANUAL {repo}: no numeric review rule; new protection requires a separately reviewed full-policy change");
            manual += 1;
            continue;
        }
        let Some(body) = review_patch(&current) else {
            println!("OK {repo}:{branch} required approvals=1 (other review and status gates unchanged)");
            continue;
        };
        let before = review_count(&current).map_or_else(|| "none".into(), |v| v.to_string());
        println!("{} {repo}:{branch} approvals {before} -> 1", if opts.apply { "APPLY" } else { "WOULD" });
        if !opts.apply { continue; }
        let path = format!("{stem}/required_pull_request_reviews");
        if let Err(e) = gh("PATCH", &path, Some(&body)) {
            eprintln!("ERROR {repo}: update rejected (other rules untouched): {e}");
            errors += 1;
            continue;
        }
        match get(&stem) {
            Ok(p) if review_count(&p) == Some(1) => {
                println!("VERIFIED {repo}:{branch} approvals=1");
                changed += 1;
            }
            Ok(p) => { eprintln!("ERROR {repo}: verification count={:?}", review_count(&p)); errors += 1; }
            Err(e) => { eprintln!("ERROR {repo}: verification request failed: {e}"); errors += 1; }
        }
    }
    println!("summary changed={changed} manual={manual} errors={errors}");
    if errors != 0 { Err(format!("{errors} updates failed")) } else { Ok(()) }
}

fn main() {
    let opts = match options_from(env::args().skip(1)) {
        Ok(o) => o,
        Err(e) => { eprintln!("{e}"); std::process::exit(2); }
    };
    if let Err(e) = run(&opts) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dry_run_default_and_apply_opt_in() {
        let defaults = options_from(Vec::<String>::new()).unwrap();
        assert!(!defaults.apply);
        assert_eq!(defaults.org, DEFAULT_ORG);
        assert!(options_from(vec!["--apply".to_owned()]).unwrap().apply);
    }

    #[test]
    fn rejects_malicious_api_path_components() {
        assert!(options_from(vec!["--repo".into(), "../../other".into()]).is_err());
        assert!(options_from(vec!["--org".into(), "evil?x=1".into()]).is_err());
    }

    #[test]
    fn reduces_only_numeric_threshold() {
        let original = json!({"required_pull_request_reviews": {
            "required_approving_review_count": 3,
            "dismiss_stale_reviews": true,
            "require_code_owner_reviews": false,
            "require_last_push_approval": true,
            "dismissal_restrictions": {"users": ["admin"]}
        }, "required_status_checks": {"strict": true, "contexts": ["ci"]}});
        let patch = review_patch(&original).unwrap();
        assert_eq!(patch, json!({"required_approving_review_count": 1}));
        assert_eq!(original["required_status_checks"]["contexts"][0], "ci");
    }

    #[test]
    fn one_approval_is_no_op() {
        assert!(review_patch(&json!({"required_pull_request_reviews": {
            "required_approving_review_count": 1
        }})).is_none());
    }

    #[test]
    fn refuses_to_modify_missing_review_requirement() {
        assert!(review_patch(&json!({"required_pull_request_reviews": null})).is_none());
    }
}
