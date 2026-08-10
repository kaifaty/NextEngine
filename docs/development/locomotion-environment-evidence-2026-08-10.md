# Canonical flat-command locomotion environment evidence — 2026-08-10

This checkpoint records the local evidence for
`nextengine.motor.env.humanoid-flat-command.v1`. It proves a deterministic
training environment and data plane, not a trained policy, learned Motor MVP,
Stage 0 completion or R5 closure.

## Implemented surface

- Engine-owned command schedule, root-local 84-value observation, 23 residual
  joint actions, ten ordered Q16 reward components and separate
  termination/truncation.
- Partial per-slot reset with independent episode ordinals, terminal-slot
  lifecycle, bounded checkpoint envelope and atomic restore.
- `motor-lab` protocol v2, Python raw-integer client/normalized adapter and an
  external-only NPZ v2 trajectory recorder.
- Isaac descriptor/mirror v2 with exact CPU schedule precomputation,
  quaternion/frame conversion and correspondence v2 gates.

## Local results

| Check | Result | Evidence |
|---|---|---|
| `fast` | `NOT_RUN` | No `fast` xtask command exists; repository guidance maps the canonical fast handoff to `host-check`. |
| `boundary-scan` | `PASS` | Source layout, publish policy, public contracts, verification and unsafe/FFI boundaries passed after splitting large motor sources. |
| `host-check` | `PASS` | Windows x86_64, pinned Rust 1.93.0; clippy, fmt, workspace tests, docs and boundary scan passed. |
| `play` | `PASS` | Production Tools composition path completed 32 ticks and saved cleanly. |
| `persistence-replay --backend physx` | `PASS` | Native-compatible PhysX path completed with exact final state/ledger roots. |
| `platform` | `PASS` | Windows SDL/ash candidate and portable contract passed. |
| `BODY-SCHEMA-P1` focused evidence | `PASS` | Body canonicalization/cycle tests and schema-to-layout compiler permutation tests passed. |
| `PHYS-JOINT-P1` / `PHYS-SNAPSHOT-P1` focused evidence | `PASS` | Native PhysX articulation, canonical snapshot/restore and 10,000-substep solver continuation tests passed. |
| `MOTOR-SCHEDULE/SAFETY/STATE/ENV-P1` | `PASS` | Native 26-test motor suite passed, including worker permutations, clamp boundaries, restore and standing behavior. |
| `MOTOR-LOCOMOTION-ENV-P1` | `PASS` | Golden seeds/schedule, bounds/modes/rate limits, partial resets, invalid batch atomicity, terminal rejection, checkpoint continuation, reward properties and at least 1,000,000 aggregate schedule steps passed. |
| `MODEL-DATAPLANE-P1` | `PASS` | Native PhysX subprocess client and checkpoint continuation passed; external NPZ v2 recorder produced a 57-step terminal trajectory with widths 84/23/10/3. |
| `MODEL-MIRROR-P1` CPU golden | `PASS` | Rust descriptor bytes, profile hashes, command schedule, root-local transform and fixed PD matched Python/Torch. |
| `MODEL-MIRROR-P1` GPU corpus | `NOT_RUN` | Isaac Lab and Isaac Sim are not installed; host is Windows rather than the pinned Linux NVIDIA profile. |
| Linux/native cross-target | `NOT_RUN` | No native Linux host was available. |

## Conditional performance

The pinned PhysX 5.9.0 SDK doctor passed. One native release
`r5-physics-16 --mode report` run completed with exact root
`0841ca8674fac89f513f1a6f3ccbc2e155265265b055bc7e45e39b2b0ae2858b`
for 1/4/8 workers and all absolute metrics passed. Motor-frame p95/p99 were
`13,076/13,560 us` (1 worker), `3,730/3,868 us` (4 workers) and
`2,110/2,327 us` (8 workers); restore p95 was `2,187,223 us`.

The verdict remains `REPORT_ONLY`: the worktree was not clean, this was one
run rather than a fixed three-run gate, no compatible fresh ten-run V5 baseline
exists, and postflight observed GPU load at `40%`. This evidence cannot close
B-12 or Stage 0.

## Remaining gates

- Train and evaluate a policy separately; PPO, imitation, export and runtime
  learned evaluation were outside this package.
- Run the 256 x 600-tick Isaac GPU correspondence corpus and satisfy byte-exact
  command/profile/component gates plus reward and physics thresholds.
- Complete Linux platform/replay and same-commit cross-target evidence.
- Produce a clean ten-run V5 baseline and one fixed three-run hard performance
  `PASS`.
- Add terrain/friction/pose randomization, pushes, stumble/recovery,
  ragdoll/get-up and motion references only through later explicit profiles.
