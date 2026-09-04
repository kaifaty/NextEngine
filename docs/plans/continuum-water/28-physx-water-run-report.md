# PhysX water lane — the lane in the run report (ADR-106 step 6)

| Field | Value |
| --- | --- |
| Research ID | `PHYSX-WATER-PRESENT-R5` (research report, not a product check) |
| Status | `RUN / G1-G5 PASS` (2026-09-04) |
| Parent | ADR-106 (Accepted 1.0); plans 25-27; plan 24 item 4 (a run option surface, the capability probe, bounded statistics) |
| Purpose | the lane's presence, its fallback reason and its bounded statistics are part of the game's run report (`RunReportV1`), the same JSON every root and tool reads, instead of a line on stderr; the request stays the `--physx-water` flag, the probe at start decides, and a host without the lane reports it as absent with a reason while the run passes |

## Frozen scope

- **Report (`crates/application`).** `RunReportV1` gains
  `presentation_fluid: Option<PresentationFluidReportV1>` (serde
  default, absent when the lane was not requested):
  `{ lane: "physx-pbd", active, fallback_reason: Option<String>, frames,
  peak_particles, emitted, absorbed, last_particles, cost_mean_us,
  cost_max_us, analysis_mean_us, inside_colliders_max,
  spray_fraction_max_permille }`. Strings and integers only; no vendor
  type enters the application crate.
- **Game.** With `--physx-water` the report carries the object: `active:
  true` with the lane's statistics, or `active: false` with the probe's
  reason (missing library, no CUDA device, …) and zero statistics. The
  stderr line stays as a developer convenience. A build without the
  feature keeps refusing `--physx-water` as before.
- **Not in scope.** A player preference (SPEC-18) for the lane, the
  adapter's own report (the lane lives in the game root), device-loss
  recreation of the fluid (recorded as open).

## Frozen gates

| Gate | Definition | Pass |
| --- | --- | --- |
| G1 roots | `host-check`, `play`, `persistence-replay` PASS; old reports without the field still deserialize (unit test) | pass |
| G2 report shape | unit test: a report with the object round-trips through JSON; one without it serialises without the key | pass |
| G3 active run | `--physx-water --maximum-frames 120`: the JSON report has `presentation_fluid.active == true`, `frames >= 120`, `peak_particles > 0` (the crate's first bounce) | pass |
| G4 fallback run | the same with `NEXTENGINE_PHYSX_GPU_LIBRARY=/nonexistent/…`: `active == false`, a non-empty `fallback_reason`, `status == "PASS"`, exit `0` | pass |
| G5 no request | a run without the flag has no `presentation_fluid` key | pass |

## Result (2026-09-04)

Implementation: `PresentationFluidReportV1` and
`RunReportV1::presentation_fluid` (`crates/application/src/report.rs`,
serde default, skipped when absent), the game fills it from the lane's
statistics or the probe's reason; `NativeFluid::create_reporting` and
`fluid_unavailable_reason` carry the bridge's reason text into the
report.

| Gate | Reading | Verdict |
| --- | --- | --- |
| G1 roots | `host-check`, `play` (root `5f0c8bcd…`), `persistence-replay` (root `03901d76…`) PASS; the xtask package fixture and the unit test cover the legacy shape | pass |
| G2 report shape | unit test: the object round-trips, a report without it serialises without the key and deserialises | pass |
| G3 active run | `--physx-water --maximum-frames 120`: `active: true`, `frames 120`, `peak_particles 608`, `emitted 726`, `cost_mean_us 718`, `analysis_mean_us 102`, `inside_colliders_max 0` | pass |
| G4 fallback run | with the library path broken: `active: false`, `fallback_reason` "PhysX fluid unavailable: PHYSX_SDK_UNAVAILABLE (gpu library or cuda device unavailable)", `status PASS`, exit `0` | pass |
| G5 no request | a run without the flag carries no `presentation_fluid` key | pass |

Observation (apparatus): an active run started while `host-check`
compiled the workspace on every core emitted nothing (`peak_particles
0`): the game loop stalled and the stage frames of the crate's first
bounce were skipped by `read_latest_snapshot`. Sessions used as evidence
run alone.
