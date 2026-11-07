//! Single-organization, dry-run-first, independently verified review-policy CLI.
//! The JSON/API policy engine and its adversarial tests are in lib.rs.
use atto_merge_review_policy::{
    audit_repo, parse_options, repositories, repository, usage, Api, Outcome, ORG,
};
use serde_json::Value;
use std::env;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

struct CliApi {
    gh_bin: String,
}

impl CliApi {
    fn new() -> Result<Self, String> {
        // Never search PATH or invoke a shell while performing org-admin writes.
        // Support explicit full path on Homebrew/macOS or elsewhere.
        let gh_bin = env::var("ATTO_GH_BIN").unwrap_or_else(|_| "/usr/bin/gh".into());
        let path = Path::new(&gh_bin);
        if !path.is_absolute() || path.file_name().and_then(|s| s.to_str()) != Some("gh") {
            return Err("ATTO_GH_BIN must be an absolute path to an executable named gh".into());
        }
        Ok(Self { gh_bin })
    }
}

impl Api for CliApi {
    fn request(&mut self, method: &str, endpoint: &str, body: Option<Value>) -> Result<Value, String> {
        if !matches!((method, body.is_some()), ("GET", false) | ("PATCH", true)) {
            return Err("unsafe or unsupported GitHub API method/body combination".into());
        }
        let mut cmd = Command::new(&self.gh_bin);
        cmd.arg("api").arg("--hostname").arg("github.com")
            .arg("--method").arg(method)
            .arg("-H").arg("Accept: application/vnd.github+json")
            .arg("-H").arg("X-GitHub-Api-Version: 2022-11-28")
            .arg(endpoint)
            .env("GH_HOST", "github.com")
            .stdout(Stdio::piped()).stderr(Stdio::piped());
        if body.is_some() { cmd.arg("--input").arg("-").stdin(Stdio::piped()); }
        let mut child = cmd.spawn().map_err(|e| format!("cannot run gh CLI: {e}"))?;
        if let Some(data) = body {
            child.stdin.take().ok_or("GitHub CLI stdin unavailable")?
                .write_all(data.to_string().as_bytes())
                .map_err(|e| format!("failed to submit JSON body: {e}"))?;
        }
        let response = child.wait_with_output()
            .map_err(|e| format!("GitHub CLI process failed: {e}"))?;
        if !response.status.success() {
            // Avoid exposing private API response bodies or credential-bearing stderr.
            return Err(format!("gh api {method} request failed (status {}) on {endpoint}", response.status));
        }
        serde_json::from_slice(&response.stdout)
            .map_err(|e| format!("invalid JSON returned for {endpoint}: {e}"))
    }
}

fn run() -> Result<(), String> {
    let raw: Vec<String> = env::args().skip(1).collect();
    if raw == ["--help"] || raw == ["-h"] {
        println!("{}", usage());
        return Ok(());
    }
    let opts = parse_options(raw)?;
    let mut api = CliApi::new()?;
    let repos = if let Some(name) = &opts.repo {
        vec![repository(&mut api, name)?]
    } else {
        repositories(&mut api)?
    };
    println!("org={ORG} mode={} accessible_nonarchived_repos={}", if opts.apply {
        "APPLY"
    } else {
        "DRY_RUN"
    }, repos.len());
    println!("CAUTION: numeric approval thresholds do not restrict reviewer identities; check CODEOWNERS or team policy separately.");
    let mut compliant = 0usize;
    let mut proposed = 0usize;
    let mut changed = 0usize;
    let mut manual = 0usize;
    let mut errors = 0usize;
    for (name, branch) in repos {
        match audit_repo(&mut api, &name, &branch, opts.apply) {
            Outcome::Compliant => { compliant += 1; println!("COMPLIANT {name}:{branch} numeric_reviews=1"); }
            Outcome::WouldChange(old) => {
                proposed += 1;
                println!("WOULD_CHANGE {name}:{branch} numeric_reviews={old}->1");
            }
            Outcome::Changed(old) => {
                changed += 1;
                println!("VERIFIED {name}:{branch} numeric_reviews={old}->1; full protection readback equal except count");
            }
            Outcome::Manual(reason) => {
                manual += 1;
                println!("MANUAL {name}:{branch} {reason}");
            }
            Outcome::Error(reason) => {
                errors += 1;
                eprintln!("ERROR {name}:{branch} {reason}");
            }
        }
    }
    println!("SUMMARY compliant={compliant} proposed={proposed} verified_changes={changed} manual={manual} errors={errors}");
    if errors > 0 || manual > 0 {
        return Err(format!("INCOMPLETE: {errors} errors and {manual} manual reviews; do not claim org-wide policy parity"));
    }
    Ok(())
}

fn main() {
    if let Err(message) = run() {
        eprintln!("{message}");
        std::process::exit(1);
    }
}
