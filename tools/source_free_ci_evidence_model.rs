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
    approved_destination_id: Option<u64>,
    destination_is_private: bool,
    independent_export_approval: bool,
    disclosure_incidents_cleared: bool,
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
    if r.source_repo_id != r.runner_repo_id {
        if !r.destination_is_private
            || r.approved_destination_id != Some(r.runner_repo_id)
            || !r.independent_export_approval
            || !r.disclosure_incidents_cleared
        {
            return Err("cross-organization source export not authorized");
        }
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
            approved_destination_id: None,
            destination_is_private: true,
            independent_export_approval: false,
            disclosure_incidents_cleared: false,
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
    #[test] fn denies_cross_org_without_each_gate() {
        let mut r=local();
        r.runner_repo_id=202;
        r.approved_destination_id=Some(202);
        r.destination_is_private=true;
        r.independent_export_approval=true;
        r.disclosure_incidents_cleared=true;
        assert_eq!(validate(&r), Ok(())); // synthetic model: NOT real authorization
        r.destination_is_private=false;
        assert!(validate(&r).is_err());
        r.destination_is_private=true;
        r.approved_destination_id=Some(203);
        assert!(validate(&r).is_err());
        r.approved_destination_id=Some(202);
        r.independent_export_approval=false;
        assert!(validate(&r).is_err());
        r.independent_export_approval=true;
        r.disclosure_incidents_cleared=false;
        assert!(validate(&r).is_err());
    }
    #[test] fn denies_empty_step_name() {
        let step=[Step {name:"",status:"completed",conclusion:"success"}];
        let mut r=local(); r.steps=&step;
        assert!(validate(&r).is_err());
    }
}
