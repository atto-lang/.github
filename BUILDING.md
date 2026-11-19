# Build and developer setup

**Rule:** commands below are checked against repository metadata/workflows, but are **examples to execute locally**, not a claim that builds or guest execution passed. Follow the target repository's current workflows and toolchain pins if they differ. Never use guessed build commands as release proof.

## Prerequisites and access

1. Obtain least-privilege GitHub access to the assigned repository; use SSH or an approved credential helper, never paste tokens into issues or logs.
2. Install Git, a recent shell, a C/C++ build toolchain as required, and the **pinned** toolchain shown in the target repository. Record the OS/arch, toolchain version, source commit and tree SHA.
3. Make a clean checkout of the assigned repository. All examples assume execution from its root. Use a local branch, not direct edits on main.
4. Check README, Cargo.toml / rust-toolchain.toml, pom.xml, CMakeLists.txt and .github/workflows before installing additional runtimes.

## AttoLang Rust browser/Wasm compiler

The [atto-source.rs CI workflow](https://github.com/atto-lang/atto-source.rs/blob/main/.github/workflows/ci.yml) specifies Rust **1.85.1** and target **wasm32-unknown-unknown** for this compiler; its Cargo.toml defines binary **attoc** and library **atto_compiler**.

~~~sh
git clone git@github.com:atto-lang/atto-source.rs.git
cd atto-source.rs
rustup toolchain install 1.85.1 --profile minimal --target wasm32-unknown-unknown
cargo +1.85.1 test --locked
cargo +1.85.1 build --locked --release --bin attoc
cargo +1.85.1 rustc --locked --release --lib --target wasm32-unknown-unknown --crate-type cdylib -- -C link-arg=--max-memory=67108864
git rev-parse HEAD
git rev-parse 'HEAD^{tree}'
~~~

The browser compiler also has guest tests run by Node and Chromium in its CI; **cargo test alone is not its full guest conformance suite**. Run the repository's current exact-head workflow/test commands before claiming Wasm execution.

## AttoLang Java/GraalVM backend

The canonical [atto-source.java](https://github.com/atto-lang/atto-source.java) repository contains **target-specific** native/host pilot workflows and source-backed conformance experiments. At the time of this documentation audit, its `main` tree does **not** contain a root `pom.xml`; therefore a generic `mvn test` instruction and the previous Maven compiler-release claims are not valid setup instructions for this repository.

~~~sh
git clone git@github.com:atto-lang/atto-source.java.git
cd atto-source.java
git rev-parse HEAD
git rev-parse 'HEAD^{tree}'
# Choose a checked-in .github/workflows profile and inspect its pinned toolchain
~~~

For a concrete GraalVM Native Image pilot inspect [the guard Native Image workflow](https://github.com/atto-lang/atto-source.java/blob/main/.github/workflows/guard-graal-native-image.yml) and `ci/verify-static-actor-native-image.sh`. Its GraalVM 25 configuration and source fixture are **profile-specific**; executing a host helper or Epsilon pilot does not certify general full-language guest parity, zero GC, or tenant isolation.

## AttoLang LLVM/C++ backend

The canonical [atto-source.llvm](https://github.com/atto-lang/atto-source.llvm) owns its native code-generation and runtime experiments. Its current root CMake configuration is a **temporary carrier-only build subset**, not a full-tree production compiler build; do not advertise `cmake --build` or `ctest` as universal success criteria.

~~~sh
git clone git@github.com:atto-lang/atto-source.llvm.git
cd atto-source.llvm
git rev-parse HEAD
git rev-parse 'HEAD^{tree}'
# Inspect .github/workflows and the exact LLVM18/Clang18 source-backed profile
~~~

For restricted native guest and rejection tests, inspect [guard native E2E](https://github.com/atto-lang/atto-source.llvm/blob/main/.github/workflows/guard-native-e2e.yml). It explicitly requires source/blob provenance and a real compiler-produced guest, and contains migration-era tooling that needs separate remediation. Host-only CMake/CTest or parser checks do **not** prove AttoLang execution; report exact toolchain, artifact digest, guest exit/output, negative diagnostics and nonzero steps.

## Zed-Pkg CLI integration

The [zed-cli rust-toolchain.toml](https://github.com/zed-pkg/zed-cli/blob/main/rust-toolchain.toml) pins Rust **1.98.1** at the inspected revision.

~~~sh
git clone https://github.com/zed-pkg/zed-cli.git
cd zed-cli
cargo +1.98.1 build --locked
cargo +1.98.1 test --locked
~~~

Package consumer verification should separately check manifest validation, lockfile reproducibility, target selection, offline/frozen installation and provenance using the CLI version under test. No compiler integration is proven merely by building Zed.

## Other backends and troubleshooting

Other `atto-source.*` targets may use different build systems and incomplete guest subsets. Before publishing a command, verify the exact repository, target and executable path. If CI does not start or its steps are empty, report **NOT_EXECUTED** with the failure reason; do not report a pass.

For reproducibility archive the source and Git-tree SHAs, toolchains, command line, target platform, test counts, output artifact digest, failure status and log URL. See [TESTING.md](TESTING.md).
