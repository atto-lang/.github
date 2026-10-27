//! Source-local native execution witness. NO release certification.
//! Pin this executable from an independently protected CI harness, not a PR.
//! This witness verifies the checked-out Git identity and observes an actual
//! native-suite process. It cannot prove that test program semantics are sound.
#[cfg(not(test))]
use std::{env, process::{Command, ExitCode}};

#[derive(Clone, Debug)]
struct Target<'a> {
    repository: &'a str,
    repo_id: u64,
    commit: &'a str,
    tree: &'a str,
    run_id: u64,
    attempt: u64,
    suite: &'a str,
}

fn oid(s: &str) -> bool {
    s.len() == 40 && s.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        && s.bytes().any(|b| b != b'0')
}
fn safe_slug(s: &str) -> bool {
    let x:Vec<_>=s.split('/').collect();
    x.len()==2 && x.iter().all(|v|!v.is_empty() && v.len()<=100
        && *v!="." && *v!=".." && v.bytes().all(|c|
        c.is_ascii_alphanumeric() || matches!(c,b'_'|b'-'|b'.')))
}
fn safe_suite(s:&str)->bool {
    !s.is_empty() && s.len()<=64 &&
    s.bytes().all(|c|c.is_ascii_alphanumeric() || matches!(c,b'_'|b'-'|b'.'))
}
#[cfg(not(test))]
fn numeric(s:&str)->Result<u64,&'static str> {
    if s.is_empty() || !s.bytes().all(|c|c.is_ascii_digit()) {
        return Err("invalid numeric identifier");
    }
    s.parse::<u64>().map_err(|_|"numeric identifier overflow")
}
fn trust_context(target:&Target<'_>, get:&impl Fn(&str)->Option<String>)->bool {
    let eq=|k:&str,v:&str|get(k).as_deref()==Some(v);
    safe_slug(target.repository) && oid(target.commit) && oid(target.tree)
        && safe_suite(target.suite)
        && target.repo_id!=0 && target.run_id!=0 && target.attempt!=0
        && eq("GITHUB_ACTIONS","true")
        && eq("GITHUB_EVENT_NAME","push")
        && eq("GITHUB_REF","refs/heads/main")
        && eq("GITHUB_REF_PROTECTED","true")
        && eq("GITHUB_REPOSITORY",target.repository)
        && eq("GITHUB_REPOSITORY_ID",&target.repo_id.to_string())
        && eq("GITHUB_SHA",target.commit)
        && eq("GITHUB_RUN_ID",&target.run_id.to_string())
        && eq("GITHUB_RUN_ATTEMPT",&target.attempt.to_string())
        && eq("GITHUB_JOB",target.suite)
        && eq("GITHUB_SERVER_URL","https://github.com")
        && eq("GITHUB_API_URL","https://api.github.com")
}
trait NativeHost {
    fn git(&self,argv:&[&str])->Result<String,&'static str>;
    fn suite(&self,argv:&[&str])->Result<i32,&'static str>;
}
#[cfg(not(test))]
struct LocalHost;
#[cfg(not(test))]
impl NativeHost for LocalHost {
    fn git(&self,argv:&[&str])->Result<String,&'static str> {
        let v=Command::new("/usr/bin/git").args(argv).output()
            .map_err(|_|"git unavailable")?;
        if !v.status.success() {return Err("git command failed");}
        let s=String::from_utf8(v.stdout).map_err(|_|"git output malformed")?;
        if s.len()>16384 || s.contains('\0') {return Err("unsafe git output");}
        Ok(s.trim().to_string())
    }
    fn suite(&self,argv:&[&str])->Result<i32,&'static str> {
        let (bin,args)=argv.split_first().ok_or("missing native test binary")?;
        if !bin.starts_with('/') || bin.as_bytes().contains(&0) {
            return Err("suite executable must be an absolute path");
        }
        Command::new(bin).args(args).status().map_err(|_|"native test could not launch")?
            .code().ok_or("native test terminated without exit code")
    }
}
fn checkpoint(host:&impl NativeHost, e:&Target<'_>)->Result<(),&'static str> {
    let commit=host.git(&["rev-parse","--verify","HEAD^{commit}"])?;
    if commit!=e.commit {return Err("checkout commit mismatch");}
    let tree=host.git(&["rev-parse","--verify","HEAD^{tree}"])?;
    if tree!=e.tree {return Err("checkout root tree mismatch");}
    // Tracked changes would mean the suite is not running the expected tree.
    // Untracked build outputs are allowed; they do not prove source isolation.
    if !host.git(&["status","--porcelain","--untracked-files=no"])?.is_empty(){
        return Err("checkout tracked files changed");
    }
    Ok(())
}
fn observe(host:&impl NativeHost, e:&Target<'_>, argv:&[&str],
           get:&impl Fn(&str)->Option<String>)->Result<(),&'static str> {
    if !trust_context(e,get){return Err("untrusted protected source-local context");}
    if argv.is_empty(){return Err("missing native suite executable");}
    checkpoint(host,e)?;
    let exit=host.suite(argv)?;
    // Always recheck Git even on test failure; no stale checkout can pass.
    checkpoint(host,e)?;
    if exit!=0 {return Err("native suite failed");}
    Ok(())
}
#[cfg(not(test))]
fn main()->ExitCode {
    let a:Vec<String>=env::args().skip(1).collect();
    let delimiter=a.iter().position(|v|v=="--");
    if delimiter!=Some(7) || a.len()<9 {
        eprintln!("NOT_CERTIFIED: usage: <repo> <repo_id> <commit> <tree> <run_id> <run_attempt> <suite_job_id> -- </absolute/test-binary> [args]");
        return ExitCode::from(3);
    }
    let target=||->Result<Target<'_>,&'static str>{
        Ok(Target{
            repository:&a[0],repo_id:numeric(&a[1])?,commit:&a[2],tree:&a[3],
            run_id:numeric(&a[4])?,attempt:numeric(&a[5])?,suite:&a[6],
        })
    };
    let result=target().and_then(|e|observe(&LocalHost,&e,
        &a[8..].iter().map(String::as_str).collect::<Vec<_>>(),
        &|k|env::var(k).ok()));
    match result {
        Ok(())=>{
            println!("OBSERVED_NATIVE_ONLY: suite process exited zero, checkout identity matched; NOT release-certified");
            ExitCode::from(3)
        }
        Err(err)=>{
            eprintln!("NOT_CERTIFIED: {err}");
            ExitCode::from(3)
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::{Cell,RefCell},collections::BTreeMap};
    const C:&str="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const T:&str="bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    fn target()->Target<'static>{
        Target{repository:"trusted/source",repo_id:901,commit:C,tree:T,run_id:400,attempt:1,suite:"native_suite"}
    }
    fn vars()->BTreeMap<String,String>{
        [
            ("GITHUB_ACTIONS","true"),("GITHUB_EVENT_NAME","push"),
            ("GITHUB_REF","refs/heads/main"),("GITHUB_REF_PROTECTED","true"),
            ("GITHUB_REPOSITORY","trusted/source"),("GITHUB_REPOSITORY_ID","901"),
            ("GITHUB_SHA",C),("GITHUB_RUN_ID","400"),("GITHUB_RUN_ATTEMPT","1"),
            ("GITHUB_JOB","native_suite"),("GITHUB_SERVER_URL","https://github.com"),
            ("GITHUB_API_URL","https://api.github.com")
        ].iter().map(|(k,v)|(k.to_string(),v.to_string())).collect()
    }
    struct Mock {
        commits:RefCell<Vec<String>>,
        trees:RefCell<Vec<String>>,
        dirty:RefCell<Vec<String>>,
        exit:Cell<i32>,git_reads:Cell<usize>,executions:Cell<usize>
    }
    impl Mock {
        fn good()->Self{
            Self{commits:RefCell::new(vec![C.into(),C.into()]),
              trees:RefCell::new(vec![T.into(),T.into()]),
              dirty:RefCell::new(vec!["".into(),"".into()]),
              exit:Cell::new(0),git_reads:Cell::new(0),executions:Cell::new(0)}
        }
        fn pop(q:&RefCell<Vec<String>>)->Result<String,&'static str>{
            let mut b=q.borrow_mut();if b.is_empty(){return Err("mock underflow");}Ok(b.remove(0))
        }
    }
    impl NativeHost for Mock{
        fn git(&self,args:&[&str])->Result<String,&'static str>{
            self.git_reads.set(self.git_reads.get()+1);
            match args {
                ["rev-parse","--verify","HEAD^{commit}"]=>Self::pop(&self.commits),
                ["rev-parse","--verify","HEAD^{tree}"]=>Self::pop(&self.trees),
                ["status","--porcelain","--untracked-files=no"]=>Self::pop(&self.dirty),
                _=>Err("unapproved git command")
            }
        }
        fn suite(&self,_:&[&str])->Result<i32,&'static str>{
            self.executions.set(self.executions.get()+1);Ok(self.exit.get())
        }
    }
    fn check(h:&Mock,t:&Target<'_>,v:&BTreeMap<String,String>)->Result<(),&'static str>{
        observe(h,t,&["/opt/trusted/run_native"],&|k|v.get(k).cloned())
    }
    #[test]fn observes_source_local_suite_without_certifying(){
        let h=Mock::good();assert_eq!(check(&h,&target(),&vars()),Ok(()));
        assert_eq!(h.executions.get(),1);assert_eq!(h.git_reads.get(),6);
    }
    #[test]fn rejects_invalid_identity_without_execution(){
        let mut t=target();let h=Mock::good();t.commit="not-a-sha";
        assert!(check(&h,&t,&vars()).is_err());assert_eq!(h.executions.get(),0);
        t=target();t.repo_id=902;
        assert!(check(&h,&t,&vars()).is_err());assert_eq!(h.git_reads.get(),0);
    }
    #[test]fn rejects_untrusted_event_or_runner_without_git_calls(){
        for (k,v) in [("GITHUB_EVENT_NAME","pull_request"),("GITHUB_REF","refs/pull/1/merge"),
            ("GITHUB_REF_PROTECTED","false"),("GITHUB_SHA","deadbeef"),
            ("GITHUB_REPOSITORY","foreign/private"),("GITHUB_REPOSITORY_ID","902"),
            ("GITHUB_RUN_ATTEMPT","2"),("GITHUB_JOB","other"),
            ("GITHUB_API_URL","https://wrong.example")] {
            let mut m=vars();m.insert(k.into(),v.into());let h=Mock::good();
            assert!(check(&h,&target(),&m).is_err(),"guard bypass for {k}");
            assert_eq!(h.git_reads.get(),0);
            assert_eq!(h.executions.get(),0);
        }
    }
    #[test]fn denies_wrong_commit_before_suite(){
        let h=Mock::good();h.commits.borrow_mut()[0]="cccccccccccccccccccccccccccccccccccccccc".into();
        assert_eq!(check(&h,&target(),&vars()),Err("checkout commit mismatch"));
        assert_eq!(h.executions.get(),0);
    }
    #[test]fn denies_wrong_tree_before_suite(){
        let h=Mock::good();h.trees.borrow_mut()[0]="cccccccccccccccccccccccccccccccccccccccc".into();
        assert_eq!(check(&h,&target(),&vars()),Err("checkout root tree mismatch"));
        assert_eq!(h.executions.get(),0);
    }
    #[test]fn denies_dirty_checkout_before_suite(){
        let h=Mock::good();h.dirty.borrow_mut()[0]=" M src/main.ores".into();
        assert_eq!(check(&h,&target(),&vars()),Err("checkout tracked files changed"));
        assert_eq!(h.executions.get(),0);
    }
    #[test]fn denies_suite_failure_even_with_clean_tree(){
        let h=Mock::good();h.exit.set(7);
        assert_eq!(check(&h,&target(),&vars()),Err("native suite failed"));
        assert_eq!(h.executions.get(),1);
    }
    #[test]fn detects_checkout_swap_after_suite(){
        let h=Mock::good();h.trees.borrow_mut()[1]="dddddddddddddddddddddddddddddddddddddddd".into();
        assert_eq!(check(&h,&target(),&vars()),Err("checkout root tree mismatch"));
        assert_eq!(h.executions.get(),1);
    }
    #[test]fn detects_tracked_modification_after_suite(){
        let h=Mock::good();h.dirty.borrow_mut()[1]=" M src/main.ores".into();
        assert_eq!(check(&h,&target(),&vars()),Err("checkout tracked files changed"));
        assert_eq!(h.executions.get(),1);
    }
    #[test]fn missing_suite_does_not_run(){
        let h=Mock::good();assert_eq!(observe(&h,&target(),&[],&|k|vars().get(k).cloned()),
            Err("missing native suite executable"));assert_eq!(h.executions.get(),0);
    }
    #[test]fn missing_environment_id_fails_closed(){
        let h=Mock::good();let mut m=vars();m.remove("GITHUB_RUN_ID");
        assert!(check(&h,&target(),&m).is_err());assert_eq!(h.executions.get(),0);
    }
}
