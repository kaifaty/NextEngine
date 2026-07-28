# Next Engine V1 Vertical Slice — implementation record

## Status

The implementation sequence `M0 → M11` is present in the repository. The full
portable/local closure passes on the pinned Rust `1.93.0` Apple Silicon
developer host. This is **not yet a shipping declaration**: native
Windows x86_64 and Linux x86_64 runtime/desktop/package gates must run on
those targets.
`v1-closure` reports them as `NOT_RUN`, never as success.

The implementation follows the Accepted architecture without a semantic
departure, so it did not add a superseding ADR.

## Milestone ledger

| Milestone | Commit | Implemented outcome |
|---|---|---|
| M0 | baseline `fd0ea61` | Clean contact-gated dialogue/quest bootstrap and green baseline checks |
| M1 | `7bcdb5f` | Aggregate envelopes, revision-checked multi-operation RPG command, immutable plan and atomic publish |
| M2 | `49034fa` | Inventory/equipment ownership, contact-bound pickup/equip and exact restore/retry |
| M3 | `19790f9` | Deterministic neutral project cook, content-addressed atomic publication and production activation |
| M4 | `2dd15e6` | Content-authored dialogue/quest/relationship interaction and shared headless activation |
| M5 | `dfe162e` | Portable `game` root, immutable presentation boundary and experimental SDL3/ash B0 adapter |
| M6 | `fbc36b4` | First-party combat through the ordinary public mechanics/effect/RPG command path |
| M7 | `f91622e` | Canonical two-chunk staging/admission and separate world-streaming persistence owner |
| M8 | `d3a000e` | Deterministic NPC planner and procedural idle/locomotion/melee fallback |
| M9 | `f45716d` | Pinned Luau sandbox, deterministic budgets, atomic proposal discard, circuit and state restore |
| M10 | `c92c66d` | Engine-owned WIT N/N−1 and pinned bounded Wasmtime Component host without ambient WASI |
| M11 | this closure change | Exact v1 closure roots, package descriptors, fallback matrix and honest target gates |

Each milestone was kept as a separate reversible local commit after its
relevant focused tests and product checks passed.

## Resulting vertical slice

The same cooked neutral project and activation lock now drive `game` and
`headless`. The authoritative path covers:

1. normalized movement;
2. contact-bound pickup;
3. inventory transfer and equipment assignment;
4. package-authored melee damage;
5. dialogue, quest and relationship transaction;
6. deterministic two-chunk transition and return;
7. save, process-state reconstruction and continuation;
8. command replay with exact state/event/ledger roots;
9. deterministic operation without network or `ai-host`;
10. optional Luau/Wasm removal with declared fallback behavior.

All domain mutation still enters through validated `WorldCommand` and the
owner-built atomic RPG transaction plan. Presentation, renderer, script VM,
Wasm runtime, worker completion order and optional AI never become gameplay
authority.

## Exact local closure

The canonical command is:

```bash
cargo run -p xtask -- v1-closure
```

At the implementation commit it binds these roots:

| Root | SHA-256 |
|---|---|
| Project composition lock | `26f9ef6523278c6c50b5219bd4e71ed5fe24ea0257f3ae07ebc5f43c162d57b0` |
| Schema registry | `a6eeccc359d0169d1cbe1c4428f47d81ace3d2e2eaec6b9be5c09f4a8618b4d3` |
| Content manifest | `76eb8adab27ada7844cf0f0ce6874d286f8b4ea574e118efc93100ec81459ef5` |
| Mechanics lock | `fc0513d102cd593ec498df316ce0f123a89b64131365c04c1b93c696806903ce` |
| World partition | `a7880df9ae6153a4d1db51f133503106ef78fa03ae152454b1d49f22d229ec36` |
| Luau manifest | `f54e7027ccf4f6ed3f1a896656f081f4e6626bad1065cf242704f0f9dda13ce8` |
| Wasm manifest | `9e59c4601037e3ffaf18a0bf4db33c9eb29e22b914b2b24611c33889c5c57f9e` |
| WIT v2 | `fb10af980777d42471fb27f8b511077518f989bf99917ff75d7545fc03a8fc20` |
| WIT v3 | `ce48d84af4035e2aa843af44993c01b94e8f5eec3378c3a750ca0cf0ca3109e4` |
| Extension compatibility | `9229ac99743b7dc5d38ed677c56385527622698934799c262337d590655e5097` |
| Local closure | `7b482d9473f73c797a9d51d50a6f35707c43ccfa5a789ed51d1bf4fff3aac6fd` |

The Windows and Linux target package descriptor hashes are respectively
`5b0d09c3adc8b0d73078d27848de640c0daa5723f4edc0fec81489280ea88e80`
and
`d0dde6560aeab748d4bfb58f2ec1512a323b805c48de2d760b80946cb33b0a9b`.
A descriptor binds target triple, `game`/`headless` roots and every exact
project/content/mechanics/extension hash; it does not claim that a target
binary was built or launched.

## Product check matrix

| Check | Local result | Shipping meaning |
|---|---|---|
| `host-check` | PASS | Formatting, clippy, workspace tests and boundary scan pass on the developer host |
| `play` | PASS | Complete offline cooked gameplay loop passes |
| `persistence-replay` | PASS | State, extension state, streaming segment and replay roots are exact |
| `content-package` | PASS | Cook/load plus data-only, Luau and Wasm ordinary package paths pass |
| `platform` | portable contract PASS; SDL/ash target run NOT_RUN | Must be rerun natively on Windows and Linux |
| `performance` | PASS | Streaming and planner numeric gates pass locally |
| `v1-closure` | `LOCAL_PASS_SHIPPING_TARGETS_NOT_RUN` | Release evidence is complete only after matching native PASS reports are collected from both targets |

Stable compatibility/fallback diagnostics in the closure are
`RPG_SCHEMA_UNSUPPORTED`, `WIT_API_N_MINUS_2_UNSUPPORTED`,
`OPTIONAL_EXTENSION_DISABLED` and `AI_HOST_OPTIONAL_FALLBACK`.

## Remaining release gates

On the exact implementation commit, run the following natively on both
`x86_64-pc-windows-msvc` and `x86_64-unknown-linux-gnu`:

```bash
cargo run -p xtask -- host-check
cargo run -p xtask -- play
cargo run -p xtask -- persistence-replay
cargo run -p xtask -- content-package
cargo run -p xtask --features desktop-sdl-ash -- platform
cargo run -p xtask -- performance
cargo run -p xtask --features desktop-sdl-ash -- v1-closure
cargo run -p xtask -- v1-package --output dist/nextengine-v1
```

`v1-package` builds release `game`/`headless` (including the desktop adapter),
publishes and reactivates the exact cooked project in a staging directory,
launches release `headless` and a bounded one-frame interactive release `game`
against that exact lock, hashes both binaries, writes a canonical package
manifest and atomically renames the complete distribution directory. It
rejects a non-shipping host, failed launch or pre-existing output. Compare the
reported package, project, state and ledger roots across the two native
reports. One invocation can certify only its current native target; release
readiness is the aggregate of matching Windows and Linux evidence. An
unavailable target remains `NOT_RUN`; it must not be manually promoted to
`PASS`.

Learned motor policy, LLM narrative and SPEC-31 runtime remain gated behind a
separate implementation plan with mandatory procedural/template fallback.
