// Read-only CI recovery decision engine. Produces a non-authorizing plan;
// DOES NOT send GitHub requests, copy Git objects, launch CI or approve exports.
// Dynamic evidence is untrusted until authenticated by an independent provider.
const SHA40 = /^[a-f0-9]{40}$/i;
const REPO = /^[A-Za-z0-9][A-Za-z0-9-]{0,38}\/[A-Za-z0-9_.-]{1,100}$/;
const id = n => Number.isSafeInteger(n) && n > 0;
const sha = s => typeof s === 'string' && SHA40.test(s);
const repo = s => typeof s === 'string' && REPO.test(s) && !s.includes('..');
const issue = (status, action, details = {}) => ({ status, action, ...details });
const SAFE_DIAGNOSTICS = new Set([
  'NOT_EXECUTED', 'TEST_FAILED', 'INCOMPLETE_JOB_EVIDENCE',
  'INCONSISTENT_JOB_EVIDENCE', 'PENDING_OR_INCONCLUSIVE',
  'STEPFUL_RUN_REPORTED_SUCCESS', 'STALE_HEAD',
]);

// This is NOT a policy authorization verifier. Even a JSON document saying
// "approved": true is never trusted by this code as an export permit.
export function decideRecovery(input) {
  if (!input || typeof input !== 'object' || Array.isArray(input)) {
    return issue('EVIDENCE_INVALID', 'collect authenticated source/run records');
  }
  const source = input.source, observed = input.observed;
  if (!source || !repo(source.repository) || !id(source.repo_id) ||
      !sha(source.head_sha) || !sha(source.tree_sha) ||
      !id(source.run_id) || !SAFE_DIAGNOSTICS.has(observed?.status)) {
    return issue('EVIDENCE_INVALID', 'obtain complete immutable source and run metadata');
  }
  const base = {
    source_repository: source.repository,
    source_repo_id: source.repo_id,
    source_head_sha: source.head_sha,
    source_tree_sha: source.tree_sha,
    observed_run_id: source.run_id,
    observed_status: observed.status,
    authorized_private_copy: false,
    mirror_performed: false,
    material_tests_verified: false,
  };

  if (observed.status === 'TEST_FAILED') {
    return issue('DIAGNOSE_ACTUAL_TEST_FAILURE', 'investigate executed step logs; fix and retest exact head', base);
  }
  if (observed.status === 'STEPFUL_RUN_REPORTED_SUCCESS') {
    return issue('TEST_COVERAGE_UNVERIFIED', 'verify required suites and exact-head provenance before any pass claim', base);
  }
  if (observed.status === 'STALE_HEAD') {
    return issue('RERUN_CURRENT_HEAD', 'recheck latest immutable PR head before recovery', base);
  }
  if (observed.status !== 'NOT_EXECUTED') {
    return issue('RECHECK_RUN_EVIDENCE', 'fetch complete exact-attempt GitHub job pages, annotations and admission reason', base);
  }
  const candidate = input.carrier;
  if (!candidate) {
    return issue('PROBE_CARRIER_CAPACITY', 'find alternate private GitHub organization and execute a source-free canary; zero steps alone do not prove minutes exhausted', base);
  }
  if (!repo(candidate.repository) || !id(candidate.repo_id) ||
      candidate.repository === source.repository ||
      candidate.repository.split('/')[0].toLowerCase() === source.repository.split('/')[0].toLowerCase() ||
      candidate.visibility !== 'private') {
    return issue('CARRIER_REJECTED', 'use a different authorized organization and genuinely private carrier', base);
  }
  const probe = candidate.probe;
  if (!probe || probe.conclusion !== 'success' || !id(probe.run_id) ||
      !id(probe.runner_id) || !id(probe.executed_steps) ||
      !sha(probe.head_sha) || !sha(probe.carrier_commit_sha) ||
      probe.head_sha !== probe.carrier_commit_sha ||
      probe.job_status !== 'completed') {
    return issue('PROBE_CARRIER_CAPACITY', 'obtain a fresh source-free, stepful local carrier Actions run and record exact immutable head', base);
  }
  const carrierReceipt = {
    carrier_repository: candidate.repository,
    carrier_repo_id: candidate.repo_id,
    probe_run_id: probe.run_id,
    probe_head_sha: probe.head_sha,
    probe_executed_steps: probe.executed_steps,
  };
  if (source.visibility === 'private' || source.visibility === 'internal') {
    // Private data boundaries require *independently checked*, live
    // source-policy approval bound to source/destination IDs and SHA.
    // Inline JSON approval claims cannot satisfy that requirement.
    return issue('PRIVATE_EXPORT_AUTHORIZATION_REQUIRED',
      'verify live independent source-owner/security approval, repo-specific DENIED holds and enforcing allowlist before any private blob transfer',
      { ...base, ...carrierReceipt, approval_claim_ignored: true });
  }
  if (source.visibility !== 'public') {
    return issue('SOURCE_VISIBILITY_UNKNOWN',
      'authenticate source repository visibility before considering transfer', { ...base, ...carrierReceipt });
  }
  return issue('PUBLIC_MIRROR_PLAN_ONLY',
    'public source may be mirrored by an independent authorized executor; first verify all blob path/mode/OIDs, exact root tree, then run material suites',
    { ...base, ...carrierReceipt });
}

export function formatRecoveryHandoff(input) {
  const result = decideRecovery(input);
  const text = [
    'AttoLang CI recovery handoff (READ ONLY, NON-AUTHORIZING)',
    'decision=' + result.status,
    'source=' + (result.source_repository ?? 'UNKNOWN'),
    'source_commit=' + (result.source_head_sha ?? 'UNKNOWN'),
    'source_tree=' + (result.source_tree_sha ?? 'UNKNOWN'),
    'observed_run=' + (result.observed_run_id ?? 'UNKNOWN'),
    'carrier=' + (result.carrier_repository ?? 'NOT_PROVEN'),
    'approved_private_export=NO',
    'source_mirror_done=NO',
    'material_tests_verified=NO',
    'next_action=' + result.action,
  ];
  return text.join('\n') + '\n';
}
