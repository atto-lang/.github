use std::{
    env,
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static COUNTER: AtomicU64 = AtomicU64::new(0);

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let path = env::temp_dir().join(format!(
            "sync-gh-org-test-{}-{stamp}-{n}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn s(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn execute(
    program: &str,
    args: &[String],
    cwd: Option<&Path>,
    extra_env: &[(String, String)],
    check: bool,
) -> Output {
    let mut command = Command::new(program);
    command.args(args);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    command.envs(extra_env.iter().cloned());
    let output = command
        .output()
        .unwrap_or_else(|e| panic!("failed to execute {program} {args:?}: {e}"));
    if check && !output.status.success() {
        panic!(
            "command failed: {program} {args:?}\n{}",
            combined(&output)
        );
    }
    output
}

fn combined(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn git(args: &[String], envs: &[(String, String)], check: bool) -> Output {
    execute("git", args, None, envs, check)
}

fn git_c(repo: &Path, args: &[&str], envs: &[(String, String)], check: bool) -> Output {
    let mut all = vec!["-C".to_string(), s(repo)];
    all.extend(args.iter().map(|v| (*v).to_string()));
    git(&all, envs, check)
}

fn git_dir(repo: &Path, args: &[&str], envs: &[(String, String)], check: bool) -> Output {
    let mut all = vec!["--git-dir".to_string(), s(repo)];
    all.extend(args.iter().map(|v| (*v).to_string()));
    git(&all, envs, check)
}

fn set_exec(path: &Path) {
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}

fn run_suite() {
    let temp = TempDir::new();
    let b = &temp.0;
    let remote = b.join("remote");
    let root = b.join("local");
    let home = b.join("home");
    let bin = b.join("bin");
    for path in [&remote, &root, &home, &bin] {
        fs::create_dir(path).unwrap();
    }

    let empty_env: Vec<(String, String)> = Vec::new();

    for name in ["repo1", "repo2"] {
        let bare = remote.join(format!("{name}.git"));
        git(
            &["init".into(), "-q".into(), "--bare".into(), s(&bare)],
            &empty_env,
            true,
        );
        let seed = b.join(format!("seed-{name}"));
        git(
            &["clone".into(), "-q".into(), s(&bare), s(&seed)],
            &empty_env,
            true,
        );
        git_c(&seed, &["config", "user.name", "Tester"], &empty_env, true);
        git_c(
            &seed,
            &["config", "user.email", "test@example.com"],
            &empty_env,
            true,
        );
        fs::write(seed.join("hello.txt"), "base\n").unwrap();
        git_c(&seed, &["add", "-A"], &empty_env, true);
        git_c(&seed, &["commit", "-qm", "base"], &empty_env, true);
        git_c(&seed, &["branch", "-M", "main"], &empty_env, true);
        git_c(&seed, &["push", "-q", "-u", "origin", "main"], &empty_env, true);
        git_dir(
            &bare,
            &["symbolic-ref", "HEAD", "refs/heads/main"],
            &empty_env,
            true,
        );
        git_c(&seed, &["tag", "v1"], &empty_env, true);
        git_c(&seed, &["push", "-q", "origin", "v1"], &empty_env, true);
    }

    let gitconfig = home.join(".gitconfig");
    git(
        &[
            "config".into(),
            "--file".into(),
            s(&gitconfig),
            format!("url.file://{}/.insteadOf", s(&remote)),
            "https://github.com/testorg/".into(),
        ],
        &empty_env,
        true,
    );

    let mock = r#"#!/usr/bin/env bash
case "$1" in
 api) printf 'repo1\nrepo2\n';;
 repo)
   if [[ "$2" == view ]]; then
     name=${3#testorg/}; test -e "$MOCK_REMOTE/$name.git" && printf '%s\n' "$name"
   elif [[ "$2" == clone ]]; then
     git clone -q "https://github.com/$3.git" "$4"
   else exit 3; fi ;;
 *) exit 3;;
esac
"#;
    let gh = bin.join("gh");
    fs::write(&gh, mock).unwrap();
    set_exec(&gh);

    let old_path = env::var("PATH").unwrap_or_default();
    let test_env = vec![
        ("HOME".into(), s(&home)),
        ("PATH".into(), format!("{}:{old_path}", s(&bin))),
        ("MOCK_REMOTE".into(), s(&remote)),
        ("GIT_CONFIG_NOSYSTEM".into(), "1".into()),
    ];

    git(
        &[
            "clone".into(),
            "-q".into(),
            "https://github.com/testorg/repo1.git".into(),
            s(&root.join("repo1")),
        ],
        &test_env,
        true,
    );
    let r = root.join("repo1");
    git_c(&r, &["config", "user.name", "Tester"], &test_env, true);
    git_c(
        &r,
        &["config", "user.email", "test@example.com"],
        &test_env,
        true,
    );

    fs::write(r.join("hello.txt"), "stashed\n").unwrap();
    git_c(
        &r,
        &["stash", "push", "-qm", "original stash"],
        &test_env,
        true,
    );
    let stash = stdout(&git_c(&r, &["rev-parse", "refs/stash"], &test_env, true));

    fs::write(r.join("new.txt"), "valuable untracked\n").unwrap();
    git_c(&r, &["branch", "topic"], &test_env, true);

    let wt = b.join("extra-wt");
    let wt_string = s(&wt);
    git_c(
        &r,
        &["worktree", "add", "--detach", &wt_string, "HEAD"],
        &test_env,
        true,
    );
    fs::write(wt.join("detached.txt"), "valuable detached\n").unwrap();

    let seed1 = b.join("seed-repo1");
    fs::write(seed1.join("remote.txt"), "remote advance\n").unwrap();
    git_c(&seed1, &["add", "-A"], &test_env, true);
    git_c(
        &seed1,
        &["commit", "-qm", "remote advance"],
        &test_env,
        true,
    );
    git_c(&seed1, &["push", "-q", "origin", "main"], &test_env, true);

    let before = stdout(&git_c(&r, &["rev-parse", "HEAD"], &test_env, true));
    let sync_script = env::var("SYNC_SCRIPT").map(PathBuf::from).unwrap_or_else(|_| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("scripts/sync-gh-org.sh")
    });
    let dry = execute(
        "bash",
        &[
            s(&sync_script),
            "testorg".into(),
            "--root".into(),
            s(&root),
            "--dry-run".into(),
        ],
        None,
        &test_env,
        true,
    );
    let dry_text = combined(&dry);
    assert!(dry_text.contains("would clone") && dry_text.contains("dirty=yes"));
    assert!(!root.join(".org-sync").exists() && !root.join("repo2").exists());
    assert_eq!(
        stdout(&git_c(&r, &["rev-parse", "HEAD"], &test_env, true)),
        before
    );
    println!("DRY_RUN_OK");

    let sync = execute(
        "bash",
        &[
            s(&sync_script),
            "testorg".into(),
            "--root".into(),
            s(&root),
            "--all".into(),
            "--checkpoint".into(),
            "--backup-remote".into(),
            "--verify".into(),
            "--report".into(),
        ],
        None,
        &test_env,
        false,
    );
    let sync_text = combined(&sync);
    assert!(sync.status.success(), "{sync_text}");
    assert!(root.join("repo2/hello.txt").exists());
    assert!(r.join("new.txt").exists());
    assert!(wt.join("detached.txt").exists());
    assert_eq!(
        stdout(&git_c(&r, &["rev-parse", "refs/stash"], &test_env, true)),
        stash
    );
    assert!(r.join("remote.txt").exists());

    let refs = combined(&git_dir(
        &remote.join("repo1.git"),
        &[
            "for-each-ref",
            "--format=%(refname)",
            "refs/heads/sync",
            "refs/tags/sync-remote",
        ],
        &test_env,
        true,
    ));
    assert!(refs.contains("refs/heads/sync/local/"));
    assert!(refs.contains("refs/heads/sync/stash/"));
    assert!(refs.contains("refs/heads/sync/detached/"));
    assert!(!refs.contains("refs/heads/sync/remote/"));
    assert!(!refs.contains("refs/tags/sync-remote/"));
    println!("MUTATION_OK");

    let clash_root = b.join("clash");
    fs::create_dir(&clash_root).unwrap();
    git(
        &[
            "clone".into(),
            "-q".into(),
            "https://github.com/testorg/repo1.git".into(),
            s(&clash_root.join("repo1")),
        ],
        &test_env,
        true,
    );
    let clash = clash_root.join("repo1");
    git_c(&clash, &["config", "user.name", "Tester"], &test_env, true);
    git_c(
        &clash,
        &["config", "user.email", "test@example.com"],
        &test_env,
        true,
    );
    fs::write(clash.join("hello.txt"), "local conflict\n").unwrap();

    let conf = b.join("conflict-seed");
    git(
        &[
            "clone".into(),
            "-q".into(),
            "https://github.com/testorg/repo1.git".into(),
            s(&conf),
        ],
        &test_env,
        true,
    );
    git_c(&conf, &["config", "user.name", "Tester"], &test_env, true);
    git_c(
        &conf,
        &["config", "user.email", "test@example.com"],
        &test_env,
        true,
    );
    fs::write(conf.join("hello.txt"), "remote conflict\n").unwrap();
    git_c(&conf, &["add", "-A"], &test_env, true);
    git_c(
        &conf,
        &["commit", "-qm", "remote conflict"],
        &test_env,
        true,
    );
    git_c(&conf, &["push", "-q", "origin", "main"], &test_env, true);

    let conflict = execute(
        "bash",
        &[
            s(&sync_script),
            "testorg".into(),
            "--root".into(),
            s(&clash_root),
            "--repo".into(),
            "repo1".into(),
            "--checkpoint".into(),
            "--backup-remote".into(),
            "--verify".into(),
            "--report".into(),
        ],
        None,
        &test_env,
        false,
    );
    let conflict_text = combined(&conflict);
    assert!(!conflict.status.success(), "{conflict_text}");
    assert!(conflict_text.to_ascii_lowercase().contains("merge conflict"));
    assert!(!stdout(&git_c(&clash, &["ls-files", "-u"], &test_env, true)).is_empty());
    let conflict_file = fs::read_to_string(clash.join("hello.txt")).unwrap();
    assert!(conflict_file.contains("local conflict") && conflict_file.contains("remote conflict"));
    println!("CONFLICT_PRESERVED_OK");

    let seed2 = b.join("seed-repo2");
    git_c(&seed2, &["switch", "-qc", "unmerged-feature"], &test_env, true);
    fs::write(seed2.join("feature.txt"), "feature base\n").unwrap();
    git_c(&seed2, &["add", "-A"], &test_env, true);
    git_c(&seed2, &["commit", "-qm", "create feature"], &test_env, true);
    git_c(
        &seed2,
        &["push", "-q", "origin", "unmerged-feature"],
        &test_env,
        true,
    );

    let feature_root = b.join("feature-sync");
    fs::create_dir(&feature_root).unwrap();
    git(
        &[
            "clone".into(),
            "-q".into(),
            "https://github.com/testorg/repo2.git".into(),
            s(&feature_root.join("repo2")),
        ],
        &test_env,
        true,
    );
    let feature_local = feature_root.join("repo2");
    git_c(
        &feature_local,
        &["config", "user.name", "Tester"],
        &test_env,
        true,
    );
    git_c(
        &feature_local,
        &["config", "user.email", "test@example.com"],
        &test_env,
        true,
    );
    git_c(
        &feature_local,
        &[
            "switch",
            "-q",
            "-c",
            "unmerged-feature",
            "origin/unmerged-feature",
        ],
        &test_env,
        true,
    );
    fs::write(feature_local.join("local-only.txt"), "feature-local\n").unwrap();
    git_c(&feature_local, &["add", "-A"], &test_env, true);
    git_c(
        &feature_local,
        &["commit", "-qm", "local feature"],
        &test_env,
        true,
    );
    fs::write(seed2.join("remote-only.txt"), "feature-remote\n").unwrap();
    git_c(&seed2, &["add", "-A"], &test_env, true);
    git_c(
        &seed2,
        &["commit", "-qm", "remote feature"],
        &test_env,
        true,
    );
    git_c(
        &seed2,
        &["push", "-q", "origin", "unmerged-feature"],
        &test_env,
        true,
    );

    let feature_sync = execute(
        "bash",
        &[
            s(&sync_script),
            "testorg".into(),
            "--root".into(),
            s(&feature_root),
            "--repo".into(),
            "repo2".into(),
            "--checkpoint".into(),
            "--backup-remote".into(),
            "--verify".into(),
            "--report".into(),
        ],
        None,
        &test_env,
        false,
    );
    let feature_text = combined(&feature_sync);
    assert!(feature_sync.status.success(), "{feature_text}");
    let feature_refs = combined(&git_dir(
        &remote.join("repo2.git"),
        &[
            "for-each-ref",
            "--format=%(refname)",
            "refs/heads/sync/remote",
        ],
        &test_env,
        true,
    ));
    assert!(feature_refs.contains("refs/heads/sync/remote/"));
    assert!(feature_text.contains("SKIPPED: remote backups:"));
    println!("SELECTIVE_REMOTE_BACKUP_OK");

    let churn_root = b.join("unrelated-churn");
    fs::create_dir(&churn_root).unwrap();
    git(
        &[
            "clone".into(),
            "-q".into(),
            "https://github.com/testorg/repo1.git".into(),
            s(&churn_root.join("repo1")),
        ],
        &test_env,
        true,
    );
    let churn = churn_root.join("repo1");
    git_c(&churn, &["config", "user.name", "Tester"], &test_env, true);
    git_c(
        &churn,
        &["config", "user.email", "test@example.com"],
        &test_env,
        true,
    );
    fs::write(churn.join("churn-local.txt"), "local valuable file\n").unwrap();
    let remote1 = remote.join("repo1.git");
    let remote_main = stdout(&git_dir(
        &remote1,
        &["rev-parse", "refs/heads/main"],
        &test_env,
        true,
    ));
    let churn_hook = remote1.join("hooks/post-receive");
    fs::write(
        &churn_hook,
        format!(
            "#!/bin/sh\nwhile read old new ref; do\n case \"$ref\" in\n refs/heads/sync/local/*) git update-ref refs/heads/unrelated-churn {remote_main};;\n esac\ndone\n"
        ),
    )
    .unwrap();
    set_exec(&churn_hook);

    let churn_run = execute(
        "bash",
        &[
            s(&sync_script),
            "testorg".into(),
            "--root".into(),
            s(&churn_root),
            "--repo".into(),
            "repo1".into(),
            "--checkpoint".into(),
            "--backup-remote".into(),
            "--verify".into(),
            "--report".into(),
        ],
        None,
        &test_env,
        false,
    );
    let churn_text = combined(&churn_run);
    assert!(churn_run.status.success(), "{churn_text}");
    assert!(churn_text.contains("CHURN: remote refs changed"));
    assert!(churn_text.contains("COMPLETE:"));
    assert_eq!(
        stdout(&git_dir(
            &remote1,
            &["rev-parse", "refs/heads/unrelated-churn"],
            &test_env,
            true,
        )),
        remote_main
    );
    fs::remove_file(&churn_hook).unwrap();
    println!("UNRELATED_REF_CHURN_OK");

    let future_source = b.join("race-future-seed");
    git(
        &[
            "clone".into(),
            "-q".into(),
            "https://github.com/testorg/repo2.git".into(),
            s(&future_source),
        ],
        &test_env,
        true,
    );
    git_c(
        &future_source,
        &["config", "user.name", "Tester"],
        &test_env,
        true,
    );
    git_c(
        &future_source,
        &["config", "user.email", "test@example.com"],
        &test_env,
        true,
    );
    git_c(
        &future_source,
        &[
            "switch",
            "-qc",
            "future-feature",
            "origin/unmerged-feature",
        ],
        &test_env,
        true,
    );
    fs::write(future_source.join("future.txt"), "concurrent feature update\n").unwrap();
    git_c(&future_source, &["add", "-A"], &test_env, true);
    git_c(
        &future_source,
        &["commit", "-qm", "future branch version"],
        &test_env,
        true,
    );
    let future_sha = stdout(&git_c(&future_source, &["rev-parse", "HEAD"], &test_env, true));
    git_c(
        &future_source,
        &["push", "-q", "origin", "HEAD:refs/heads/race-staging"],
        &test_env,
        true,
    );

    let race_root = b.join("target-churn");
    fs::create_dir(&race_root).unwrap();
    git(
        &[
            "clone".into(),
            "-q".into(),
            "https://github.com/testorg/repo2.git".into(),
            s(&race_root.join("repo2")),
        ],
        &test_env,
        true,
    );
    let race = race_root.join("repo2");
    git_c(&race, &["config", "user.name", "Tester"], &test_env, true);
    git_c(
        &race,
        &["config", "user.email", "test@example.com"],
        &test_env,
        true,
    );
    git_c(
        &race,
        &[
            "switch",
            "-qc",
            "unmerged-feature",
            "origin/unmerged-feature",
        ],
        &test_env,
        true,
    );
    fs::write(race.join("race-local.txt"), "local feature update\n").unwrap();
    git_c(&race, &["add", "-A"], &test_env, true);
    git_c(
        &race,
        &["commit", "-qm", "local feature version"],
        &test_env,
        true,
    );

    let remote2 = remote.join("repo2.git");
    let race_hook = remote2.join("hooks/post-receive");
    fs::write(
        &race_hook,
        format!(
            "#!/bin/sh\nwhile read old new ref; do\n case \"$ref\" in\n refs/heads/sync/local/*) git update-ref refs/heads/unmerged-feature {future_sha};;\n esac\ndone\n"
        ),
    )
    .unwrap();
    set_exec(&race_hook);

    let race_run = execute(
        "bash",
        &[
            s(&sync_script),
            "testorg".into(),
            "--root".into(),
            s(&race_root),
            "--repo".into(),
            "repo2".into(),
            "--checkpoint".into(),
            "--backup-remote".into(),
            "--verify".into(),
            "--report".into(),
        ],
        None,
        &test_env,
        false,
    );
    let race_text = combined(&race_run);
    assert!(race_run.status.success(), "{race_text}");
    assert!(race_text.contains("CHURN: remote refs changed"));
    assert!(race_text.contains("refs/heads/unmerged-feature"));
    assert!(race_text.contains("BACKED_UP:"));
    fs::remove_file(&race_hook).unwrap();
    assert!(
        git_c(
            &race,
            &["merge-base", "--is-ancestor", &future_sha, "unmerged-feature"],
            &test_env,
            false,
        )
        .status
        .success()
    );
    println!("TARGET_REF_CHURN_REFRESH_OK");

    let gate_root = b.join("backup-gate");
    fs::create_dir(&gate_root).unwrap();
    git(
        &[
            "clone".into(),
            "-q".into(),
            "https://github.com/testorg/repo2.git".into(),
            s(&gate_root.join("repo2")),
        ],
        &test_env,
        true,
    );
    let gated = gate_root.join("repo2");
    git_c(&gated, &["config", "user.name", "Tester"], &test_env, true);
    git_c(
        &gated,
        &["config", "user.email", "test@example.com"],
        &test_env,
        true,
    );
    fs::write(gated.join("gated-local.txt"), "must be preserved locally\n").unwrap();

    let pre_receive = remote2.join("hooks/pre-receive");
    fs::write(
        &pre_receive,
        "#!/bin/sh\nwhile read old new ref; do\n case \"$ref\" in refs/heads/sync/*) exit 1;; esac\ndone\nexit 0\n",
    )
    .unwrap();
    set_exec(&pre_receive);
    let previous = stdout(&git_dir(&remote2, &["rev-parse", "main"], &test_env, true));

    let gated_run = execute(
        "bash",
        &[
            s(&sync_script),
            "testorg".into(),
            "--root".into(),
            s(&gate_root),
            "--repo".into(),
            "repo2".into(),
            "--checkpoint".into(),
            "--backup-remote".into(),
            "--verify".into(),
        ],
        None,
        &test_env,
        false,
    );
    let gated_text = combined(&gated_run);
    assert!(!gated_run.status.success());
    assert!(gated_text.contains("local safety-ref push rejected"));
    assert_eq!(
        stdout(&git_dir(&remote2, &["rev-parse", "main"], &test_env, true)),
        previous
    );
    assert!(gated.join("gated-local.txt").exists());
    assert!(
        stdout(&git_c(
            &gated,
            &["log", "-1", "--pretty=%s"],
            &test_env,
            true,
        ))
        .starts_with("sync: checkpoint")
    );
    fs::remove_file(&pre_receive).unwrap();
    println!("FAILED_BACKUP_BLOCKS_INTEGRATION_OK");

    let cache_root = b.join("aux-clone");
    fs::create_dir(&cache_root).unwrap();
    git(
        &[
            "clone".into(),
            "-q".into(),
            "https://github.com/testorg/repo2.git".into(),
            s(&cache_root.join("repo2")),
        ],
        &test_env,
        true,
    );
    let parent = cache_root.join("some-dependent-repo/.cache");
    fs::create_dir_all(&parent).unwrap();
    let cache = parent.join("compiler-abcdef");
    git(
        &[
            "clone".into(),
            "-q".into(),
            "https://github.com/testorg/repo2.git".into(),
            s(&cache),
        ],
        &test_env,
        true,
    );
    git_c(&cache, &["switch", "--detach", "HEAD"], &test_env, true);
    let old_head = stdout(&git_c(&cache, &["rev-parse", "HEAD"], &test_env, true));

    let cache_run = execute(
        "bash",
        &[
            s(&sync_script),
            "testorg".into(),
            "--root".into(),
            s(&cache_root),
            "--repo".into(),
            "repo2".into(),
            "--checkpoint".into(),
            "--backup-remote".into(),
            "--verify".into(),
            "--report".into(),
        ],
        None,
        &test_env,
        false,
    );
    let cache_text = combined(&cache_run);
    assert!(cache_run.status.success(), "{cache_text}");
    assert_eq!(cache_text.matches("[repo2] START:").count(), 1);
    assert!(cache_text.contains("[repo2] SKIPPED: secondary clone"));
    assert_eq!(
        stdout(&git_c(&cache, &["rev-parse", "HEAD"], &test_env, true)),
        old_head
    );
    assert!(
        !git_c(
            &cache,
            &["symbolic-ref", "-q", "--short", "HEAD"],
            &test_env,
            false,
        )
        .status
        .success()
    );
    println!("CACHED_CLONE_NO_DUPLICATE_SYNC_OK");

    fs::write(cache.join("valuable.txt"), "unique uncommitted source\n").unwrap();
    let dirty_cache = execute(
        "bash",
        &[
            s(&sync_script),
            "testorg".into(),
            "--root".into(),
            s(&cache_root),
            "--repo".into(),
            "repo2".into(),
            "--checkpoint".into(),
            "--backup-remote".into(),
            "--verify".into(),
            "--report".into(),
        ],
        None,
        &test_env,
        false,
    );
    let dirty_text = combined(&dirty_cache);
    assert!(!dirty_cache.status.success());
    assert_eq!(dirty_text.matches("[repo2] START:").count(), 1);
    assert!(dirty_text.contains("[repo2] BLOCKED: secondary clone needs separate preservation:"));
    assert_eq!(
        fs::read_to_string(cache.join("valuable.txt")).unwrap(),
        "unique uncommitted source\n"
    );
    assert!(
        stdout(&git_c(
            &cache,
            &["ls-files", "valuable.txt"],
            &test_env,
            true,
        ))
        .is_empty()
    );
    println!("DIRTY_CACHED_CLONE_FAIL_CLOSED_OK");

    fs::remove_file(cache.join("valuable.txt")).unwrap();
    git_c(&cache, &["config", "user.name", "Tester"], &test_env, true);
    git_c(
        &cache,
        &["config", "user.email", "test@example.com"],
        &test_env,
        true,
    );
    fs::write(
        cache.join("committed-but-private.txt"),
        "unpublished commit\n",
    )
    .unwrap();
    git_c(&cache, &["add", "-A"], &test_env, true);
    git_c(
        &cache,
        &["commit", "-qm", "unpublished cache code"],
        &test_env,
        true,
    );
    let commit_sha = stdout(&git_c(&cache, &["rev-parse", "HEAD"], &test_env, true));

    let private_cache = execute(
        "bash",
        &[
            s(&sync_script),
            "testorg".into(),
            "--root".into(),
            s(&cache_root),
            "--repo".into(),
            "repo2".into(),
            "--checkpoint".into(),
            "--backup-remote".into(),
            "--verify".into(),
            "--report".into(),
        ],
        None,
        &test_env,
        false,
    );
    let private_text = combined(&private_cache);
    assert!(!private_cache.status.success());
    assert!(private_text.contains("[repo2] BLOCKED: secondary clone needs separate preservation:"));
    assert_eq!(
        stdout(&git_c(&cache, &["rev-parse", "HEAD"], &test_env, true)),
        commit_sha
    );
    assert!(private_text.contains("branch-or-HEAD-only-commits=1"));
    println!("UNPUBLISHED_CACHED_COMMIT_FAIL_CLOSED_OK");
}

fn main() {
    run_suite();
}

#[cfg(test)]
mod tests {
    #[test]
    fn sync_org_safety_regression() {
        super::run_suite();
    }
}
