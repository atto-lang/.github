//! Independent operator-approved repository-and-branch inventory gate.
//! Manifest is UTF-8, exactly one repo + TAB + branch entry per line.
//! This is not a cryptographic signature: sign-off is out of band.
use crate::valid_repo_name;
use std::collections::BTreeSet;

pub fn parse_manifest(text: &str) -> Result<Vec<(String, String)>, String> {
    if text.is_empty() { return Err("approved inventory file is empty".into()); }
    if text.contains('\r') { return Err("inventory must use LF, not CRLF, line endings".into()); }
    let mut found = Vec::new();
    let mut seen = BTreeSet::new();
    for (index, line) in text.lines().enumerate() {
        let line_no = index + 1;
        let (repo, branch) = line.split_once('\t')
            .ok_or_else(|| format!("inventory line {line_no}: expected repo<TAB>branch"))?;
        if !valid_repo_name(repo) {
            return Err(format!("inventory line {line_no}: invalid repository name"));
        }
        if branch.is_empty() || branch.trim() != branch || branch.chars().any(char::is_control) {
            return Err(format!("inventory line {line_no}: unsafe default branch"));
        }
        if !seen.insert(repo.to_owned()) {
            return Err(format!("inventory line {line_no}: duplicate repository"));
        }
        found.push((repo.to_owned(), branch.to_owned()));
    }
    if found.is_empty() { return Err("approved inventory contains no repository entries".into()); }
    found.sort();
    Ok(found)
}

pub fn verify_manifest(text: &str, live: &[(String, String)]) -> Result<(), String> {
    let approved = parse_manifest(text)?;
    if live.is_empty() { return Err("live organization inventory is empty".into()); }
    let mut actual = live.to_vec();
    actual.sort();
    if actual.windows(2).any(|w| w[0].0 == w[1].0) {
        return Err("live organization inventory contains duplicate repositories".into());
    }
    if approved != actual {
        return Err(format!(
            "approved inventory does not equal live org repository/default-branch identities (approved={}, visible={}); no writes authorized",
            approved.len(), actual.len()
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_repo_and_branch_matches() {
        let v = vec![("alpha".into(), "main".into()), ("beta".into(), "release/v1".into())];
        assert!(verify_manifest("beta\trelease/v1\nalpha\tmain\n", &v).is_ok());
    }
    #[test]
    fn trailing_newline_is_accepted() {
        assert!(parse_manifest("alpha\tmain\n").is_ok());
    }
    #[test]
    fn empty_manifest_rejected() {
        assert!(parse_manifest("").is_err());
    }
    #[test]
    fn blank_line_rejected() {
        assert!(parse_manifest("alpha\tmain\n\nbeta\tmain\n").is_err());
    }
    #[test]
    fn comments_not_accepted_as_scope() {
        assert!(parse_manifest("#approved\tsigned\n").is_err());
    }
    #[test]
    fn missing_tab_rejected() {
        assert!(parse_manifest("alpha main\n").is_err());
    }
    #[test]
    fn repeated_tab_rejected() {
        assert!(parse_manifest("alpha\tmain\textra\n").is_err());
    }
    #[test]
    fn duplicate_repo_rejected() {
        assert!(parse_manifest("alpha\tmain\nalpha\tdev\n").is_err());
    }
    #[test]
    fn invalid_repo_rejected() {
        assert!(parse_manifest("../escape\tmain\n").is_err());
    }
    #[test]
    fn trailing_spaces_in_branch_rejected() {
        assert!(parse_manifest("alpha\tmain \n").is_err());
    }
    #[test]
    fn control_chars_in_branch_rejected() {
        assert!(parse_manifest("alpha\tm\u{0001}ain\n").is_err());
    }
    #[test]
    fn equal_count_with_swapped_repo_rejected() {
        let v = vec![("alpha".into(), "main".into()), ("beta".into(), "main".into())];
        assert!(verify_manifest("alpha\tmain\ncharlie\tmain\n", &v).is_err());
    }
    #[test]
    fn changed_branch_rejected() {
        let v = vec![("alpha".into(), "main".into())];
        assert!(verify_manifest("alpha\tdev\n", &v).is_err());
    }
    #[test]
    fn partial_live_visibility_rejected() {
        let v = vec![("alpha".into(), "main".into())];
        assert!(verify_manifest("alpha\tmain\nbeta\tmain\n", &v).is_err());
    }
    #[test]
    fn duplicate_live_repo_rejected() {
        let v = vec![("alpha".into(), "main".into()), ("alpha".into(), "main".into())];
        assert!(verify_manifest("alpha\tmain\n", &v).is_err());
    }
    #[test]
    fn empty_live_rejected() {
        assert!(verify_manifest("alpha\tmain\n", &[]).is_err());
    }
    #[test]
    fn windows_crlf_rejected() {
        assert!(parse_manifest("alpha\tmain\r\n").is_err());
    }
    #[test]
    fn unicode_repo_rejected() {
        assert!(parse_manifest("café\tmain\n").is_err());
    }
}
