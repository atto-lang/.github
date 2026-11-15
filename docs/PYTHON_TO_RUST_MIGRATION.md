# Python-to-Rust migration inventory

Python is prohibited across `ores-truffle-oreslang`. Required functionality currently implemented in Python must be ported to Rust with equivalent behavior and test coverage; deleting required behavior is not a migration.

## Audit baseline — 2026-10-21

The default branches of all 60 organization repositories were enumerated through recursive Git trees. Every tree response used for this inventory was complete (not truncated). Filename/manifest classification found **452 Python artifacts across 40 repositories**.

This baseline counts `.py`, `.pyi`, `.ipynb`, and recognized Python package/tool manifests. It is a migration inventory, not proof that repositories with zero counted files have no Python command embedded in another build/config file; the Rust policy gate separately scans execution/configuration paths.

| Repository | Python artifacts |
| --- | ---: |
| oreslang-source.llvm | 99 |
| oreslang-common-source | 84 |
| oreslang-source.java | 43 |
| oreslang-source.forge | 26 |
| oreslang-source.wasm | 25 |
| oreslang-proof-of-concepts | 16 |
| oreslang-source.net | 16 |
| oreslang-platform.embedded | 15 |
| gh-linguist-fork | 13 |
| oreslang-spin | 12 |
| oreslang-test.cli | 11 |
| rx-ores | 9 |
| oreslang-restful-server-perf-benchmarks | 8 |
| oreslang-platform-conformance | 8 |
| oreslang-demo-restful-server | 6 |
| rx-oreslang-channels | 5 |
| rx-ores-callbacks | 5 |
| oreslang-source.hdl | 5 |
| oreslang-design-patterns | 5 |
| oreslang-format | 4 |
| oreslang-runtime.realtime-java | 4 |
| oreslang-platform.asic | 3 |
| oreslang-data-science | 3 |
| oreslang-http-routing | 3 |
| oreslang-platform.fpga | 3 |
| oreslang-rpc | 3 |
| ores-truffle-oreslang.github.io | 2 |
| oreslang-source.js | 2 |
| oreslang-linked-queue | 2 |
| sublime-text-plugins | 2 |
| oreslang-ssr | 1 |
| oreslang-tensor | 1 |
| oreslang-ml-conformance | 1 |
| .github | 1 |
| async-testing | 1 |
| oreslang-otel | 1 |
| oreslang-autograd | 1 |
| oreslang-wasm | 1 |
| oreslang-stack-cli | 1 |
| oreslang-serialization-and-validation | 1 |

The other 20 repositories had zero filename/manifest artifacts at this baseline.

## Migration rules

1. Port behavior to Rust first. Preserve positive, negative, fail-closed, race, integrity, and boundary cases represented by the Python implementation.
2. Compile Rust ports with warnings denied and execute their tests before removing the Python source.
3. Remove Python callers from workflows, build scripts, documentation command examples, and package/runtime dependencies.
4. Re-enumerate the resulting Git tree and require zero Python artifacts.
5. Run the repository's normal tests on an **authorized source-local runner**. If private Actions jobs fail before runner assignment, record the run/job IDs and classify the tests as **NOT_EXECUTED**. Do **not** use a cross-organization private-source mirror as a fallback: [conformance incident #22](https://github.com/ores-truffle-oreslang/oreslang-platform-conformance/issues/22) and source provenance/security gates currently prohibit private-source export. Independently review and resolve those restrictions before considering any future alternate-runner design.
6. Do not claim an isolated Rust-port smoke test as exact full-tree validation.

## Work in progress

- `oreslang-stack-cli`: migration PR #21 replaces its sole Python identity checker with Rust; the migration branch has zero Python files.
- `oreslang-tensor`: migration PR #9 replaces its sole Python descriptor oracle with Rust; the migration branch has zero Python files. Funded-run testing caught a warnings-as-errors defect in the first Rust port, which was corrected before review.
- `oreslang-source.rs`: baseline already has zero Python artifacts; the no-Python policy gate is being hardened there.
- `.github`, `oreslang-source.llvm`, `oreslang-source.java`, and `oreslang-source.wasm`: policy PRs exist, but legacy Python remains a blocker until functional ports are complete.

This inventory must be updated when migrations merge; counts on unmerged branches do not reduce the default-branch baseline.
