# Organization security baseline — 2026-10-31

## Method and boundaries

Authenticated GitHub repository search returned **78 repositories** under `ores-truffle-oreslang`. For each, an authenticated recursive `main` Git tree was read for **path-level inventory** (workflows, test-like paths, CODEOWNERS, security policy, dependency automation and tracked link modes). Each `main` branch was separately read from the GitHub branch API.

These checks establish **repository metadata and file presence only**: they do not scan every source file for vulnerabilities, run a compiler, prove native execution, attest runtime provenance, or certify a repository as secure. The inventory can become stale as branches move.

## Confirmed issues

1. **P0: 78 / 78 main branches returned `protected=false`.** No repository in the returned inventory had active branch-level protection according to the authenticated branch API. Do not claim checks/review policies are enforced merely because a workflow or CODEOWNERS file exists. Private ruleset reads also returned **403 with GitHub plan upgrade-required**, so no effective private rulesets could be verified. Administrators must choose an approved plan/policy-enforcement path and verify the effective settings.
2. **7 / 78 had no tracked workflow YAML at `.github/workflows/*.yml|yaml`:** `oreslang-logic.ores`, `oreslang-db`, `oreslang-physics-demo.ores`, `oreslang-api-server.ores`, `ores-fm-mac-agent.ores`, `oreslang-n8n-make-zapier-app`, `oreslana.ores`. Several are near-empty scaffolds: absence of workflow does not by itself establish a vulnerable production runtime. Audit before claiming runnable tests.
3. **0 / 78 had a repository-local `SECURITY.md` at root or `.github/SECURITY.md`.** A private organization `.github` repository does not establish a public inherited security policy for every repository. Confirm whether a separate inherited or private disclosure process is active; add per-repository policies where needed.
4. **Only one tree had a tracked `.github/dependabot.yml|yaml`**, `gh-linguist-fork`. This is a configuration inventory, not proof that dependency updates, scanning, or alternative dependency management are disabled.
5. Many repositories lack repository-local CODEOWNERS and some contain test fixtures/benchmarks but no files matching conventional test-path heuristics; do not treat file-name counts as coverage or test execution. Tracked symlinks and submodules exist in a few repos and require project-specific policy rather than blanket assumptions.

## Remediation / acceptance

- **Administrator-controlled P0:** enable enforced protections on all 78 default branches, with independent review (including CODEOWNERS for sensitive paths), stale-review dismissal, no direct/force pushes or bypasses, non-bypassable checks bound to a trusted issuer, and validated negative controls. If private branch/ruleset protection is plan-limited, record the exact restriction and approved alternative; do not claim it has been applied. Existing issue #2 tracks the five core compiler branches and runner admission.
- **CI evidence P0:** a zero-step run, status-only check, local/source-free verifier, or a green synthetic test is **not** compiler-native evidence. Record run attempt, job ID, executed steps, checkout commit/tree, compiler binary identity and guest results separately. Never move private source to another organization without explicit authorization plus independent security review.
- **Shared hardening candidate:** the companion change to `.github/workflows/reusable-source-tree-preflight.yml` hashes **each physical tracked file** and compares it with the stage-0 Git object, compares physical file modes, rejects tracked symlinks and gitlinks, rejects altered index entries, and detects untracked **and ignored** files before emitting commit/tree identity. This still proves only a checkout **at that moment**, never execution/attestation or safety of downstream code.
- **Per-repo rollout:** create focused changes for missing CI, required reviewers, dependency review, and vulnerability-reporting instructions; preserve existing CI semantics and avoid breaking projects that deliberately use gitlinks/LFS until separately reviewed. Re-check branch SHA and exact tree before any merge.

## Org repository inventory (78; main unprotected on the inspection date)

| Repository | Main protected | Tracked workflow path status |
|---|---|---|
| `oreslang-restful-server-perf-benchmarks` | No | One or more observed |
| `oreslang-static-analysis.rs` | No | One or more observed |
| `oreslang-ssr` | No | One or more observed |
| `oreslang-format` | No | One or more observed |
| `oreslang-platform.mobile` | No | One or more observed |
| `oreslang-web-browser.ores` | No | One or more observed |
| `oreslang-reverse-proxy-demo.ores` | No | One or more observed |
| `oreslang-spin` | No | One or more observed |
| `oreslang-ai-agent-coordination-via-actors-demo` | No | One or more observed |
| `gh-linguist-fork` | No | One or more observed |
| `rx-oreslang-channels` | No | One or more observed |
| `oreslang-platform.asic` | No | One or more observed |
| `ores-truffle-oreslang.github.io` | No | One or more observed |
| `oreslang-serialization-and-validation` | No | One or more observed |
| `oreslang-rpc` | No | One or more observed |
| `oreslang-durable-objects.ores` | No | One or more observed |
| `vscode-plugins` | No | One or more observed |
| `rx-ores` | No | One or more observed |
| `oreslang-tensor` | No | One or more observed |
| `oreslang-orie-ieor` | No | One or more observed |
| `oreslang-ml-conformance` | No | One or more observed |
| `oreslang-source.js` | No | One or more observed |
| `oreslang-common-source` | No | One or more observed |
| `rx-ores-tasks` | No | One or more observed |
| `oreslang-cli` | No | One or more observed |
| `oreslang-platform.embedded` | No | One or more observed |
| `oreslang-runtime.realtime-java` | No | One or more observed |
| `oreslang-demo-restful-server` | No | One or more observed |
| `oreslang-hpc-finance` | No | One or more observed |
| `oreslang-data-science` | No | One or more observed |
| `oreslang-message-queue-demo.ores` | No | One or more observed |
| `oreslang-source.rs` | No | One or more observed |
| `oreslang-phoenix-server.ores` | No | One or more observed |
| `oreslang-websocket-server-demo.ores` | No | One or more observed |
| `oreslang-sg-stategraph.ores` | No | One or more observed |
| `oreslang-proof-of-concepts` | No | One or more observed |
| `oreslang-test.cli` | No | One or more observed |
| `oreslang-logic.ores` | No | None observed |
| `oreslang-db` | No | None observed |
| `oreslang-runtime-ui` | No | One or more observed |
| `rx-ores-callbacks` | No | One or more observed |
| `oreslang-linked-queue` | No | One or more observed |
| `oreslang-target-profiles` | No | One or more observed |
| `oreslang-physics-demo.ores` | No | None observed |
| `oreslang-source.net` | No | One or more observed |
| `oreslang-source.forge` | No | One or more observed |
| `oreslang-source.hdl` | No | One or more observed |
| `oreslang-http-routing` | No | One or more observed |
| `oreslang-source.java` | No | One or more observed |
| `postgres-orm.ores` | No | One or more observed |
| `oreslang-api-server.ores` | No | None observed |
| `oreslang-compilers-parity-testing` | No | One or more observed |
| `oreslang-demo` | No | One or more observed |
| `ores-fm-mac-agent.ores` | No | None observed |
| `oreslang-platform.iot` | No | One or more observed |
| `oreslang-n8n-make-zapier-app` | No | None observed |
| `oreslang-design-patterns` | No | One or more observed |
| `oreslang-platform.desktop` | No | One or more observed |
| `oreslang-ml` | No | One or more observed |
| `oreslang-source.llvm` | No | One or more observed |
| `sublime-text-plugins` | No | One or more observed |
| `oreslana.ores` | No | None observed |
| `oreslang-source.cranelift` | No | One or more observed |
| `oreslang-platform-conformance` | No | One or more observed |
| `.github` | No | One or more observed |
| `async-testing` | No | One or more observed |
| `oreslang-otel` | No | One or more observed |
| `oreslang-platform.fpga` | No | One or more observed |
| `oreslang-browser` | No | One or more observed |
| `oreslang-hot-loading-tests` | No | One or more observed |
| `oreslang-autograd` | No | One or more observed |
| `oreslang-step-function-demo` | No | One or more observed |
| `html-js-css-syntax-highlighter` | No | One or more observed |
| `oreslang-wasm` | No | One or more observed |
| `oreslang-stack-cli` | No | One or more observed |
| `oreslang-source.wasm` | No | One or more observed |
| `oreslang-forward-proxy-demo.ores` | No | One or more observed |
| `oreslang-demos` | No | One or more observed |

## Verification status

- **Confirmed:** 78 repository entries, 78 recursive Git trees returned without truncation, 78 branch API readings with `protected=false`.
- **Local-only integrity prototype:** a temporary Git repository accepted its intact checkout and rejected six independent mutations: tracked content edit, chmod drift, untracked file, ignored file, symlink replacement, staged/index drift (6 / 6 rejection). This is **not** a GitHub-hosted run against the PR head.
- **Not yet confirmed:** this PR's exact-head Actions steps, any native Oreslang backend, administrator protection activation, reviewer independence, or security of every repository's source.

Owner approval and real runner admission are required before promoting any of these gates to enforced production release evidence.
