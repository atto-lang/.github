# Why the REST demo spent about 10 ms in routing/dispatch

Follow-up investigation on 2026-10-07 found a runtime hot spot, rather than a
10 ms requirement for matching a few HTTP routes. See [execution modes and the
original measurements](execution-modes.md) for baseline context.

## Cause

`OresEvalRootNode.Evaluator.expressionContainsPotentialSuspension` conservatively
examines member calls to decide whether execution may suspend. For each such
call it constructed `new LinkedHashSet<>(classes.values())` to deduplicate
qualified/unqualified aliases of class declarations.

Class declarations are Java records. Their generated `hashCode()` recursively
hashes methods, statements and expressions. Thus deduplicating a handful of
class references repeatedly walked entire class syntax trees. The check runs
throughout interpreted execution, affecting routing and telemetry helpers as
well as handler code. This cost is unrelated to socket ownership transfer.

In one Java Flight Recorder run of 200 HTTP requests, 185 of 296 recorded
execution samples contained `Ast.ClassDecl.hashCode`. Samples include startup
and all request phases; this ratio is evidence of a hot spot, not an exact
percentage of routing latency. After the change it appeared in 9 of 84 samples,
so structural hashing elsewhere was not completely eliminated.

## Narrow fix

Deduplicate declarations using a fresh identity set for each scan, preserving
the existing class iteration order:

```java
Set<Ast.ClassDecl> visitedClasses = Collections.newSetFromMap(new IdentityHashMap<>());
for (Ast.ClassDecl klass : classes.values()) {
    if (!visitedClasses.add(klass)) continue;
    // Existing conservative method/suspension analysis follows unchanged.
}
```

Declaration aliases point to the same AST instance. This avoids recursively
hashing their contents. No suspension checks, borrow checks, actor ownership
rules or logging are disabled. The identity set is local to each scan; it is
not a long-lived cache and does not retain guest heaps.

## Evidence

A temporary instrumented copy split the main loop into span creation/logging,
`app.dispatch`, and span completion/export. Across 200 requests per version,
discarding the initial ten, median times were:

| Interval | Original | Identity deduplication |
|---|---:|---:|
| Create routing span and emit routing event | 0.735 ms | 0.218 ms |
| `app.dispatch` | 8.118 ms | 0.644 ms |
| Finish/export routing span | 0.842 ms | 0.288 ms |

The original routing span begins inside its creation call and stops inside
`finish`; its boundaries differ from these explicit outer timers. Do not add
these medians to reconstruct the original span. Instrumented measurements also
include profiling effects.

Then the **unchanged application and curl script** ran without JFR or added
timers, in baseline/fixed/baseline/fixed order. Each fresh process received six
batches of ten requests; the first batch was excluded from warm medians.
Telemetry remained enabled. All **240 requests** passed expected status/body
assertions, with **zero admission retries**.

| Measurement | Baseline, two repetitions | Fixed, two repetitions |
|---|---:|---:|
| Warm client median per request | 13.74 / 13.98 ms | 3.76 / 3.75 ms |
| Warm routing-span median | 8.97 / 9.30 ms | 1.07 / 1.05 ms |
| Warm ten-curl script median | 263 / 267 ms | 161 / 161 ms |

This is approximately a 73% reduction in median client latency and an 88%
reduction in the routing span in these runs. Script time improves less because
it also includes ten curl launches, shell processing and gaps between requests.

Environment: macOS ARM64, Temurin 25.0.4.1, baseline runtime `c82416067`, demo
`e6e7cf6`, with only the identity-deduplication runtime change for the fixed runs.
Temporary directories and ephemeral loopback ports kept these tests separate
from other servers/data. Stdout was redirected to a file.

These are short sequential JVM tests, not concurrent-throughput benchmarks or
claims about native AOT/hybrid improvement. Existing native packages have not
been rebuilt with this patch. Remaining latency includes interpretation,
conservative suspension analysis, telemetry serialization, scheduling and I/O;
this experiment does not attribute every remaining millisecond.

## Reproducing the investigation

With the pinned compiler/dependencies installed, record a JVM server with:

```bash
JAVA_TOOL_OPTIONS='-XX:StartFlightRecording=filename=dispatch.jfr,settings=profile,dumponexit=true' \
  ./scripts/run-server.sh
```

In another terminal run the ten-request script twenty times, then stop the
server normally so the recording is written:

```bash
for iteration in {1..20}; do ./scripts/curl-10.sh; done
jfr print --json --stack-depth 128 --events jdk.ExecutionSample dispatch.jfr > samples.json
```

Use an isolated data directory: the curl script deliberately replaces/deletes
the demo resource. For timing comparisons, disable JFR and any added diagnostic
timers, use the same telemetry settings, start fresh processes in alternating
baseline/fixed order, and correlate request IDs with completed server spans.

## Delivery and validation

The fix originated in commit `39f07082b` in [runtime PR #442](https://github.com/ores-truffle-oreslang/oreslang-source.java/pull/442).
It is now merged into runtime `main` through the #440/#439 dependency stack.
[Demo PR #4](https://github.com/ores-truffle-oreslang/oreslang-demo-restful-server/pull/4)
puts the logging/native integration on demo `main` and pins the merged runtime
`3f709fbf`, including this fix. Standard main-branch setup now uses that pin;
a local SOURCE_REF override is no longer required for the dispatch fix.

The performance-fix tree passed 1,238 runtime tests, 55 Spin HTTP assertions,
and 31 REST demo checks. Subsequent runtime changes have their own validation;
the measurements in this document remain measurements of the stated revisions.
Previously generated native archives are not updated by a Git merge: rebuild
them from the desired clean compiler/application pins before claiming this fix
is present in a native deployment.
