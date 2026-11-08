//! Offline Rust validation of the dated private repository ownership matrix.
//! Source-only checks are not GitHub transfer authorization or compiler guest evidence.
use std::collections::BTreeSet;
use std::{env, fs, process};
const Q: char = 0x60 as char;
const HOLDS: &[&str] = &[
    ".github", "gh-linguist-fork", "html-js-css-syntax-highlighter",
    "ores-truffle-oreslang.github.io", "oreslana.ores", "oreslang-language-specs.md",
];
struct Row<'a> {
    name: &'a str,
    domain: &'a str,
    belongs: &'a str,
    excludes: &'a str,
    decision: &'a str,
}
fn parse(input: &str) -> Result<Vec<Row<'_>>, String> {
    let mut rows = Vec::new();
    let marker = format!("| {Q}");
    for (i, line) in input.lines().enumerate() {
        if !line.starts_with(&marker) { continue; }
        let cells: Vec<_> = line.split('|').map(str::trim).collect();
        if cells.len() != 7 || !cells[0].is_empty() || !cells[6].is_empty() {
            return Err(format!("line {}: malformed table row", i + 1));
        }
        let name = cells[1].strip_prefix(Q).and_then(|s| s.strip_suffix(Q))
            .ok_or_else(|| format!("line {}: invalid repo quoting", i + 1))?;
        if name.is_empty() || cells[2..6].iter().any(|s| s.is_empty()) {
            return Err(format!("line {}: empty ownership field", i + 1));
        }
        rows.push(Row { name, domain: cells[2], belongs: cells[3], excludes: cells[4], decision: cells[5] });
    }
    Ok(rows)
}
fn validate(input: &str, total: usize, holds: &[&str]) -> Result<(), String> {
    let rows = parse(input)?;
    if rows.len() != total { return Err(format!("expected {total} rows, got {}", rows.len())); }
    let allowed_holds: BTreeSet<_> = holds.iter().copied().collect();
    if allowed_holds.len() != holds.len() { return Err("duplicate expected hold".into()); }
    let mut names = BTreeSet::new();
    let mut found_holds = BTreeSet::new();
    let mut proposed = 0;
    for row in rows {
        if !names.insert(row.name) { return Err(format!("duplicate repository {}", row.name)); }
        if row.domain.is_empty() || row.belongs.is_empty() || row.excludes.is_empty() {
            return Err(format!("empty responsibility for {}", row.name));
        }
        if row.decision == "HOLD — owner/exit decision" {
            found_holds.insert(row.name);
        } else {
            let destination = format!("Proposed {Q}atto-lang/{}{Q} (not transferred)", row.name);
            if row.decision != destination {
                return Err(format!("unauthorized destination for {}", row.name));
            }
            proposed += 1;
        }
    }
    if found_holds != allowed_holds { return Err("hold identity mismatch".into()); }
    if proposed != total - holds.len() { return Err("transfer/hold count mismatch".into()); }
    for phrase in ["## Maintainer migration checklist", "## Organization contract",
                   "numeric GitHub repository ID", "NOT_EXECUTED"] {
        if !input.contains(phrase) { return Err(format!("missing required text: {phrase}")); }
    }
    Ok(())
}
fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: validate_repository_ownership_guide FILE");
        process::exit(2);
    }
    let input = fs::read_to_string(&args[1]).unwrap_or_else(|e| {
        eprintln!("cannot read {}: {e}", args[1]);
        process::exit(2);
    });
    if let Err(e) = validate(&input, 80, HOLDS) {
        eprintln!("ownership guide FAIL: {e}");
        process::exit(1);
    }
    println!("ownership guide OK: 80 rows, 74 AttoLang proposals, 6 holds, zero direct AttoCo transfers");
}
#[cfg(test)]
mod tests {
    use super::*;
    const PREFIX: &str = "## Organization contract\n## Maintainer migration checklist\nnumeric GitHub repository ID\nNOT_EXECUTED\n";
    fn row(name: &str, target: &str) -> String {
        format!("| {Q}{name}{Q} | Area | Included | Excluded | {target} |\n")
    }
    fn source(rows: &[String]) -> String { format!("{PREFIX}{}", rows.concat()) }
    #[test]
    fn valid_sample() {
        let text = source(&[row("a", &format!("Proposed {Q}atto-lang/a{Q} (not transferred)")),
                            row("hold", "HOLD — owner/exit decision")]);
        assert!(validate(&text, 2, &["hold"]).is_ok());
    }
    #[test]
    fn reject_duplicate_repo() {
        let target = format!("Proposed {Q}atto-lang/a{Q} (not transferred)");
        let text = source(&[row("a", &target), row("a", &target)]);
        assert!(validate(&text, 2, &[]).unwrap_err().contains("duplicate"));
    }
    #[test]
    fn reject_direct_atto_co() {
        let text = source(&[row("a", &format!("Proposed {Q}atto-co/a{Q} (not transferred)"))]);
        assert!(validate(&text, 1, &[]).unwrap_err().contains("unauthorized"));
    }
    #[test]
    fn reject_renamed_target() {
        let text = source(&[row("a", &format!("Proposed {Q}atto-lang/atto-a{Q} (not transferred)"))]);
        assert!(validate(&text, 1, &[]).unwrap_err().contains("unauthorized"));
    }
    #[test]
    fn reject_unapproved_hold() {
        let text = source(&[row("a", "HOLD — owner/exit decision")]);
        assert!(validate(&text, 1, &[]).unwrap_err().contains("hold identity"));
    }
    #[test]
    fn reject_missing_and_empty_scope() {
        let text = source(&[row("a", &format!("Proposed {Q}atto-lang/a{Q} (not transferred)"))]);
        assert!(validate(&text, 2, &[]).unwrap_err().contains("expected"));
        let bad = format!("{PREFIX}| {Q}a{Q} | Area |  | Excluded | Proposed {Q}atto-lang/a{Q} (not transferred) |\n");
        assert!(validate(&bad, 1, &[]).unwrap_err().contains("empty"));
    }
}