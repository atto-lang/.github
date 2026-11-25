//! GitHub CLI adapter. With --apply, this program mutates only the specified
//! AttoLang PR's Draft state and a narrow commit-status context. It NEVER
//! approves, merges, or alters labels. Invocation must come from a trusted
//! repository-local workflow/app, not code checked out from an untrusted PR.
use atto_draft_hold::{parse_snapshot, validate_repo, Decision, Snapshot, PR_JQ, STATUS_CONTEXT};
use std::env;
use std::process::Command;

#[derive(Debug)]
struct Args {
    repo: String,
    pr: u64,
    apply: bool,
}

fn parse_args() -> Result<Args, String> {
    let mut repo = None;
    let mut pr = None;
    let mut apply = false;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--repo" if repo.is_none() => repo = args.next(),
            "--pr" if pr.is_none() => pr = args.next(),
            "--apply" if !apply => apply = true,
            "--help" | "-h" => {
                println!("Usage: atto-draft-hold --repo atto-lang/REPO --pr NUMBER [--apply]");
                println!(
                    "By default, read-only. --apply also requires protected workflow activation."
                );
                std::process::exit(0);
            }
            _ => return Err("invalid or duplicate CLI option".to_owned()),
        }
    }
    let repo = repo.ok_or("missing --repo")?;
    validate_repo(&repo).map_err(str::to_owned)?;
    let pr = pr
        .ok_or("missing --pr")?
        .parse::<u64>()
        .map_err(|_| "invalid PR number")?;
    if pr == 0 {
        return Err("zero PR number".to_owned());
    }
    Ok(Args { repo, pr, apply })
}

fn gh(args: &[&str]) -> Result<String, String> {
    let output = Command::new("gh")
        .args(args)
        .output()
        .map_err(|_| "cannot execute GitHub CLI")?;
    if !output.status.success() {
        // Never forward raw CLI stderr, remote response bodies, or token data.
        return Err(format!(
            "GitHub operation failed (exit {:?})",
            output.status.code()
        ));
    }
    String::from_utf8(output.stdout).map_err(|_| "GitHub returned non-UTF8 data".to_owned())
}

fn fetch(repo: &str, number: u64) -> Result<Snapshot, String> {
    let path = format!("repos/{repo}/pulls/{number}");
    let output = gh(&["api", &path, "--jq", PR_JQ])?;
    parse_snapshot(&output, repo, number).map_err(str::to_owned)
}

fn commit_status(repo: &str, sha: &str, state: &str, description: &str) -> Result<(), String> {
    if !matches!(state, "success" | "failure" | "pending") {
        return Err("invalid commit state".to_owned());
    }
    let path = format!("repos/{repo}/statuses/{sha}");
    let args = [
        "api",
        "--method",
        "POST",
        &path,
        "-f",
        &format!("state={state}"),
        "-f",
        &format!("context={STATUS_CONTEXT}"),
        "-f",
        &format!("description={description}"),
    ];
    gh(&args)?;
    Ok(())
}

fn restore_draft(repo: &str, number: u64) -> Result<(), String> {
    let number = number.to_string();
    gh(&["pr", "ready", &number, "--undo", "--repo", repo])?;
    Ok(())
}

fn allowed_to_apply(repo: &str) -> Result<(), String> {
    // Neither GITHUB_TOKEN nor --apply alone grants the actor policy
    // authority. Privileged dispatchers must opt in explicitly.
    if env::var("ATTO_DRAFT_HOLD_APPLY").as_deref() != Ok("1") {
        return Err("missing explicit trusted-controller activation".to_owned());
    }
    if env::var("GITHUB_REPOSITORY").as_deref() != Ok(repo) {
        return Err("token repository scope mismatch".to_owned());
    }
    let event = env::var("GITHUB_EVENT_NAME").unwrap_or_default();
    if !matches!(event.as_str(), "pull_request_target" | "workflow_dispatch") {
        return Err("unapproved GitHub event for write mode".to_owned());
    }
    if env::var_os("GH_TOKEN").is_none() {
        return Err("missing GH_TOKEN for authenticated GitHub operations".to_owned());
    }
    Ok(())
}

fn run() -> Result<(), String> {
    let args = parse_args()?;
    if args.apply {
        allowed_to_apply(&args.repo)?;
    }
    let first = fetch(&args.repo, args.pr)?;
    let decision = first.decision();
    if !args.apply {
        println!(
            "READ_ONLY repo={} pr={} head={} decision={decision}",
            first.repository, first.number, first.head_sha
        );
        return Ok(());
    }

    // Fail or pend the exact verified PR head before any potentially
    // destructive operation. Never authorize a merge or set a full-gate pass.
    let initial_state = if decision == Decision::ClearOfDraftHolds {
        "pending"
    } else {
        "failure"
    };
    commit_status(
        &args.repo,
        &first.head_sha,
        initial_state,
        "AttoLang draft-hold controller revalidating current PR state",
    )?;
    let second = fetch(&args.repo, args.pr)?;
    if !first.still_same(&second) {
        return Err("PR changed during guard check; pending/failure retained".to_owned());
    }

    match decision {
        Decision::RestoreDraft => {
            // Recheck immediately before conversion. There is no GitHub
            // atomic labels+draft mutation; a trusted required check plus
            // periodic reconciliation is required for stronger safety.
            let just_before = fetch(&args.repo, args.pr)?;
            if !second.still_same(&just_before)
                || !just_before.persistent_hold()
                || just_before.draft
                || just_before.state != "open"
            {
                return Err("PR changed before Draft restoration; fail closed".to_owned());
            }
            restore_draft(&args.repo, args.pr)?;
            let after = fetch(&args.repo, args.pr)?;
            if after.head_sha != first.head_sha || !after.draft {
                return Err("cannot verify Draft restoration on same head".to_owned());
            }
            // Hold removal during remediation does not retroactively permit
            // a merge: a new ready transition must start a new review cycle.
            commit_status(
                &args.repo,
                &after.head_sha,
                "failure",
                "Persistent Draft hold: merge denied",
            )?;
            println!(
                "DRAFT_RESTORED repo={} pr={} head={}",
                args.repo, args.pr, after.head_sha
            );
        }
        Decision::Blocked => {
            commit_status(
                &args.repo,
                &first.head_sha,
                "failure",
                "Draft, closed PR, or blocking governance label",
            )?;
            println!(
                "HELD repo={} pr={} head={}",
                args.repo, args.pr, first.head_sha
            );
        }
        Decision::ClearOfDraftHolds => {
            let just_before = fetch(&args.repo, args.pr)?;
            if !second.still_same(&just_before) {
                return Err("PR changed before status success; pending retained".to_owned());
            }
            commit_status(
                &args.repo,
                &first.head_sha,
                "success",
                "No Draft hold; NOT full merge approval",
            )?;
            let after = fetch(&args.repo, args.pr)?;
            if !first.still_same(&after) {
                // Current-head/same-head label changes may occur during
                // the API call; fail our issuer context instead of trusting
                // a stale success. Schedule periodic reconciliation too.
                commit_status(
                    &args.repo,
                    &first.head_sha,
                    "failure",
                    "PR changed during draft-hold revalidation",
                )?;
                return Err("concurrent PR transition after posting status".to_owned());
            }
            println!(
                "DRAFT_HOLD_CLEAR_ONLY repo={} pr={} head={}",
                args.repo, args.pr, first.head_sha
            );
        }
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("FAIL_CLOSED: {error}");
        std::process::exit(1);
    }
}
