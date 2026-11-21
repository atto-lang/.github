//! Read-only GitHub API collector and conservative ready-cycle audit.
//! It MUST NOT produce a merge-eligible status until a trusted push event store
//! and independently verified OpenAI/Anthropic issuer bindings are installed.
//! All remote reads use authenticated gh CLI; no PR head code is executed.
use atto_draft_hold::review_cycle::{
    assess, AiCheck, Evidence, HumanReview, Provider, ReviewState, Transition, TransitionKind,
};
use atto_draft_hold::{parse_snapshot, validate_repo, PR_JQ};
use std::env;
use std::process::Command;

// Head, label state and updated_at must be stable across every paginated read.
const META_JQ: &str = r#"[.base.repo.full_name,.number,.user.id,.created_at,.head.sha,.draft,.updated_at,.base.sha] | @tsv"#;
const TIMELINE_JQ: &str = r#".[] | select(.event == "ready_for_review" or .event == "converted_to_draft") | [.id,.created_at,.event] | @tsv"#;
const REVIEWS_JQ: &str =
    r#".[] | [.id,.user.login,.user.id,.state,.submitted_at,.commit_id] | @tsv"#;
const CHECKS_JQ: &str = r#".check_runs[] | select(.name == "atto-ai/openai/review-attested" or .name == "atto-ai/anthropic/review-attested") | [.id,.app.id,.name,.head_sha,.completed_at,.conclusion] | @tsv"#;

fn gh(parts: &[&str]) -> Result<String, String> {
    let result = Command::new("gh")
        .args(parts)
        .output()
        .map_err(|_| "GitHub CLI unavailable".to_string())?;
    if !result.status.success() {
        return Err("authenticated GitHub read failed; no merge evidence".to_owned());
    }
    String::from_utf8(result.stdout).map_err(|_| "non-UTF8 GitHub response".into())
}

fn rows(s: &str, columns: usize) -> Result<Vec<Vec<&str>>, String> {
    let mut output = Vec::new();
    for line in s.lines() {
        if line.contains('\r') {
            return Err("carriage return in API projection".into());
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != columns {
            return Err("malformed API projection".into());
        }
        output.push(fields);
    }
    Ok(output)
}
fn is_sha(s: &str) -> bool {
    s.len() == 40
        && s.bytes()
            .all(|x| x.is_ascii_digit() || (b'a'..=b'f').contains(&x))
}
fn positive(s: &str) -> Result<u64, String> {
    let number = s
        .parse::<u64>()
        .map_err(|_| "invalid API numeric identity")?;
    if number == 0 {
        return Err("zero API numeric identity".into());
    }
    Ok(number)
}

fn collect(repo: &str, pr: u64) -> Result<Evidence, String> {
    validate_repo(repo).map_err(str::to_owned)?;
    if pr == 0 {
        return Err("missing PR number".into());
    }
    let address = format!("repos/{repo}/pulls/{pr}");
    let state =
        parse_snapshot(&gh(&["api", &address, "--jq", PR_JQ])?, repo, pr).map_err(str::to_owned)?;
    let meta_text = gh(&["api", &address, "--jq", META_JQ])?;
    let meta = rows(&meta_text, 8)?;
    if meta.len() != 1 || meta[0][0] != repo || positive(meta[0][1])? != pr {
        return Err("PR metadata identity changed".into());
    }
    // Reject unrelated/moving PR head, Draft change and invalid metadata before
    // walking any pages. Also reject missing update watermark or base identity.
    let head = &meta[0][4];
    let draft = if state.draft { "true" } else { "false" };
    if *head != state.head_sha
        || meta[0][5] != draft
        || atto_draft_hold::review_cycle::utc_seconds(meta[0][6]).is_none()
        || !is_sha(meta[0][7])
    {
        return Err("live PR metadata differs from immutable starting snapshot".into());
    }
    let author = positive(meta[0][2])?;
    let creation = meta[0][3].to_string();

    let timeline_address = format!("repos/{repo}/issues/{pr}/timeline?per_page=100");
    let timeline_text = gh(&["api", "--paginate", &timeline_address, "--jq", TIMELINE_JQ])?;
    let timeline = rows(&timeline_text, 3)?;
    let mut transitions = Vec::new();
    for row in &timeline {
        let kind = match row[2] {
            "ready_for_review" => TransitionKind::Ready,
            "converted_to_draft" => TransitionKind::Draft,
            _ => return Err("unknown lifecycle event".into()),
        };
        transitions.push(Transition {
            id: positive(row[0])?,
            at: row[1].to_owned(),
            kind,
        });
    }
    // With a *complete* timeline, first transition type determines the
    // opening state. Without transitions, current draft equals opening state.
    let created_as_draft = transitions
        .first()
        .map_or(state.draft, |first| first.kind == TransitionKind::Ready);
    let reviews_address = format!("repos/{repo}/pulls/{pr}/reviews?per_page=100");
    let review_text = gh(&["api", "--paginate", &reviews_address, "--jq", REVIEWS_JQ])?;
    let reviews = rows(&review_text, 6)?;
    let mut human_reviews = Vec::new();
    for row in &reviews {
        // GitHub COMMENTED events are retained, but never approvals.
        let state = match row[3] {
            "APPROVED" => ReviewState::Approved,
            "DISMISSED" => ReviewState::Dismissed,
            "CHANGES_REQUESTED" => ReviewState::ChangesRequested,
            "COMMENTED" => ReviewState::Commented,
            _ => return Err("unknown GitHub review state".into()),
        };
        human_reviews.push(HumanReview {
            id: positive(row[0])?,
            login: row[1].to_owned(),
            github_id: positive(row[2])?,
            state,
            submitted_at: row[4].to_owned(),
            head_sha: row[5].to_owned(),
        });
    }

    let checks_address = format!(
        "repos/{repo}/commits/{}/check-runs?per_page=100",
        state.head_sha
    );
    let check_text = gh(&["api", "--paginate", &checks_address, "--jq", CHECKS_JQ])?;
    let checks = rows(&check_text, 6)?;
    let mut ai_checks = Vec::new();
    for row in &checks {
        let provider = match row[2] {
            "atto-ai/openai/review-attested" => Provider::OpenAi,
            "atto-ai/anthropic/review-attested" => Provider::Anthropic,
            _ => return Err("unknown AI check-run name".into()),
        };
        ai_checks.push(AiCheck {
            run_id: positive(row[0])?,
            app_id: positive(row[1])?,
            provider,
            name: row[2].to_owned(),
            head_sha: row[3].to_owned(),
            completed_at: row[4].to_owned(),
            successful: row[5] == "success",
        });
    }
    let current = Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .output()
        .map_err(|_| "UTC clock unavailable")?;
    if !current.status.success() {
        return Err("UTC clock failed".into());
    }
    let checked = String::from_utf8(current.stdout)
        .map_err(|_| "invalid UTC clock output")?
        .trim()
        .to_owned();
    let mut labels = Vec::new();
    if state.keep_as_draft {
        labels.push("keep-as-draft".into());
    }
    if state.keep_draft {
        labels.push("keep-draft".into());
    }
    if state.not_ready {
        labels.push("not-ready".into());
    }

    // Read the same live PR again after all paginated evidence API calls. A
    // stable SHA alone is NOT sufficient: label changes, Draft transitions,
    // base retargets and review modifications can occur on the same commit.
    let final_state =
        parse_snapshot(&gh(&["api", &address, "--jq", PR_JQ])?, repo, pr).map_err(str::to_owned)?;
    let final_meta_text = gh(&["api", &address, "--jq", META_JQ])?;
    let final_meta = rows(&final_meta_text, 8)?;
    if final_state != state || final_meta.len() != 1 || final_meta[0] != meta[0] {
        return Err("live PR changed while collecting evidence; fail closed".into());
    }

    // SAFETY: GitHub timeline commit records have no authenticated last-push
    // timestamp, and arbitrary repo check runs do not establish AI provider
    // ownership. Leave both unverified until a protected installation provides
    // independently recorded push webhook receipts and approved issuer IDs.
    Ok(Evidence {
        head_sha: state.head_sha.clone(),
        open: state.state == "open",
        draft: state.draft,
        created_at: creation,
        created_as_draft,
        lifecycle_verified: true,
        transitions_complete: true,
        transitions,
        checked_at: checked,
        last_push_at: "1970-01-01T00:00:00Z".into(),
        push_verified: false,
        push_head_sha: state.head_sha,
        author_github_id: author,
        labels_verified: state.labels_verified,
        labels,
        reviews_complete: true,
        human_reviews,
        ai_checks_complete: true,
        ai_checks,
        ai_bindings: None,
    })
}

fn main() {
    let mut args = env::args().skip(1);
    let result = match (
        args.next(),
        args.next(),
        args.next(),
        args.next(),
        args.next(),
    ) {
        (Some(k1), Some(repo), Some(k2), Some(n), None) if k1 == "--repo" && k2 == "--pr" => n
            .parse::<u64>()
            .map_err(|_| "invalid PR number".into())
            .and_then(|number| collect(&repo, number)),
        _ => Err("usage: atto-ready-audit --repo atto-lang/REPO --pr NUMBER".into()),
    };
    match result {
        Ok(e) => {
            let out = assess(&e);
            println!("READ_ONLY_REVIEW_AUDIT state={} lane={:?} ready_cycle_started={:?} issues={:?} authorized={}",
                out.status,out.lane,out.ready_cycle_started_at,out.failures,out.merge_authorized);
            if out.status == "HOLD" {
                std::process::exit(1);
            }
            // Deliberately never exit success as authority to merge even
            // if a future collector supplies complete authenticated evidence.
            std::process::exit(2);
        }
        Err(reason) => {
            eprintln!("HOLD: {reason}");
            std::process::exit(1);
        }
    }
}
