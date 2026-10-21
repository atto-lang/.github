use std::{env,fs,path::{Path,PathBuf}};
fn scan(root:&Path, p:&Path, bad:&mut Vec<PathBuf>) {
    let Ok(meta)=fs::symlink_metadata(p) else {return};
    if meta.file_type().is_symlink(){return}
    if meta.is_dir(){
        if p!=root {
            if let Some(s)=p.file_name().and_then(|x|x.to_str()){
                if [".git","target","node_modules",".venv","venv","vendor","dist","build"].contains(&s){return}
            }
        }
        if let Ok(entries)=fs::read_dir(p){
            for entry in entries.flatten(){scan(root,&entry.path(),bad)}
        }
    }else if meta.is_file(){
        let name=p.file_name().and_then(|x|x.to_str()).unwrap_or("");
        let lower=name.to_ascii_lowercase();
        if lower.ends_with(".py")||lower.ends_with(".pyi")||lower.ends_with(".ipynb")
          ||["requirements.txt","pipfile","pipfile.lock","poetry.lock","pytest.ini","tox.ini","setup.py","pyproject.toml","uv.lock"].contains(&lower.as_str()) {
            bad.push(p.strip_prefix(root).unwrap_or(p).to_path_buf());
        }
    }
}
fn main(){
    let root=env::args_os().nth(1).map(PathBuf::from).unwrap_or_else(||PathBuf::from("."));
    let mut bad=Vec::new();scan(&root,&root,&mut bad);bad.sort();
    if !bad.is_empty(){
       eprintln!("Python artifacts prohibited. Port required functionality to Rust:");
       for p in &bad{eprintln!("  {}",p.display())}
       std::process::exit(1)
    }
    println!("PASS: no Python source, notebooks or Python package manifests");
}
