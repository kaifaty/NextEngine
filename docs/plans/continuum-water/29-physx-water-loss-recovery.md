# PhysX water lane — surviving a loss (ADR-106 step 7)

| Field | Value |
| --- | --- |
| Research ID | `PHYSX-WATER-PRESENT-R6` (research report, not a product check) |
| Status | `RUN / G1-G5 PASS` (2026-09-04, one apparatus correction in the adapter) |
| Parent | ADR-106 (Accepted 1.0); plans 25-28; plan 24 item 4 (device-loss handling) |
| Purpose | a loss during the session never ends the run: a Vulkan device loss is recovered by the adapter while the lane keeps publishing, and a failure of the fluid itself (a CUDA fault, a bridge error) demotes the lane to the stage's droplets for the rest of the session with the reason in the run report; both losses can be injected so the paths are exercised on a healthy host |

## Frozen scope

- **Fluid failure (game, `apps/game/src/physx_water.rs`).** A failed
  step, read or set of the fluid no longer fails the frame. The lane
  records the reason (the error code and text), frees the fluid, publishes
  one empty particle update so the last picture does not freeze on the
  screen, and answers every later frame with no update; the stage's
  droplets resume with the next simulation snapshot. The statistics up to
  the failure stay. A new flag `--physx-water-fail-after <frames>`
  (needs `--physx-water`) injects the failure after that many lane frames
  with the reason `PHYSX_WATER_INJECTED_FAILURE`.
- **Device loss (game and adapter).** The adapter already rebuilds the
  graphics after a recoverable presentation loss (`recover_graphics`);
  the lane publishes every frame, so the particle pass refills after the
  rebuild. A new flag `--inject-device-loss-after-frames <frames>`
  (needs `--interactive`) passes the adapter's existing injection option
  through so the path is exercised with the lane on. The stderr session
  line gains `particle_frames` (frames rendered with a particle update).
- **Report.** `presentation_fluid` after a demotion: `active: false`,
  `fallback_reason` with the recorded reason, the statistics up to the
  failure (`frames` counts the frames the fluid ran).
- **Not in scope.** Re-creating the fluid after its failure (the session
  continues with droplets; a restart brings the lane back), a real CUDA
  fault on demand (none can be produced safely; the injected failure
  follows the same code path as the bridge's error).

## Frozen gates

| Gate | Clause |
| --- | --- |
| G1 roots | `host-check`, `play`, `persistence-replay` PASS; `water-present` digest unchanged |
| G2 fluid failure | `--physx-water --physx-water-fail-after 60 --maximum-frames 120`: exit 0, `status PASS`; report `active: false`, `fallback_reason` containing `PHYSX_WATER_INJECTED_FAILURE`, `frames == 60`, `peak_particles > 0`; the adapter renders all 120 frames |
| G3 device loss | `--physx-water --inject-device-loss-after-frames 60 --maximum-frames 120`: exit 0, `status PASS`, `recoveries == 1` on the session line; report `active: true`, `frames >= 110`; `particle_frames >= 100` (the pass shows particles after the rebuild) |
| G4 clean run | `--physx-water --maximum-frames 120` unchanged: `active: true`, `peak_particles > 0` |
| G5 flags | `--physx-water-fail-after` without `--physx-water` and `--inject-device-loss-after-frames` without `--interactive` are refused as arguments |

## Result (2026-09-04)

| Gate | Reading | Verdict |
| --- | --- | --- |
| G1 roots | `host-check`, `play` (root `5f0c8bcd…`), `persistence-replay` (root `03901d76…`), `water-present` PASS | pass |
| G2 fluid failure | `--physx-water-fail-after 60 --maximum-frames 120`: exit 0, `PASS`; stderr `PHYSX_WATER_DEMOTED after 60 frames: PHYSX_WATER_INJECTED_FAILURE: injected after 60 lane frames`; report `active: false`, the same reason, `frames 60`, `peak_particles 458`; the adapter rendered 120 frames | pass |
| G3 device loss | `--physx-water --inject-device-loss-after-frames 60 --maximum-frames 120`: exit 0, `PASS`, `recoveries=1`, `particle_frames=115`; report `active: true`, `frames 122`, `peak_particles 674` | pass |
| G4 clean run | `active: true`, `frames 120`, `peak_particles 608` (as plan 28) | pass |
| G5 flags | both misuses refused with `CLI_ARGUMENT_INVALID` | pass |

Apparatus correction (adapter, found by G3 and reproduced without the
lane): the first frame after a device recovery failed with `B0 frame plan
invalid: skinning vertex-stream index is outside the uploaded frame
stream`. The reflection pass (plan 15) draws the plan's skinned meshes
before the main pass, and the skinned vertex stream of the frame slot was
prepared only inside the main pass's `record`; a fresh context after the
recovery had no offsets yet, while a fresh context at start was hidden by
the first frame having no water surface (no reflection) — and the
reflection of every later frame used the previous frame's stream. The
preparation now runs once per frame before the first pass
(`GraphicsContext::render`), so the reflection sees the frame's own
poses. A recovery without the lane passes the same way.
