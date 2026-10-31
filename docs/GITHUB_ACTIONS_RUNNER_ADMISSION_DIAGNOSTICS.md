# Diagnose zero-step Oreslang GitHub Actions jobs

Status: proposed source-free diagnostic tooling. This is **not a runner fix**,
CI success, permission grant, branch-protection change, or source-export
authorization.

The repository policy [requires Rust rather than Python](RUST_ONLY_POLICY.md).
`tools/gh_run_admission.rs` is a std-only Rust, read-only GitHub CLI wrapper
with an explicit allowlist for the two private compiler source repositories.
It reads only run metadata, the selected job, GitHub check annotations,
the default branch's protected flag, ruleset list, and Actions policy.
It never accesses repository contents, Git objects, artifacts, secrets,
or write endpoints. Every GitHub API request pins the host to `github.com`
and ignores any ambient `GH_HOST` setting. It cannot inspect private Actions billing settings.

## Independent administrator action

From a trusted administrator workstation with the correct GitHub organization
permissions and authenticated GitHub CLI (`gh auth status`), build the
tool from a reviewed Oreslang organization checkout and run:

```sh
rustc --edition=2021 -D warnings --test tools/gh_run_admission.rs -o /tmp/ores-gh-diagnostic-tests
/tmp/ores-gh-diagnostic-tests
rustc --edition=2021 -D warnings tools/gh_run_admission.rs -o /tmp/ores-gh-diagnostic

# Secure local capture: annotations may contain private identifiers.
umask 077
/tmp/ores-gh-diagnostic ores-truffle-oreslang/oreslang-source.llvm 41236569150 124641494087 > llvm-admission-local.txt
/tmp/ores-gh-diagnostic ores-truffle-oreslang/oreslang-source.net 41250696587 124683413992 > net-admission-local.txt
```

The sample run/job IDs refer to real matching jobs. Replace both IDs with
the current **matching** run and job values when diagnosing a later head.
The tool independently reads the selected job's `run_id` **and its actual `check_run_url`**, rejects mismatched run/job IDs, derives the check-run ID only from the exact audited-repository URL, then verifies the check-run `head_sha` and `github-actions` app identity against the workflow run **before querying annotations**. It never assumes a workflow job ID and check-run ID are interchangeable identity domains.
Never equate a successful diagnostic with a successful compiler run.

Review the local output privately before quoting or sanitizing the exact
annotation message in the existing security issues. Do not post tokens,
authenticated command output, private source paths, credentials, or customer
metadata. The diagnostic emits read errors as `UNAVAILABLE` without
printing stderr from `gh` or the selected token. A check-run annotations
403/404 means the caller must inspect the GitHub web UI or retry from an
explicitly authorized independent administrator account; it is **not** a
license to broaden the GitHub App installation or bypass source governance.

## What to inspect in GitHub administration

1. Original [LLVM CI issue #159](https://github.com/ores-truffle-oreslang/oreslang-source.llvm/issues/159)
   and [.NET security issue #221](https://github.com/ores-truffle-oreslang/oreslang-source.net/issues/221)
   already contain exact zero-step job examples and their run attempts.
2. Inspect the **check-run annotation**, job runner ID, labels and dates.
   A job that fails before any step is not a compiler test failure. GitHub
   may refuse to generate job logs at all.
3. In the GitHub organization Actions settings and billing/usage UI,
   verify the exact admission restriction, hosted runner spending limit,
   workflow policy, private-repo entitlement, and selected-repository
   runner groups. Do not guess which one is responsible.
4. Private source repo rulesets currently return an upgrade-required
   HTTP 403; classic protection reads return a separate GitHub App
   permission 403. Organization administrators must verify actual plan
   entitlement and effective private branch protection. Do not make
   source public or change billing automatically.
5. Use a source-free canary on an authorized runner. Record a nonzero-step
   job, exact SHA/tree, run attempt, runner ID and executed step names.
   Only then rerun exact-head compiler/V3/NativeAOT tests.
6. Private-source mirroring must remain disabled until independently
   approved origin/destination snapshot, private branch protections and
   safe temporary-ref cleanup. See [LLVM cleanup #1154](https://github.com/ores-truffle-oreslang/oreslang-source.llvm/issues/1154)
   and [.NET proposed export #277](https://github.com/ores-truffle-oreslang/oreslang-source.net/issues/277).

## Test receipt

The exact Rust source in this draft was compiled and executed on
[Beamscale run 41252832319](https://github.com/beamscale/bmscl-compiler/actions/runs/41252832319),
job `124689745230`: **15/15 tests passed**, `rustc -D warnings`,
and the source-free execution marker was recorded in a real job log.
The validated Git blob is
`09f06dd09e0a4acd1afa5cca49005f4fcbd3cf96`,
identical to the proposed Oreslang organization Rust file.

The tests cover bounded argument identities, strict run/job membership,
check-run URL repository binding, expected head SHA and GitHub Actions
application identity, spoofed URL and malformed input rejection, and
pinning authenticated API reads to `github.com` rather than trusting
ambient `GH_HOST`. No private Oreslang compiler source or secrets
were copied. This does **not** diagnose or repair source-organization
runner admission; source-org CI must execute separately.
