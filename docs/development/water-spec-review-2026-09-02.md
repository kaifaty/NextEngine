# Water specification review — 2026-09-02

Scope: SPEC-38, ADR-076/100/101/102/103/104, the water parts of
SPEC-03/21/26, routing, traceability, README, roadmap, the water
task-state and plans 07/24, checked against `water.rs`, `water_flow.rs`,
`snapshot.rs`, `physics_step.rs`, the reference scene and
`verification/water_flow.rs`. Verified consistent without change:
priority classes `290/291`, capability ids, schema `3` / segment `v3` /
hash domain `...v3`, field layout 4/5, bounds `64`/`256`, the `core_r8d`
registry, the reference geometry and coefficients, the flux laws, the
largest-remainder scaling and the submersion classification.

## Applied in this revision

| Finding | Fix |
| --- | --- |
| SPEC-38 header still said 1.9; normative dependencies omitted ADR-100/103/104 | 2.2 with the revision note; dependencies added |
| SPEC-26/ADR-100 said the water table changes only through the command; the flow step rewrites cell levels every tick without command, event or revision bump | both documents name the flow step as the second writer for cells; ramped cells reject at validation |
| ADR-103 product check expected a common level; the scene's equilibrium is vessel A at its floor | ADR-103 check text matches the implemented gate; the common-level case is named as the contract unit test |
| Windows/Linux root equality still a promotion requirement (SPEC-38 twice, ADR-076) | Linux-only same-target gate (ADR-090); ADR-076 clause marked retired |
| ADR-104 called the reaction batch "existing"; none exists | "defined by ADR-076/081, first implemented by the buoyancy consumer", with the plan `continuum-water/08` contents named |
| Buoyancy consumer ownership contradictory between ADR-100/103/SPEC-38 | ADR-103 and SPEC-38 point to the ADR-104 consumer, which needs its own short ADR narrowing ADR-058 |
| ADR-103 said `SetLevel` rewrites the cell volume at once and suspends a ramp | resynchronisation at the next step via the synced revision; cells cannot carry a ramp |
| schema `2` / `v2` statements in ADR-100, SPEC-26, task-state | schema `3` everywhere, with the R8c history kept |
| rejection list of four; code has six; `SetSource` also drives sinks; unconditional flux limits; "record order"; record contents without tick rate and synced revision | ADR-103 and SPEC-26 enumerate the six codes, the per-kind limits, the ascending edge-id order and the full record contents; SPEC-21 names sinks |
| SPEC-38 authority section still defined a canonical particle state; persistence/fallback/warm-start paragraphs read as production | research qualifiers added; the real V1 persistence and fallback stated; `ContinuumWaterCanonicalStateV1` demoted to a research tool type |
| flow-step failure and world-revision semantics undocumented | SPEC-26: invariant fault stops the run with the prior checkpoint; water motion alone does not advance the world revision but changes the physics checkpoint hash and every root |
| lattice tier without bounds; face-sharing cells reject as overlapping | SPEC-38 practice 1 states the first-increment bounds and the closed-interval rule the lattice increment must resolve |
| practices written as present fact | practices 1-5 marked planned; skipped edges record flux `0`; presentation reads `effective_level` at the published tick |
| G6 cost failure not surfaced | SPEC-38 network performance row (`183 us`, report-only), ADR-103 consequence, ADR-104 row |
| research check ids reused the product ids | `CONTINUUM-PARTICLE-COUPLING-R1`, `CONTINUUM-PARTICLE-PERSISTENCE-R1` in SPEC-38, ADR-104, routing, traceability |
| stale README/traceability/roadmap/task-state rows; routing physics row without ADR-103 and `water-flow` | refreshed |
| ADR-102 stability clause referred to the frame-rate-confounded apparatus | per published set (plan 24 revision 3) |

## Open (code follows, not documents)

- **Tick rate cross-check.** Done 2026-09-03: `GroundedCapsuleWorld`
  rejects a non-empty network whose `ticks_per_second` differs from the
  world `TickRateProfileV1::gameplay_hz` with `PHYS_WATER_FLOW_INVALID`
  at activation and restore (unit test
  `water_flow_network_tick_rate_must_match_the_world_profile`).
- **Face-sharing cells.** `WaterVolumeDefinitionV1::overlaps` uses closed
  intervals, so a contiguous lattice cannot be authored; the lattice
  increment must choose half-open extents or an explicit gap rule.
- **Buoyancy consumer.** ADR-105 and plan `continuum-water/08` written
  2026-09-03 (frozen, not run); code follows.
- **Step cost.** `183 us` at the record bounds in release; a frozen bound
  waits for a consumer; in-place stepping and `i64` fast paths are the
  candidates.
- **Retired hash domain reuse.** `articulated.rs` uses the string
  `nextengine.physics-world-checkpoint.v2` for its own `checkpoint_hash_v2`;
  document or rename in the articulated increment so no two records share
  a domain string.
