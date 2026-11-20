//! Read-only, fail-closed evidence for the *named* AttoLang review policy.
//!
//! This is a witness evaluator, NOT a GitHub merge-protection mechanism.
//! A green result does not prove that GitHub rulesets enforce reviewer identity,
//! status checks, or branch protection. Keep those as separate live receipts.
use serde_json::Value;
use std::collections::BTreeMap;

pub const ALLOWED: [&str; 2] = ["oresoftware", "the1mills"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApprovalEvidence {
    Approved { reviewer: String, review_id: u64 },
    Missing,
    BlockedByChangesRequested { reviewer: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Verdict {
    Approved { id: u64, on_head: bool },
    ChangesRequested,
    Dismissed,
}

fn valid_login(login: &str) -> bool {
    !login.is_empty()
        && login.len() <= 39
        && login.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-')
}

fn valid_sha(sha: &str) -> bool {
    sha.len() == 40 && sha.bytes().all(|c| c.is_ascii_hexdigit())
}

/// Consumes the complete review history sorted by increasing numeric review ID.
/// A PR's latest decisive review from each author wins; comments do not revoke an
/// approval. A CHANGES_REQUESTED review from *anyone* blocks until dismissed or
/// superseded by that reviewer, consistent with conservative GitHub governance.
/// Only exact-head approvals from one of two named independent accounts count.
pub fn evaluate(
    reviews: &Value,
    pr_author: &str,
    expected_head: &str,
) -> Result<ApprovalEvidence, String> {
    if !valid_login(pr_author) || !valid_sha(expected_head) {
        return Err("invalid PR author or exact head SHA".into());
    }
    let entries = reviews.as_array().ok_or("reviews must be a complete array")?;
    let mut last_id = 0u64;
    let mut latest: BTreeMap<String, Verdict> = BTreeMap::new();
    for review in entries {
        let id = review["id"].as_u64().ok_or("review missing numeric ID")?;
        if id == 0 || id <= last_id {
            return Err("duplicate or nonmonotonic review ID; incomplete/unsorted history".into());
        }
        last_id = id;
        let login = review.pointer("/user/login").and_then(Value::as_str)
            .ok_or("review missing reviewer login")?;
        if !valid_login(login) { return Err("invalid reviewer login".into()); }
        let reviewer = login.to_ascii_lowercase();
        let state = review["state"].as_str().ok_or("review missing state")?;
        match state {
            "PENDING" | "COMMENTED" => {}
            "APPROVED" => {
                // GitHub's stale-review setting may allow old approvals; this
                // witness intentionally requires an exact-head review.
                let sha = review["commit_id"].as_str().ok_or("approval missing commit ID")?;
                if !valid_sha(sha) { return Err("approval contains invalid commit ID".into()); }
                latest.insert(reviewer, Verdict::Approved {
                    id, on_head: sha.eq_ignore_ascii_case(expected_head)
                });
            }
            "CHANGES_REQUESTED" => {
                latest.insert(reviewer, Verdict::ChangesRequested);
            }
            "DISMISSED" => {
                latest.insert(reviewer, Verdict::Dismissed);
            }
            other => return Err(format!("unknown review state: {other}")),
        };
    }
    if let Some((who, _)) = latest.iter()
        .find(|(_, verdict)| matches!(verdict, Verdict::ChangesRequested))
    {
        return Ok(ApprovalEvidence::BlockedByChangesRequested {
            reviewer: who.clone()
        });
    }
    let author = pr_author.to_ascii_lowercase();
    for named in ALLOWED {
        if named == author { continue; }
        if let Some(Verdict::Approved { id, on_head: true }) = latest.get(named) {
            return Ok(ApprovalEvidence::Approved {
                reviewer: named.into(), review_id: *id
            });
        }
    }
    Ok(ApprovalEvidence::Missing)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    const HEAD: &str = "ba15cfe08e85bf51389a8c9c33e9caafa225dbd0";
    const OLD: &str = "0000000000000000000000000000000000000000";
    fn review(id: u64, who: &str, state: &str, commit: &str) -> Value {
        json!({"id":id,"user":{"login":who},"state":state,"commit_id":commit})
    }
    #[test]
    fn no_review_means_missing() {
        assert_eq!(evaluate(&json!([]),"the1mills",HEAD),Ok(ApprovalEvidence::Missing));
    }
    #[test]
    fn oresoftware_approves_the1mills() {
        assert_eq!(evaluate(&json!([review(1,"ORESoftware","APPROVED",HEAD)]),"the1mills",HEAD),
            Ok(ApprovalEvidence::Approved { reviewer:"oresoftware".into(), review_id:1 }));
    }
    #[test]
    fn the1mills_approves_oresoftware() {
        assert_eq!(evaluate(&json!([review(5,"the1mills","APPROVED",HEAD)]),"ORESOFTWARE",HEAD),
            Ok(ApprovalEvidence::Approved { reviewer:"the1mills".into(), review_id:5 }));
    }
    #[test]
    fn unrelated_approval_is_insufficient() {
        assert_eq!(evaluate(&json!([review(1,"other","APPROVED",HEAD)]),"the1mills",HEAD),
            Ok(ApprovalEvidence::Missing));
    }
    #[test]
    fn authors_own_approval_cannot_count() {
        assert_eq!(evaluate(&json!([review(1,"the1mills","APPROVED",HEAD)]),"the1mills",HEAD),
            Ok(ApprovalEvidence::Missing));
    }
    #[test]
    fn earlier_head_is_insufficient() {
        assert_eq!(evaluate(&json!([review(1,"ORESoftware","APPROVED",OLD)]),"the1mills",HEAD),
            Ok(ApprovalEvidence::Missing));
    }
    #[test]
    fn latest_changes_requested_overrides_own_prior_approval() {
        let v = json!([review(1,"ORESoftware","APPROVED",HEAD),
            review(2,"ORESoftware","CHANGES_REQUESTED",HEAD)]);
        assert_eq!(evaluate(&v,"the1mills",HEAD),
            Ok(ApprovalEvidence::BlockedByChangesRequested { reviewer:"oresoftware".into() }));
    }
    #[test]
    fn unrelated_changes_requested_blocks_qualified_approval() {
        let v = json!([review(1,"ORESoftware","APPROVED",HEAD),
            review(2,"other","CHANGES_REQUESTED",HEAD)]);
        assert_eq!(evaluate(&v,"the1mills",HEAD),
            Ok(ApprovalEvidence::BlockedByChangesRequested { reviewer:"other".into() }));
    }
    #[test]
    fn dismissed_request_for_changes_no_longer_blocks() {
        let v = json!([review(1,"other","CHANGES_REQUESTED",HEAD),
            review(2,"other","DISMISSED",HEAD),review(3,"ORESoftware","APPROVED",HEAD)]);
        assert_eq!(evaluate(&v,"the1mills",HEAD),
            Ok(ApprovalEvidence::Approved { reviewer:"oresoftware".into(), review_id:3 }));
    }
    #[test]
    fn dismissed_approval_does_not_count() {
        let v = json!([review(1,"ORESoftware","APPROVED",HEAD),
            review(2,"ORESoftware","DISMISSED",HEAD)]);
        assert_eq!(evaluate(&v,"the1mills",HEAD),Ok(ApprovalEvidence::Missing));
    }
    #[test]
    fn commented_review_does_not_revoke_approval() {
        let v = json!([review(1,"ORESoftware","APPROVED",HEAD),
            review(2,"ORESoftware","COMMENTED",HEAD)]);
        assert!(matches!(evaluate(&v,"the1mills",HEAD),Ok(ApprovalEvidence::Approved{..})));
    }
    #[test]
    fn pending_review_does_not_count() {
        assert_eq!(evaluate(&json!([review(1,"ORESoftware","PENDING",HEAD)]),"the1mills",HEAD),
            Ok(ApprovalEvidence::Missing));
    }
    #[test]
    fn invalid_head_is_rejected() {
        assert!(evaluate(&json!([]),"the1mills","main").is_err());
    }
    #[test]
    fn absent_commit_id_is_rejected() {
        assert!(evaluate(&json!([{"id":1,"user":{"login":"ORESoftware"},"state":"APPROVED"}]),
            "the1mills",HEAD).is_err());
    }
    #[test]
    fn missing_reviewer_is_rejected() {
        assert!(evaluate(&json!([{"id":1,"state":"APPROVED","commit_id":HEAD}]),"the1mills",HEAD).is_err());
    }
    #[test]
    fn duplicate_review_ids_are_rejected() {
        let v = json!([review(1,"ORESoftware","APPROVED",HEAD),review(1,"other","COMMENTED",HEAD)]);
        assert!(evaluate(&v,"the1mills",HEAD).is_err());
    }
    #[test]
    fn unsorted_review_ids_are_rejected() {
        let v = json!([review(2,"ORESoftware","APPROVED",HEAD),review(1,"other","COMMENTED",HEAD)]);
        assert!(evaluate(&v,"the1mills",HEAD).is_err());
    }
    #[test]
    fn unknown_state_is_rejected() {
        assert!(evaluate(&json!([review(1,"ORESoftware","AUTO_APPROVED",HEAD)]),
            "the1mills",HEAD).is_err());
    }
    #[test]
    fn comments_from_unrelated_users_do_not_approve() {
        assert_eq!(evaluate(&json!([review(1,"other","COMMENTED",HEAD)]),"the1mills",HEAD),
            Ok(ApprovalEvidence::Missing));
    }
    #[test]
    fn invalid_reviewer_login_is_rejected() {
        assert!(evaluate(&json!([review(1,"some/bad","APPROVED",HEAD)]),"the1mills",HEAD).is_err());
    }
}
