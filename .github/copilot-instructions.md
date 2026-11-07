# AttoLang AI contributor instructions

Follow [AGENTS.md](../AGENTS.md) and [the AOT/zero-GC policy](../docs/AOT_ZERO_GC_DYNAMISM_POLICY.md). Avoid unbounded runtime dynamism in sealed full-guest-AOT profiles. Produce static proxies/reflection metadata, check closure/actor/task-frame lifetime and bounded cleanup, and separate host AOT from guest AOT and process GC from guest GC. Reject unsupported constructs before IR. Require real positive/negative `.atto` compiler tests, executed target evidence and independent review; zero-step CI is NOT_EXECUTED.
