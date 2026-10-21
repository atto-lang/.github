//! Dependency-free lexical gate; real compiler ownership checking is still required.
use std::{env,fs,path::{Path,PathBuf},process};
#[derive(Clone,Debug,PartialEq,Eq)]
struct Token { text:String, line:usize }
fn lex(src:&str)->Vec<Token> {
    let b=src.as_bytes();let mut i=0;let mut line=1;let mut out=Vec::new();
    while i<b.len(){
        let c=b[i];
        if c==b'\n'{line+=1;i+=1;continue;}
        if c.is_ascii_whitespace(){i+=1;continue;}
        if i+1<b.len() && c==b'/' && b[i+1]==b'/' {
            i+=2;while i<b.len() && b[i]!=b'\n'{i+=1;}continue;
        }
        if i+1<b.len() && c==b'/' && b[i+1]==b'*'{
            i+=2;let mut depth=1;
            while i<b.len() && depth>0{
                if b[i]==b'\n'{line+=1;}
                if i+1<b.len() && b[i]==b'/' && b[i+1]==b'*'{depth+=1;i+=2;}
                else if i+1<b.len() && b[i]==b'*' && b[i+1]==b'/'{depth-=1;i+=2;}
                else{i+=1;}
            }
            continue;
        }
        if c==b'\'' || c==b'"'{
            let triple=i+2<b.len() && b[i+1]==c && b[i+2]==c;
            i+=if triple{3}else{1};
            while i<b.len(){
                if b[i]==b'\n'{line+=1;}
                if b[i]==b'\\' && !triple{i=(i+2).min(b.len());continue;}
                if triple && i+2<b.len() && b[i]==c && b[i+1]==c && b[i+2]==c{i+=3;break;}
                if !triple && b[i]==c{i+=1;break;}
                i+=1;
            }
            out.push(Token{text:"<literal>".into(),line});continue;
        }
        let start=i;
        if c.is_ascii_alphabetic() || c==b'_'{
            i+=1;while i<b.len() && (b[i].is_ascii_alphanumeric() || b[i]==b'_'){i+=1;}
        }else if c.is_ascii_digit(){
            i+=1;while i<b.len() && (b[i].is_ascii_alphanumeric() || b[i]==b'_'){i+=1;}
        }else if i+1<b.len() && ((c==b'&' && b[i+1]==b'&') || (c==b'*' && b[i+1]==b'*')){
            i+=2;
        }else{i+=1;}
        out.push(Token{text:String::from_utf8_lossy(&b[start..i]).into_owned(),line});
    }
    out
}
fn can_end_expr(s:&str)->bool{
    s==")" || s=="]" || s=="}" || s=="<literal>" ||
    (s.as_bytes().first().is_some_and(|b|b.is_ascii_alphanumeric() || *b==b'_') &&
    !matches!(s,"return"|"let"|"val"|"const"|"as"|"in"|"then"|"else"|"on"|"case"|"when"|"await"|"rt"|"borrow"|"mut"|"take"|"share"|"copy"))
}
fn violations(src:&str)->Vec<String>{
    let ts=lex(src);let mut issues=Vec::new();
    for i in 0..ts.len(){
        let x=&ts[i];
        if x.text=="rt" && ts.get(i+1).is_some_and(|t|t.text=="borrow") && ts.get(i+2).is_some_and(|t|t.text=="mut"){
            issues.push(format!("{}: noncanonical 'rt borrow mut'; use 'rt mut borrow'",x.line));
        }
        if x.text=="rt" && ts.get(i+1).is_some_and(|t|matches!(t.text.as_str(),"ref"|"deref"|"unref")){
            issues.push(format!("{}: raw pointer intrinsic is forbidden",x.line));
        }
        if matches!(x.text.as_str(),"Pointer"|"RawPointer") && ts.get(i+1).is_some_and(|t|t.text=="<"){
            issues.push(format!("{}: public pointer type is forbidden",x.line));
        }
        if x.text!="&" && x.text!="*"{continue;}
        if i>0 && can_end_expr(&ts[i-1].text){continue;}
        let next=ts.get(i+1).map_or("",|t|t.text.as_str());
        if next.is_empty() || next==";" || next=="}"{continue;}
        let msg=if x.text=="&"{"borrow/address '&' forbidden; use 'rt borrow' or 'rt mut borrow'"}else{"pointer dereference '*' forbidden; use explicit rt ownership operations"};
        issues.push(format!("{}: {}",x.line,msg));
    }
    issues
}
fn inspect(p:&Path,src:&str,issues:&mut Vec<String>){
    for v in violations(src){issues.push(format!("{}:{}",p.display(),v));}
}
fn walk(dir:&Path,issues:&mut Vec<String>)->std::io::Result<()>{
    for ent in fs::read_dir(dir)?{
        let ent=ent?;let p=ent.path();let name=ent.file_name();let name=name.to_string_lossy();
        if p.is_dir(){
            if !matches!(name.as_ref(),".git"|"target"|"node_modules"|".gradle"|"build"|"dist"){walk(&p,issues)?;}
        }else if p.is_file() && matches!(p.extension().and_then(|x|x.to_str()),Some("ores"|"oreslang"|"ore")){
            if name.contains(".negative.") || name.contains(".compile_fail."){continue;}
            inspect(&p,&fs::read_to_string(&p)?,issues);
        }
    }
    Ok(())
}
fn main(){
    let args:Vec<String>=env::args().skip(1).collect();
    if args.is_empty(){eprintln!("usage: check_oreslang_ownership PATH [PATH...]");process::exit(2);}
    let mut issues=Vec::new();
    for a in args{
        let p=PathBuf::from(a);
        let result=if p.is_dir(){walk(&p,&mut issues)}
            else if p.is_file(){fs::read_to_string(&p).map(|s|inspect(&p,&s,&mut issues))}
            else{eprintln!("not found: {}",p.display());process::exit(2)};
        if let Err(e)=result{eprintln!("scan error: {}: {e}",p.display());process::exit(2);}
    }
    if !issues.is_empty(){for issue in &issues{eprintln!("{issue}");}process::exit(1);}
    println!("Oreslang pointerless lexical gate: PASS");
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn canonical_and_arithmetic(){
        assert!(violations("const x=rt borrow owner; let y=rt mut borrow owner; rt copy x; rt share x; rt take x; return a * b & c;").is_empty());
    }
    #[test]fn pointer_operations_rejected(){
        for s in ["let y=&value;","return &mut value;","let y=*ptr;","foo(*ptr);","return rt borrow mut foo;","let p=Pointer<int>;","rt ref p;","rt deref p;","rt unref p;"]{
            assert!(!violations(s).is_empty(),"{s}");
        }
    }
    #[test]fn comments_strings_binary_are_safe(){
        assert!(violations("/* &thing /* *other */ &nested */ // &nope\n let s=\"*ptr &x\"; return (a * b) & (c * d);").is_empty());
    }
    #[test]fn line_numbers(){assert!(violations("let a=1;\nreturn &a;")[0].starts_with("2:"));}
    #[test]fn logical_and(){assert!(violations("return x && y;").is_empty());}
}