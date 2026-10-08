# Oreslang execution modes: JVM, native AOT and native hybrid

Oreslang's Java-written runtime uses Truffle. The executable's build mode and
how guest Oreslang functions execute are distinct choices.

| | JVM | Native AOT | Native hybrid |
|---|---|---|---|
| What is deployed? | Java runtime classes/dependencies plus application sources | Native executable, embedded application sources, carrier library | Native executable, embedded application sources, carrier library |
| Java installation on the server? | Yes | No | No |
| How is the Java-written runtime executed? | By the JVM, with its own Java compilation/optimization | Compiled to machine code by GraalVM Native Image during the build | Compiled to machine code by GraalVM Native Image during the build |
| Guest Oreslang execution | Interpreted; optimizing guest JIT requires a compatible Truffle runtime/toolchain | Interpreted by the native runtime; guest JIT excluded | Guest JIT capability included; individual functions may still be interpreted |
| Capability selected by | Java/Truffle launch configuration | `scripts/build-native.sh aot` | `scripts/build-native.sh hybrid` |
| Runtime mode selection | Subject to runtime/toolchain support | Defaults to `aot`; rejects `jit` and `hybrid` | Defaults to `hybrid`; accepts `aot` to disable guest JIT |

Here “native AOT” means the **runtime/interpreter and reachable Java libraries**
are compiled ahead of time. It does not mean each Oreslang HTTP handler has been
lowered to application-specific native machine code. Ores sources are still
parsed/typechecked and executed by the embedded runtime. Neither native variant
needs a separately installed JVM or GraalVM at deployment time. GraalVM's build
and runtime components are incorporated as needed into the executable; the
whole GraalVM installation is not shipped alongside it.

Native mode is a build-time capability. A runtime flag cannot add a compiler to
an AOT-only executable. `--build-info` reports the compiled capability. Both native
packages include a required carrier library and depend on OS system libraries;
they are not fully static or cross-platform binaries. Build for the target OS/CPU.

## Measured REST demo, 2026-10-07

On macOS ARM64, the same ten-request `curl-10.sh` workflow ran six times per build:
one initial batch and five repeated batches, giving **180 requests total**. All
expected status/body assertions passed, with **zero admission retries**.

| Build | Warm median per request | Warm median for entire ten-curl script |
|---|---:|---:|
| JVM | 15.06 ms | 293 ms |
| Native AOT | 14.64 ms | 275 ms |
| Native hybrid | 14.59 ms | 269 ms |

“Warm” here means the five repeated batches; it does not establish that hybrid
JIT compilation reached steady state. Each variant ran sequentially in a fresh
server process, using its own temporary data directory and loopback port.
Telemetry was enabled and stdout went to a file. The JVM used Temurin 25.0.4.1;
native images used GraalVM CE 25.3.4.1+1.1.

Per-request latency is curl's `time_total`. Script wall time also includes curl
process launches, shell assertions, header parsing and gaps between requests.
Neither column includes server startup. These small samples do not establish a
statistically significant native speedup. They do not measure concurrent
throughput, saturation, network deployment latency or terminal rendering costs.
The demo intentionally admits one active request to serialize file-store writes.
Shared and isolated handlers perform different work, so this is not an isolated
measurement of actor isolation overhead.

The measured routing/dispatch span was approximately 10 ms. That interval includes
instrumentation and any scheduling inside dispatch: it is **not a measurement of
pure route lookup**. Actor startup and ownership transfer each had medians below
0.2 ms, while actor handling measured roughly 3 ms. Intervals can overlap and must
not be summed. Follow-up profiling is needed to attribute the dispatch interval.

Provenance:
- Runtime: [`c82416067`](https://github.com/ores-truffle-oreslang/oreslang-source.java/commit/c82416067)
- Native application: [`0b7900b`](https://github.com/ores-truffle-oreslang/oreslang-demo-restful-server/commit/0b7900b)
- JVM application: [`e6e7cf6`](https://github.com/ores-truffle-oreslang/oreslang-demo-restful-server/commit/e6e7cf6), differing only in native validation summary text
- Native telemetry library: [`d5535a5`](https://github.com/ores-truffle-oreslang/oreslang-otel/commit/d5535a5)

## Running and building

See the [REST demo](https://github.com/ores-truffle-oreslang/oreslang-demo-restful-server)
and its [native distribution PR](https://github.com/ores-truffle-oreslang/oreslang-demo-restful-server/pull/2).
After installing the pinned compiler/dependencies, `scripts/run-server.sh` launches
the JVM version. With the matching GraalVM build toolchain, use
`scripts/build-native.sh aot` or `scripts/build-native.sh hybrid`.
Extract a matching native archive and run `bin/rest-server --port=3000`;
`curl-10.sh` exercises its ten-request workflow.
