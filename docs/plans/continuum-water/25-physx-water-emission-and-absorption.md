# PhysX water lane — emission from exact data and absorption at the level (ADR-106 step 3)

| Field | Value |
| --- | --- |
| Research ID | `PHYSX-WATER-PRESENT-R2` (research report, not a product check) |
| Status | `RUN / G1-G5 PASS / G6 partial (reading recorded)` (2026-09-04) |
| Parent | ADR-106 (Accepted 1.0); plan 24 (the demo and the list of increments); plans 17 and 21 (the stage's splash and edge records); ADR-102 |
| Purpose | the PhysX fluid stops being a seeded block and becomes a transient layer on the exact level: particles are born where the exact model says water leaves it (a floating box's waterline while it moves, the crest of a jet or fall) and die when they settle on the level; the fluid's particle count is bounded and returns to zero at rest |

## Frozen scope

- **Stage input.** `WaterPresentationFrameV1` gains `boxes:
  Vec<WaterFloatingBoxV1>` (the committed dynamic boxes the stage already
  reads) so the lane sees the same exact inputs as the stage's splash.
- **Emission (lane, pure function `emission_for_frame`).** Once per
  published stage frame: for every floating box whose vertical speed
  exceeds `0.3 m/s` (plan 17's rule), `min(64, (|v| - 0.3) / 0.02)`
  particles on the box's waterline perimeter, launched outward and up at
  `0.8 m/s` (the plan 17 split); for every `Jet` or `Fall` edge record
  whose crest lies inside the fluid's box, `min(256, |flux| / 500,000
  mm^3)` particles at the crest with the record's direction and the weir
  or orifice speed of the head (plan 21's rule). Sources outside the
  fluid's box (the vessels, in this increment) emit nothing. Emission
  stops when the fluid holds `16,384` particles.
- **Absorption (lane, pure function `absorb`).** After every readback, a
  particle whose height is below the level plus `2` spacings (`0.1 m`)
  and whose speed is below `0.3 m/s`, or which lies below the floor, is
  removed: it has returned to the exact water. The remaining set (plus
  the frame's emission) is uploaded to the fluid
  (`ne_physx_fluid_set`: positions, velocities, active count).
- **Fluid box.** The basin: floor at the exact initial level, walls at
  the rim's inner faces, capacity `16,384`, spacing `0.05 m`, `60 Hz`
  from the frame clock; created empty. `--physx-water` runs the lane;
  `--physx-water-pour` adds plan 24's block for the look.
- **Statistics.** At session end the game prints `PHYSX_WATER:` frames,
  the peak particle count, the emitted and absorbed totals, the mean and
  max of step plus readback plus upload per frame.
- **Not in scope.** Body colliders in the fluid (the crate as a moving
  box: plan 26), the vessels' fluid box, level changes moving the floor,
  kernels and spray from the fluid, a run option beyond the flags.

## Frozen gates

| Gate | Definition | Pass |
| --- | --- | --- |
| G1 roots | `host-check`, `play`, `persistence-replay` PASS; `water-present` unchanged (the frame's new field is not in its digest) | pass |
| G2 emission rule | unit tests: a box at `1 m/s` yields `35` particles on its waterline with upward speed; a box at rest and an empty edge list yield none; a `Fall` record inside the box yields particles at its crest, one outside yields none; the cap holds | pass |
| G3 absorption rule | unit test: a particle `0.05 m` above the level at `0.1 m/s` is absorbed, one `0.5 m` above at rest is kept (it is still falling), one below the floor is removed | pass |
| G4 bounded and returning | a `900`-frame `--physx-water` session (no pour): the peak particle count is above `0` (the crate's first bounce) and the count at the end is `0` (everything absorbed after the crate settles); no `PRESENTATION_PARTICLE_SURFACE_INVALID` | pass |
| G5 cost | step plus readback plus upload mean `<= 4,000 us` per frame over the session | pass |
| G6 look (captures) | frames `40` and `140` from the water start with `--physx-water`: droplets around the rising crate at `40`; few or none at `140` | pass |

The rules, thresholds and caps are frozen; a change after the run is a new
revision.

## Result (2026-09-04)

Implementation: `WaterPresentationFrameV1::boxes` (stage);
`ne_physx_fluid_set` and capacity-sized buffers in the bridge,
`NativeFluid::set` / `max_particles`; `apps/game/src/physx_water.rs`
(`PhysxWaterLane`, pure `emission_for_frame` and `absorb`,
`LaneStats`), `--physx-water-pour` for plan 24's block.

| Gate | Reading | Verdict |
| --- | --- | --- |
| G1 roots | `host-check`, `play`, `persistence-replay`, `water-present`: see the task-state entry (the frame's new field is not in the present digest) | pass |
| G2 emission rule | unit test: the crate at `1 m/s` yields `35` particles on its waterline with upward speed inside the box; at rest or without edges none; a `Fall` inside the box yields `10` particles at its crest, one at the vessels none; the cap holds | pass |
| G3 absorption rule | unit test: `0.05 m` above the level at `0.1 m/s` absorbed, `0.5 m` above at rest kept, below the floor removed, fast at the level kept | pass |
| G4 bounded and returning | a clean `900`-frame `--physx-water` session (`controls=0`): peak `608` particles (the crate's first bounces), `921` emitted, `921` absorbed, `0` at the end; no bounds rejection. An earlier session with the user's input on the keyboard during the run (`controls=149`) emitted nothing (no published stage frames while paused), recorded as an observation | pass |
| G5 cost | step plus readback plus upload: `764 us` mean, `4,816 us` max per frame over the session | pass |
| G6 look | frame `40`: PhysX droplets around the rising crate; frame `140`: `513` particles remain as a thin sheet spreading around the still-bobbing crate (`757` emitted, `244` absorbed by then) — the frozen "few or none" did not hold because the crate keeps bobbing and re-emitting over the first seconds | partial; reading recorded |

Captures stay outside Git. Next increments per plan 24: the crate as a
moving collider in the fluid (plan 26), kernels and spray from the fluid,
a run option and statistics through the adapter.
