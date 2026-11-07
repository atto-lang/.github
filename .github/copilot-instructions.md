# AttoLang GitHub Copilot instructions

Apply [the organization AOT / zero-GC controlled-dynamism directive](../docs/AOT_ZERO_GC_DYNAMISM_POLICY.md) and [AGENTS.md](../AGENTS.md). Avoid runtime eval, runtime class/method generation, unbounded reflection or hidden interpreter fallback in sealed AOT. Favor compile-time proxies/dispatch metadata and lifetime-checked actor/task/closure allocations. Annotate execution profile and memory claim, add negative-before-IR `.atto` tests, and never call parser-only or zero-step CI guest execution. Preserve the no-Python/Rust-only tooling rule.
