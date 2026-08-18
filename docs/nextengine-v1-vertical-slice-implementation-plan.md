# Next Engine V1 Vertical Slice — implementation record

## Status

This file is a historical implementation record for the `M0 → M11` sequence.
Its local-result table describes the closure state when that sequence
completed; it is not the current native-target status or execution queue.
Current stage status lives in [roadmap.md](roadmap.md). Native Linux is now the
active development host under
[ADR-082](architecture/adr/082-linux-first-development-and-deferred-windows-host.md),
and deferred Windows work lives in
[development/windows-validation-backlog.md](development/windows-validation-backlog.md).

The implementation sequence is present in the repository. At record time, the
full portable/local closure passed on the pinned Rust `1.93.0` Apple Silicon
developer host while native Windows/Linux slots remained `NOT_RUN`. Later
native evidence must not be inferred from this historical snapshot.

The implementation follows the Accepted architecture without a semantic
departure, so it did not add a superseding ADR.

The 2026-07-29 refactoring closure additionally delivers:

- domain-namespaced current-only `next_contracts` API and
  `ProjectCompositionLockV2`;
- `next_reference_game` as owner of the first-party project/source/scenario;
- `next_application` as the production coordinator shared by `game`,
  `headless` and runtime-bearing `tools`;
- Runtime-owned SPEC-29 lifecycle decisions and Assets-owned atomic durable
  session generations;
- exactly-once final save/terminal receipt, restart recovery, required-save
  failure recovery and 1,000-cycle live-session/receipt stress coverage;
- versioned typed application/xtask JSON reports and a boundary rule that
  forbids production dependencies on `next_verification`.

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

The verified 2026-07-29 local closure binds these roots:

| Root | SHA-256 |
|---|---|
| Project composition lock | `1858ada0e1d5314d0d4992041668fbe35154e4c8039d149204728ee9b29fe0b8` |
| Schema registry | `a6eeccc359d0169d1cbe1c4428f47d81ace3d2e2eaec6b9be5c09f4a8618b4d3` |
| Content manifest | `3308ca242003e035db22f533f9a056b05f9542a896a1ff787da53aeab218ee1d` |
| Mechanics lock | `7e5006f72a5eb6d6772aa91a97623c8e2fe7ba85efa63fb7cac7743ee97709c1` |
| World partition | `05b0a58f35d0b25be9409262f1c363598ff0b863fea5dbe897640be5f4d55ef0` |
| Luau manifest | `f54e7027ccf4f6ed3f1a896656f081f4e6626bad1065cf242704f0f9dda13ce8` |
| Wasm manifest | `9e59c4601037e3ffaf18a0bf4db33c9eb29e22b914b2b24611c33889c5c57f9e` |
| WIT v2 | `fb10af980777d42471fb27f8b511077518f989bf99917ff75d7545fc03a8fc20` |
| WIT v3 | `ce48d84af4035e2aa843af44993c01b94e8f5eec3378c3a750ca0cf0ca3109e4` |
| Extension compatibility | `9229ac99743b7dc5d38ed677c56385527622698934799c262337d590655e5097` |
| Play authoritative state | `6a8497c815b769d887e59c88a3e87dfd631968a77fc11fcbc2824afb3131e63b` |
| Play command ledger | `505f38f1d5cfa93cb39514593823b8abb2b6e113f0e945ad3ec4c8acf523a98a` |
| Replay authoritative state | `dfaf8d0da19986b02f6595912bd81eb442c4da8f73daf103e738fe76d21e8924` |
| Replay command ledger | `5745d14b1f61943afa171f9bce0852c9fb54c4cec527ae0c8f2f05167d2dfc01` |
| Local closure | `233fbece67817ee62f276ca357042aabce3a6064fdec4e1575101cb0111cab44` |

The Windows and Linux target package descriptor hashes are respectively
`8700dd9a04a8b4e59401a360b4c35114621b145d7f6577f305e150b2d29b6bca`
and
`65598d7b168862ab64d2cf1e6b259908288c8455c3328057d63891b08d79d732`.
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

## Current native validation entry point

The earlier manual per-check matrix has been replaced by the native gate
harness. On each selected exact clean checkpoint, run:

```bash
cargo run --locked -p xtask --features desktop-sdl-ash -- native-gate-run --output artifacts/native-gate/<commit>
```

The authoritative command semantics remain in SPEC-12. This section preserves
the historical paired-gate entry point; current Linux-first execution and the
deferred Windows bundle/compare procedure are maintained in ADR-082 and
[development/windows-validation-backlog.md](development/windows-validation-backlog.md).

Learned motor policy, LLM narrative and SPEC-31 runtime remain gated behind a
separate implementation plan with mandatory procedural/template fallback.
