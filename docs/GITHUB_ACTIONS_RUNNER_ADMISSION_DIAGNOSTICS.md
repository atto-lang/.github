# Diagnose zero-step Oreslang GitHub Actions jobs

Status: proposed source-free diagnostic tooling. This is **not a runner fix**,
CI success, permission grant, branch-protection change, or source-export
authorization.

The repository policy [requires Rust rather than Python](RUST_ONLY_POLICY.md).
`tools/gh_run_admission.rs` is a std-only Rust, read-only GitHub CLI wrapper
with an explicit allowlist for the two private compiler source repositories **and the `.github` governance repository**.
It reads only run metadata, the selected job, GitHub check annotations,
the default branch's protected flag, ruleset list, and Actions policy.
It never accesses repository contents, Git objects, artifacts, secrets,
or write endpoints. Every GitHub API request pins the host to `github.com`, ignores any ambient `GH_HOST` setting, and invokes the reviewed `/usr/bin/gh` binary by default rather than resolving `gh` through `PATH`. On macOS, an administrator may explicitly set `ATTO_GH_BIN=/opt/homebrew/bin/gh` (or the trusted absolute executable path ending in `/gh`); parent traversals, shell strings and relative paths are rejected. It cannot inspect private Actions billing settings.

## Current administrator handoff — private governance runner (2026-11-07)

The most recent pinned governance head before this documentation revision was `91a1b30c7d510d7bbf1881316c4b19695f4049aa` ([governance PR #36](https://github.com/ores-truffle-oreslang/.github/pull/36)). Its [source-free canary run 42412905146](https://github.com/ores-truffle-oreslang/.github/actions/runs/42412905146), job `128524666921`, and [Rust conformance run 42412905144](https://github.com/ores-truffle-oreslang/.github/actions/runs/42412905144), job `128524667011`, both completed with `steps=null`. These are **runner-admission failures**, not Rust test failures.

With a trusted administrator-authenticated GitHub CLI, use the latest source-free example:

```sh
gh auth status
umask 077
gh api --hostname github.com --method GET \
  repos/ores-truffle-oreslang/.github/actions/runs/42412905146 \
  --jq '{id,run_attempt,head_sha,status,conclusion}'
gh api --hostname github.com --method GET \
  repos/ores-truffle-oreslang/.github/actions/jobs/128524666921 \
  --jq '{id,run_id,head_sha,status,conclusion,runner_id,steps,check_run_url}'
```

Confirm the run/job association, the failed status and empty/null steps. **Do not treat `128524666921` as a check-run ID.** Extract the actual check-run ID from `check_run_url`, and use the dedicated read-only check-run and annotations commands below. Copy only the **redacted failure reason**, not raw private account metadata or tokens, into [governance issue #37](https://github.com/ores-truffle-oreslang/.github/issues/37).

GitHub's [current branch-protection documentation](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches) confirms that **private** repository branch protections require an eligible paid plan. The connected integration's `.github/branches/main/protection` read returned **403: Resource not accessible by integration**; its private rulesets read returned **403: Upgrade to GitHub Pro or make this repository public to enable this feature**. `.github/main` currently reports `protected=false`. These are **distinct plan/permission blockers**; do not infer that runner admission and protection necessarily have the same cause. Verify the organization plan and exact Actions restriction with authorized administrators. **Do not change visibility, billing or security protections without explicit authorization.**

## Independent administrator action

From a trusted administrator workstation with the correct GitHub organization
permissions and authenticated GitHub CLI (`gh auth status`), select the executable
explicitly when it is not at `/usr/bin/gh`. On Apple Silicon macOS with
Homebrew this is commonly `/opt/homebrew/bin/gh`, but verify the path first.
The diagnostic does not search PATH; it rejects nonabsolute paths and dot
traversal. Example for a trusted macOS installation:

```sh
export ATTO_GH_BIN=/opt/homebrew/bin/gh
"$ATTO_GH_BIN" auth status
```

Then build the
tool from a reviewed Oreslang organization checkout and run:

```sh
rustc --edition=2021 -D warnings --test tools/gh_run_admission.rs -o /tmp/ores-gh-diagnostic-tests
/tmp/ores-gh-diagnostic-tests
rustc --edition=2021 -D warnings tools/gh_run_admission.rs -o /tmp/ores-gh-diagnostic

# Secure local capture: annotations may contain private identifiers.
umask 077
/tmp/ores-gh-diagnostic ores-truffle-oreslang/oreslang-source.llvm 41236569150 124641494087 > llvm-admission-local.txt
/tmp/ores-gh-diagnostic ores-truffle-oreslang/oreslang-source.net 41250696587 124683413992 > net-admission-local.txt

# PR #36: exact-head Rust conformance job and source-free runner canary.
# These reads are source-free diagnostics, NOT running Rust tests.
/tmp/ores-gh-diagnostic ores-truffle-oreslang/.github 42398019612 128479490053 > governance-rust-admission-local.txt
/tmp/ores-gh-diagnostic ores-truffle-oreslang/.github 42398019416 128479489595 > governance-canary-admission-local.txt
```

The sample run/job IDs refer to real matching jobs. Replace both IDs with
the current **matching** run and job values when diagnosing a later head.
The tool independently reads the selected job's `run_id` **and its actual `check_run_url`**, rejects mismatched run/job IDs, and refuses to proceed unless the selected job is a **completed failure with zero steps**. It derives the check-run ID only from the exact audited-repository URL, then verifies the check-run `head_sha`, `github-actions` app identity, completed status, and failed conclusion against the workflow run **before querying annotations**. It never assumes a workflow job ID and check-run ID are interchangeable identity domains.
Never equate a successful diagnostic with a successful compiler run.

Review the local output privately before quoting or sanitizing the exact
annotation message in the existing security issues. Do not post tokens,
authenticated command output, private source paths, credentials, or customer
metadata. The diagnostic emits read errors as `UNAVAILABLE` without
printing stderr from `gh` or the selected token. A check-run annotations
403/404 means the caller must inspect the GitHub web UI or retry from an
explicitly authorized independent administrator account; it is **not** a
license to broaden the GitHub App installation or bypass source governance.

## Differential evidence and next administrator receipt (2026-11-07)

**Latest source:** the tool now contains **25 Rust tests**, including three path-validation tests for absolute trusted Linux/macOS GitHub CLI binaries, and the workflow checks for all 25. The 25 tests are not yet verified as executed on the current private-repository head.



The following same-organization, GitHub-hosted workflow results help rule out a **total** runner outage:

- Public `oreslang-cli` [CI run 42338255919](https://github.com/ores-truffle-oreslang/oreslang-cli/actions/runs/42338255919) passed with **11 recorded job steps**.
- Public `ores-truffle-oreslang.github.io` [site-validation run 42387087806](https://github.com/ores-truffle-oreslang/ores-truffle-oreslang.github.io/actions/runs/42387087806) passed with **31 recorded job steps**.
- Private `.github` [review-policy Rust run 42398019612](https://github.com/ores-truffle-oreslang/.github/actions/runs/42398019612) had `runner_id=0`, `steps=[]` (job `128479490053`) and failed.
- Private `.github` [source-free canary 42398019416](https://github.com/ores-truffle-oreslang/.github/actions/runs/42398019416) also had `runner_id=0`, `steps=[]` (job `128479489595`) and failed.

These facts narrow the cause to **private-repository billing/spending/entitlements, repository Actions policy, or private-job scheduling**, but do not establish which. Only the real check-run annotation or authorized billing/runners settings can distinguish them. Inspect GitHub organization **Settings → Billing and licensing** and **Settings → Actions**, then privately obtain the annotation using this read-only diagnostic. Never assume the runner failure was caused by the Rust source.

**Evidence classification:** PR #22's historical 18/18 executed source-free tests cover only its earlier exact diagnostic blob (`2891937bf...`). The expanded `.github` allowlist and four new tests must execute at the **new** head before any new passed-test claim. No private compiler source may be mirrored to public repositories to evade the missing runner.
## Direct administrator-only GitHub CLI readback (no Rust installation needed)

When the administrator has `gh` but no Rust toolchain, collect the failure reason using *read-only* requests from an authenticated workstation. These commands make **no branch-protection or billing changes**:

```sh
gh auth status
gh api --hostname github.com --method GET \
  repos/ores-truffle-oreslang/.github/actions/runs/42402642383 \
  --jq '{id,run_attempt,head_sha,status,conclusion}'
gh api --hostname github.com --method GET \
  repos/ores-truffle-oreslang/.github/actions/jobs/128493550909 \
  --jq '{id,run_id,head_sha,status,conclusion,runner_id,steps,check_run_url}'
```

Confirm the job belongs to run `42402642383`, both heads match, the job is completed/failed, and `steps` is an empty array. **Do not guess a check-run ID from the job ID**. Copy the exact numeric ID from the job's `check_run_url` under the *same repository* into the next commands (replace `CHECK_RUN_ID` with that number):

```sh
gh api --hostname github.com --method GET \
  repos/ores-truffle-oreslang/.github/check-runs/CHECK_RUN_ID \
  --jq '{id,head_sha,status,conclusion,app_slug:.app.slug,output}'
gh api --hostname github.com --method GET \
  'repos/ores-truffle-oreslang/.github/check-runs/CHECK_RUN_ID/annotations?per_page=100' \
  --jq '[.[] | {annotation_level,title,message}]'
```

Before relying on annotations, confirm the check-run head equals the workflow head, `app_slug` is `github-actions`, status is `completed`, and conclusion is `failure`. Output may contain sensitive administrative details; **review it locally, redact secrets, and share only the factual blocker**. If these GET requests return 403/404, use the original failed [Actions job](https://github.com/ores-truffle-oreslang/.github/actions/runs/42402642383/job/128493550909) in the browser as an authorized organization administrator. A GitHub App exposing `permissions.admin=true` in repository metadata **does not** prove it has the required branch-protection or check-run annotation permissions.

Public same-organization workflows have executed steps while the private `.github` source-free canary has not. That supports a private-repository admission investigation, **not** a confirmed billing diagnosis.
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

A historical, earlier version of the Rust source (not the present 25-test draft) was compiled and executed on
[Beamscale run 41253433658](https://github.com/beamscale/bmscl-compiler/actions/runs/41253433658),
job `124691512059` in Beamscale run `41253433658`: **18/18 tests passed**, `rustc -D warnings`, the pinned `/usr/bin/gh` preflight passed, and `SOURCE_FREE_GH_ZERO_STEP_HARDENING_TEST=PASS` was recorded in a real job log. The historically validated Rust blob was `2891937bf07b37ef9c993432fe3bae172de50a50`. **It is not the current proposed source**: the present diagnostic tool blob is `681e41c4b014c915ac4dc1b594b748a3e4a4005a`, including the expanded governance allowlist and 25 source tests. That newer blob has **no verified executed-test receipt** on its private PR head.

The tests cover bounded argument identities, strict run/job membership, completed-failure/zero-step gating, check-run URL repository binding, expected head SHA and GitHub Actions application identity, spoofed URL and malformed input rejection, an absolute `/usr/bin/gh` executable path, and pinning authenticated API reads to `github.com` rather than trusting ambient `GH_HOST`. No private Oreslang compiler source or secrets
were copied. This does **not** diagnose or repair source-organization
runner admission; source-org CI must execute separately.
