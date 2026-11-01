#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
t=$(mktemp -d)
trap 'rm -rf "$t"' EXIT
sha=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
printf '{"id":42,"head_sha":"%s","status":"completed","conclusion":"success","run_attempt":2}\n' "$sha" > "$t/run.json"
cat > "$t/jobs.json" <<'JSON'
{"total_count":1,"jobs":[{"id":7,"name":"test","status":"completed","conclusion":"success","runner_id":33,"runner_name":"hosted","steps":[{"name":"Set up job","status":"completed","conclusion":"success"},{"name":"Run actual test","status":"completed","conclusion":"success"},{"name":"Complete job","status":"completed","conclusion":"success"}]}]}
JSON
verifier=./scripts/ci-evidence/verify-actions-admission.sh
# Run from repository root; the verifier uses only local fixture JSON.
cd ../..
[[ -x "$verifier" ]] || verifier="bash $verifier"
if ! bash scripts/ci-evidence/verify-actions-admission.sh fixture "$t/run.json" "$t/jobs.json" "$sha" 2 >/dev/null; then
  echo 'FAIL pristine executed receipt' >&2; exit 1
fi
reject=0
rejectcase() {
  label=$1; expression=$2; which=$3
  jq "$expression" "$t/$which.json" > "$t/bad.json"
  if [[ "$which" == run ]]; then
    if bash scripts/ci-evidence/verify-actions-admission.sh fixture "$t/bad.json" "$t/jobs.json" "$sha" 2 >/dev/null 2>&1; then echo "ACCEPTED $label" >&2; exit 2; fi
  else
    if bash scripts/ci-evidence/verify-actions-admission.sh fixture "$t/run.json" "$t/bad.json" "$sha" 2 >/dev/null 2>&1; then echo "ACCEPTED $label" >&2; exit 3; fi
  fi
  reject=$((reject+1))
  echo "REJECT $label"
}
rejectcase wrong-head '.head_sha="bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"' run
rejectcase failed-run '.conclusion="failure"' run
rejectcase queued-run '.status="queued"' run
rejectcase missing-attempt 'del(.run_attempt)' run
rejectcase wrong-attempt '.run_attempt=3' run
rejectcase no-jobs '.total_count=0 | .jobs=[]' jobs
rejectcase partial-page '.total_count=101' jobs
rejectcase missing-page '.total_count=2' jobs
rejectcase unassigned-runner '.jobs[0].runner_id=0' jobs
rejectcase empty-runner '.jobs[0].runner_name=""' jobs
rejectcase zero-steps '.jobs[0].steps=[]' jobs
rejectcase internal-steps-only '.jobs[0].steps=[.jobs[0].steps[0],.jobs[0].steps[2]]' jobs
rejectcase failed-step '.jobs[0].steps[1].conclusion="failure"' jobs
rejectcase stale-job '.jobs[0].status="queued"' jobs
rejectcase duplicated-name '.jobs += [.jobs[0]] | .total_count=2 | .jobs[1].id=8' jobs
rejectcase skipped-job '.jobs[0].conclusion="skipped"' jobs
rejectcase duplicate-job-id '.jobs += [.jobs[0]] | .total_count=2 | .jobs[1].name="second"' jobs
rejectcase negative-runner '.jobs[0].runner_id=-1' jobs
rejectcase missing-job-id '.jobs[0] |= del(.id)' jobs
[[ "$reject" -eq 19 ]] || { echo "Expected 19 rejections; observed $reject" >&2; exit 4; }
echo 'PASS 20/20 synthetic admission controls; no compiler execution'
