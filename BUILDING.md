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

## Existing Oreslang Java/Graal reference

The [Java pom.xml](https://github.com/ores-truffle-oreslang/oreslang-source.java/blob/main/pom.xml) specifies Maven compiler release **21** and GraalVM dependency version **25.3.4.1** at the inspected revision. Use a compatible JDK, Maven and the distribution required by the relevant build profile:

~~~sh
git clone git@github.com:ores-truffle-oreslang/oreslang-source.java.git
cd oreslang-source.java
java -version
mvn -version
mvn -B test
git rev-parse HEAD
~~~

Maven unit tests are **not** a GraalVM Native Image execution receipt. Native Image requires the target repository's explicit build profile, installed GraalVM/native-image tooling and a verified execution of the output binary. Do not infer a JDK 25 requirement from the Maven release=21 setting; pin the chosen qualification matrix separately.

## Existing Oreslang native LLVM/C++ backend

Its [CMakeLists.txt](https://github.com/ores-truffle-oreslang/oreslang-source.llvm/blob/main/CMakeLists.txt) requires CMake **3.20+** and C++ **17**. The repository has special host-only configurations and target-dependent LLVM/Clang requirements. First follow its current README/workflow for the desired profile; a generic starting point is:

~~~sh
git clone git@github.com:ores-truffle-oreslang/oreslang-source.llvm.git
cd oreslang-source.llvm
cmake -S . -B build
cmake --build build --parallel 2
ctest --test-dir build --output-on-failure
~~~

A successful host-only CTest profile does **not** show actual .atto/.ores source lowered and executed natively. Record the executable, inputs, and native exit/output evidence separately.

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

Other atto-source.* / oreslang-source.* targets may use different build systems and incomplete subsets. Before publishing a command, verify the exact repository, target and executable path. If CI does not start or its steps are empty, report **NOT_EXECUTED** with the failure reason; do not report a pass.

For reproducibility archive the source and Git-tree SHAs, toolchains, command line, target platform, test counts, output artifact digest, failure status and log URL. See [TESTING.md](TESTING.md).
