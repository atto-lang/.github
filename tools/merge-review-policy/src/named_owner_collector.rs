//! A strictly read-only, scope-limited GitHub evidence collector.
//! No result of this module authorizes a PATCH or proves a live merge gate.
use crate::named_owner_preflight::{
    assess_pilot, PilotEvidence, ReadOnlyCandidate, PILOT_BRANCH, PILOT_REPO,
};
use crate::{Api, ORG};
use serde_json::Value;
use std::collections::BTreeSet;

const PREFIX: &str = "repos/ores-truffle-oreslang/.github";
const MAX_PAGES: usize = 100;
const MAX_OWNERS_BYTES: usize = 131_072;
const PAGE_SIZE: usize = 100;

fn sha40(s: &str) -> bool {
    s.len() == 40 && s.bytes().all(|c| c.is_ascii_hexdigit())
}

fn branch_head<A: Api>(api: &mut A) -> Result<String, String> {
    let branch = api.get(&format!("{PREFIX}/branches/main"))
        .map_err(|_| "cannot read default branch; permission/plan or network blocked")?;
    if branch["name"].as_str() != Some(PILOT_BRANCH)
        || branch["protected"].as_bool() != Some(true)
    {
        return Err("main branch is not verified protected".into());
    }
    let head = branch.pointer("/commit/sha").and_then(Value::as_str)
        .ok_or("base branch commit SHA missing")?;
    if !sha40(head) {
        return Err("base branch commit SHA malformed".into());
    }
    Ok(head.to_owned())
}

fn read_rulesets<A: Api>(api: &mut A) -> Result<Value, String> {
    let mut result = Vec::new();
    let mut ids = BTreeSet::new();
    for page in 1..=MAX_PAGES {
        let url = format!("{PREFIX}/rulesets?includes_parents=true&per_page={PAGE_SIZE}&page={page}");
        let batch = api.get(&url)
            .map_err(|_| "ruleset inventory inaccessible; no negative evidence")?;
        let values = batch.as_array().ok_or("rulesets response must be an array")?;
        if values.len() > PAGE_SIZE { return Err("rulesets response exceeded page size".into()); }
        for entry in values {
            let id = entry["id"].as_u64().filter(|id| *id > 0)
                .ok_or("ruleset missing numeric identity")?;
            if !ids.insert(id) { return Err("duplicate ruleset across pages".into()); }
            result.push(entry.clone());
        }
        if values.len() < PAGE_SIZE { return Ok(Value::Array(result)); }
    }
    Err("ruleset pagination limit exceeded; inventory incomplete".into())
}

/// GitHub's contents API returns MIME-base64 data, possibly with LF separators.
/// Strictly reject padding errors, unknown alphabet, and oversized content.
fn decode_base64(text: &str) -> Result<Vec<u8>, String> {
    if text.len() > MAX_OWNERS_BYTES * 2 {
        return Err("CODEOWNERS encoded content exceeds safety limit".into());
    }
    let raw: Vec<u8> = text.bytes().filter(|c| *c != b'\n' && *c != b'\r').collect();
    if raw.is_empty() || raw.len() % 4 != 0 {
        return Err("CODEOWNERS encoding has invalid base64 length".into());
    }
    let mut output = Vec::new();
    for (index, c) in raw.chunks_exact(4).enumerate() {
        let last = index == raw.len() / 4 - 1;
        let mut vals = [0u8; 4];
        let mut padding = 0usize;
        for (i, ch) in c.iter().enumerate() {
            vals[i] = match *ch {
                b'A'..=b'Z' => ch - b'A',
                b'a'..=b'z' => ch - b'a' + 26,
                b'0'..=b'9' => ch - b'0' + 52,
                b'+' => 62,
                b'/' => 63,
                b'=' if last && i >= 2 => { padding += 1; 0 },
                _ => return Err("CODEOWNERS encoding contains invalid base64 data".into()),
            };
        }
        if padding > 2 || (c[2] == b'=' && c[3] != b'=')
            || (padding == 1 && vals[2] & 0b11 != 0)
            || (padding == 2 && vals[1] & 0b1111 != 0)
        {
            return Err("CODEOWNERS encoding has invalid padding".into());
        }
        output.push((vals[0] << 2) | (vals[1] >> 4));
        if padding < 2 { output.push((vals[1] << 4) | (vals[2] >> 2)); }
        if padding == 0 { output.push((vals[2] << 6) | vals[3]); }
        if output.len() > MAX_OWNERS_BYTES {
            return Err("CODEOWNERS content too large to verify".into());
        }
    }
    Ok(output)
}

fn get_codeowners<A: Api>(api: &mut A, sha: &str) -> Result<String, String> {
    let contents = api.get(&format!("{PREFIX}/contents/.github/CODEOWNERS?ref={sha}"))
        .map_err(|_| "pinned base CODEOWNERS missing or inaccessible")?;
    if contents["type"].as_str() != Some("file")
        || contents["path"].as_str() != Some(".github/CODEOWNERS")
        || contents["encoding"].as_str() != Some("base64")
        || !contents["sha"].as_str().is_some_and(sha40)
    {
        return Err("pinned CODEOWNERS metadata malformed or unsupported".into());
    }
    let encoded = contents["content"].as_str()
        .ok_or("pinned CODEOWNERS response is missing content")?;
    let bytes = decode_base64(encoded)?;
    String::from_utf8(bytes).map_err(|_| "CODEOWNERS content must be UTF-8".into())
}

/// Reads only the .github/main pilot. Every GET must be explicitly authorized
/// on the administrator's chosen credential. No endpoint here writes settings.
pub fn collect_pilot<A: Api>(api: &mut A) -> Result<ReadOnlyCandidate, String> {
    let repo = api.get(PREFIX)
        .map_err(|_| "repository metadata inaccessible")?;
    if repo["full_name"].as_str() != Some("ores-truffle-oreslang/.github")
        || repo["name"].as_str() != Some(PILOT_REPO)
        || repo["default_branch"].as_str() != Some(PILOT_BRANCH)
        || repo["archived"].as_bool() != Some(false)
    {
        return Err("repository identity, default branch, or archive status changed".into());
    }
    if ORG != "ores-truffle-oreslang" { return Err("organization identity mismatch".into()); }
    let before = branch_head(api)?;
    let codeowners = get_codeowners(api, &before)?;
    let errors = api.get(&format!("{PREFIX}/codeowners/errors?ref={before}"))
        .map_err(|_| "CODEOWNERS syntax diagnostics unavailable")?;
    let software = api.get(&format!("{PREFIX}/collaborators/oresoftware/permission"))
        .map_err(|_| "oresoftware collaborator permissions unavailable")?;
    let mills = api.get(&format!("{PREFIX}/collaborators/the1mills/permission"))
        .map_err(|_| "the1mills collaborator permissions unavailable")?;
    let protection = api.get(&format!("{PREFIX}/branches/main/protection"))
        .map_err(|_| "branch protection read denied; cannot prove merge requirements")?;
    let rulesets = read_rulesets(api)?;
    let after = branch_head(api)?;
    if before != after {
        return Err("base branch changed while evidence was being collected".into());
    }
    assess_pilot(&PilotEvidence {
        repo: PILOT_REPO,
        branch: PILOT_BRANCH,
        base_sha: &before,
        codeowners_ref_sha: &before,
        codeowners: &codeowners,
        codeowners_errors: &errors,
        protection: &protection,
        rulesets: &rulesets,
        oresoftware_access: &software,
        the1mills_access: &mills,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::VecDeque;

    const HEAD: &str = "fedcba9876543210fedcba9876543210fedcba98";
    const NEXT: &str = "0000000000000000000000000000000000000000";
    const OWNERS_B64: &str = "KiBAb3Jlc29mdHdhcmUgQHRoZTFtaWxscwo=";

    struct Mock {
        calls: VecDeque<(String, Result<Value, String>)>,
        writes: usize,
    }
    impl Mock {
        fn new() -> Self { Self { calls: VecDeque::new(), writes: 0 } }
        fn ok(mut self, path: impl Into<String>, value: Value) -> Self {
            self.calls.push_back((path.into(), Ok(value))); self
        }
        fn fail(mut self, path: impl Into<String>) -> Self {
            self.calls.push_back((path.into(), Err("denied".into()))); self
        }
        fn done(&self) {
            assert!(self.calls.is_empty(), "unconsumed mocked GETs");
            assert_eq!(self.writes, 0, "read-only collector cannot write");
        }
    }
    impl Api for Mock {
        fn request(&mut self, method: &str, path: &str, body: Option<Value>) -> Result<Value,String> {
            assert_eq!(method, "GET");
            assert!(body.is_none());
            let (expected, answer) = self.calls.pop_front().expect("unexpected request");
            assert_eq!(expected, path);
            answer
        }
    }
    fn repo() -> Value {
        json!({"full_name":"ores-truffle-oreslang/.github", "name":".github",
              "default_branch":"main","archived":false})
    }
    fn branch(sha: &str) -> Value {
        json!({"name":"main","protected":true,"commit":{"sha":sha}})
    }
    fn contents() -> Value {
        json!({"type":"file","path":".github/CODEOWNERS","sha":HEAD,
               "encoding":"base64","content":OWNERS_B64})
    }
    fn policy() -> Value {
        json!({"required_pull_request_reviews":{
            "require_code_owner_reviews":true,"required_approving_review_count":1,
            "dismiss_stale_reviews":true,"require_last_push_approval":true,
            "bypass_pull_request_allowances":{"users":[],"teams":[],"apps":[]}
        },"required_status_checks":{"strict":true,"checks":[{"context":"ci","app_id":15368}]},
        "enforce_admins":{"enabled":true}})
    }
    fn fixture() -> Mock {
        Mock::new()
            .ok(PREFIX, repo())
            .ok(format!("{PREFIX}/branches/main"),branch(HEAD))
            .ok(format!("{PREFIX}/contents/.github/CODEOWNERS?ref={HEAD}"),contents())
            .ok(format!("{PREFIX}/codeowners/errors?ref={HEAD}"),json!({"errors":[]}))
            .ok(format!("{PREFIX}/collaborators/oresoftware/permission"),json!({"permission":"write"}))
            .ok(format!("{PREFIX}/collaborators/the1mills/permission"),json!({"permission":"admin"}))
            .ok(format!("{PREFIX}/branches/main/protection"),policy())
            .ok(format!("{PREFIX}/rulesets?includes_parents=true&per_page=100&page=1"),json!([]))
            .ok(format!("{PREFIX}/branches/main"),branch(HEAD))
    }
    #[test]
    fn administrator_candidate_reads_only_pinned_full_evidence() {
        let mut api=fixture();
        assert_eq!(collect_pilot(&mut api),Ok(ReadOnlyCandidate{base_sha:HEAD.into()}));
        api.done();
    }
    #[test]
    fn no_gh_patch_even_on_complete_evidence() {
        let mut api=fixture();let _=collect_pilot(&mut api);
        assert_eq!(api.writes,0);api.done();
    }
    #[test]
    fn absent_codeowners_on_base_is_not_a_candidate() {
        let mut api=Mock::new().ok(PREFIX,repo())
            .ok(format!("{PREFIX}/branches/main"),branch(HEAD))
            .fail(format!("{PREFIX}/contents/.github/CODEOWNERS?ref={HEAD}"));
        assert!(collect_pilot(&mut api).is_err());api.done();
    }
    #[test]
    fn wrong_repo_identity_stops_before_other_requests() {
        let mut api=Mock::new().ok(PREFIX,json!({"full_name":"attacker/.github"}));
        assert!(collect_pilot(&mut api).is_err());api.done();
    }
    #[test]
    fn unprotected_branch_is_rejected() {
        let mut api=Mock::new().ok(PREFIX,repo()).ok(format!("{PREFIX}/branches/main"),
            json!({"name":"main","protected":false,"commit":{"sha":HEAD}}));
        assert!(collect_pilot(&mut api).is_err());api.done();
    }
    #[test]
    fn malformed_base_sha_is_rejected() {
        let mut api=Mock::new().ok(PREFIX,repo()).ok(format!("{PREFIX}/branches/main"),branch("main"));
        assert!(collect_pilot(&mut api).is_err());api.done();
    }
    #[test]
    fn moved_base_branch_invalidates_receipt() {
        let mut api=Mock::new().ok(PREFIX,repo())
            .ok(format!("{PREFIX}/branches/main"),branch(HEAD))
            .ok(format!("{PREFIX}/contents/.github/CODEOWNERS?ref={HEAD}"),contents())
            .ok(format!("{PREFIX}/codeowners/errors?ref={HEAD}"),json!({"errors":[]}))
            .ok(format!("{PREFIX}/collaborators/oresoftware/permission"),json!({"permission":"write"}))
            .ok(format!("{PREFIX}/collaborators/the1mills/permission"),json!({"permission":"write"}))
            .ok(format!("{PREFIX}/branches/main/protection"),policy())
            .ok(format!("{PREFIX}/rulesets?includes_parents=true&per_page=100&page=1"),json!([]))
            .ok(format!("{PREFIX}/branches/main"),branch(NEXT));
        assert!(collect_pilot(&mut api).is_err());api.done();
    }
    #[test]
    fn inaccessible_protection_stops_before_ruleset_claim() {
        let mut api=Mock::new().ok(PREFIX,repo())
            .ok(format!("{PREFIX}/branches/main"),branch(HEAD))
            .ok(format!("{PREFIX}/contents/.github/CODEOWNERS?ref={HEAD}"),contents())
            .ok(format!("{PREFIX}/codeowners/errors?ref={HEAD}"),json!({"errors":[]}))
            .ok(format!("{PREFIX}/collaborators/oresoftware/permission"),json!({"permission":"write"}))
            .ok(format!("{PREFIX}/collaborators/the1mills/permission"),json!({"permission":"write"}))
            .fail(format!("{PREFIX}/branches/main/protection"));
        assert!(collect_pilot(&mut api).is_err());api.done();
    }
    #[test]
    fn active_rule_rejected_without_ignoring_other_protections() {
        let mut api=Mock::new().ok(PREFIX,repo())
            .ok(format!("{PREFIX}/branches/main"),branch(HEAD))
            .ok(format!("{PREFIX}/contents/.github/CODEOWNERS?ref={HEAD}"),contents())
            .ok(format!("{PREFIX}/codeowners/errors?ref={HEAD}"),json!({"errors":[]}))
            .ok(format!("{PREFIX}/collaborators/oresoftware/permission"),json!({"permission":"write"}))
            .ok(format!("{PREFIX}/collaborators/the1mills/permission"),json!({"permission":"write"}))
            .ok(format!("{PREFIX}/branches/main/protection"),policy())
            .ok(format!("{PREFIX}/rulesets?includes_parents=true&per_page=100&page=1"),
                json!([{"id":1,"enforcement":"active"}]))
            .ok(format!("{PREFIX}/branches/main"),branch(HEAD));
        assert!(collect_pilot(&mut api).is_err());api.done();
    }
    #[test]
    fn base64_decodes_exact_owner_rule() {
        assert_eq!(decode_base64(OWNERS_B64).unwrap(),b"* @oresoftware @the1mills\n");
        assert_eq!(decode_base64("YQ==").unwrap(),b"a");
        assert_eq!(decode_base64("YWI=").unwrap(),b"ab");
    }
    #[test]
    fn base64_rejects_noncanonical_padding_and_invalid_alphabet() {
        for value in ["", "YQ=", "YQ===", "YQ=A", "YR==", "YWJ=", "YQ??",
                      "YQ==AAAA", "YQ==\0", "YQ==%"] {
            assert!(decode_base64(value).is_err(), "accepted {value:?}");
        }
    }
    #[test]
    fn base64_rejects_large_encoded_payload() {
        assert!(decode_base64(&"A".repeat(MAX_OWNERS_BYTES*2+1)).is_err());
    }
    #[test]
    fn ruleset_pagination_rejects_duplication() {
        let page:Vec<Value>=(1..=PAGE_SIZE).map(|id|json!({"id":id,"enforcement":"disabled"})).collect();
        let mut api=Mock::new()
            .ok(format!("{PREFIX}/rulesets?includes_parents=true&per_page=100&page=1"),json!(page))
            .ok(format!("{PREFIX}/rulesets?includes_parents=true&per_page=100&page=2"),
                json!([{"id":1,"enforcement":"active"}]));
        assert!(read_rulesets(&mut api).is_err());api.done();
    }
    #[test]
    fn rule_on_second_page_is_not_ignored() {
        let page:Vec<Value>=(1..=PAGE_SIZE).map(|id|json!({"id":id,"enforcement":"disabled"})).collect();
        let mut api=Mock::new()
            .ok(format!("{PREFIX}/rulesets?includes_parents=true&per_page=100&page=1"),json!(page))
            .ok(format!("{PREFIX}/rulesets?includes_parents=true&per_page=100&page=2"),
                json!([{"id":999,"enforcement":"active"}]));
        let result=read_rulesets(&mut api).unwrap();
        assert_eq!(result.as_array().unwrap().len(),PAGE_SIZE+1);
        assert!(result.as_array().unwrap().iter().any(|x|x["enforcement"]=="active"));
        api.done();
    }
    #[test]
    fn missing_ruleset_page_is_a_failure_not_an_empty_list() {
        let page:Vec<Value>=(1..=PAGE_SIZE).map(|id|json!({"id":id,"enforcement":"disabled"})).collect();
        let mut api=Mock::new()
            .ok(format!("{PREFIX}/rulesets?includes_parents=true&per_page=100&page=1"),json!(page))
            .fail(format!("{PREFIX}/rulesets?includes_parents=true&per_page=100&page=2"));
        assert!(read_rulesets(&mut api).is_err());api.done();
    }
}
