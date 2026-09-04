# PhysX water lane — recreation after a failure and kernels smoothed in time (plan 31 item 8)

| Field | Value |
| --- | --- |
| Research ID | `PHYSX-WATER-PRESENT-R8` (research report, not a product check) |
| Status | `RUN / G1-G5 PASS` (2026-09-04) |
| Parent | plan 31 item 8; ADR-106 (Accepted 1.0); plans 25-30 (the lane), 29 (demotion on failure), 30 (kernels) |
| Purpose | a failed fluid stays failed for the session and the kernels flicker as neighbourhoods change from frame to frame. The lane recreates its fluid after a bounded pause and blends every particle's kernel with its own previous one, which needs particles to keep an identity across the lane's absorption and emission |

## Frozen scope

- **Identity.** The lane keeps an id per particle slot, carried through
  absorption (the kept slots keep their ids) and emission (new ids from
  a counter); `absorb` returns the kept indices.
- **Kernel blending.** A particle's kernel for the frame is
  `0.7 · previous + 0.3 · new` when it had a non-zero kernel the frame
  before and has one now; a zero kernel (spray) stays zero and forgets;
  a particle without a previous kernel takes the new one. The stderr
  line gains `kernels_blended_permille` (particles blended with a
  previous kernel) and `kernel_change_permille` (`Σ|kₜ − kₜ₋₁| / Σ|kₜ|`
  over blended particles, the flicker measure).
- **Recreation.** After a demotion the lane counts frames; after `300`
  frames (`5 s` at the presentation clock) it recreates the fluid (empty,
  the basin box, the same profile) and resumes with the reason cleared;
  a failed recreation keeps the lane demoted and retries after another
  `300` frames, at most `3` attempts, then the lane stays demoted with
  the last reason. The injected failure fires once. The run report
  gains `recoveries` (serde default).
- **Not in scope (recorded).** The neighbourhood analysis on the GPU
  (`2.2 ms` on four threads at five thousand particles is within the
  frame; a compute pass is its own increment), a run on a host without
  CUDA beyond the broken-path fallback of plan 28 (no AMD host here),
  the retention decision of ADR-106 (the user's, with D-012).

## Frozen gates

| Gate | Clause |
| --- | --- |
| G1 roots | `play`, `persistence-replay`, `water-present` equal plan 39's; `host-check` PASS |
| G2 unit | `absorb` returns the kept indices in order; the blend rule (previous and new → the weighted mix; zero new → zero; no previous → new); the flicker measure of an alternating kernel sequence halves under the blend |
| G3 recreation | `--physx-water --physx-water-fail-after 60 --maximum-frames 600`: `PASS`; report `active: true`, `recoveries: 1`, `frames ≥ 200` (the lane's frames continue after the pause), emission after the recovery (`emitted` grows past its value at frame 60) |
| G4 flicker | the pour demo (`--physx-water --physx-water-pour --maximum-frames 120`): `kernel_change_permille` on the stderr line `≤ 300` at the end and `kernels_blended_permille ≥ 700` |
| G5 cost | the pour demo's `analysis_mean_us` stays `≤ 2 500` (plan 30's gate) with the blend included |

## Result (2026-09-04)

Implementation: particle ids carried through `absorb` (which now returns
the kept indices) and emission; `blend_kernel` and
`kernel_change_permille`; `FluidRecipe`, `create_fluid` and
`try_recreate` in the lane with `recoveries` in the statistics, the run
report and the stderr line; the injected failure fires once.

| Gate | Reading | Verdict |
| --- | --- | --- |
| G1 roots | recorded from the chain below | pass |
| G2 unit | `absorb_returns_the_kept_indices_in_order` (the first expectation counted a particle `0.2 m` up as absorbed; the band is `0.1 m`, corrected), `kernels_blend_with_their_previous_and_the_flicker_halves` | pass |
| G3 recreation | `--physx-water --physx-water-fail-after 60 --maximum-frames 600`: `PASS`; demoted after `60` frames, `PHYSX_WATER_RECREATED after attempt 1` `300` frames later; report `active: true`, `recoveries: 1`, `frames 299`, `emitted 909` (`458` at the demotion) | pass |
| G4 flicker | the pour demo: `kernels_blended_permille 737`, `kernel_change_permille 26` at the end; the clean run `1000` and `5` | pass |
| G5 cost | the pour demo `analysis_mean_us 2342` (plan 30: `2176`; the blend's map adds about `0.2 ms` at five thousand particles), under `2 500` | pass |

Recorded as open: the analysis on the GPU (a compute pass of the
particle pass), a run on an AMD or CUDA-less host (only the broken-path
fallback of plan 28 is exercised here), and the retention decision of
ADR-106 with D-012.
