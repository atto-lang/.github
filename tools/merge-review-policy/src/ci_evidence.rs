//! Independent, exact-head classification of GitHub Actions job receipts.
//! "steps=[]" (or null) and runner_id=0 are NEVER executed test evidence.
//! This module does not infer which compiler, suite, or guest workload ran.
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobEvidence {
    AwaitingRunner,
    RunnerAdmissionFailure,
    StepsRunning,
    ExecutedFailed,
    ExecutedPassed,
}

/// A completed job with no executed steps is a runner-admission failure.
/// A successful job with nonempty steps is only proof of *those host job steps*,
/// not proof of guest execution or parity. The caller must independently
/// authenticate which steps ran and tie those to executable artifacts.
pub fn classify_job(job: &Value, expected_head: &str) -> Result<JobEvidence, String> {
    if expected_head.len() != 40 || !expected_head.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("expected head must be an exact 40-hex SHA".into());
    }
    if job["head_sha"].as_str() != Some(expected_head) {
        return Err("job is not attached to the requested exact head".into());
    }
    let attempt = job["run_attempt"].as_u64().ok_or("missing run attempt")?;
    if attempt == 0 { return Err("run attempt cannot be zero".into()); }
    let status = job["status"].as_str().ok_or("missing job status")?;
    let runner = job["runner_id"].as_u64().unwrap_or(0);
    let steps = match job.get("steps") {
        None | Some(Value::Null) => &[][..],
        Some(Value::Array(v)) => v.as_slice(),
        _ => return Err("malformed step list".into()),
    };
    match status {
        "queued" | "waiting" | "pending" | "requested" => {
            if runner != 0 || !steps.is_empty() {
                return Err("queued job contains impossible runner or executed steps".into());
            }
            return Ok(JobEvidence::AwaitingRunner);
        }
        "in_progress" => {
            if steps.is_empty() && runner == 0 { return Ok(JobEvidence::AwaitingRunner); }
            if runner == 0 { return Err("running job has steps but no runner assignment".into()); }
            return Ok(JobEvidence::StepsRunning);
        }
        "completed" => {}
        _ => return Err("unknown job status".into()),
    }
    if steps.is_empty() {
        return Ok(JobEvidence::RunnerAdmissionFailure);
    }
    if runner == 0 {
        return Err("completed job reports steps but no runner assignment".into());
    }
    let mut succeeded = 0_u32;
    for step in steps {
        if !step.is_object() { return Err("malformed job step".into()); }
        match step["conclusion"].as_str() {
            Some("success") => succeeded += 1,
            Some("skipped") => {},
            Some("failure" | "cancelled" | "timed_out" | "action_required") => {},
            Some(_) | None => return Err("step has unknown or missing conclusion".into()),
        }
    }
    if succeeded == 0 {
        return Err("no successfully executed steps; cannot claim execution".into());
    }
    match job["conclusion"].as_str() {
        Some("success") if steps.iter().all(|s| matches!(s["conclusion"].as_str(), Some("success" | "skipped"))) =>
            Ok(JobEvidence::ExecutedPassed),
        Some("success") => Err("successful job conflicts with failed or cancelled step".into()),
        Some("failure" | "timed_out" | "cancelled" | "action_required") => Ok(JobEvidence::ExecutedFailed),
        _ => Err("missing or unknown completed-job conclusion".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const SHA: &str = "ba15cfe08e85bf51389a8c9c33e9caafa225dbd0";
    fn job(status: &str, conclusion: &str, runner_id: u64, steps: Value) -> Value {
        json!({"head_sha":SHA,"run_attempt":1,"status":status,"conclusion":conclusion,
            "runner_id":runner_id,"steps":steps})
    }

    #[test]
    fn zero_step_failed_job_is_runner_admission_not_failing_test() {
        assert_eq!(classify_job(&job("completed","failure",0,json!([])),SHA),
            Ok(JobEvidence::RunnerAdmissionFailure));
    }
    #[test]
    fn null_steps_cannot_be_test_evidence() {
        assert_eq!(classify_job(&job("completed","success",0,Value::Null),SHA),
            Ok(JobEvidence::RunnerAdmissionFailure));
    }
    #[test]
    fn missing_steps_cannot_be_test_evidence() {
        let mut j = job("completed","failure",0,Value::Null);
        j.as_object_mut().unwrap().remove("steps");
        assert_eq!(classify_job(&j,SHA),Ok(JobEvidence::RunnerAdmissionFailure));
    }
    #[test]
    fn queued_job_is_not_a_pass() {
        assert_eq!(classify_job(&job("queued","",0,json!([])),SHA),
            Ok(JobEvidence::AwaitingRunner));
    }
    #[test]
    fn in_progress_with_runner_is_not_a_pass() {
        assert_eq!(classify_job(&job("in_progress","",7,json!([{"conclusion":"success"}])),SHA),
            Ok(JobEvidence::StepsRunning));
    }
    #[test]
    fn successful_real_steps_are_host_evidence_only() {
        assert_eq!(classify_job(&job("completed","success",77,
            json!([{"conclusion":"success"},{"conclusion":"skipped"}])),SHA),
            Ok(JobEvidence::ExecutedPassed));
    }
    #[test]
    fn failing_executed_steps_are_not_admission_failure() {
        assert_eq!(classify_job(&job("completed","failure",77,
            json!([{"conclusion":"success"},{"conclusion":"failure"}])),SHA),
            Ok(JobEvidence::ExecutedFailed));
    }
    #[test]
    fn forged_success_with_failed_step_is_rejected() {
        assert!(classify_job(&job("completed","success",77,
            json!([{"conclusion":"success"},{"conclusion":"failure"}])),SHA).is_err());
    }
    #[test]
    fn zero_runner_with_forged_steps_is_rejected() {
        assert!(classify_job(&job("completed","success",0,
            json!([{"conclusion":"success"}])),SHA).is_err());
    }
    #[test]
    fn wrong_exact_head_is_rejected() {
        assert!(classify_job(&job("completed","success",77,
            json!([{"conclusion":"success"}])),
            "0000000000000000000000000000000000000000").is_err());
    }
    #[test]
    fn malformed_head_is_rejected() {
        assert!(classify_job(&job("completed","success",77,
            json!([{"conclusion":"success"}])),"main").is_err());
    }
    #[test]
    fn missing_run_attempt_is_rejected() {
        let mut j = job("completed","success",77,json!([{"conclusion":"success"}]));
        j.as_object_mut().unwrap().remove("run_attempt");
        assert!(classify_job(&j,SHA).is_err());
    }
    #[test]
    fn zero_run_attempt_is_rejected() {
        let mut j = job("completed","success",77,json!([{"conclusion":"success"}]));
        j["run_attempt"] = json!(0);
        assert!(classify_job(&j,SHA).is_err());
    }
    #[test]
    fn malformed_step_list_is_rejected() {
        assert!(classify_job(&job("completed","success",77,json!({})),SHA).is_err());
    }
    #[test]
    fn all_skipped_steps_are_not_execution_proof() {
        assert!(classify_job(&job("completed","success",77,
            json!([{"conclusion":"skipped"}])),SHA).is_err());
    }
    #[test]
    fn cancelled_without_steps_is_runner_admission_category() {
        assert_eq!(classify_job(&job("completed","cancelled",0,json!([])),SHA),
            Ok(JobEvidence::RunnerAdmissionFailure));
    }
    #[test]
    fn unknown_step_conclusion_fails_closed() {
        assert!(classify_job(&job("completed","success",77,
            json!([{"conclusion":"magic"}])),SHA).is_err());
    }

    // Source-contract tests guard workflow provenance even while runner
    // admission is unavailable. They do not constitute executed CI evidence.
    fn exact_head_workflow_contract(workflow: &str) -> bool {
        workflow.contains("ref: ${{ github.event.pull_request.head.sha || github.sha }}")
            && workflow.contains("EXPECTED_HEAD: ${{ github.event.pull_request.head.sha || github.sha }}")
            && workflow.contains("test \"${HEAD}\" = \"${EXPECTED_HEAD}\"")
            && workflow.contains("git rev-parse 'HEAD^{tree}'")
            && workflow.contains("persist-credentials: false")
    }

    #[test]
    fn rust_workflow_runs_all_checks_even_if_earlier_check_fails() {
        let workflow = include_str!("../../../.github/workflows/merge-review-policy-rust.yml");
        let test = workflow.find("- name: Execute Rust policy unit tests").unwrap();
        let clippy = workflow.find("- name: Check Rust policy with clippy").unwrap();
        let fmt = workflow.find("- name: Check Rust formatting").unwrap();
        assert!(test < clippy && clippy < fmt, "Cargo tests must execute before formatting");
        assert_eq!(workflow.matches("if: success() || failure()").count(), 2);
        assert!(!workflow.contains("continue-on-error: true"), "all gates must remain required");
    }

    #[test]
    fn mutation_that_skips_clippy_on_test_failure_is_rejected() {
        let workflow = include_str!("../../../.github/workflows/merge-review-policy-rust.yml");
        let mutation = workflow.replacen(
            "if: success() || failure()", "if: success()", 1
        );
        assert_ne!(mutation, workflow);
        assert_ne!(mutation.matches("if: success() || failure()").count(), 2);
    }

    #[test]
    fn rust_workflow_checks_out_actual_pr_head_not_synthetic_merge() {
        let workflow = include_str!("../../../.github/workflows/merge-review-policy-rust.yml");
        assert!(exact_head_workflow_contract(workflow));
    }

    #[test]
    fn mutation_removing_explicit_pr_head_checkout_is_rejected() {
        let workflow = include_str!("../../../.github/workflows/merge-review-policy-rust.yml");
        let mutation = workflow.replace(
            "ref: ${{ github.event.pull_request.head.sha || github.sha }}",
            "ref: ${{ github.sha }}",
        );
        assert_ne!(mutation, workflow);
        assert!(!exact_head_workflow_contract(&mutation));
    }

    #[test]
    fn mutation_removing_exact_head_comparison_is_rejected() {
        let workflow = include_str!("../../../.github/workflows/merge-review-policy-rust.yml");
        let mutation = workflow.replace("test \"${HEAD}\" = \"${EXPECTED_HEAD}\"", "true");
        assert_ne!(mutation, workflow);
        assert!(!exact_head_workflow_contract(&mutation));
    }

    #[test]
    fn mutation_removing_tree_receipt_is_rejected() {
        let workflow = include_str!("../../../.github/workflows/merge-review-policy-rust.yml");
        let mutation = workflow.replace("git rev-parse 'HEAD^{tree}'", "git status --porcelain");
        assert_ne!(mutation, workflow);
        assert!(!exact_head_workflow_contract(&mutation));
    }

    #[test]
    fn queued_with_steps_is_invalid() {
        assert!(classify_job(&job("queued","",0,json!([{"conclusion":"success"}])),SHA).is_err());
    }
}
