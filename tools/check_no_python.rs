#[cfg(not(test))]
use std::env;
use std::{fs, io, path::{Path, PathBuf}};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Finding { path: PathBuf, detail: String }

fn name(p: &Path) -> String {
    p.file_name().and_then(|x| x.to_str()).unwrap_or("").to_ascii_lowercase()
}

fn artifact(p: &Path) -> bool {
    let n=name(p);
    n.ends_with(".py") || n.ends_with(".pyi") || n.ends_with(".ipynb")
        || matches!(n.as_str(),"pipfile"|"pipfile.lock"|"poetry.lock"|"pytest.ini"|"tox.ini"|"setup.py"|"pyproject.toml"|"uv.lock"|".python-version")
        || n=="requirements.txt" || (n.starts_with("requirements-") && n.ends_with(".txt"))
        || n=="__pycache__" || n.ends_with(".egg-info")
}

fn doc(p:&Path)->bool {
    matches!(p.extension().and_then(|x|x.to_str()).map(|x|x.to_ascii_lowercase()).as_deref(),Some("md"|"mdx"|"rst"|"adoc"))
}

fn text_candidate(p:&Path)->bool {
    let n=name(p);
    if matches!(n.as_str(),"makefile"|"gnumakefile"|"cmakelists.txt"|"dockerfile"|"justfile"|"taskfile"){return true}
    matches!(p.extension().and_then(|x|x.to_str()).map(|x|x.to_ascii_lowercase()).as_deref(),
        Some("sh"|"bash"|"zsh"|"yml"|"yaml"|"toml"|"json"|"jsonc"|"xml"|"gradle"|"kts"|"rs"|"java"|"kt"|"c"|"cc"|"cpp"|"h"|"hpp"|"js"|"mjs"|"cjs"|"ts"|"tsx"|"jsx"|"cs"|"ps1"|"cmd"|"bat"|"cmake"|"mk"|"go"|"swift"|"scala"))
}

fn base(t:&str)->&str { t.rsplit('/').next().unwrap_or(t).trim_end_matches(".exe") }
fn pyexec(b:&str)->bool {
    if matches!(b,"python"|"python2"|"python3"){return true}
    b.strip_prefix("python").is_some_and(|r|!r.is_empty()&&r.chars().all(|c|c.is_ascii_digit()||c=='.'))
}
fn pipexec(b:&str)->bool {
    if matches!(b,"pip"|"pip2"|"pip3"|"pipenv"){return true}
    b.strip_prefix("pip").is_some_and(|r|!r.is_empty()&&r.chars().all(|c|c.is_ascii_digit()||c=='.'))
}

fn tokens(line:&str)->Vec<String>{
    line.split(|c:char|c.is_whitespace()||matches!(c,'|'|'&'|';'|'('|')'|'{'|'}'|'['|']'|'='|':'|','|'"'|'\''))
        .filter(|x|!x.is_empty()).map(|x|x.trim_matches('$').to_ascii_lowercase()).collect()
}

fn invocation(line:&str)->Option<&'static str>{
    let t=line.trim();
    if t.is_empty() || ((t.starts_with('#')&&!t.starts_with("#!"))||t.starts_with("//")){return None}
    let l=t.to_ascii_lowercase();
    if l.contains("actions/setup-python@")||l.contains("python-version:"){return Some("workflow config")}
    if l.starts_with("#!")&&l.contains("python"){return Some("shebang")}
    if l.contains("find_package(python")||l.contains("python3_add_")||l.contains("python_add_"){return Some("build dependency")}
    if l.contains("pythonpath")||l.contains("pythonhome"){return Some("runtime environment")}
    if ["command::new(","processbuilder(","runtime.getruntime().exec(","process.start(","child_process","spawn("].iter().any(|x|l.contains(x))
        && ["\"python\"","\"python3\"","'python'","'python3'","\"pip\"","\"pytest\""].iter().any(|x|l.contains(x)){return Some("host process launch")}
    let v=tokens(t);
    let wrap=|s:&str|matches!(s,"run"|"shell"|"command"|"exec"|"spawn"|"env"|"sudo"|"time"|"xargs");
    for i in 0..v.len(){
        let b=base(&v[i]); let pos=i==0||v.get(i.wrapping_sub(1)).is_some_and(|p|wrap(base(p)));
        if pos&&pyexec(b){return Some("interpreter")}
        if pos&&pipexec(b){return Some("package manager")}
        if pos&&matches!(b,"pytest"|"poetry"){return Some("tool")}
        if pos&&b=="uv"&&v.get(i+1).is_some_and(|n|matches!(n.as_str(),"run"|"pip"|"sync"|"lock"|"venv")){return Some("uv tool")}
        if pos&&(b.ends_with(".py")||b.ends_with(".pyi")){return Some("script reference")}
    }
    None
}

fn inspect(root:&Path,p:&Path,out:&mut Vec<Finding>)->io::Result<()>{
    let rel=p.strip_prefix(root).unwrap_or(p);
    if rel!=Path::new("")&&artifact(rel){out.push(Finding{path:rel.to_path_buf(),detail:"Python artifact".into()})}
    let m=fs::symlink_metadata(p)?;
    if m.file_type().is_symlink(){return Ok(())}
    if m.is_dir(){
        if p!=root&&name(p)==".git"{return Ok(())}
        for e in fs::read_dir(p)?{inspect(root,&e?.path(),out)?}
        return Ok(())
    }
    if !m.is_file(){return Ok(())}
    if rel.to_string_lossy().replace('\\',"/")=="tools/check_no_python.rs"{return Ok(())}
    if !text_candidate(p)&&!doc(p){return Ok(())}
    let Ok(s)=String::from_utf8(fs::read(p)?) else{return Ok(())};
    if doc(p){
        let mut fenced=false;
        for (i,line) in s.lines().enumerate(){
            let b=line.trim_start().as_bytes();
            if b.starts_with(&[96,96,96])||line.trim_start().starts_with("~~~"){fenced=!fenced;continue}
            let low=line.to_ascii_lowercase();
            if (fenced&&invocation(line).is_some()) || (!fenced&&(low.contains("requires")||low.contains("requirement")||low.contains("dependency"))&&low.contains("python")){
                out.push(Finding{path:rel.to_path_buf(),detail:format!("Python dependency/command at line {}",i+1)});break
            }
        }
    }else{
        for (i,line) in s.lines().enumerate(){if let Some(k)=invocation(line){out.push(Finding{path:rel.to_path_buf(),detail:format!("{k} at line {}",i+1)});break}}
    }
    Ok(())
}

fn run(root:&Path)->io::Result<Vec<Finding>>{
    let mut v=Vec::new(); inspect(root,root,&mut v)?;
    v.sort_by(|a,b|a.path.cmp(&b.path).then(a.detail.cmp(&b.detail)));v.dedup();Ok(v)
}

#[cfg(not(test))]
fn main(){
    let root=env::args_os().nth(1).map(PathBuf::from).unwrap_or_else(||PathBuf::from("."));
    match run(&root){
        Ok(v) if v.is_empty()=>println!("PASS: no Python artifacts, dependencies, or execution paths detected"),
        Ok(v)=>{eprintln!("Python is prohibited. Port required functionality to Rust:");for f in v{eprintln!("  {}: {}",f.path.display(),f.detail)}std::process::exit(1)}
        Err(e)=>{eprintln!("FAIL-CLOSED: incomplete scan: {e}");std::process::exit(2)}
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test] fn paths(){for p in ["x.py","x.pyi","x.ipynb","pyproject.toml","requirements-dev.txt"]{assert!(artifact(Path::new(p)))}assert!(!artifact(Path::new("main.rs")))}
    #[test] fn commands(){for l in ["python3 x.py","uses: actions/setup-python@v5","pip install x","pytest -q","poetry install","uv run x.py","let x=Command::new(\"python\");","find_package(Python3 REQUIRED)","#!/usr/bin/env python3"]{assert!(invocation(l).is_some(),"{l}")}}
    #[test] fn labels(){assert!(invocation("name: Rust-only / no Python").is_none());assert!(invocation("# Python is prohibited").is_none());assert!(invocation("echo pythonic").is_none())}
    #[test] fn full_scan(){let r=std::env::temp_dir().join(format!("no-python-{}",std::process::id()));let _=fs::remove_dir_all(&r);fs::create_dir_all(&r).unwrap();fs::write(r.join("legacy.py"),"legacy").unwrap();assert!(!run(&r).unwrap().is_empty());fs::remove_dir_all(r).unwrap()}
}
