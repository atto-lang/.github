//! SYNTHETIC, SOURCE-FREE MODEL ONLY.
//! Tests classification logic; does NOT attest real Oreslang source, workflows,
//! branch rules, review signatures, Actions budget, or native compiler behavior.

#[derive(Debug, Clone)]
struct Step<'a> {
    name: &'a str,
    status: &'a str,
    conclusion: &'a str,
}

#[derive(Debug, Clone)]
struct Receipt<'a> {
    source_repo_id: u64,
    runner_repo_id: u64,
    trusted_workflow_on_protected_ref: bool,
    expected_commit: &'a str,
    actual_commit: &'a str,
    expected_tree: &'a str,
    actual_tree: &'a str,
    runner_id: Option<u64>,
    run_status: &'a str,
    run_conclusion: &'a str,
    job_status: &'a str,
    job_conclusion: &'a str,
    steps: &'a [Step<'a>],
}

fn canonical_sha(s: &str) -> bool {
    s.len() == 40 && s.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        && s.bytes().any(|b| b != b'0')
}

// A fail-closed *illustrative* gate. Real evidence must come from authenticated
// GitHub APIs and an independent trusted policy, not PR-supplied fields.
fn validate(r: &Receipt<'_>) -> Result<(), &'static str> {
    if !r.trusted_workflow_on_protected_ref {
        return Err("untrusted workflow/protection");
    }
    if r.source_repo_id == 0 || r.runner_repo_id == 0 {
        return Err("missing immutable repository ID");
    }
    // Source export is forbidden while confidentiality incidents and
    // independent destination authorization are unresolved. Never promote
    // PR-controlled booleans into permission to move private source.
    if r.source_repo_id != r.runner_repo_id {
        return Err("cross-organization source export not authorized");
    }
    if !canonical_sha(r.expected_commit)
        || !canonical_sha(r.actual_commit)
        || !canonical_sha(r.expected_tree)
        || !canonical_sha(r.actual_tree)
        || r.expected_commit != r.actual_commit
        || r.expected_tree != r.actual_tree
    {
        return Err("unverified exact commit/tree");
    }
    if r.runner_id.unwrap_or(0) == 0 {
        return Err("runner not assigned");
    }
    if r.run_status != "completed" || r.job_status != "completed"
        || r.run_conclusion != "success" || r.job_conclusion != "success"
    {
        return Err("run/job not successful");
    }
    if r.steps.is_empty() {
        return Err("no executed steps");
    }
    if r.steps.iter().any(|s| {
        s.name.is_empty() || s.status != "completed" || s.conclusion != "success"
    }) {
        return Err("step not successfully executed");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const S: &[Step<'static>] = &[
        Step { name: "Set up job", status: "completed", conclusion: "success" },
        Step { name: "Compile", status: "completed", conclusion: "success" },
        Step { name: "Run tests", status: "completed", conclusion: "success" },
    ];

    fn local<'a>() -> Receipt<'a> {
        Receipt {
            source_repo_id: 101,
            runner_repo_id: 101,
            trusted_workflow_on_protected_ref: true,
            expected_commit: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            actual_commit: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            expected_tree: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            actual_tree: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            runner_id: Some(818),
            run_status: "completed",
            run_conclusion: "success",
            job_status: "completed",
            job_conclusion: "success",
            steps: S,
        }
    }

    #[test] fn permits_synthetic_executed_local_success() {
        assert_eq!(validate(&local()), Ok(()));
    }
    #[test] fn rejects_no_runner() {
        let mut r=local(); r.runner_id = Some(0);
        assert!(validate(&r).is_err());
        r.runner_id = None;
        assert!(validate(&r).is_err());
    }
    #[test] fn rejects_empty_and_missing_steps() {
        let mut r=local(); r.steps = &[];
        assert_eq!(validate(&r), Err("no executed steps"));
    }
    #[test] fn rejects_skipped_failed_and_queued_steps() {
        for (status,conclusion) in [("completed","skipped"),("completed","failure"),("queued","success")] {
            let step = [Step { name:"test",status,conclusion }];
            let mut r=local(); r.steps = &step;
            assert!(validate(&r).is_err());
        }
    }
    #[test] fn rejects_partial_or_failed_workflow() {
        for (status,conclusion) in [("in_progress",""),("completed","failure"),("completed","cancelled")] {
            let mut r=local(); r.run_status=status; r.run_conclusion=conclusion;
            assert!(validate(&r).is_err());
        }
    }
    #[test] fn rejects_failed_or_pending_job() {
        let mut r=local(); r.job_conclusion="failure";
        assert!(validate(&r).is_err());
        r.job_status="queued"; r.job_conclusion="success";
        assert!(validate(&r).is_err());
    }
    #[test] fn rejects_wrong_commit_and_tree() {
        let mut r=local(); r.actual_commit="cccccccccccccccccccccccccccccccccccccccc";
        assert!(validate(&r).is_err());
        r=local(); r.actual_tree="dddddddddddddddddddddddddddddddddddddddd";
        assert!(validate(&r).is_err());
    }
    #[test] fn rejects_invalid_sha_strings() {
        for invalid in ["", "not-a-git-sha", "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            "0000000000000000000000000000000000000000", "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaZ"] {
            let mut r=local(); r.actual_commit=invalid;
            assert!(validate(&r).is_err());
        }
    }
    #[test] fn rejects_wrong_repo_and_untrusted_workflow() {
        let mut r=local(); r.source_repo_id=0;
        assert!(validate(&r).is_err());
        r=local(); r.trusted_workflow_on_protected_ref=false;
        assert!(validate(&r).is_err());
    }
    #[test] fn rejects_unapproved_cross_org_carrier() {
        let mut r=local(); r.runner_repo_id=202;
        assert_eq!(validate(&r), Err("cross-organization source export not authorized"));
    }
    #[test]
    fn denies_foreign_destination_for_many_repository_ids() {
        let mut r = local();
        for destination_id in [0, 1, 100, 102, 202, u64::MAX] {
            r.runner_repo_id = destination_id;
            assert_eq!(
                validate(&r),
                Err(if destination_id == 0 {
                    "missing immutable repository ID"
                } else {
                    "cross-organization source export not authorized"
                }),
                "unapproved destination {destination_id}"
            );
        }
    }
    #[test]
    fn denies_foreign_destination_even_when_other_evidence_is_valid() {
        let mut r = local();
        r.runner_repo_id = 202;
        assert_eq!(
            validate(&r),
            Err("cross-organization source export not authorized")
        );
        assert!(canonical_sha(r.expected_commit));
        assert!(canonical_sha(r.actual_tree));
        assert_eq!(r.run_conclusion, "success");
    }
    #[test] fn denies_empty_step_name() {
        let step=[Step {name:"",status:"completed",conclusion:"success"}];
        let mut r=local(); r.steps=&step;
        assert!(validate(&r).is_err());
    }
}


/// Authenticated API *shaped* test fixtures. This module does NOT perform HTTP,
/// authenticate API data, or decide repository-wide export authorization.
/// A production caller must obtain these facts from trusted, independently
/// protected infrastructure and must never accept PR-supplied JSON as facts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Execution {
    NotExecuted,
    ExecutedFailed,
    ExecutedPassed,
}

#[derive(Debug, Clone)]
struct GithubRun<'a> {
    id: u64,
    repo_id: u64,
    workflow_id: u64,
    head_sha: &'a str,
    event: &'a str,
    status: &'a str,
    conclusion: &'a str,
}

#[derive(Debug, Clone)]
struct GithubJob<'a> {
    id: u64,
    run_id: u64,
    runner_id: Option<u64>,
    status: &'a str,
    conclusion: &'a str,
    steps: Option<&'a [Step<'a>]>,
}

#[derive(Debug, Clone)]
struct GithubCheck<'a> {
    id: u64,
    repo_id: u64,
    run_id: u64,
    job_id: u64,
    head_sha: &'a str,
    app_id: u64,
    status: &'a str,
    conclusion: &'a str,
}

#[derive(Debug, Clone)]
struct GithubBranchProtection<'a> {
    repo_id: u64,
    base_ref: &'a str,
    protected: bool,
    requires_reviews: bool,
    requires_codeowners: bool,
    requires_checks: bool,
    applies_to_admins: bool,
    permits_bypass: bool,
}

#[derive(Debug, Clone)]
struct GithubReview<'a> {
    review_id: u64,
    reviewer_id: u64,
    reviewed_head_sha: &'a str,
    state: &'a str,
    // Must be fetched independently through GitHub collaborator permissions,
    // never taken from a user-submitted PR review body.
    permission: &'a str,
}

#[derive(Debug, Clone)]
struct TrustedReleasePolicy<'a> {
    source_repo_id: u64,
    expected_workflow_id: u64,
    expected_app_id: u64,
    expected_commit: &'a str,
    expected_root_tree: &'a str,
    base_ref: &'a str,
    pr_author_id: u64,
}

// An independently collected checkout tree receipt would be a separate
// authenticated input. This structure alone is mock data, not a signature,
// identity attestation, or a GitHub Actions run API field.
#[derive(Debug, Clone)]
struct GithubCheckoutReceipt<'a> {
    source_repo_id: u64,
    run_id: u64,
    job_id: u64,
    checked_commit: &'a str,
    checked_root_tree: &'a str,
}

fn classify_execution(run: &GithubRun<'_>, job: &GithubJob<'_>) -> Execution {
    if run.id == 0 || job.id == 0 || run.id != job.run_id
        || job.runner_id.unwrap_or(0) == 0
        || job.steps.is_none_or(|steps| steps.is_empty())
        || run.status != "completed" || job.status != "completed"
    {
        return Execution::NotExecuted;
    }
    let steps = job.steps.expect("nonempty step evidence required");
    if run.conclusion != "success" || job.conclusion != "success"
        || steps.iter().any(|s| s.name.trim().is_empty()
            || s.status != "completed" || s.conclusion != "success")
    {
        return Execution::ExecutedFailed;
    }
    Execution::ExecutedPassed
}

fn verify_native_release(
    policy: &TrustedReleasePolicy<'_>,
    run: &GithubRun<'_>,
    job: &GithubJob<'_>,
    check: &GithubCheck<'_>,
    protection: &GithubBranchProtection<'_>,
    reviews: &[GithubReview<'_>],
    checkout: Option<&GithubCheckoutReceipt<'_>>,
) -> Result<(), &'static str> {
    if policy.source_repo_id == 0
        || run.repo_id != policy.source_repo_id
        || check.repo_id != policy.source_repo_id
        || protection.repo_id != policy.source_repo_id
    {
        return Err("immutable source repository identity mismatch");
    }
    // This evidence gate is source-local only. Private source export is
    // categorically NOT authorized by this validator.
    if run.workflow_id == 0 || run.workflow_id != policy.expected_workflow_id
        || check.app_id != policy.expected_app_id || check.id == 0
        || check.run_id != run.id || check.job_id != job.id
    {
        return Err("untrusted workflow or issuing app / check linkage");
    }
    if !canonical_sha(policy.expected_commit)
        || run.head_sha != policy.expected_commit
        || check.head_sha != policy.expected_commit
    {
        return Err("stale commit or check");
    }
    let receipt = checkout.ok_or("missing independent checkout tree receipt")?;
    if receipt.source_repo_id != policy.source_repo_id
        || receipt.run_id != run.id || receipt.job_id != job.id
        || !canonical_sha(policy.expected_root_tree)
        || !canonical_sha(receipt.checked_commit)
        || !canonical_sha(receipt.checked_root_tree)
        || receipt.checked_commit != policy.expected_commit
        || receipt.checked_root_tree != policy.expected_root_tree
    {
        return Err("wrong or unverified checkout commit/root tree");
    }
    if run.event != "push" && run.event != "pull_request" && run.event != "merge_group" {
        return Err("unexpected workflow event");
    }
    if protection.base_ref != policy.base_ref
        || !protection.protected
        || !protection.requires_reviews
        || !protection.requires_codeowners
        || !protection.requires_checks
        || !protection.applies_to_admins
        || protection.permits_bypass
    {
        return Err("missing effective base-branch protection");
    }
    if policy.pr_author_id == 0 || reviews.is_empty()
        || !reviews.iter().any(|r| {
            r.review_id != 0
                && r.reviewer_id != 0 && r.reviewer_id != policy.pr_author_id
                && r.reviewed_head_sha == policy.expected_commit
                && r.state == "APPROVED"
                && matches!(r.permission, "write" | "maintain" | "admin")
        })
    {
        return Err("no independent exact-head approval");
    }
    if reviews.iter().any(|r| r.state == "CHANGES_REQUESTED") {
        return Err("outstanding change request");
    }
    if check.status != "completed" || check.conclusion != "success" {
        return Err("trusted check did not succeed");
    }
    match classify_execution(run, job) {
        Execution::NotExecuted => Err("not executed"),
        Execution::ExecutedFailed => Err("executed tests failed"),
        Execution::ExecutedPassed => Ok(()),
    }
}

#[cfg(test)]
mod api_fact_tests {
    use super::*;
    const STEPS: &[Step<'static>] = &[
        Step { name: "Checkout", status: "completed", conclusion: "success" },
        Step { name: "Native tests", status: "completed", conclusion: "success" },
    ];

    fn fixture<'a>() -> (
        TrustedReleasePolicy<'a>, GithubRun<'a>, GithubJob<'a>,
        GithubCheck<'a>, GithubBranchProtection<'a>, Vec<GithubReview<'a>>,
        GithubCheckoutReceipt<'a>
    ) {
        let sha = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        (
            TrustedReleasePolicy {
                source_repo_id: 901, expected_workflow_id: 200, expected_app_id: 15368,
                expected_commit: sha,
                expected_root_tree: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                base_ref: "main", pr_author_id: 100,
            },
            GithubRun {
                id: 400, repo_id: 901, workflow_id: 200, head_sha: sha,
                event: "push", status: "completed", conclusion: "success",
            },
            GithubJob {
                id: 500, run_id: 400, runner_id: Some(1),
                status: "completed", conclusion: "success", steps: Some(STEPS),
            },
            GithubCheck {
                id: 600, repo_id: 901, run_id: 400, job_id: 500,
                head_sha: sha, app_id: 15368, status: "completed", conclusion: "success",
            },
            GithubBranchProtection {
                repo_id: 901, base_ref: "main", protected: true,
                requires_reviews: true, requires_codeowners: true,
                requires_checks: true, applies_to_admins: true, permits_bypass: false,
            },
            vec![GithubReview {
                review_id: 700, reviewer_id: 101, reviewed_head_sha: sha,
                state: "APPROVED", permission: "write",
            }],
            GithubCheckoutReceipt {
                source_repo_id: 901, run_id: 400, job_id: 500,
                checked_commit: sha,
                checked_root_tree: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            },
        )
    }

    #[test]
    fn allows_only_synthetic_fully_reviewed_source_local_receipt() {
        let (p,r,j,c,b,v, proof) = fixture();
        assert_eq!(verify_native_release(&p,&r,&j,&c,&b,&v,Some(&proof)), Ok(()));
    }

    #[test]
    fn classifies_observed_zero_step_source_runner_failure() {
        let (_,mut r,mut j,_,_,_, _proof) = fixture();
        r.conclusion="failure"; j.conclusion="failure";
        j.runner_id=None; j.steps=None;
        assert_eq!(classify_execution(&r,&j), Execution::NotExecuted);
        j.runner_id=Some(11); j.steps=Some(&[]);
        assert_eq!(classify_execution(&r,&j), Execution::NotExecuted);
    }

    #[test]
    fn distinguishes_real_native_test_failures_from_admission() {
        let (_,mut r,mut j,_,_,_, _proof) = fixture();
        r.conclusion="failure"; j.conclusion="failure";
        assert_eq!(classify_execution(&r,&j), Execution::ExecutedFailed);
        r.conclusion="success"; j.conclusion="success";
        assert_eq!(classify_execution(&r,&j), Execution::ExecutedPassed);
    }

    #[test]
    fn denies_cross_repository_and_forged_check_references() {
        let (p,mut r,mut j,mut c,b,v, proof) = fixture();
        r.repo_id=902;
        assert!(verify_native_release(&p,&r,&j,&c,&b,&v,Some(&proof)).is_err());
        r.repo_id=901; c.repo_id=902;
        assert!(verify_native_release(&p,&r,&j,&c,&b,&v,Some(&proof)).is_err());
        c.repo_id=901; c.run_id=401;
        assert!(verify_native_release(&p,&r,&j,&c,&b,&v,Some(&proof)).is_err());
        c.run_id=400; j.run_id=401;
        assert!(verify_native_release(&p,&r,&j,&c,&b,&v,Some(&proof)).is_err());
    }

    #[test]
    fn denies_wrong_check_issuer_or_workflow_id() {
        let (mut p,mut r,j,mut c,b,v, proof) = fixture();
        c.app_id=1234;
        assert!(verify_native_release(&p,&r,&j,&c,&b,&v,Some(&proof)).is_err());
        c.app_id=15368; r.workflow_id=201;
        assert!(verify_native_release(&p,&r,&j,&c,&b,&v,Some(&proof)).is_err());
        r.workflow_id=200; p.expected_workflow_id=0;
        assert!(verify_native_release(&p,&r,&j,&c,&b,&v,Some(&proof)).is_err());
    }

    #[test]
    fn denies_stale_sha_or_forged_merge_check() {
        let (p,mut r,j,mut c,b,v, proof) = fixture();
        c.head_sha="bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        assert!(verify_native_release(&p,&r,&j,&c,&b,&v,Some(&proof)).is_err());
        c.head_sha=p.expected_commit;
        r.head_sha="bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        assert!(verify_native_release(&p,&r,&j,&c,&b,&v,Some(&proof)).is_err());
    }

    #[test]
    fn denies_missing_branch_protection_rules() {
        let (p,r,j,c,mut b,v, proof) = fixture();
        b.protected=false; assert!(verify_native_release(&p,&r,&j,&c,&b,&v,Some(&proof)).is_err());
        b.protected=true; b.requires_checks=false;
        assert!(verify_native_release(&p,&r,&j,&c,&b,&v,Some(&proof)).is_err());
        b.requires_checks=true; b.requires_codeowners=false;
        assert!(verify_native_release(&p,&r,&j,&c,&b,&v,Some(&proof)).is_err());
        b.requires_codeowners=true; b.applies_to_admins=false;
        assert!(verify_native_release(&p,&r,&j,&c,&b,&v,Some(&proof)).is_err());
        b.applies_to_admins=true; b.permits_bypass=true;
        assert!(verify_native_release(&p,&r,&j,&c,&b,&v,Some(&proof)).is_err());
    }

    #[test]
    fn denies_author_self_approval_or_unknown_permission() {
        let (p,r,j,c,b,mut v, proof) = fixture();
        v[0].reviewer_id=100;
        assert!(verify_native_release(&p,&r,&j,&c,&b,&v,Some(&proof)).is_err());
        v[0].reviewer_id=101; v[0].permission="read";
        assert!(verify_native_release(&p,&r,&j,&c,&b,&v,Some(&proof)).is_err());
        v[0].permission="write"; v[0].reviewed_head_sha="bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        assert!(verify_native_release(&p,&r,&j,&c,&b,&v,Some(&proof)).is_err());
    }

    #[test]
    fn denies_outstanding_change_request_even_with_approval() {
        let (p,r,j,c,b,mut v, proof) = fixture();
        v.push(GithubReview {
            review_id: 701, reviewer_id: 102, reviewed_head_sha: p.expected_commit,
            state: "CHANGES_REQUESTED", permission: "write",
        });
        assert!(verify_native_release(&p,&r,&j,&c,&b,&v,Some(&proof)).is_err());
    }

    #[test]
    fn denies_skipped_job_or_workflow_event_spoof() {
        let (p,mut r,mut j,c,b,v, proof) = fixture();
        j.status="queued";
        assert!(verify_native_release(&p,&r,&j,&c,&b,&v,Some(&proof)).is_err());
        j.status="completed"; r.event="workflow_dispatch";
        assert!(verify_native_release(&p,&r,&j,&c,&b,&v,Some(&proof)).is_err());
    }

    #[test]
    fn denies_github_check_without_success() {
        let (p,r,j,mut c,b,v, proof) = fixture();
        c.conclusion="neutral";
        assert!(verify_native_release(&p,&r,&j,&c,&b,&v,Some(&proof)).is_err());
        c.conclusion="success"; c.status="in_progress";
        assert!(verify_native_release(&p,&r,&j,&c,&b,&v,Some(&proof)).is_err());
    }
    #[test]
    fn denies_missing_or_wrong_checkout_tree_receipt() {
        let (p,r,j,c,b,v,mut proof)=fixture();
        assert_eq!(
            verify_native_release(&p,&r,&j,&c,&b,&v,None),
            Err("missing independent checkout tree receipt")
        );
        proof.checked_root_tree="cccccccccccccccccccccccccccccccccccccccc";
        assert_eq!(
            verify_native_release(&p,&r,&j,&c,&b,&v,Some(&proof)),
            Err("wrong or unverified checkout commit/root tree")
        );
        proof.checked_root_tree=p.expected_root_tree;
        proof.run_id=401;
        assert!(verify_native_release(&p,&r,&j,&c,&b,&v,Some(&proof)).is_err());
        proof.run_id=400;
        proof.checked_commit="cccccccccccccccccccccccccccccccccccccccc";
        assert!(verify_native_release(&p,&r,&j,&c,&b,&v,Some(&proof)).is_err());
        proof.checked_commit=p.expected_commit;
        proof.source_repo_id=902;
        assert!(verify_native_release(&p,&r,&j,&c,&b,&v,Some(&proof)).is_err());
    }

}
