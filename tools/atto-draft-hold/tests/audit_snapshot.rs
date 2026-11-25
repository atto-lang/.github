#![cfg(unix)]
// End-to-end read-only collector tests. All gh API responses are synthetic;
// neither credentials nor a live repository are used for this race harness.
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);
const FAKE_GH: &str = r#"#!/bin/sh
set -eu
count=$(cat "$MOCK_COUNT" 2>/dev/null || printf 0)
count=$((count + 1))
printf '%s\n' "$count" > "$MOCK_COUNT"
printf '%s\n' "$*" >> "$MOCK_LOG"
if [ "${MOCK_ERROR_ON:-0}" = "$count" ]; then exit 77; fi
case "$*" in
  *"/issues/49/timeline"*) exit 0 ;;
  *"/pulls/49/reviews"*) exit 0 ;;
  *"/check-runs"*) exit 0 ;;
esac
case "$*" in
  *".user.id,.created_at"*)
    sha=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
    updated=2026-11-20T00:01:00Z
    if [ "${MOCK_RACE:-}" = metadata ] && [ "$count" -eq 7 ]; then
      updated=2026-11-20T00:02:00Z
    fi
    if [ "${MOCK_RACE:-}" = first_head ] && [ "$count" -eq 2 ]; then
      sha=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
    fi
    if [ "${MOCK_RACE:-}" = malformed ] && [ "$count" -eq 2 ]; then
      printf 'unexpected\n'
      exit 0
    fi
    printf 'atto-lang/.github\t49\t1000\t2026-11-19T00:00:00Z\t%s\tfalse\t%s\taaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n' "$sha" "$updated"
    exit 0 ;;
  *".base.repo.full_name"*)
    hold=false
    if [ "${MOCK_RACE:-}" = labels ] && [ "$count" -eq 6 ]; then
      hold=true
    fi
    printf 'atto-lang/.github\t49\topen\tfalse\tfalse\taaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\ttrue\t%s\tfalse\tfalse\n' "$hold"
    exit 0 ;;
esac
exit 71
"#;

struct Fixture {
    root: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let id = NEXT.fetch_add(1, Ordering::SeqCst);
        let root = env::temp_dir().join(format!("atto-ready-audit-{}-{}", std::process::id(), id));
        fs::create_dir_all(&root).unwrap();
        let gh_path = root.join("gh");
        fs::write(&gh_path, FAKE_GH).unwrap();
        let mut perms = fs::metadata(&gh_path).unwrap().permissions();
        perms.set_mode(0o700);
        fs::set_permissions(&gh_path, perms).unwrap();
        Self { root }
    }
    fn call(&self, race: &str, fail_on: &str) -> Output {
        let exe = env!("CARGO_BIN_EXE_atto-ready-audit");
        let mut paths = vec![self.root.clone()];
        paths.extend(env::split_paths(&env::var_os("PATH").unwrap_or_default()));
        Command::new(exe)
            .args(["--repo", "atto-lang/.github", "--pr", "49"])
            .env("PATH", env::join_paths(paths).unwrap())
            .env("MOCK_RACE", race)
            .env("MOCK_ERROR_ON", fail_on)
            .env("MOCK_COUNT", self.root.join("count"))
            .env("MOCK_LOG", self.root.join("log"))
            .output()
            .unwrap()
    }
    fn count(&self) -> usize {
        fs::read_to_string(self.root.join("count"))
            .unwrap()
            .trim()
            .parse()
            .unwrap()
    }
    fn log(&self) -> String {
        fs::read_to_string(self.root.join("log")).unwrap_or_default()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
fn both(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

#[test]
fn stable_authenticated_fixture_is_still_hold_without_push_and_ai_issuers() {
    let f = Fixture::new();
    let out = f.call("", "0");
    assert_eq!(out.status.code(), Some(1), "{}", both(&out));
    assert!(both(&out).contains("READ_ONLY_REVIEW_AUDIT state=HOLD"));
    assert!(both(&out).contains("authorized=false"));
    assert_eq!(f.count(), 7);
    assert!(!f.log().contains("--method POST"));
    assert!(!f.log().contains("ready --undo"));
}
#[test]
fn concurrent_same_sha_label_hold_blocks_mixed_snapshot() {
    let f = Fixture::new();
    let out = f.call("labels", "0");
    assert_eq!(out.status.code(), Some(1), "{}", both(&out));
    assert!(both(&out).contains("live PR changed while collecting evidence"));
    assert!(!both(&out).contains("READ_ONLY_REVIEW_AUDIT"));
    assert_eq!(f.count(), 7);
}
#[test]
fn changed_update_watermark_blocks_same_sha_and_same_labels() {
    let f = Fixture::new();
    let out = f.call("metadata", "0");
    assert_eq!(out.status.code(), Some(1), "{}", both(&out));
    assert!(both(&out).contains("live PR changed while collecting evidence"));
    assert_eq!(f.count(), 7);
}
#[test]
fn initial_live_metadata_head_mismatch_is_rejected_before_pagination() {
    let f = Fixture::new();
    let out = f.call("first_head", "0");
    assert_eq!(out.status.code(), Some(1), "{}", both(&out));
    assert!(both(&out).contains("immutable starting snapshot"));
    assert_eq!(f.count(), 2);
}
#[test]
fn malformed_api_projection_fails_closed() {
    let f = Fixture::new();
    let out = f.call("malformed", "0");
    assert_eq!(out.status.code(), Some(1), "{}", both(&out));
    assert!(both(&out).contains("malformed API projection"));
    assert_eq!(f.count(), 2);
}
#[test]
fn failed_paginated_github_read_fails_closed() {
    let f = Fixture::new();
    let out = f.call("", "3");
    assert_eq!(out.status.code(), Some(1), "{}", both(&out));
    assert!(both(&out).contains("authenticated GitHub read failed"));
    assert_eq!(f.count(), 3);
}
