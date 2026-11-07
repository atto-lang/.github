//! Single-organization, dry-run-first, independently verified review-policy CLI.
//! The JSON/API policy engine and its adversarial tests are in lib.rs.
use atto_merge_review_policy::{
    apply_planned_repo, audit_repo, parse_options, preflight_all, repositories, repository, usage,
    Api, Outcome, ORG,
};
use atto_merge_review_policy::inventory::verify_manifest;
use serde_json::Value;
use std::fs;
use std::env;
use std::io::Write;
use std::path::{Component, Path};
use std::process::{Command, Stdio};

struct CliApi {
    gh_bin: String,
}

/// Reject relative or ambiguous executable names before making privileged
/// GitHub organization API calls. Never use shell/PATH resolution for writes.
/// The administrator must independently trust the selected absolute binary.
fn valid_gh_bin(value: &str) -> bool {
    let path = Path::new(value);
    path.is_absolute()
        && path.file_name().and_then(|name| name.to_str()) == Some("gh")
        && !value.chars().any(char::is_control)
        && !value.split('/').any(|part| part == "." || part == "..")
        && !path.components().any(|part| matches!(part, Component::ParentDir | Component::CurDir))
}

impl CliApi {
    fn new() -> Result<Self, String> {
        // Never search PATH or invoke a shell while performing org-admin writes.
        // Support an explicitly trusted absolute Homebrew/macOS path.
        let gh_bin = match env::var("ATTO_GH_BIN") {
            Ok(value) => value,
            Err(env::VarError::NotPresent) => "/usr/bin/gh".into(),
            Err(env::VarError::NotUnicode(_)) => {
                return Err("ATTO_GH_BIN must be valid UTF-8".into());
            }
        };
        if !valid_gh_bin(&gh_bin) {
            return Err("ATTO_GH_BIN must be a trusted absolute path ending /gh without control characters or dot traversal".into());
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
    if raw.len() == 1 && matches!(raw[0].as_str(), "--help" | "-h") {
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
    // The connected repository inventory may omit inaccessible private repos.
    // The expected count is an independent operator-supplied upper-level guard,
    // never inferred from the same incomplete API response.
    if opts.all && opts.expected_repos != Some(repos.len()) {
        return Err(format!("ABORTED WITHOUT WRITES: discovered {} repos, but operator confirmed {:?}",
            repos.len(), opts.expected_repos));
    }
    if opts.apply && opts.all {
        // Cardinality alone cannot detect same-count substitutions. Require a
        // separately reviewed file enumerating every exact repo/default branch.
        // A missing, symlinked, malformed, or stale inventory is never a write.
        let inventory_path = env::var("ATTO_APPROVED_REPO_INVENTORY")
            .map_err(|_| "ABORTED WITHOUT WRITES: ATTO_APPROVED_REPO_INVENTORY is required for --all --apply")?;
        let inventory_path = Path::new(&inventory_path);
        if !inventory_path.is_absolute() {
            return Err("ABORTED WITHOUT WRITES: approved inventory path must be absolute".into());
        }
        let metadata = fs::symlink_metadata(inventory_path)
            .map_err(|_| "ABORTED WITHOUT WRITES: approved inventory file unavailable")?;
        if !metadata.file_type().is_file() {
            return Err("ABORTED WITHOUT WRITES: approved inventory must be a regular, non-symlinked file".into());
        }
        let manifest = fs::read_to_string(inventory_path)
            .map_err(|_| "ABORTED WITHOUT WRITES: cannot read approved inventory file")?;
        verify_manifest(&manifest, &repos)
            .map_err(|reason| format!("ABORTED WITHOUT WRITES: {reason}"))?;
        println!("APPROVED_INVENTORY_MATCH repos={} exact repository and branch identities", repos.len());
    }
    println!("org={ORG} mode={} accessible_nonarchived_repos={}", if opts.apply {
        "APPLY"
    } else {
        "DRY_RUN"
    }, repos.len());
    println!("CAUTION: numeric approval thresholds do not restrict reviewer identities; check CODEOWNERS or team policy separately.");
    // A bulk apply is an all-or-nothing *preflight*. No writes are allowed
    // until every repository is fully inspectable and policy-eligible.
    // Actual GitHub writes cannot be transactionally rolled back; stop on
    // the first failed apply and reconcile the partial batch explicitly.
    let plans = if opts.apply && opts.all {
        match preflight_all(&mut api, &repos) {
            Ok(plans) => {
                // Ensure the org inventory did not change during the potentially
                // long all-repository read-only preflight. Do not begin a partial
                // rollout to a stale list of default branches.
                let fresh_inventory = repositories(&mut api)?;
                if fresh_inventory != repos {
                    return Err("ABORTED WITHOUT WRITES: repository list or default branches changed during preflight".into());
                }
                println!("PREFLIGHT_OK repos={} snapshots and inventory revalidated", plans.len());
                Some(plans)
            }
            Err(reasons) => {
                for reason in &reasons { eprintln!("PREFLIGHT_BLOCKED {reason}"); }
                return Err(format!("ABORTED WITHOUT WRITES: {} repository preflight failures", reasons.len()));
            }
        }
    } else { None };
    let mut compliant = 0usize;
    let mut proposed = 0usize;
    let mut changed = 0usize;
    let mut manual = 0usize;
    let mut errors = 0usize;
    for (index, (name, branch)) in repos.into_iter().enumerate() {
        if opts.apply {
            // The organization may change a repo's default branch while a
            // bulk policy rollout is running. Never mutate the old default
            // just because it appeared in an earlier inventory snapshot.
            let live = repository(&mut api, &name)?;
            if live != (name.clone(), branch.clone()) {
                return Err(format!(
                    "ABORTED: default branch or repository metadata changed for {name}; no further writes issued"
                ));
            }
        }
        let outcome = if let Some(planned) = &plans {
            apply_planned_repo(&mut api, &planned[index])
        } else {
            audit_repo(&mut api, &name, &branch, opts.apply)
        };
        let bulk_failure = matches!(&outcome, Outcome::Manual(_) | Outcome::Error(_));
        match outcome {
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
        if opts.apply && opts.all && bulk_failure {
            return Err(format!("BULK APPLY HALTED after {changed} verified writes; reconcile any partial batch before resuming"));
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

#[cfg(test)]
mod tests {
    use super::valid_gh_bin;

    #[test]
    fn accepts_absolute_linux_and_macos_cli() {
        for value in ["/usr/bin/gh", "/usr/local/bin/gh", "/opt/homebrew/bin/gh"] {
            assert!(valid_gh_bin(value), "rejected {value}");
        }
    }

    #[test]
    fn rejects_path_lookup_and_shell_commands() {
        for value in ["", "gh", "./gh", "../gh", "gh --version", "/bin/sh",
                      "/usr/bin/gh\n--method PATCH", "/usr/bin/gh\0payload"] {
            assert!(!valid_gh_bin(value), "accepted {value:?}");
        }
    }

    #[test]
    fn rejects_ambiguous_dot_components_and_bad_final_names() {
        for value in ["/usr/./bin/gh", "/tmp/../usr/bin/gh", "/usr/bin/gh/../gh",
                      "/usr/bin/gh?method=PATCH", "/usr/bin/gh "] {
            assert!(!valid_gh_bin(value), "accepted {value:?}");
        }
    }
}
