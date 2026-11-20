# Python to Rust: 50-organization migration rollout

Status snapshot: **2026-10-21**. This tracks **50 distinct GitHub organizations** with review PRs proposing **51 Python-script replacements** (the Orestra PR replaces two scripts). These are **unmerged changes**; CI passing is not proof of merge. The canonical [no-Python policy](RUST_ONLY_POLICY.md) is already on `main`.

**CI audit:** 47 organizations' reviewed migration heads had all checked workflows pass; three were blocked by jobs failing without any steps executing, even after retry. Source runners/billing or policy must be resolved before those three can be considered verified. The 47 include privacy fixes and lint workflow remediation made during review. Do not merge blocked work merely to increase this count.

| Organization | Repository | Migration PR | Work | CI snapshot |
| --- | --- | --- | --- | --- |
| hypeblitz | `.github` | [#18](https://github.com/hypeblitz/.github/pull/18) | Public privacy validator | Passed |
| file-tunnel | `.github` | [#21](https://github.com/file-tunnel/.github/pull/21) | Public privacy validator | Passed |
| unreal-unity-poc | `.github` | [#18](https://github.com/unreal-unity-poc/.github/pull/18) | Public privacy validator | Passed |
| fifa-math | `.github` | [#20](https://github.com/fifa-math/.github/pull/20) | Public privacy validator | Passed |
| agent-pontifex | `.github` | [#40](https://github.com/agent-pontifex/.github/pull/40) | Public privacy validator | Passed |
| hypesiege | `.github` | [#27](https://github.com/hypesiege/.github/pull/27) | Public privacy validator | Passed |
| discrete-event-systems | `.github` | [#28](https://github.com/discrete-event-systems/.github/pull/28) | Public privacy validator | Passed |
| drone-mngr | `.github` | [#24](https://github.com/drone-mngr/.github/pull/24) | Public privacy validator | Passed |
| fanwaave | `.github` | [#27](https://github.com/fanwaave/.github/pull/27) | Public privacy validator | Passed |
| apostille-me | `.github` | [#34](https://github.com/apostille-me/.github/pull/34) | Public privacy validator | Passed |
| embedded-alerts | `.github` | [#41](https://github.com/embedded-alerts/.github/pull/41) | Public privacy validator | Passed |
| evento-globolo | `.github` | [#36](https://github.com/evento-globolo/.github/pull/36) | Public privacy validator | Passed |
| hacker-house-medellin | `.github` | [#40](https://github.com/hacker-house-medellin/.github/pull/40) | Public privacy validator | Passed |
| led-dynamo | `.github` | [#33](https://github.com/led-dynamo/.github/pull/33) | Public privacy validator | Passed |
| 3FA-app | `.github` | [#43](https://github.com/3FA-app/.github/pull/43) | Public privacy validator | Passed |
| messaging-intel | `.github` | [#47](https://github.com/messaging-intel/.github/pull/47) | Public privacy validator | Passed |
| akrion-sim | `.github` | [#20](https://github.com/akrion-sim/.github/pull/20) | Public privacy validator | Passed |
| athlet-o | `.github` | [#26](https://github.com/athlet-o/.github/pull/26) | Public privacy validator | Passed |
| benefactor-cc | `.github` | [#39](https://github.com/benefactor-cc/.github/pull/39) | Public privacy validator | Passed |
| canonical-cloud | `.github` | [#83](https://github.com/canonical-cloud/.github/pull/83) | Public privacy validator | Passed |
| claritas-viz | `.github` | [#32](https://github.com/claritas-viz/.github/pull/32) | Public privacy validator | Passed |
| cliptown | `.github` | [#21](https://github.com/cliptown/.github/pull/21) | Public privacy validator | Passed |
| daedalus-fab | `.github` | [#29](https://github.com/daedalus-fab/.github/pull/29) | Public privacy validator | Passed |
| declarative-migrations | `.github` | [#25](https://github.com/declarative-migrations/.github/pull/25) | Public privacy validator | Passed |
| fiducia-cloud | `.github` | [#64](https://github.com/fiducia-cloud/.github/pull/64) | Public privacy validator | Passed |
| anticaptrad | `.github` | [#19](https://github.com/anticaptrad/.github/pull/19) | Public privacy validator | Passed |
| opto-sync | `.github` | [#28](https://github.com/opto-sync/.github/pull/28) | Public privacy validator | Passed |
| quaestor-ledger | `.github` | [#36](https://github.com/quaestor-ledger/.github/pull/36) | Public privacy validator | Passed |
| sagitta-stack | `.github` | [#18](https://github.com/sagitta-stack/.github/pull/18) | Public privacy validator | Passed |
| shared-auth | `.github` | [#74](https://github.com/shared-auth/.github/pull/74) | Public privacy validator | Passed |
| scintilla-run | `.github` | [#52](https://github.com/scintilla-run/.github/pull/52) | Public privacy validator | Passed |
| rust-ssr-demos | `.github` | [#16](https://github.com/rust-ssr-demos/.github/pull/16) | Public privacy validator | Passed |
| sonus-auris | `.github` | [#56](https://github.com/sonus-auris/.github/pull/56) | Public privacy validator | Passed |
| usa-acc | `.github` | [#17](https://github.com/usa-acc/.github/pull/17) | Public privacy validator | Passed |
| voxletra | `.github` | [#22](https://github.com/voxletra/.github/pull/22) | Public privacy validator | Passed |
| zed-pkg | `.github` | [#117](https://github.com/zed-pkg/.github/pull/117) | Public privacy validator | Passed |
| ORESoftware | `.github` | [#247](https://github.com/ORESoftware/.github/pull/247) | Public privacy validator | Runner did not execute |
| channelsiege | `.github` | [#22](https://github.com/channelsiege/.github/pull/22) | Public privacy validator | Passed |
| memebank | `.github` | [#43](https://github.com/memebank/.github/pull/43) | Public privacy validator | Passed |
| meta-agents-demo | `.github` | [#28](https://github.com/meta-agents-demo/.github/pull/28) | Public privacy validator | Passed |
| networking-components | `.github` | [#18](https://github.com/networking-components/.github/pull/18) | Public privacy validator | Passed |
| OmniBlitz | `.github` | [#18](https://github.com/OmniBlitz/.github/pull/18) | Public privacy validator | Passed |
| StreemPilot | `.github` | [#30](https://github.com/StreemPilot/.github/pull/30) | Public privacy validator | Passed |
| streamkore | `.github` | [#20](https://github.com/streamkore/.github/pull/20) | Public privacy validator | Passed |
| gha-indie-worker | `.github` | [#46](https://github.com/gha-indie-worker/.github/pull/46) | Public privacy validator | Passed |
| ores-chat | `.github` | [#17](https://github.com/ores-chat/.github/pull/17) | Organization policy checker | Passed |
| ores-stack | `.github` | [#4](https://github.com/ores-stack/.github/pull/4) | Organization identity checker | Runner did not execute |
| declarative-migrations-test | `.github` | [#20](https://github.com/declarative-migrations-test/.github/pull/20) | Immutable source pin verifier | Passed |
| orestra | `orestra-docs` | [#7](https://github.com/orestra/orestra-docs/pull/7) | Markdown links and integration-lock checks (2 scripts) | Runner did not execute |
| ores-otel | `.github` | [#23](https://github.com/ores-otel/.github/pull/23) | SHA-256 template manifest renderer | Passed |

## Validation and boundaries

- These PRs preserve required behavior with Rust implementations and migrate their known GitHub Actions callers; no first-party Python script should be added in their place.
- The three unverified projects are `ORESoftware/.github`, `ores-stack/.github` and `orestra/orestra-docs`. Github Actions shows failures with no job steps on these PRs, including on retry. The code cannot be declared tested by their own workflow until a run actually executes.
- `hacker-house-medellin/.github` was hardened to add read-only workflow permissions, a timeout, and pinned actions, fixing a pre-existing baseline failure.
- `voxletra/.github` had a direct private-registry URL removed from public-facing documentation after the Rust privacy validator correctly rejected it.
- The separate [canonical enforcement PR #12](https://github.com/ores-truffle-oreslang/.github/pull/12) introduces a Rust no-Python CI gate and ports the Python organization-sync test suite. It is **not counted in the 50 new organization PRs** and still needs a passing workflow before merge.
- Count migrated scripts after merge and verify `main` per organization separately. The numbers here refer to opened PRs, not merged or released code.
