//! Read-only administrator command for the scoped .github/main pilot.
//! Usage: cargo run --bin named-owner-evidence -- --read-only
//! No flags or code path permit PATCH or any GitHub mutation.
use atto_merge_review_policy::named_owner_collector::collect_pilot;
use atto_merge_review_policy::Api;
use serde_json::Value;
use std::env;
use std::path::{Component, Path};
use std::process::{Command, Stdio};

const PREFIX: &str = "repos/ores-truffle-oreslang/.github";
const MAX_RESPONSE_BYTES: usize = 512 * 1024;

struct ReadOnlyGh {
    executable: String,
}

fn validated_binary(value: &str) -> bool {
    let path = Path::new(value);
    path.is_absolute()
        && path.file_name().and_then(|file| file.to_str()) == Some("gh")
        && !value.chars().any(char::is_control)
        && !value.split('/').any(|part| part == "." || part == "..")
        && !path.components().any(|part| matches!(part, Component::ParentDir | Component::CurDir))
}

fn valid_endpoint(value: &str) -> bool {
    (value == PREFIX || value.starts_with(&format!("{PREFIX}/")))
        && !value.bytes().any(|c| c.is_ascii_control() || c == b' ')
        && !value.contains("..")
        && !value.contains('#')
        && !value.contains('%')
}

impl ReadOnlyGh {
    fn new() -> Result<Self, String> {
        let executable = match env::var("ATTO_GH_BIN") {
            Ok(value) => value,
            Err(env::VarError::NotPresent) => "/usr/bin/gh".into(),
            Err(env::VarError::NotUnicode(_)) => {
                return Err("ATTO_GH_BIN must be UTF-8".into());
            }
        };
        if !validated_binary(&executable) {
            return Err("ATTO_GH_BIN must name a trusted absolute gh executable without traversal".into());
        }
        Ok(Self { executable })
    }
}

impl Api for ReadOnlyGh {
    fn request(&mut self, method: &str, endpoint: &str, body: Option<Value>) -> Result<Value, String> {
        if method != "GET" || body.is_some() {
            return Err("refused: evidence collector supports only GET with no body".into());
        }
        if !valid_endpoint(endpoint) {
            return Err("refused: GitHub endpoint is outside fixed governance pilot scope".into());
        }
        let response = Command::new(&self.executable)
            .arg("api")
            .arg("--hostname").arg("github.com")
            .arg("--method").arg("GET")
            .arg("-H").arg("Accept: application/vnd.github+json")
            .arg("-H").arg("X-GitHub-Api-Version: 2022-11-28")
            .arg(endpoint)
            .env_remove("GH_HOST")
            .env("GH_PROMPT_DISABLED", "1")
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .map_err(|_| "cannot execute administrator-selected gh binary".to_owned())?;
        if !response.status.success() {
            // Never log raw API error bodies or privileged account metadata.
            return Err(format!("GitHub read-only GET refused (exit {:?}); check admin credentials, plan and repository access",
                response.status.code()));
        }
        if response.stdout.len() > MAX_RESPONSE_BYTES {
            return Err("GitHub evidence response exceeded bounded size".into());
        }
        serde_json::from_slice(&response.stdout)
            .map_err(|_| "GitHub evidence response was not valid JSON".into())
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() == 1 && args[0] == "--help" {
        println!("Usage: named-owner-evidence --read-only\n");
        println!("Queries only ores-truffle-oreslang/.github/main with GET.");
        println!("A candidate is NOT merge-enforcement proof and NEVER authorizes --apply.");
        return Ok(());
    }
    if args.len() != 1 || args[0] != "--read-only" {
        return Err("invalid arguments: only --read-only is accepted; no apply or write mode exists".into());
    }
    let mut api = ReadOnlyGh::new()?;
    let candidate = collect_pilot(&mut api)?;
    println!("READ_ONLY_CANDIDATE repo=ores-truffle-oreslang/.github branch=main base_sha={}",
        candidate.base_sha);
    println!("NOT_ENFORCEMENT_PROOF: verify required CI publisher, independent reviews and live merge-gate negative tests");
    Ok(())
}

fn main() {
    if let Err(reason) = run() {
        eprintln!("MANUAL_NO_WRITE: {reason}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accepts_explicit_absolute_cli_paths() {
        for value in ["/usr/bin/gh", "/opt/homebrew/bin/gh", "/usr/local/bin/gh"] {
            assert!(validated_binary(value));
        }
    }
    #[test]
    fn denies_relative_and_dot_paths() {
        for value in ["gh", "./gh", "/usr/./bin/gh", "/tmp/../bin/gh",
                      "/usr/bin/gh\n--method PATCH", "/usr/bin/sh"] {
            assert!(!validated_binary(value));
        }
    }
    #[test]
    fn fixed_scope_refuses_any_other_repo_or_ambiguous_endpoint() {
        for value in ["repos/ores-truffle-oreslang/oreslang-source.java",
                      "repos/ores-truffle-oreslang/.github-other",
                      "repos/ores-truffle-oreslang/.github/../settings",
                      "repos/ores-truffle-oreslang/.github%2fsettings",
                      "repos/ores-truffle-oreslang/.github/a\nb"] {
            assert!(!valid_endpoint(value), "{value}");
        }
        assert!(valid_endpoint(PREFIX));
        assert!(valid_endpoint(&format!("{PREFIX}/branches/main")));
    }
    #[test]
    fn hard_rejects_all_mutations_before_subprocess_execution() {
        let mut api = ReadOnlyGh { executable: "/nonexistent/gh".into() };
        assert!(api.request("PATCH",PREFIX,Some(serde_json::json!({"foo":"bar"}))).is_err());
        assert!(api.request("POST",PREFIX,None).is_err());
        assert!(api.request("GET",PREFIX,Some(serde_json::json!({}))).is_err());
    }
}
