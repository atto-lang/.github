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
or write endpoints. It cannot inspect private Actions billing settings.

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
/tmp/ores-gh-diagnostic ores-truffle-oreslang/oreslang-source.net 41250696587 124680332733 > net-admission-local.txt
```

For **the .NET example**, the run/job IDs shown above are from different
attempts, so replace them with matching run and job values before executing.
The tool prints both IDs and the job's `run_id` to support this cross-check.
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

This tool was originally created as source-free code and built on
[Beamscale run 41251167458](https://github.com/beamscale/bmscl-compiler/actions/runs/41251167458):
job `124684797331` executed real runner steps; `rustc -D warnings`
compiled the test binary and standalone CLI and **8/8 Rust tests passed**.
It did **not** access Oreslang private compiler source or use organization
administrator credentials, so it did not retrieve the actual check-run
annotations. Its outcome is tool validation only.
