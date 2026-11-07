//! Read-only, source-free GitHub Actions admission diagnostics.
//!
//! Does not fetch repository contents, secrets, artifacts, or change settings.
//! Annotation text is potentially sensitive: review locally before sharing.
//! Usage: gh-run-admission OWNER/REPO RUN_ID JOB_ID
use std::env;
use std::process::{Command, ExitCode};

const GITHUB_API_HOST: &str = "github.com";
const GH_CLI_PATH: &str = "/usr/bin/gh";

const ALLOWED_REPOS: [&str; 3] = [
    "ores-truffle-oreslang/oreslang-source.llvm",
    "ores-truffle-oreslang/oreslang-source.net",
    "ores-truffle-oreslang/.github",
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

fn canonical_sha(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn parse(args: &[String]) -> Result<Args, &'static str> {
    if args.len() != 4 {
        return Err("usage: gh-run-admission OWNER/REPO RUN_ID JOB_ID");
    }
    let repo = &args[1];
    if !ALLOWED_REPOS.contains(&repo.as_str()) {
        return Err("repository is not an explicitly authorized Oreslang source or governance repository");
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

fn check_run_id_from_url(repository: &str, url: &str) -> Result<String, &'static str> {
    let prefix = format!("https://api.github.com/repos/{repository}/check-runs/");
    let id = url
        .strip_prefix(&prefix)
        .ok_or("selected job check_run_url is outside the audited repository")?;
    if !canonical_id(id) {
        return Err("selected job check_run_url does not end in one canonical check-run id");
    }
    Ok(id.to_owned())
}

fn endpoint(parts: &Args, part: &str, check_run_id: Option<&str>) -> String {
    let prefix = format!("repos/{}/", parts.repository);
    match part {
        "run" | "run_head" => format!("{prefix}actions/runs/{}", parts.run_id),
        "job" | "job_identity" => format!("{prefix}actions/jobs/{}", parts.job_id),
        "check_run" | "check_identity" => format!(
            "{prefix}check-runs/{}",
            check_run_id.expect("check-run id required for check-run endpoint")
        ),
        "annotations" => format!(
            "{prefix}check-runs/{}/annotations?per_page=100",
            check_run_id.expect("check-run id required for annotation endpoint")
        ),
        "branch" => format!("{prefix}branches/main"),
        "rulesets" => format!("{prefix}rulesets?per_page=100"),
        "permissions" => format!("{prefix}actions/permissions"),
        _ => unreachable!("only compile-time named endpoints are allowed"),
    }
}

fn projection(part: &str) -> &'static str {
    match part {
        "run" => r#"{id,run_attempt,head_sha,event,status,conclusion,created_at,updated_at}"#,
        "run_head" => ".head_sha",
        "job" => r#"{id,run_id,name,status,conclusion,runner_id,runner_name,started_at,completed_at,step_count:((.steps // [])|length),check_run_url}"#,
        "job_identity" => r#"[.run_id,.check_run_url,.status,.conclusion,((.steps // [])|length)]|@tsv"#,
        "check_run" => r#"{id,head_sha,status,conclusion,app_slug:.app.slug,started_at,completed_at,annotations_count:.output.annotations_count}"#,
        "check_identity" => r#"[.head_sha,.app.slug,.status,.conclusion]|@tsv"#,
        "annotations" => r#"[.[]|{level:.annotation_level,title,message,path,start_line,end_line}]"#,
        "branch" => r#"{name,protected,sha:.commit.sha}"#,
        "rulesets" => r#"[.[]|{id,name,enforcement,source_type,source}]"#,
        "permissions" => r#"{enabled,allowed_actions,selected_actions_url}"#,
        _ => unreachable!("only compile-time named projections are allowed"),
    }
}

fn gh_read(parts: &Args, part: &str, check_run_id: Option<&str>) -> Result<String, String> {
    let path = endpoint(parts, part, check_run_id);
    let result = Command::new(GH_CLI_PATH)
        // Preserve the administrator's chosen credentials, but never allow
        // ambient GH_HOST to redirect a privileged API request elsewhere.
        .env_remove("GH_HOST")
        .args(["api", "--hostname", GITHUB_API_HOST, "--method", "GET",
               &path, "--jq", projection(part)])
        .output()
        .map_err(|_| "GitHub CLI unavailable (install gh and authenticate)".to_owned())?;
    if !result.status.success() {
        return Err(format!(
            "GitHub API refused read-only {} request (exit {:?}); inspect access/plan in the UI",
            part,
            result.status.code()
        ));
    }
    let output =
        String::from_utf8(result.stdout).map_err(|_| format!("{part}: non-UTF8 API output"))?;
    if output.len() > 64 * 1024 {
        return Err(format!("{part}: diagnostic output exceeded 64 KiB limit"));
    }
    Ok(output)
}

fn parse_job_identity(parts: &Args, observed: &str) -> Result<String, &'static str> {
    let line = observed.trim_end_matches(['\r', '\n']);
    if line.contains('\n') || line.contains('\r') {
        return Err("selected job identity projection returned multiple records");
    }
    let mut fields = line.split('\t');
    let run_id = fields.next().ok_or("selected job identity is missing run_id")?;
    let check_run_url = fields
        .next()
        .ok_or("selected job identity is missing check_run_url")?;
    let status = fields.next().ok_or("selected job identity is missing status")?;
    let conclusion = fields
        .next()
        .ok_or("selected job identity is missing conclusion")?;
    let step_count = fields
        .next()
        .ok_or("selected job identity is missing step_count")?;
    if fields.next().is_some() || !canonical_id(run_id) || run_id != parts.run_id {
        return Err("selected job does not belong to supplied workflow run");
    }
    if status != "completed" || conclusion != "failure" || step_count != "0" {
        return Err("selected job is not one completed zero-step failure");
    }
    check_run_id_from_url(&parts.repository, check_run_url)
}

fn verify_check_identity(run_head: &str, observed: &str) -> Result<(), &'static str> {
    let expected_head = run_head.trim();
    if !canonical_sha(expected_head) {
        return Err("workflow run head_sha is not one canonical lowercase 40-hex SHA");
    }
    let line = observed.trim_end_matches(['\r', '\n']);
    if line.contains('\n') || line.contains('\r') {
        return Err("check-run identity projection returned multiple records");
    }
    let mut fields = line.split('\t');
    let check_head = fields
        .next()
        .ok_or("check-run identity is missing head_sha")?;
    let app_slug = fields.next().ok_or("check-run identity is missing app slug")?;
    let status = fields.next().ok_or("check-run identity is missing status")?;
    let conclusion = fields
        .next()
        .ok_or("check-run identity is missing conclusion")?;
    if fields.next().is_some()
        || !canonical_sha(check_head)
        || check_head != expected_head
        || app_slug != "github-actions"
        || status != "completed"
        || conclusion != "failure"
    {
        return Err(
            "selected check-run does not match completed failed workflow head/GitHub Actions identity",
        );
    }
    Ok(())
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

    // A workflow job has a check_run_url; job IDs and check-run IDs are
    // different identity domains and must never be assumed equal.
    let job_identity = match gh_read(&args, "job_identity", None) {
        Ok(s) => s,
        Err(reason) => {
            eprintln!("Cannot attest job/run identity: {reason}");
            return ExitCode::from(1);
        }
    };
    let check_run_id = match parse_job_identity(&args, &job_identity) {
        Ok(id) => id,
        Err(reason) => {
            eprintln!("{reason}");
            return ExitCode::from(1);
        }
    };

    let run_head = match gh_read(&args, "run_head", None) {
        Ok(s) => s,
        Err(reason) => {
            eprintln!("Cannot attest workflow run head: {reason}");
            return ExitCode::from(1);
        }
    };
    let check_identity = match gh_read(&args, "check_identity", Some(&check_run_id)) {
        Ok(s) => s,
        Err(reason) => {
            eprintln!("Cannot attest check-run identity: {reason}");
            return ExitCode::from(1);
        }
    };
    if let Err(reason) = verify_check_identity(&run_head, &check_identity) {
        eprintln!("{reason}");
        return ExitCode::from(1);
    }

    println!("SOURCE_FREE_READ_ONLY_ADMISSION_REPORT");
    println!(
        "repo={} run={} job={} check_run={}",
        args.repository, args.run_id, args.job_id, check_run_id
    );

    let mut essential_ok = true;
    for part in [
        "run",
        "job",
        "check_run",
        "annotations",
        "branch",
        "rulesets",
        "permissions",
    ] {
        println!("--- {part} ---");
        let selected = matches!(part, "check_run" | "annotations").then_some(check_run_id.as_str());
        match gh_read(&args, part, selected) {
            Ok(data) => print!("{data}"),
            Err(message) => {
                println!("UNAVAILABLE: {message}");
                if matches!(part, "run" | "job" | "check_run" | "annotations") {
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
            assert_eq!(
                parse(&argv(repo, "41236569150", "124641494087"))
                    .unwrap()
                    .repository,
                repo
            );
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
            "someone/private",
            "ores-truffle-oreslang/../admin",
            "ores-truffle-oreslang/oreslang-source.llvm?ref=main",
            "ores-truffle-oreslang/oreslang-source.net\n--method DELETE",
        ] {
            assert!(parse(&argv(bad, "12", "34")).is_err(), "{bad:?}");
        }
    }


    #[test]
    fn accepts_exact_governance_source_free_admission_job() {
        let args = parse(&argv("ores-truffle-oreslang/.github",
            "42398019612", "128479490053")).unwrap();
        assert_eq!(
            endpoint(&args, "job_identity", None),
            "repos/ores-truffle-oreslang/.github/actions/jobs/128479490053"
        );
        assert_eq!(
            endpoint(&args, "run_head", None),
            "repos/ores-truffle-oreslang/.github/actions/runs/42398019612"
        );
        assert_eq!(
            endpoint(&args, "annotations", Some("128479490053")),
            "repos/ores-truffle-oreslang/.github/check-runs/128479490053/annotations?per_page=100"
        );
    }

    #[test]
    fn governance_job_must_not_trust_check_run_url_in_another_repo() {
        let args = parse(&argv("ores-truffle-oreslang/.github",
            "42398019612", "128479490053")).unwrap();
        let attack = concat!(
            "42398019612\t",
            "https://api.github.com/repos/ores-truffle-oreslang/",
            "oreslang-source.llvm/check-runs/128479490053\tcompleted\tfailure\t0\n"
        );
        assert!(parse_job_identity(&args, attack).is_err());
    }

    #[test]
    fn governance_job_identity_requires_zero_steps_and_failed_state() {
        let args = parse(&argv("ores-truffle-oreslang/.github",
            "42398019612", "128479490053")).unwrap();
        for bad in [
            "42398019612\thttps://api.github.com/repos/ores-truffle-oreslang/.github/check-runs/128479490053\tcompleted\tsuccess\t0\n",
            "42398019612\thttps://api.github.com/repos/ores-truffle-oreslang/.github/check-runs/128479490053\tcompleted\tfailure\t1\n",
            "42398019612\thttps://api.github.com/repos/ores-truffle-oreslang/.github/check-runs/128479490053\tin_progress\tfailure\t0\n",
        ] {
            assert!(parse_job_identity(&args, bad).is_err());
        }
    }

    #[test]
    fn cannot_add_unrelated_repo_by_using_github_prefix() {
        for bad in [
            "ores-truffle-oreslang/.github/../oreslang-source.java",
            "ores-truffle-oreslang/.github?ref=main",
            "ores-truffle-oreslang/.github\n--method PATCH",
            "other/.github",
        ] {
            assert!(parse(&argv(bad, "42398019612", "128479490053")).is_err());
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
        let args =
            parse(&argv(ALLOWED_REPOS[1], "41250696587", "124683413992")).unwrap();
        assert_eq!(
            endpoint(&args, "job", None),
            "repos/ores-truffle-oreslang/oreslang-source.net/actions/jobs/124683413992"
        );
        assert_eq!(
            endpoint(&args, "annotations", Some("124683413992")),
            "repos/ores-truffle-oreslang/oreslang-source.net/check-runs/124683413992/annotations?per_page=100"
        );
    }

    #[test]
    fn extracts_check_run_from_selected_job_identity() {
        let args =
            parse(&argv(ALLOWED_REPOS[1], "41250696587", "124683413992")).unwrap();
        let observed = concat!(
            "41250696587\t",
            "https://api.github.com/repos/ores-truffle-oreslang/",
            "oreslang-source.net/check-runs/124683413992\tcompleted\tfailure\t0\n"
        );
        assert_eq!(parse_job_identity(&args, observed).unwrap(), "124683413992");
    }

    #[test]
    fn rejects_wrong_job_run_even_if_ids_valid() {
        let args =
            parse(&argv(ALLOWED_REPOS[1], "41250696587", "124683413992")).unwrap();
        let observed = concat!(
            "41250696586\t",
            "https://api.github.com/repos/ores-truffle-oreslang/",
            "oreslang-source.net/check-runs/124683413992\tcompleted\tfailure\t0\n"
        );
        assert!(parse_job_identity(&args, observed).is_err());
    }

    #[test]
    fn rejects_spoofed_check_run_urls() {
        let args =
            parse(&argv(ALLOWED_REPOS[1], "41250696587", "124683413992")).unwrap();
        for url in [
            "https://api.github.com/repos/ores-truffle-oreslang/oreslang-source.llvm/check-runs/124683413992",
            "https://api.github.com/repos/ores-truffle-oreslang/oreslang-source.net/check-runs/124683413992?x=1",
            "https://api.github.com/repos/ores-truffle-oreslang/oreslang-source.net/check-runs/01",
            "https://evil.example/repos/ores-truffle-oreslang/oreslang-source.net/check-runs/124683413992",
        ] {
            let observed = format!("{}\t{}\tcompleted\tfailure\t0\n", args.run_id, url);
            assert!(parse_job_identity(&args, &observed).is_err(), "{url}");
        }
    }

    #[test]
    fn verifies_check_run_head_and_actions_app() {
        let head = "025e88eca0c308c5f34fe8e4d58c88c915eb8d8e\n";
        let good =
            "025e88eca0c308c5f34fe8e4d58c88c915eb8d8e\tgithub-actions\tcompleted\tfailure\n";
        assert!(verify_check_identity(head, good).is_ok());
        assert!(verify_check_identity(
            head,
            "125e88eca0c308c5f34fe8e4d58c88c915eb8d8e\tgithub-actions\tcompleted\tfailure\n"
        )
        .is_err());
        assert!(verify_check_identity(
            head,
            "025e88eca0c308c5f34fe8e4d58c88c915eb8d8e\tuntrusted-app\tcompleted\tfailure\n"
        )
        .is_err());
    }

    #[test]
    fn rejects_malformed_shas() {
        for bad in [
            "",
            "025e88eca0c308c5f34fe8e4d58c88c915eb8d8",
            "025E88ECA0C308C5F34FE8E4D58C88C915EB8D8E",
            "g25e88eca0c308c5f34fe8e4d58c88c915eb8d8",
        ] {
            assert!(!canonical_sha(bad), "{bad:?}");
        }
    }

    #[test]
    fn job_identity_uses_read_only_selected_job_route() {
        let a = parse(&argv(ALLOWED_REPOS[0], "2", "3")).unwrap();
        assert_eq!(endpoint(&a, "job_identity", None), endpoint(&a, "job", None));
        assert_eq!(
            projection("job_identity"),
            r#"[.run_id,.check_run_url,.status,.conclusion,((.steps // [])|length)]|@tsv"#
        );
    }

    #[test]
    fn api_host_is_pinned_to_official_github() {
        assert_eq!(GITHUB_API_HOST, "github.com");
        assert!(ALLOWED_REPOS.iter().all(|r| r.starts_with("ores-truffle-oreslang/")));
    }
    #[test]
    fn rejects_nonzero_or_nonfailed_job_evidence() {
        let args =
            parse(&argv(ALLOWED_REPOS[1], "41250696587", "124683413992")).unwrap();
        let base = concat!(
            "41250696587\thttps://api.github.com/repos/ores-truffle-oreslang/",
            "oreslang-source.net/check-runs/124683413992"
        );
        for suffix in [
            "\tcompleted\tsuccess\t0\n",
            "\tcompleted\tfailure\t1\n",
            "\tin_progress\t\t0\n",
            "\tqueued\t\t0\n",
        ] {
            assert!(parse_job_identity(&args, &format!("{base}{suffix}")).is_err());
        }
    }

    #[test]
    fn rejects_nonfailed_or_incomplete_check_evidence() {
        let head = "025e88eca0c308c5f34fe8e4d58c88c915eb8d8e\n";
        for observed in [
            "025e88eca0c308c5f34fe8e4d58c88c915eb8d8e\tgithub-actions\tcompleted\tsuccess\n",
            "025e88eca0c308c5f34fe8e4d58c88c915eb8d8e\tgithub-actions\tin_progress\t\n",
            "025e88eca0c308c5f34fe8e4d58c88c915eb8d8e\tgithub-actions\tqueued\t\n",
        ] {
            assert!(verify_check_identity(head, observed).is_err());
        }
    }

    #[test]
    fn gh_executable_is_absolute_and_pinned() {
        assert_eq!(GH_CLI_PATH, "/usr/bin/gh");
        assert!(GH_CLI_PATH.starts_with('/'));
        assert!(!GH_CLI_PATH.contains(".."));
    }

    #[test]
    fn all_projections_are_literal_and_constrained() {
        for part in [
            "run",
            "run_head",
            "job",
            "job_identity",
            "check_run",
            "check_identity",
            "annotations",
            "branch",
            "rulesets",
            "permissions",
        ] {
            assert!(!projection(part).contains("secret"));
            assert!(!projection(part).contains("token"));
            assert!(!projection(part).contains("environment"));
        }
    }
}
