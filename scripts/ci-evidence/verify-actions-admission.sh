#!/usr/bin/env bash
# Executed GitHub Actions job admission verifier; not native Oreslang evidence.
set -euo pipefail
[[ $# -ge 4 && $# -le 5 ]] || { echo 'usage: verifier fixture RUN_JSON JOBS_JSON HEAD [ATTEMPT] | live OWNER/REPO RUN_ID HEAD [ATTEMPT]' >&2; exit 64; }
mode=$1
expected_sha=$4
expected_attempt=''
if [[ $# == 5 ]]; then expected_attempt=$5; fi
[[ "$expected_sha" =~ ^[a-f0-9]{40}$ ]] || { echo 'REJECT invalid commit SHA' >&2; exit 65; }
[[ -z "$expected_attempt" || "$expected_attempt" =~ ^[1-9][0-9]*$ ]] || { echo 'REJECT invalid run attempt' >&2; exit 65; }
command -v jq >/dev/null || { echo 'REJECT requires jq' >&2; exit 69; }
required_step=${ORES_CI_REQUIRED_STEP-}
[[ -n "$required_step" && "$required_step" != 'Set up job' && "$required_step" != 'Complete job' ]] || {
  echo 'REJECT requires explicit ORES_CI_REQUIRED_STEP matching the executed test step' >&2
  exit 65
}
t=$(mktemp -d)
trap 'rm -rf "$t"' EXIT
run=$t/run.json
jobs=$t/jobs.json
case "$mode" in
  fixture)
    [[ -f "$2" && -f "$3" && ! -L "$2" && ! -L "$3" ]] || { echo 'REJECT invalid fixture files' >&2; exit 66; }
    cp -- "$2" "$run"
    cp -- "$3" "$jobs"
    ;;
  live)
    [[ "$2" =~ ^[a-zA-Z0-9_.-]+/[a-zA-Z0-9_.-]+$ && "$3" =~ ^[1-9][0-9]*$ ]] || { echo 'REJECT bad repository or run ID' >&2; exit 65; }
    [[ -x /usr/bin/gh ]] || { echo 'REJECT requires audited /usr/bin/gh' >&2; exit 69; }
    /usr/bin/gh api --hostname github.com --method GET "repos/$2/actions/runs/$3" > "$run"
    attempt=$(jq -er '.run_attempt | select(type=="number" and .>=1)' "$run") || { echo 'REJECT missing attempt' >&2; exit 67; }
    jq -e --argjson expected "$3" '.id == $expected' "$run" >/dev/null || { echo 'REJECT run ID mismatch' >&2; exit 67; }
    /usr/bin/gh api --hostname github.com --method GET "repos/$2/actions/runs/$3/attempts/$attempt/jobs?per_page=100" > "$jobs"
    ;;
  *) echo 'REJECT invalid mode' >&2; exit 64 ;;
esac
jq -e . "$run" >/dev/null && jq -e . "$jobs" >/dev/null || { echo 'REJECT malformed JSON' >&2; exit 67; }
jq -e --arg sha "$expected_sha" --arg attempt "$expected_attempt" '
  (.head_sha == $sha) and (.status=="completed") and (.conclusion=="success")
  and (.run_attempt|type=="number") and (.run_attempt>=1)
  and ($attempt=="" or (.run_attempt|tostring)==$attempt)
  and (.id|type=="number") and (.id>0)
' "$run" >/dev/null || { echo 'REJECT run head/status/attempt/ID mismatch' >&2; exit 68; }
run_id=$(jq -er '.id | select(type=="number" and .>0)' "$run") || exit 68
run_attempt=$(jq -er '.run_attempt | select(type=="number" and .>=1)' "$run") || exit 68
jq -e --arg required "$required_step" --arg sha "$expected_sha" --argjson run_id "$run_id" --argjson attempt "$run_attempt" '
  (. as $doc
    | (.total_count|type=="number") and (.total_count>=1) and (.total_count<=100)
      and (.jobs|type=="array") and ((.jobs|length)==$doc.total_count)
      and ((.jobs|map(.id)|unique|length)==$doc.total_count)
      and ((.jobs|map(.name)|unique|length)==$doc.total_count)
      and all(.jobs[];
        (.id|type=="number") and (.id>0)
        and (.run_id==$run_id)
        and (.run_attempt==$attempt)
        and (.head_sha==$sha)
        and (.name|type=="string") and (.name|length>0)
        and (.status=="completed") and (.conclusion=="success")
        and (.runner_id|type=="number") and (.runner_id>0)
        and (.runner_name|type=="string") and (.runner_name|length>0)
        and (.steps|type=="array") and (.steps|length>=3)
        and all(.steps[]; .status=="completed" and .conclusion=="success")
        and any(.steps[]; .name==$required and .status=="completed" and .conclusion=="success")
      )
  )
' "$jobs" >/dev/null || { echo 'REJECT unassigned/zero-step/partial/duplicate/failed jobs' >&2; exit 68; }
echo "EXECUTED-CI-ONLY: head=$expected_sha run=$(jq -r .id "$run") attempt=$(jq -r .run_attempt "$run") jobs=$(jq -r .total_count "$jobs") steps=$(jq -r '[.jobs[].steps[]]|length' "$jobs") required_step=$required_step"
echo 'NOT_NATIVE_GUEST_EVIDENCE: separate compiler digest, tree and guest status required'
