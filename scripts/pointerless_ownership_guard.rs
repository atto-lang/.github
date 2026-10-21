// Compatibility path: execute the exact canonical Rust checker.
// Retained older implementation is compile-time disabled.
include!("../tools/check_oreslang_ownership.rs");
#[cfg(any())]
mod legacy {
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
struct Token {
    text: String,
    line: usize,
}

fn strip_comments_and_strings(input: &str) -> String {
    let b = input.as_bytes();
    let mut out = vec![b' '; b.len()];
    let mut i = 0usize;
    let mut line_comment = false;
    let mut block_depth = 0usize;
    let mut string = false;
    let mut quote = b'"';
    let mut escape = false;

    while i < b.len() {
        if line_comment {
            if b[i] == b'\n' {
                line_comment = false;
                out[i] = b'\n';
            }
            i += 1;
            continue;
        }
        if block_depth > 0 {
            if i + 1 < b.len() && b[i] == b'/' && b[i + 1] == b'*' {
                block_depth += 1;
                i += 2;
                continue;
            }
            if i + 1 < b.len() && b[i] == b'*' && b[i + 1] == b'/' {
                block_depth -= 1;
                i += 2;
                continue;
            }
            if b[i] == b'\n' {
                out[i] = b'\n';
            }
            i += 1;
            continue;
        }
        if string {
            if b[i] == b'\n' {
                out[i] = b'\n';
            }
            if escape {
                escape = false;
            } else if b[i] == b'\\' {
                escape = true;
            } else if b[i] == quote {
                string = false;
            }
            i += 1;
            continue;
        }

        if i + 1 < b.len() && b[i] == b'/' && b[i + 1] == b'/' {
            line_comment = true;
            i += 2;
            continue;
        }
        if i + 1 < b.len() && b[i] == b'/' && b[i + 1] == b'*' {
            block_depth = 1;
            i += 2;
            continue;
        }
        if b[i] == b'"' || b[i] == b'\'' {
            string = true;
            quote = b[i];
            i += 1;
            continue;
        }
        out[i] = b[i];
        i += 1;
    }
    String::from_utf8(out).expect("ASCII-preserving sanitizer")
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut i = 0usize;
    let mut line = 1usize;
    let b = input.as_bytes();
    while i < b.len() {
        if b[i] == b'\n' {
            line += 1;
            i += 1;
            continue;
        }
        if b[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if b[i].is_ascii_alphabetic() || b[i] == b'_' {
            let start = i;
            i += 1;
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                i += 1;
            }
            tokens.push(Token { text: input[start..i].to_string(), line });
            continue;
        }
        if !b[i].is_ascii() {
            let len = input[i..].chars().next().expect("valid UTF-8").len_utf8();
            tokens.push(Token { text: input[i..i + len].to_string(), line });
            i += len;
            continue;
        }
        let two = if i + 1 < b.len() && b[i + 1].is_ascii() {
            &input[i..i + 2]
        } else {
            ""
        };
        if matches!(two, "=>" | "->" | "::" | "==" | "!=" | "<=" | ">=" | "&&" | "||") {
            tokens.push(Token { text: two.to_string(), line });
            i += 2;
            continue;
        }
        tokens.push(Token { text: (b[i] as char).to_string(), line });
        i += 1;
    }
    tokens
}

fn expression_start(prev: Option<&str>) -> bool {
    matches!(
        prev,
        None
            | Some("=")
            | Some("(")
            | Some("[")
            | Some("{")
            | Some(",")
            | Some(":")
            | Some("=>")
            | Some("->")
            | Some("return")
            | Some("then")
            | Some("else")
            | Some("do")
            | Some("case")
    )
}

fn looks_like_ident(s: Option<&str>) -> bool {
    s.and_then(|v| v.chars().next())
        .map(|c| c.is_ascii_alphabetic() || c == '_')
        .unwrap_or(false)
}

fn audit_tokens(tokens: &[Token]) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    for i in 0..tokens.len() {
        let cur = tokens[i].text.as_str();
        let next = tokens.get(i + 1).map(|t| t.text.as_str());
        let next2 = tokens.get(i + 2).map(|t| t.text.as_str());
        let prev = i.checked_sub(1)
            .and_then(|j| tokens.get(j))
            .map(|t| t.text.as_str());

        if cur == "rt" && next == Some("borrow") && next2 == Some("mut") {
            out.push((tokens[i].line, "legacy rt borrow mut; use rt mut borrow".into()));
        }
        if cur == "rt" && matches!(next, Some("ref") | Some("deref") | Some("unref") | Some("raw")) {
            out.push((
                tokens[i].line,
                format!("pointer-style rt {} is not Oreslang source", next.unwrap()),
            ));
        }
        if matches!(cur, "Pointer" | "RawPointer") && next == Some("<") {
            out.push((tokens[i].line, format!("public {}<T> pointer type is forbidden", cur)));
        }
        if cur == "&" && next == Some("mut") {
            out.push((tokens[i].line, "source &mut pointer/borrow syntax is forbidden".into()));
        }
        if cur == "&" && expression_start(prev) && (next == Some("(") || looks_like_ident(next)) {
            out.push((tokens[i].line, "source unary & address/borrow syntax is forbidden".into()));
        }
        if cur == "*" && expression_start(prev) && (next == Some("(") || looks_like_ident(next)) {
            out.push((tokens[i].line, "source unary * dereference syntax is forbidden".into()));
        }
    }
    out.sort();
    out.dedup();
    out
}

fn is_negative_fixture(path: &Path, source: &str) -> bool {
    let lower = path.to_string_lossy().to_ascii_lowercase();
    lower.contains("unsupported-")
        || lower.contains("reject-")
        || lower.contains("/negative/")
        || lower.contains("/invalid/")
        || source.lines().take(8).any(|l| l.contains("POINTER_SYNTAX_NEGATIVE_TEST"))
}

fn collect(root: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        if name == ".git" || name == "target" || name == "build" || name == "node_modules" {
            continue;
        }
        let ty = entry.file_type()?;
        if ty.is_symlink() {
            continue;
        }
        if ty.is_dir() {
            collect(&path, out)?;
        } else if ty.is_file() && path.extension().and_then(|x| x.to_str()) == Some("ores") {
            out.push(path);
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = env::args().nth(1).unwrap_or_else(|| ".".into());
    let mut files = Vec::new();
    collect(Path::new(&root), &mut files)?;
    files.sort();

    let mut violations = 0usize;
    for path in files {
        let source = fs::read_to_string(&path)?;
        if is_negative_fixture(&path, &source) {
            continue;
        }
        let clean = strip_comments_and_strings(&source);
        for (line, message) in audit_tokens(&tokenize(&clean)) {
            eprintln!("{}:{}: {}", path.display(), line, message);
            violations += 1;
        }
    }
    if violations > 0 {
        eprintln!("pointerless ownership syntax guard: {} violation(s)", violations);
        std::process::exit(1);
    }
    println!("pointerless ownership syntax guard: clean");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn findings(s: &str) -> Vec<String> {
        audit_tokens(&tokenize(&strip_comments_and_strings(s)))
            .into_iter()
            .map(|(_, m)| m)
            .collect()
    }

    #[test]
    fn accepts_canonical_ownership_and_binary_ops() {
        assert!(findings("val a = rt borrow x; val b = rt mut borrow y; val c = x * y; val d = x & y;").is_empty());
    }

    #[test]
    fn rejects_legacy_runtime_pointer_forms() {
        for src in [
            "val x = rt borrow mut y;",
            "val x = rt ref y;",
            "val x = rt deref y;",
            "val x = rt unref y;",
            "val x = rt raw y;",
            "val x: Pointer<int> = y;",
            "val x: RawPointer<int> = y;",
        ] {
            assert!(!findings(src).is_empty(), "{}", src);
        }
    }

    #[test]
    fn rejects_unary_pointer_sigils_but_not_binary_ops() {
        assert!(!findings("val x = &value;").is_empty());
        assert!(!findings("val x = &mut value;").is_empty());
        assert!(!findings("val x = *pointer;").is_empty());
        assert!(findings("val x = a * b; val y = a & b;").is_empty());
    }

    #[test]
    fn handles_non_ascii_without_panicking() {
        assert!(findings("val café = x * y;").is_empty());
        assert!(!findings("val café = rt deref owner;").is_empty());
    }

    #[test]
    fn ignores_comments_and_strings() {
        assert!(findings("// rt deref p\nval s = \"&mut Pointer<int>\";").is_empty());
    }
}

}
