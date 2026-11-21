#![cfg(unix)]
// End-to-end CLI tests with a deliberately fake "gh" executable.
// No network, real GitHub mutation or credentials are used here.
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

const SHA: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
static NEXT: AtomicU64 = AtomicU64::new(0);

const FAKE_GH: &str = r#"#!/bin/sh
set -eu
printf '%s\n' "$*" >> "$MOCK_LOG"

if [ "$1" = api ] && [ "$2" = --method ]; then
  if [ "${MOCK_STATUS_ERROR:-0}" = 1 ]; then
    exit 7
  fi
  printf '{}\n'
  exit 0
fi

if [ "$1" = api ]; then
  n=$(cat "$MOCK_COUNT" 2>/dev/null || printf 0)
  n=$((n + 1))
  printf '%s\n' "$n" > "$MOCK_COUNT"
  if [ "${MOCK_MUTATE_ON_FETCH:-0}" = "$n" ]; then
    awk -F '\t' 'BEGIN { OFS = "\t" } { $8 = "true"; print }' \
      "$MOCK_STATE" > "$MOCK_NEXT"
    mv "$MOCK_NEXT" "$MOCK_STATE"
  fi
  cat "$MOCK_STATE"
  exit 0
fi

if [ "$1" = pr ] && [ "$2" = ready ] && [ "$4" = --undo ]; then
  awk -F '\t' 'BEGIN { OFS = "\t" } { $4 = "true"; print }' \
    "$MOCK_STATE" > "$MOCK_NEXT"
  mv "$MOCK_NEXT" "$MOCK_STATE"
  exit 0
fi

exit 12
"#;

struct Fixture {
    root: PathBuf,
}

impl Fixture {
    fn new(keep: bool, draft: bool, not_ready: bool) -> Self {
        let id = NEXT.fetch_add(1, Ordering::SeqCst);
        let root = env::temp_dir().join(format!("atto-draft-hold-cli-{}-{}", std::process::id(), id));
        fs::create_dir_all(&root).unwrap();
        let gh_path = root.join("gh");
        fs::write(&gh_path, FAKE_GH).unwrap();
        let mut perms = fs::metadata(&gh_path).unwrap().permissions();
        perms.set_mode(0o700);
        fs::set_permissions(&gh_path, perms).unwrap();

        fs::write(root.join("state"), format!(
            "atto-lang/.github\t49\topen\t{}\tfalse\t{}\ttrue\t{}\tfalse\t{}\n",
            draft, SHA, keep, not_ready
        )).unwrap();
        Self { root }
    }

    fn invoke(&self, apply: bool, authorized: bool, mutate_on: Option<&str>,
        status_error: bool) -> Output
    {
        let exe = env!("CARGO_BIN_EXE_atto-draft-hold");
        let mut args = vec!["--repo", "atto-lang/.github", "--pr", "49"];
        if apply {
            args.push("--apply");
        }
        let mut paths = vec![self.root.clone()];
        paths.extend(env::split_paths(&env::var_os("PATH").unwrap_or_default()));
        let path = env::join_paths(paths).unwrap();
        let mut cmd = Command::new(exe);
        cmd.args(args)
            .env("PATH", path)
            .env("GH_TOKEN", "fixture-not-a-real-token")
            .env("GITHUB_REPOSITORY", "atto-lang/.github")
            .env("GITHUB_EVENT_NAME", "pull_request_target")
            .env("MOCK_LOG", self.root.join("log"))
            .env("MOCK_COUNT", self.root.join("count"))
            .env("MOCK_STATE", self.root.join("state"))
            .env("MOCK_NEXT", self.root.join("next"))
            .env("MOCK_STATUS_ERROR", if status_error { "1" } else { "0" });
        if authorized {
            cmd.env("ATTO_DRAFT_HOLD_APPLY", "1");
        } else {
            cmd.env_remove("ATTO_DRAFT_HOLD_APPLY");
        }
        if let Some(fetch_number) = mutate_on {
            cmd.env("MOCK_MUTATE_ON_FETCH", fetch_number);
        }
        cmd.output().unwrap()
    }

    fn log(&self) -> String {
        fs::read_to_string(self.root.join("log")).unwrap_or_default()
    }

    fn state(&self) -> String {
        fs::read_to_string(self.root.join("state")).unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn dry_run_never_posts_status_or_converts_draft() {
    let fixture = Fixture::new(true, false, false);
    let output = fixture.invoke(false, false, None, false);
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("READ_ONLY"));
    assert!(!fixture.log().contains("POST"));
    assert!(!fixture.log().contains("pr ready"));
}

#[test]
fn missing_write_activation_fails_before_any_gh_request() {
    let fixture = Fixture::new(true, false, false);
    let result = fixture.invoke(true, false, None, false);
    assert!(!result.status.success());
    assert!(fixture.log().is_empty());
}

#[test]
fn held_ready_pr_is_failed_first_then_restored_to_draft() {
    let fixture = Fixture::new(true, false, false);
    let output = fixture.invoke(true, true, None, false);
    assert!(output.status.success(),
        "{}", String::from_utf8_lossy(&output.stderr));
    let log = fixture.log();
    assert!(log.contains("state=failure"));
    assert!(!log.contains("state=success"));
    assert!(log.contains("pr ready 49 --undo --repo atto-lang/.github"));
    assert!(log.find("state=failure").unwrap() < log.find("pr ready").unwrap());
    assert!(fixture.state().contains("\topen\ttrue\tfalse\t"));
}

#[test]
fn unlabelled_ready_pr_gets_only_the_narrow_status() {
    let fixture = Fixture::new(false, false, false);
    let output = fixture.invoke(true, true, None, false);
    assert!(output.status.success(),
        "{}", String::from_utf8_lossy(&output.stderr));
    let log = fixture.log();
    assert!(log.find("state=pending").unwrap() < log.find("state=success").unwrap());
    assert!(log.contains("context=atto-governance/draft-hold"));
    assert!(!log.contains("pr ready"));
}

#[test]
fn not_ready_blocks_without_draft_conversion() {
    let fixture = Fixture::new(false, false, true);
    let output = fixture.invoke(true, true, None, false);
    assert!(output.status.success());
    assert!(fixture.log().contains("state=failure"));
    assert!(!fixture.log().contains("pr ready"));
}

#[test]
fn same_sha_label_race_cannot_emit_success() {
    let fixture = Fixture::new(false, false, false);
    let output = fixture.invoke(true, true, Some("2"), false);
    assert!(!output.status.success());
    let log = fixture.log();
    assert!(log.contains("state=pending"));
    assert!(!log.contains("state=success"));
    assert!(!log.contains("pr ready"));
}

#[test]
fn failed_status_api_never_converts_draft() {
    let fixture = Fixture::new(true, false, false);
    let output = fixture.invoke(true, true, None, true);
    assert!(!output.status.success());
    let log = fixture.log();
    assert!(log.contains("state=failure"));
    assert!(!log.contains("pr ready"));
}
