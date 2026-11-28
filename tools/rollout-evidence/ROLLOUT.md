# Oreslang 50-repository evidence-tooling rollout

Requested scope: at least 50 repositories in ores-truffle-oreslang.
Delivered scope: 50 additive draft PRs, each with an isolated dependency-free
Rust receipt inspector, a repository-specific acceptance matrix, 10 unit tests,
4 CLI-process tests, a lockfile and a read-only self-test workflow.
This is verification tooling and acceptance-target work, not implementation of
50 runtime features, migration of retained scripts, or a claim of backend parity.

## Validation and publication

All 50 repository-specific packages passed locally with rustc 1.99.1:
cargo fmt --check; cargo clippy --offline --locked --all-targets -- -D warnings;
cargo test --offline --locked. Total: 700 checker tests, no ignored tests.
Synthetic receipts test only the checker. Actual repository acceptance suites
and real receipt-producing adapters are not implemented by these PRs.

All 49 child PRs below were read back through GitHub compare APIs. All eight
published file blob IDs per PR exactly match the locally tested package and
workflow; each diff contains eight additions and no modifications/deletions.
The organization PR contains the same package plus this inventory and its
machine-readable evidence file; its own commit identity is in PR metadata.

The initial hosted snapshot for the 49 child PRs found:
- oreslang-cli: checker self-test run 40441710791 succeeded with executed
  format, lint, test and CLI-listing steps (job 121983354052).
- Other 48: checker jobs concluded failure with steps=null; classify as
  NOT_EXECUTED / runner-admission failure, not Rust test failure.
- This organization PR: consult current Actions; it is created after this snapshot.

The successful checker workflow does not execute the Oreslang compiler.
No private tree was mirrored, no source-transfer policy was widened, and no
release gate or automatic merge was activated. Existing compiler/runtime
candidates remain independent and require their own exact-head execution.

## Inventory

| Repository | Draft PR | Exact candidate head | Checker CI snapshot |
| --- | --- | --- | --- |
| async-testing | [#26](https://github.com/ores-truffle-oreslang/async-testing/pull/26) | `588e211df561cc4174bb4c4d4f0733f3a33fdaa1` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/async-testing/actions/runs/40441861549) |
| oreslang-autograd | [#8](https://github.com/ores-truffle-oreslang/oreslang-autograd/pull/8) | `cc015d1a6d545a6efb2ad0aa021621dabc8b3149` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-autograd/actions/runs/40441888681) |
| oreslang-browser | [#27](https://github.com/ores-truffle-oreslang/oreslang-browser/pull/27) | `7f26d5c71fb627d7670bbc89fa233fc298d86da3` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-browser/actions/runs/40441866819) |
| oreslang-cli | [#17](https://github.com/ores-truffle-oreslang/oreslang-cli/pull/17) | `5a0bf92d89ebf1bd6e214700938e1aa5f71794ec` | [executed PASS](https://github.com/ores-truffle-oreslang/oreslang-cli/actions/runs/40441710791) |
| oreslang-common-source | [#165](https://github.com/ores-truffle-oreslang/oreslang-common-source/pull/165) | `d03505f9c8ca8719c1445cd3f4db92fdb40f6aa8` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-common-source/actions/runs/40441689448) |
| oreslang-compilers-parity-testing | [#6](https://github.com/ores-truffle-oreslang/oreslang-compilers-parity-testing/pull/6) | `313c13edf1ca06e4b7d286372fb28e897e3a53d4` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-compilers-parity-testing/actions/runs/40441808373) |
| oreslang-data-science | [#16](https://github.com/ores-truffle-oreslang/oreslang-data-science/pull/16) | `dde6cf4d8655e965d31a88beb14f7fbac605ce37` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-data-science/actions/runs/40441723270) |
| oreslang-db | [#3](https://github.com/ores-truffle-oreslang/oreslang-db/pull/3) | `40fe642fe0b9aadf40ad0bfdea793eb3e2877904` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-db/actions/runs/40441755350) |
| oreslang-design-patterns | [#9](https://github.com/ores-truffle-oreslang/oreslang-design-patterns/pull/9) | `79e7abbc01242e31faaa994f4b3c921343bb18b0` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-design-patterns/actions/runs/40441831102) |
| oreslang-format | [#101](https://github.com/ores-truffle-oreslang/oreslang-format/pull/101) | `37714d854440b2fd9874f2c50e04a1501e07da22` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-format/actions/runs/40441567629) |
| oreslang-hot-loading-tests | [#4](https://github.com/ores-truffle-oreslang/oreslang-hot-loading-tests/pull/4) | `f11379756b2724d01788732754e28569fd400143` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-hot-loading-tests/actions/runs/40441854807) |
| oreslang-hpc-finance | [#9](https://github.com/ores-truffle-oreslang/oreslang-hpc-finance/pull/9) | `0fef966ef26b7ce8c3d7c31aacd2e2a8288808c4` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-hpc-finance/actions/runs/40441716184) |
| oreslang-http-routing | [#23](https://github.com/ores-truffle-oreslang/oreslang-http-routing/pull/23) | `2b542ef9ef3f4d04ab1cd95251239511d92074c4` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-http-routing/actions/runs/40441774926) |
| oreslang-linked-queue | [#10](https://github.com/ores-truffle-oreslang/oreslang-linked-queue/pull/10) | `f943a651fee9d1d1a8f598a005117017c60d26c9` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-linked-queue/actions/runs/40441760849) |
| oreslang-ml | [#21](https://github.com/ores-truffle-oreslang/oreslang-ml/pull/21) | `5c4eaf4c007fe458dcd0e6b4e51c2228a25d3008` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-ml/actions/runs/40441825915) |
| oreslang-ml-conformance | [#10](https://github.com/ores-truffle-oreslang/oreslang-ml-conformance/pull/10) | `953dcd5c98a310eb961132e866bed0d18eae79ca` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-ml-conformance/actions/runs/40441673849) |
| oreslang-otel | [#12](https://github.com/ores-truffle-oreslang/oreslang-otel/pull/12) | `cd978119d40ccad5651e15262aea731ec4378030` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-otel/actions/runs/40441874440) |
| oreslang-platform-conformance | [#185](https://github.com/ores-truffle-oreslang/oreslang-platform-conformance/pull/185) | `4df27742b028061ad44eb3f2a2446166b66303eb` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-platform-conformance/actions/runs/40441849457) |
| oreslang-platform.asic | [#11](https://github.com/ores-truffle-oreslang/oreslang-platform.asic/pull/11) | `0a1338224614c4653bceb5b765aae8aa08c57038` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-platform.asic/actions/runs/40441594357) |
| oreslang-platform.desktop | [#12](https://github.com/ores-truffle-oreslang/oreslang-platform.desktop/pull/12) | `5496aac78fedb1724c6640a9569f06e907fa9d46` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-platform.desktop/actions/runs/40441836282) |
| oreslang-platform.embedded | [#27](https://github.com/ores-truffle-oreslang/oreslang-platform.embedded/pull/27) | `c68075c7a60ec44860f291160ed81f9979c7ed6d` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-platform.embedded/actions/runs/40441695711) |
| oreslang-platform.fpga | [#11](https://github.com/ores-truffle-oreslang/oreslang-platform.fpga/pull/11) | `06713f9687b45a7df2f65c0138588f235937259e` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-platform.fpga/actions/runs/40441882299) |
| oreslang-platform.iot | [#15](https://github.com/ores-truffle-oreslang/oreslang-platform.iot/pull/15) | `d39f0cbdf00048f21088eec868b0ae8261ea6def` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-platform.iot/actions/runs/40441819939) |
| oreslang-platform.mobile | [#14](https://github.com/ores-truffle-oreslang/oreslang-platform.mobile/pull/14) | `cdfb33e0b4d87a06b0cfb876cc9998891de407f7` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-platform.mobile/actions/runs/40441574991) |
| oreslang-proof-of-concepts | [#14](https://github.com/ores-truffle-oreslang/oreslang-proof-of-concepts/pull/14) | `4ffbb823e93d9a94b499b95744a049c777129173` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-proof-of-concepts/actions/runs/40441740937) |
| oreslang-restful-server-perf-benchmarks | [#13](https://github.com/ores-truffle-oreslang/oreslang-restful-server-perf-benchmarks/pull/13) | `5b3e184d32691137118755aee7e9eb3589c4e108` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-restful-server-perf-benchmarks/actions/runs/40441556478) |
| oreslang-rpc | [#12](https://github.com/ores-truffle-oreslang/oreslang-rpc/pull/12) | `dc131a9f4696830af7875e7004fa6019e7066afc` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-rpc/actions/runs/40441610429) |
| oreslang-runtime.realtime-java | [#7](https://github.com/ores-truffle-oreslang/oreslang-runtime.realtime-java/pull/7) | `651cd96140f79b05a642ccc8f760fe8b6f9d9e84` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-runtime.realtime-java/actions/runs/40441701317) |
| oreslang-serialization-and-validation | [#38](https://github.com/ores-truffle-oreslang/oreslang-serialization-and-validation/pull/38) | `a2957b52dec27c2d9be94f86e0b2b60550782c71` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-serialization-and-validation/actions/runs/40441607792) |
| oreslang-source.forge | [#112](https://github.com/ores-truffle-oreslang/oreslang-source.forge/pull/112) | `fa7d7f2a68d96291e139f633f9a31275bf57b065` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-source.forge/actions/runs/40441788765) |
| oreslang-source.hdl | [#21](https://github.com/ores-truffle-oreslang/oreslang-source.hdl/pull/21) | `f807594ddfc2ba8d8e7f00b48ee703029b4bd35e` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-source.hdl/actions/runs/40441782733) |
| oreslang-source.java | [#2133](https://github.com/ores-truffle-oreslang/oreslang-source.java/pull/2133) | `20be7c1cdcdb71316ee1e6571e3c5725fb7f320a` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-source.java/actions/runs/40441794640) |
| oreslang-source.js | [#86](https://github.com/ores-truffle-oreslang/oreslang-source.js/pull/86) | `8ba300e41556734200b33c88067b420ef786de54` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-source.js/actions/runs/40441681129) |
| oreslang-source.llvm | [#1078](https://github.com/ores-truffle-oreslang/oreslang-source.llvm/pull/1078) | `77119348891c77c8bd97037e62da869a212ab328` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-source.llvm/actions/runs/40441843883) |
| oreslang-source.net | [#181](https://github.com/ores-truffle-oreslang/oreslang-source.net/pull/181) | `37405d64322fd42fda3de90be4fff6417913f711` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-source.net/actions/runs/40441801927) |
| oreslang-source.rs | [#18](https://github.com/ores-truffle-oreslang/oreslang-source.rs/pull/18) | `709cd7be93b9e2959f820e95a0ee119d2460134f` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-source.rs/actions/runs/40441731456) |
| oreslang-source.wasm | [#147](https://github.com/ores-truffle-oreslang/oreslang-source.wasm/pull/147) | `0c3c62e6e559d1da802e043e380692a5bb6581ba` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-source.wasm/actions/runs/40441910253) |
| oreslang-spin | [#25](https://github.com/ores-truffle-oreslang/oreslang-spin/pull/25) | `4e5c38d54c7e9577784538f30f11c9df862a00fc` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-spin/actions/runs/40441583077) |
| oreslang-ssr | [#10](https://github.com/ores-truffle-oreslang/oreslang-ssr/pull/10) | `f4f2212cd5c1f041eae7249d532e83bcf0c1e2d3` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-ssr/actions/runs/40441563894) |
| oreslang-stack-cli | [#27](https://github.com/ores-truffle-oreslang/oreslang-stack-cli/pull/27) | `9498059db1b9fe97610ba137f9cee220f9803397` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-stack-cli/actions/runs/40441896346) |
| oreslang-static-analysis.rs | [#17](https://github.com/ores-truffle-oreslang/oreslang-static-analysis.rs/pull/17) | `c30b917124e37860c0a529aa715aeb392a7889bc` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-static-analysis.rs/actions/runs/40441510409) |
| oreslang-target-profiles | [#33](https://github.com/ores-truffle-oreslang/oreslang-target-profiles/pull/33) | `4c0c71d25e5df13205b68181b30d5145af4ab6c1` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-target-profiles/actions/runs/40441768894) |
| oreslang-tensor | [#11](https://github.com/ores-truffle-oreslang/oreslang-tensor/pull/11) | `4b85ca9b52bd0d71a13619e6e20aa9b54cc8b95f` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-tensor/actions/runs/40441655862) |
| oreslang-test.cli | [#20](https://github.com/ores-truffle-oreslang/oreslang-test.cli/pull/20) | `99986c40b86cdca0763012d087c1e1dc55a13f4a` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-test.cli/actions/runs/40441746547) |
| oreslang-wasm | [#42](https://github.com/ores-truffle-oreslang/oreslang-wasm/pull/42) | `07318a68ae89e601306274def640ebebe2ab4ca7` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/oreslang-wasm/actions/runs/40441903456) |
| postgres-orm.ores | [#3](https://github.com/ores-truffle-oreslang/postgres-orm.ores/pull/3) | `ae4edfa90fca4097c702df712c5cce18d64fa184` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/postgres-orm.ores/actions/runs/40441818607) |
| rx-ores | [#17](https://github.com/ores-truffle-oreslang/rx-ores/pull/17) | `465ca4226205ead9a1422324aa999834fc121c9f` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/rx-ores/actions/runs/40441664209) |
| rx-ores-callbacks | [#11](https://github.com/ores-truffle-oreslang/rx-ores-callbacks/pull/11) | `4f9102f3f3433ee9ba83571256985fdd32b129a9` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/rx-ores-callbacks/actions/runs/40441599754) |
| rx-oreslang-channels | [#25](https://github.com/ores-truffle-oreslang/rx-oreslang-channels/pull/25) | `a00ce808d14ba7a44c367abd788f7139c4c58181` | [NOT_EXECUTED (steps=null)](https://github.com/ores-truffle-oreslang/rx-oreslang-channels/actions/runs/40441586622) |
| .github | this PR | see PR metadata | pending at inventory creation |

## Review and next integration

Read each package's README and cases.tsv before connecting a producer.
A case is a requirement, not proof the feature exists. Bind real commands,
diagnostics and output oracles; unsupported required execution cases must fail.
Keep negative rejection, host-only checks, model results and guest execution
separate. Independently authenticate run/job/check issuer, commands and
artifacts, all dependency compiler pins and fixture hashes.

Use source-authorized execution and independent review before merging.
The checker intentionally outputs CONSISTENT_UNAUTHENTICATED_RECEIPT.
It does not authorize a release, private-source export or native/AOT capability.
