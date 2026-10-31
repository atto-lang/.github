//! Read-only, source-free GitHub Actions admission diagnostics.
//!
//! Does not fetch repository contents, secrets, artifacts, or change settings.
//! Annotation text is potentially sensitive: review locally before sharing.
//! Usage: gh-run-admission OWNER/REPO RUN_ID JOB_ID
use std::env;
use std::process::{Command, ExitCode};

const ALLOWED_REPOS: [&str; 2] = [
    "ores-truffle-oreslang/oreslang-source.llvm",
    "ores-truffle-oreslang/oreslang-source.net",
];

#[derive(Debug, PartialEq, Eq)]
struct Args {
    repository: String,
    run_id: String,
    job_id: String,
}

fn canonical_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 19
        && value.as_bytes()[0].is_ascii_digit()
        && value.as_bytes()[0] != b'0'
        && value.bytes().all(|b| b.is_ascii_digit())
        && value.parse::<u64>().is_ok()
}

fn parse(args: &[String]) -> Result<Args, &'static str> {
    if args.len() != 4 {
        return Err("usage: gh-run-admission OWNER/REPO RUN_ID JOB_ID");
    }
    let repo = &args[1];
    if !ALLOWED_REPOS.contains(&repo.as_str()) {
        return Err("repository is not one of the two audited Oreslang source backends");
    }
    if !canonical_id(&args[2]) || !canonical_id(&args[3]) {
        return Err("run_id and job_id must be canonical positive decimal u64 values");
    }
    Ok(Args {
        repository: repo.to_owned(),
        run_id: args[2].clone(),
        job_id: args[3].clone(),
    })
}

fn endpoint(parts: &Args, part: &str) -> String {
    let prefix = format!("repos/{}/", parts.repository);
    match part {
        "run" => format!("{prefix}actions/runs/{}", parts.run_id),
        "job" => format!("{prefix}actions/jobs/{}", parts.job_id),
        "annotations" => format!("{prefix}check-runs/{}/annotations?per_page=100", parts.job_id),
        "branch" => format!("{prefix}branches/main"),
        "rulesets" => format!("{prefix}rulesets?per_page=100"),
        "permissions" => format!("{prefix}actions/permissions"),
        _ => unreachable!("only compile-time named endpoints are allowed"),
    }
}

fn projection(part: &str) -> &'static str {
    match part {
        "run" => r#"{id,run_attempt,head_sha,event,status,conclusion,created_at,updated_at}"#,
        "job" => r#"{id,run_id,name,status,conclusion,runner_id,runner_name,started_at,completed_at,step_count:((.steps // [])|length)}"#,
        "annotations" => r#"[.[]|{level:.annotation_level,title,message,path,start_line,end_line}]"#,
        "branch" => r#"{name,protected,sha:.commit.sha}"#,
        "rulesets" => r#"[.[]|{id,name,enforcement,source_type,source}]"#,
        "permissions" => r#"{enabled,allowed_actions,selected_actions_url}"#,
        _ => unreachable!("only compile-time named projections are allowed"),
    }
}

fn gh_read(parts: &Args, part: &str) -> Result<String, String> {
    let path = endpoint(parts, part);
    let result = Command::new("gh")
        .args(["api", "--method", "GET", &path, "--jq", projection(part)])
        .output()
        .map_err(|_| "GitHub CLI unavailable (install gh and authenticate)".to_owned())?;
    if !result.status.success() {
        // No credential values or unfiltered stderr are printed.
        return Err(format!(
            "GitHub API refused read-only {} request (exit {:?}); inspect access/plan in the UI",
            part, result.status.code()
        ));
    }
    let output = String::from_utf8(result.stdout)
        .map_err(|_| format!("{part}: non-UTF8 API output"))?;
    if output.len() > 64 * 1024 {
        return Err(format!("{part}: diagnostic output exceeded 64 KiB limit"));
    }
    Ok(output)
}

fn main() -> ExitCode {
    let argv: Vec<_> = env::args().collect();
    let args = match parse(&argv) {
        Ok(args) => args,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::from(2);
        }
    };
    println!("SOURCE_FREE_READ_ONLY_ADMISSION_REPORT");
    println!("repo={} run={} job={}", args.repository, args.run_id, args.job_id);
    // Check job.run_id against the supplied run_id when reviewing results.
    // Never assume a zero-step failure is a compile/test failure.
    let mut essential_ok = true;
    for part in ["run", "job", "annotations", "branch", "rulesets", "permissions"] {
        println!("--- {part} ---");
        match gh_read(&args, part) {
            Ok(data) => print!("{data}"),
            Err(message) => {
                println!("UNAVAILABLE: {message}");
                if part == "run" || part == "job" || part == "annotations" {
                    essential_ok = false;
                }
            }
        }
    }
    eprintln!("Annotation messages may be sensitive: review before copying to any issue.");
    if essential_ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(repo: &str, run: &str, job: &str) -> Vec<String> {
        vec!["tool".into(), repo.into(), run.into(), job.into()]
    }
    #[test]
    fn accepts_pinned_audit_repos() {
        for repo in ALLOWED_REPOS {
            assert_eq!(parse(&argv(repo, "41236569150", "124641494087")).unwrap().repository, repo);
        }
    }
    #[test]
    fn rejects_empty_and_zero_ids() {
        for bad in ["", "0", "00", "01", "-1", "+1", "1 ", "1/2", "1?x", "1\n2"] {
            assert!(!canonical_id(bad), "accepted {bad:?}");
        }
    }
    #[test]
    fn rejects_overflow_and_unbounded_ids() {
        for bad in ["18446744073709551616", "99999999999999999999999"] {
            assert!(!canonical_id(bad));
        }
    }
    #[test]
    fn accepts_canonical_positive_id() {
        for good in ["1", "123", "41236569150", "9999999999999999999"] {
            assert!(canonical_id(good));
        }
    }
    #[test]
    fn rejects_arbitrary_or_injected_repositories() {
        for bad in [
            "someone/private", "ores-truffle-oreslang/../admin",
            "ores-truffle-oreslang/oreslang-source.llvm?ref=main",
            "ores-truffle-oreslang/oreslang-source.net\n--method DELETE",
            "ores-truffle-oreslang/.github",
        ] {
            assert!(parse(&argv(bad, "12", "34")).is_err(), "{bad:?}");
        }
    }
    #[test]
    fn rejects_missing_and_extra_arguments() {
        assert!(parse(&["tool".into()]).is_err());
        let mut many = argv(ALLOWED_REPOS[0], "1", "2");
        many.push("--write".into());
        assert!(parse(&many).is_err());
    }
    #[test]
    fn accepts_read_only_explicit_paths() {
        let args = parse(&argv(ALLOWED_REPOS[1], "41249665953", "124680332733")).unwrap();
        assert_eq!(
            endpoint(&args, "annotations"),
            "repos/ores-truffle-oreslang/oreslang-source.net/check-runs/124680332733/annotations?per_page=100"
        );
        assert_eq!(
            endpoint(&args, "job"),
            "repos/ores-truffle-oreslang/oreslang-source.net/actions/jobs/124680332733"
        );
    }
    #[test]
    fn all_projections_are_literal_and_constrained() {
        for part in ["run", "job", "annotations", "branch", "rulesets", "permissions"] {
            assert!(!projection(part).contains("secret"));
            assert!(!projection(part).contains("token"));
            assert!(!projection(part).contains("environment"));
        }
    }
}
