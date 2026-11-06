//! Offline, read-only migration-manifest validator.
//! No GitHub credentials, network, secret reads, or repository mutations.
use std::collections::HashSet;
use std::env;
use std::fs;
use std::process;

const MANIFEST: &str = "docs/atto-org-migration/repo-manifest.csv";
const HEADER: &str = "source,source_repo_id,source_visibility,default_branch,source_head_sha,target_owner,target_repo,wave,disposition,status,risk,evidence_gate,approval_gate,dependency_hints,receipt";
const REQUIRED_APPROVERS: &str = "source_admin;target_admin;independent_security_reviewer";
const REQUIRED_ROWS: usize = 80;

fn parse_row(line: &str) -> Result<Vec<String>, String> {
    enum State { Start, Quoted, Closed }
    let mut state = State::Start;
    let mut fields = Vec::new();
    let mut value = String::new();
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match state {
            State::Start => {
                if c == '"' { state = State::Quoted; }
                else { return Err("every data field must begin with a quote".into()); }
            }
            State::Quoted => {
                if c == '"' {
                    if chars.peek() == Some(&'"') {
                        chars.next();
                        value.push('"');
                    } else { state = State::Closed; }
                } else if c == '\0' || c == '\r' || c == '\n' {
                    return Err("embedded control character".into());
                } else {
                    value.push(c);
                }
            }
            State::Closed => {
                if c != ',' { return Err("unexpected characters after quoted field".into()); }
                fields.push(std::mem::take(&mut value));
                state = State::Start;
            }
        }
    }
    match state {
        State::Closed => {
            fields.push(value);
            Ok(fields)
        }
        _ => Err("truncated or unclosed CSV field".into()),
    }
}

fn sha40(s: &str) -> bool {
    s.len() == 40 && s.bytes().all(|b| b.is_ascii_hexdigit())
}

fn validate_csv(input: &str) -> Result<String, String> {
    if input.contains('\0') || input.starts_with('\u{feff}') {
        return Err("NUL or UTF-8 BOM not permitted".into());
    }
    let mut lines = input.lines();
    if lines.next() != Some(HEADER) {
        return Err("unexpected manifest schema/header".into());
    }
    let mut source_names = HashSet::new();
    let mut source_ids = HashSet::new();
    let mut destinations = HashSet::new();
    let mut counts = [0usize; 3];
    let mut total = 0usize;
    for (index, line) in lines.enumerate() {
        let lineno = index + 2;
        let fields = parse_row(line).map_err(|e| format!("line {lineno}: {e}"))?;
        if fields.len() != 15 {
            return Err(format!("line {lineno}: expected 15 fields, got {}", fields.len()));
        }
        let field = |i: usize| fields[i].as_str();
        let source = field(0);
        let name = source.strip_prefix("ores-truffle-oreslang/")
            .ok_or_else(|| format!("line {lineno}: wrong source organization"))?;
        if name.is_empty() || name.contains('/') || name == "." || name == ".." {
            return Err(format!("line {lineno}: invalid repository name"));
        }
        if !source_names.insert(source.to_owned()) {
            return Err(format!("line {lineno}: duplicate source {source}"));
        }
        let id = field(1).parse::<u64>()
            .map_err(|_| format!("line {lineno}: invalid stable repository ID"))?;
        if id == 0 || !source_ids.insert(id) {
            return Err(format!("line {lineno}: duplicate/zero stable repository ID"));
        }
        if !matches!(field(2), "private" | "public" | "internal") {
            return Err(format!("line {lineno}: unsupported source visibility"));
        }
        if field(3).is_empty() || field(3).contains(char::is_whitespace) {
            return Err(format!("line {lineno}: invalid default branch"));
        }
        let target = field(5);
        let slot = match target {
            "atto-lang" => 0,
            "atto-co" => 1,
            "ores-truffle-oreslang" => 2,
            _ => return Err(format!("line {lineno}: unauthorized destination organization")),
        };
        if field(6) != name {
            return Err(format!("line {lineno}: simultaneous rename is prohibited"));
        }
        if !destinations.insert(format!("{target}/{}", field(6))) {
            return Err(format!("line {lineno}: duplicate destination repository"));
        }
        let wave = field(7);
        let expected_slot = if matches!(wave,
            "A-language-spec" | "B-developer-tooling" | "B-docs-examples" |
            "C-compilers" | "D-runtime-targets" | "E-conformance") {
            0
        } else if wave.starts_with("F-") {
            1
        } else if wave == "H-hold" {
            2
        } else {
            return Err(format!("line {lineno}: unknown migration wave"));
        };
        if slot != expected_slot {
            return Err(format!("line {lineno}: target/wave mismatch"));
        }
        if field(8) != (if slot == 2 { "hold" } else { "propose_transfer" }) {
            return Err(format!("line {lineno}: wrong disposition for destination"));
        }
        let status = field(9);
        if !matches!(status, "not_started" | "preflight" | "blocked" | "approved" |
            "in_transfer" | "verified" | "held") {
            return Err(format!("line {lineno}: unknown transfer status"));
        }
        if slot == 2 && matches!(status, "approved" | "in_transfer" | "verified") {
            return Err(format!("line {lineno}: legacy hold must be adjudicated first"));
        }
        let head = field(4);
        if head != "capture_at_transfer_preflight" && !sha40(head) {
            return Err(format!("line {lineno}: invalid pinned source head SHA"));
        }
        let active = matches!(status, "preflight" | "approved" | "in_transfer" | "verified");
        if active && !sha40(head) {
            return Err(format!("line {lineno}: active transfer missing exact source SHA"));
        }
        let evidence = field(11);
        if !matches!(evidence,
            "ownership_security_decision" | "exact_head_guest_compile_execute_negative_tests" |
            "exact_head_executed_steps_conformance_receipts" |
            "target_abi_and_isolation_negative_tests" | "source_contract_and_tooling_smoke" |
            "pinned_compiler_guest_e2e_and_security_tests") {
            return Err(format!("line {lineno}: unrecognized evidence gate"));
        }
        if field(12) != REQUIRED_APPROVERS {
            return Err(format!("line {lineno}: approval gate weakened"));
        }
        if field(10).trim().is_empty() || field(13).trim().is_empty() {
            return Err(format!("line {lineno}: risk or dependency audit missing"));
        }
        if matches!(status, "approved" | "in_transfer" | "verified")
            && field(13) == "unknown_requires_dependency_audit" {
            return Err(format!("line {lineno}: dependency audit incomplete"));
        }
        if matches!(status, "approved" | "in_transfer" | "verified")
            && field(14) == "pending" {
            return Err(format!("line {lineno}: approval/preflight receipt missing"));
        }
        if field(14).trim().is_empty() {
            return Err(format!("line {lineno}: empty receipt state"));
        }
        counts[slot] += 1;
        total += 1;
    }
    if total != REQUIRED_ROWS || counts != [41, 33, 6] {
        return Err(format!("baseline mismatch: total={total}, destinations={counts:?}; update invariant only with reviewed inventory delta"));
    }
    for essential in [".github", "ores-truffle-oreslang.github.io", "oreslang-source.java",
        "oreslang-platform-conformance", "oreslang-durable-objects.ores",
        "attolang-language-spec.md"] {
        if !source_names.contains(&format!("ores-truffle-oreslang/{essential}")) {
            return Err(format!("mandatory repository missing: {essential}"));
        }
    }
    Ok(format!("validated {total} unique source repository IDs and destinations; atto-lang={}, atto-co={}, holds={}; no transfers executed", counts[0], counts[1], counts[2]))
}

fn main() {
    let path = env::args().nth(1).unwrap_or_else(|| MANIFEST.to_owned());
    match fs::read_to_string(&path).map_err(|e| e.to_string())
        .and_then(|content| validate_csv(&content)) {
        Ok(message) => println!("{message}"),
        Err(e) => {
            eprintln!("migration manifest validation FAILED: {e}");
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const BASE: &str = include_str!("../docs/atto-org-migration/repo-manifest.csv");

    #[test]
    fn inventory_baseline_is_valid() {
        assert!(validate_csv(BASE).is_ok());
    }

    #[test]
    fn rejects_unterminated_and_unquoted_fields() {
        assert!(parse_row("\"source\",\"unfinished").is_err());
        assert!(parse_row("source,\"other\"").is_err());
        assert!(parse_row("\"source\"junk,\"other\"").is_err());
    }

    #[test]
    fn quoted_csv_field_roundtrip() {
        assert_eq!(parse_row("\"a\"\"b\",\"c,d\"").unwrap(), vec!["a\"b", "c,d"]);
    }

    #[test]
    fn rejects_duplicate_repository_id() {
        let tampered = BASE.replacen("\"1444921662\"", "\"1411040701\"", 1);
        assert!(validate_csv(&tampered).is_err());
    }

    #[test]
    fn rejects_wrong_destination_and_rename() {
        let tampered = BASE.replacen("\"atto-lang\",\"attolang-language-spec.md\"",
            "\"unapproved-org\",\"attolang-language-spec.md\"", 1);
        assert!(validate_csv(&tampered).is_err());
        let renamed = BASE.replacen("\"atto-lang\",\"attolang-language-spec.md\"",
            "\"atto-lang\",\"renamed-spec.md\"", 1);
        assert!(validate_csv(&renamed).is_err());
    }

    #[test]
    fn active_transfer_requires_exact_source_sha() {
        let tampered = BASE.replacen("\"propose_transfer\",\"not_started\"",
            "\"propose_transfer\",\"approved\"", 1);
        assert!(validate_csv(&tampered).is_err());
    }

    #[test]
    fn cannot_weaken_independent_approval_gate() {
        let tampered = BASE.replacen(REQUIRED_APPROVERS, "source_admin", 1);
        assert!(validate_csv(&tampered).is_err());
    }

    #[test]
    fn cannot_silently_change_baseline() {
        let mut lines: Vec<&str> = BASE.lines().collect();
        lines.pop();
        let missing = format!("{}\n", lines.join("\n"));
        assert!(validate_csv(&missing).is_err());
    }
}