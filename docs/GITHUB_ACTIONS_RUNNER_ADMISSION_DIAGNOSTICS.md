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
[Beamscale run 41253433658](https://github.com/beamscale/bmscl-compiler/actions/runs/41253433658),
job `124691512059` in Beamscale run `41253433658`: **18/18 tests passed**, `rustc -D warnings`, the pinned `/usr/bin/gh` preflight passed, and `SOURCE_FREE_GH_ZERO_STEP_HARDENING_TEST=PASS` was recorded in a real job log. The exact validated Rust blob is `2891937bf07b37ef9c993432fe3bae172de50a50`, identical to the current proposed organization tool.

The tests cover bounded argument identities, strict run/job membership, completed-failure/zero-step gating, check-run URL repository binding, expected head SHA and GitHub Actions application identity, spoofed URL and malformed input rejection, an absolute `/usr/bin/gh` executable path, and pinning authenticated API reads to `github.com` rather than trusting ambient `GH_HOST`. No private Oreslang compiler source or secrets
were copied. This does **not** diagnose or repair source-organization
runner admission; source-org CI must execute separately.
