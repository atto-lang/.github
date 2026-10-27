//! Source-free process-level tests of the *real* Rust collector executable.
//! Synthetic token only, fake gh CLI only: never access GitHub or private source.
#![cfg(unix)]
use std::{collections::BTreeMap, env, fs, os::unix::fs::PermissionsExt,
          path::PathBuf, process::{Command, Output}};

const COMMIT: &str="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const TREE: &str="bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

struct FakeGh {
    root: PathBuf,
    invocations: PathBuf,
}
impl FakeGh {
    fn create(name:&str) -> Self {
        let root=env::temp_dir().join(format!("ores-ci-fake-gh-{}-{}",std::process::id(),name));
        let _=fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let invocations=root.join("calls.txt");
        let bin=root.join("gh");
        fs::write(&bin,"#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$CI_GH_CALL_LOG\"\nexit 0\n").unwrap();
        let mut perms=fs::metadata(&bin).unwrap().permissions();
        perms.set_mode(0o700);
        fs::set_permissions(&bin,perms).unwrap();
        Self{root,invocations}
    }
    fn calls(&self)->String {
        fs::read_to_string(&self.invocations).unwrap_or_default()
    }
}
impl Drop for FakeGh {
    fn drop(&mut self) { let _=fs::remove_dir_all(&self.root); }
}
fn binary()->String {
    env::var("CI_EVIDENCE_BINARY").expect("CI_EVIDENCE_BINARY must be a compiled collector CLI path")
}
fn context()->BTreeMap<String,String> {
    [
        ("GITHUB_ACTIONS","true"),
        ("GITHUB_REF_PROTECTED","true"),
        ("GITHUB_EVENT_NAME","push"),
        ("GITHUB_REF","refs/heads/main"),
        ("GITHUB_REPOSITORY","trusted/source"),
        ("GITHUB_SHA",COMMIT),
        ("GITHUB_SERVER_URL","https://github.com"),
        ("GITHUB_API_URL","https://api.github.com"),
        ("GH_TOKEN","synthetic-invalid-token-no-api-requests"),
    ].into_iter().map(|(k,v)|(k.to_owned(),v.to_owned())).collect()
}
fn run(fake:&FakeGh,envs:&BTreeMap<String,String>)->Output {
    let system_path=env::var_os("PATH").unwrap_or_default();
    let path=format!("{}:{}",fake.root.display(),system_path.to_string_lossy());
    let mut cmd=Command::new(binary());
    cmd.env_clear().envs(envs).env("PATH",path)
        .env("CI_GH_CALL_LOG",&fake.invocations)
        .args(["trusted/source","main",COMMIT,TREE,"400","500","600","200","15368","test"]);
    cmd.output().expect("collector must start")
}
fn expect_guard(output:&Output) {
    assert_eq!(output.status.code(),Some(3));
    assert!(output.stdout.is_empty(),"collector printed untrusted success");
    let stderr=String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("authenticated protected source-local push context required"),
            "unexpected guard failure: {stderr}");
    assert!(!stderr.contains("synthetic-invalid-token"),"token unexpectedly logged");
}
#[test]
fn rejects_untrusted_events_refs_repositories_shas_and_hosts_without_api_calls() {
    let fake=FakeGh::create("negative-contexts");
    for (name,value) in [
        ("GITHUB_ACTIONS","false"),
        ("GITHUB_REF_PROTECTED","false"),
        ("GITHUB_EVENT_NAME","pull_request"),
        ("GITHUB_EVENT_NAME","workflow_dispatch"),
        ("GITHUB_REF","refs/pull/1/merge"),
        ("GITHUB_REF","refs/heads/feature"),
        ("GITHUB_REPOSITORY","other/private"),
        ("GITHUB_SHA","cccccccccccccccccccccccccccccccccccccccc"),
        ("GITHUB_SERVER_URL","https://example.org"),
        ("GITHUB_API_URL","https://api.not-github.example"),
        ("GH_HOST","another.example"),
    ] {
        let mut vars=context();vars.insert(name.into(),value.into());
        expect_guard(&run(&fake,&vars));
        assert!(fake.calls().is_empty(),"untrusted {name} reached gh API");
    }
}
#[test]
fn refuses_missing_or_empty_authentication_without_api_calls() {
    let fake=FakeGh::create("missing-token");
    for token in [None,Some("")] {
        let mut vars=context();
        match token { None=>{vars.remove("GH_TOKEN");},Some(v)=>{vars.insert("GH_TOKEN".into(),v.into());} }
        expect_guard(&run(&fake,&vars));
        assert!(fake.calls().is_empty(),"missing/empty token reached gh API");
    }
}
#[test]
fn synthetic_matching_context_can_read_only_attempt_but_never_certify() {
    let fake=FakeGh::create("get-only");
    let output=run(&fake,&context());
    assert_eq!(output.status.code(),Some(3));
    assert!(output.stdout.is_empty(),"mock data must never issue passing release proof");
    let stderr=String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("NOT_CERTIFIED"),"not fail-closed: {stderr}");
    assert!(!stderr.contains("synthetic-invalid-token"),"token unexpectedly logged");
    let calls=fake.calls();
    assert!(!calls.is_empty(),"positive synthetic guard never reached fake gh");
    assert!(calls.lines().all(|l|l.starts_with("api --method GET ")),
      "mutating/non-GET invocation observed: {calls}");
}
