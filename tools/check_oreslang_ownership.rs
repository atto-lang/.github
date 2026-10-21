// Dependency-free lexical gate; real compiler ownership checking is still required.
use std::{env,fs,path::{Path,PathBuf},process};
#[derive(Clone,Debug,PartialEq,Eq)]
struct Token { text:String, line:usize }
struct Lex<'a> {
    b: &'a [u8], i: usize, line: usize, out: Vec<Token>, errors: Vec<String>
}
impl<'a> Lex<'a> {
    fn new(s: &'a str)->Self {
        Self { b:s.as_bytes(), i:0, line:1, out:Vec::new(), errors:Vec::new() }
    }
    fn pair(&self,c:u8)->bool {self.b.get(self.i+1)==Some(&c)}
    fn token(&mut self,s:&str,line:usize){self.out.push(Token{text:s.into(),line});}
    fn quote(&mut self,q:u8){
        let start=self.line;
        let triple=q!=b'\x60' && self.b.get(self.i+1)==Some(&q) && self.b.get(self.i+2)==Some(&q);
        self.i+=if triple{3}else{1};
        while self.i<self.b.len(){
            let c=self.b[self.i];
            if !triple && c==b'\\'{
                if self.i+1>=self.b.len(){
                    self.errors.push(format!("{start}: incomplete quoted escape"));self.i+=1;return;
                }
                if self.b[self.i+1]==b'\n'{self.line+=1;}
                self.i+=2;continue;
            }
            if q==b'\x60' && c==b'$' && self.pair(b'{'){
                let line=self.line;
                self.i+=2;
                self.token("__tpl_start__",line);
                self.code(true);
                self.token("__tpl_end__",self.line);
                continue;
            }
            if triple && self.b.get(self.i..self.i+3)==Some([q,q,q].as_slice()){
                self.i+=3;self.token("<literal>",start);return;
            }
            if !triple && c==q{
                self.i+=1;self.token("<literal>",start);return;
            }
            if c==b'\n'{self.line+=1;}
            self.i+=1;
        }
        self.errors.push(format!("{start}: unterminated literal"));
    }
    fn comment(&mut self){
        let start=self.line;self.i+=2;let mut depth=1usize;
        while self.i<self.b.len(){
            if self.b[self.i]==b'\n'{self.line+=1;}
            if self.b[self.i]==b'/' && self.pair(b'*'){depth+=1;self.i+=2;}
            else if self.b[self.i]==b'*' && self.pair(b'/'){
                depth-=1;self.i+=2;if depth==0{return;}
            }else{self.i+=1;}
        }
        self.errors.push(format!("{start}: unterminated block comment"));
    }
    fn code(&mut self,interpolation:bool){
        let mut depth=0usize;
        while self.i<self.b.len(){
            let c=self.b[self.i];
            if c.is_ascii_whitespace(){
                if c==b'\n'{self.line+=1;}self.i+=1;continue;
            }
            if c==b'/' && self.pair(b'/'){
                self.i+=2;while self.i<self.b.len() && self.b[self.i]!=b'\n'{self.i+=1;}continue;
            }
            if c==b'/' && self.pair(b'*'){self.comment();continue;}
            if matches!(c,b'\''|b'"'|b'\x60'){self.quote(c);continue;}
            if interpolation && c==b'}'{
                self.i+=1;
                if depth==0{return;}
                depth-=1;self.token("}",self.line);continue;
            }
            if interpolation && c==b'{'{depth+=1;}
            let start=self.i;let line=self.line;
            if c.is_ascii_alphanumeric() || c==b'_'{
                self.i+=1;
                while self.i<self.b.len()
                    && (self.b[self.i].is_ascii_alphanumeric()||self.b[self.i]==b'_'){self.i+=1;}
            }else if (c==b'&' && self.pair(b'&')) || (c==b'*' && self.pair(b'*')){
                self.i+=2;
            }else{self.i+=1;}
            self.out.push(Token{
                text:String::from_utf8_lossy(&self.b[start..self.i]).into_owned(),line
            });
        }
        if interpolation{self.errors.push(format!("{}: unterminated interpolation",self.line));}
    }
}
fn lex(src:&str)->(Vec<Token>,Vec<String>){
    let mut scanner=Lex::new(src);scanner.code(false);
    (scanner.out,scanner.errors)
}
fn can_end_expr(s:&str)->bool{
    s==")" || s=="]" || s=="}" || s=="<literal>" || s=="__tpl_end__" ||
    (s.as_bytes().first().is_some_and(|b|b.is_ascii_alphanumeric() || *b==b'_') &&
    !matches!(s,"return"|"let"|"val"|"const"|"as"|"in"|"then"|"else"|"on"|"case"|"when"|"await"|"rt"|"borrow"|"mut"|"take"|"share"|"copy"|"__tpl_start__"))
}
fn violations(src:&str)->Vec<String>{
    let (ts,mut issues)=lex(src);
    for i in 0..ts.len(){
        let x=&ts[i];
        if x.text=="rt" && ts.get(i+1).is_some_and(|t|t.text=="borrow") && ts.get(i+2).is_some_and(|t|t.text=="mut"){
            issues.push(format!("{}: noncanonical 'rt borrow mut'; use 'rt mut borrow'",x.line));
        }
        if x.text=="rt" && ts.get(i+1).is_some_and(|t|matches!(t.text.as_str(),"ref"|"deref"|"unref"|"raw")){
            issues.push(format!("{}: raw pointer intrinsic is forbidden",x.line));
        }
        if matches!(x.text.as_str(),"Pointer"|"RawPointer") && ts.get(i+1).is_some_and(|t|t.text=="<"){
            issues.push(format!("{}: public pointer type is forbidden",x.line));
        }
        // Only classify these sigils in Oreslang source, not host-language code.
        if x.text=="*" && i>=2 {
            let before=ts[i-2].text.as_str();
            let name=ts[i-1].text.as_str();
            let after=ts.get(i+1).map_or("",|t|t.text.as_str());
            let is_type=name.as_bytes().first().is_some_and(|b|b.is_ascii_alphabetic());
            let in_parameter=matches!(before,"("|",") && {
                let mut depth=0usize;let mut opener=None;
                for j in (0..i-1).rev(){
                    if ts[j].text==")"{depth+=1;}
                    else if ts[j].text=="("{
                        if depth==0{opener=Some(j);break;}else{depth-=1;}
                    }
                }
                opener.is_some_and(|open|open>=2
                    && matches!(ts[open-2].text.as_str(),"fnc"|"method"|"routine"))
            });
            let in_declaration=matches!(before,"let"|"val"|"const"|":") || in_parameter;
            if is_type && in_declaration
                && (after.is_empty() || after=="{" || after=="," || after==")"
                    || after.as_bytes().first().is_some_and(|b|b.is_ascii_alphabetic())) {
                issues.push(format!("{}: source pointer type T* is forbidden",x.line));
                continue;
            }
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
        let kind=ent.file_type()?;
        if kind.is_symlink(){
            if matches!(p.extension().and_then(|x|x.to_str()),Some("ores"|"oreslang"|"ore")) || p.is_dir(){
                issues.push(format!("{}: symlinked guest source or directory cannot bypass source scan",p.display()));
            }
        }else if kind.is_dir(){
            if !matches!(name.as_ref(),".git"|"target"|"node_modules"|".gradle"|"build"|"dist"){walk(&p,issues)?;}
        }else if kind.is_file() && matches!(p.extension().and_then(|x|x.to_str()),Some("ores"|"oreslang"|"ore")){
            // Compile-fail fixtures are verified by each real compiler, not by this advisory gate.
            let in_tests=p.components().any(|c|matches!(c.as_os_str().to_str(),Some("tests"|"test"|"fixtures")));
            if in_tests && (name.contains(".negative.") || name.contains(".compile_fail.") || name.starts_with("unsupported-") || name.ends_with("-reject.ores")){continue;}
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
        for s in ["let y=&value;","return &mut value;","let y=*ptr;","foo(*ptr);","return rt borrow mut foo;","let p=Pointer<int>;","rt ref p;","rt deref p;","rt unref p;","rt raw p;","let Foo* raw;","let p=RawPointer<int>;"]{
            assert!(!violations(s).is_empty(),"{s}");
        }
    }
    #[test]fn comments_strings_binary_are_safe(){
        assert!(violations("/* &thing /* *other */ &nested */ // &nope\n let s=\"*ptr &x\"; return (a * b) & (c * d);").is_empty());
    }
    #[test]fn line_numbers(){assert!(violations("let a=1;\nreturn &a;")[0].starts_with("2:"));}
    #[test]fn logical_and(){assert!(violations("return x && y;").is_empty());}
    #[test]fn canonical_mutable_borrow(){assert!(violations("let loan=rt mut borrow value;").is_empty());}
    #[test]fn legacy_order_rejected(){assert!(violations("rt borrow mut value;")[0].contains("noncanonical"));}
    #[test]fn string_and_template_text_are_not_code(){assert!(violations("val quoted='&ptr'; val template=`*ptr &x`;").is_empty());}
    #[test]fn typed_pointer_detection(){assert!(violations("let Box* address;")[0].contains("pointer type"));}
    #[test]fn scan_reports_path(){let mut issues=Vec::new();inspect(Path::new("example.ores"),"return &foo;",&mut issues);assert!(issues[0].contains("example.ores:1:"));}

    #[test]fn rejects_pointer_types_in_function_signatures(){
        for source in [
            "fnc get(): Box* { return value; }",
            "fnc use(Box* arg): int { return 1; }",
            "fnc use2(int ok, Box* arg): int { return ok; }",
            "pub fnc use3(int a, int b, Box* arg): int { return a+b; }",
        ]{
            assert!(violations(source).iter().any(|s|s.contains("pointer type")),"{source}");
        }
    }
    #[test]fn detects_pointer_operations_in_template_interpolations(){
        let tick=char::from(96);
        let dollar=char::from(36);
        let src=format!("let s={tick}{dollar}{{ &ptr }}{tick};");
        assert!(!violations(&src).is_empty());
        let nested=format!("let s={tick}{dollar}{{ fnc(){{return rt ref ptr;}} }}{tick};");
        assert!(!violations(&nested).is_empty());
        let escaped=format!("let s={tick}\\{dollar}{{ &ptr }}{tick};");
        assert!(violations(&escaped).is_empty());
    }
    #[test]fn rejects_unterminated_comments_and_literals(){
        for source in ["/* unterminated","\"unterminated","'unterminated"]{
            assert!(!violations(source).is_empty(),"{source}");
        }
        let tick=char::from(96);
        assert!(!violations(&format!("let s={tick}unterminated")).is_empty());
    }
    #[test]fn binary_ops_remain_valid_inside_templates(){
        let tick=char::from(96);
        let dollar=char::from(36);
        let src=format!("let s={tick}calc {dollar}{{ a*b & c }}{tick};");
        assert!(violations(&src).is_empty(),"{:?}",violations(&src));
    }
    #[cfg(unix)]
    #[test]fn source_symlink_is_reported(){
        use std::os::unix::fs::symlink;
        let root=env::temp_dir().join(format!("ores-ownership-check-{}",process::id()));
        fs::create_dir_all(&root).unwrap();
        let target=root.join("source.ores");
        fs::write(&target,"let x=1;").unwrap();
        symlink(&target,root.join("alias.ores")).unwrap();
        let mut issues=Vec::new();
        walk(&root,&mut issues).unwrap();
        assert!(issues.iter().any(|s|s.contains("symlinked guest source")));
        fs::remove_dir_all(&root).unwrap();
    }

}