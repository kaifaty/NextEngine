# Water rotational presentation — the whirlpool over a sink (SPEC-38 practice 4)

| Field | Value |
| --- | --- |
| Research ID | `WL-VORTEX` |
| Status | `RUN / G1 G2 PASS / G3 human open` (2026-09-03) |
| Parent | plan `continuum-water/11` section 3 (after plan 21); SPEC-38 2.2 practice 4 ("rotational flow is presentation"); plan 09 (the ring stage); plan 14 (ambient waves, the `20 mm` cap) |
| Purpose | whirlpools are a presentation-only layer of the bound surface, derived from the exact flux of a sink (or a pump drawing from the cell) and the cell geometry, never read by gameplay; the authored vortex variant of the practice, inside the existing ring cap and the stage budget |

## Frozen scope

- **Vortex rule (stage).** For every bound volume, every `Sink` edge on
  it and every `Pump` edge whose flux leaves it with non-zero last flux
  contributes one vortex centred at the volume's plan centre (the mouth
  of one-cell edges; the shared face midpoint for a pump), radius
  `WATER_VORTEX_RADIUS_MICROMETRES = 400_000`, dip depth
  `min(WATER_VORTEX_DEPTH_CAP_MICROMETRES = 12_000, |flux| / 2)`
  micrometres (`flux` in cubic millimetres per tick), spiral ripple
  amplitude `WATER_VORTEX_RIPPLE_MICROMETRES = 3_000`, two arms, one
  radial turn per `150 mm`, one revolution per second (the frame clock).
  Height at a vertex at distance `r < R` from the centre, with `f = (R -
  r) / R`: `-depth * f^2 + amplitude * f * sin(2 theta + k r - omega t)`,
  evaluated with `sin_q15` and the double-angle identities on integer
  `dx`, `dz`; outside `R` nothing. The result stays inside the existing
  `[-cap, cap - 1]` clamp of the ring (`WATER_RIPPLE_CAP_MICROMETRES`).
- **Nothing else.** No new record kind (the `Mouth` record of plan 21
  already names the sink), no particle emission, no gameplay read: the
  vortex is a pure function of the published flux and the frame index.
- **Reference scene.** Vessel B carries the sink (`0.5 L/s`); its
  surface ring shows the whirlpool while the sink drains
  (`|flux| = 16,666 mm^3` per tick: dip `8.3 mm`).

## Frozen gates

| Gate | Definition | Pass |
| --- | --- | --- |
| G1 roots, purity, bounds, cost | `water-present` PASS: roots identical, purity identical, every vertex inside the cap, stage cost `<= 1.0 ms` per frame | pass |
| G2 flux-driven | unit test: with the sink draining, vessel B's ring is lower at the centre than the same frame computed from a network whose sink flux is `0`, by at most the cap; with flux `0` the two frames are identical; the basin and vessel A rings are byte-identical in both | pass |
| G3 look (human) | walking to vessel B in the game shows a rotating whirlpool over the sink while it drains (the `--start-at-water` view does not show vessel B; the gate is the human's) | human |

The radius, depth rule, ripple amplitude, arms and speeds are frozen; a
change after the run is a new revision with its own evidence.

## Result (2026-09-03)

Implementation: `vortices_of` and `vortex_height` in
`crates/reference-game/src/water_presentation.rs`, applied inside
`surface_grid` before the ring's cap clamp; no new record, no particle,
no gameplay read.

| Gate | Reading | Verdict |
| --- | --- | --- |
| G1 roots, purity, bounds, cost | `water-present` PASS: roots identical, purity identical, `0` bounds violations, max ripple `20,000 um` (the cap), stage cost `141 us` max / `85 us` mean (release); the check's matrix digest moved from `a8904f22…` to `88655c7b…` as vessel B's ring now carries the whirlpool while the sink drains | pass |
| G2 flux-driven | unit test: with vessel B filled and draining (`sink flux < 0`) its ring is lower at the sink than the same frame from a network with the sink's flux at `0`, by at most the dip cap plus the ripple; the basin and vessel A rings are byte-identical; the frame recomputes identically | pass |
| G3 look (human) | not taken: the `--start-at-water` view does not show vessel B (`x 15-17.8 m`); the whirlpool shows while the sink drains vessel B, which needs water from vessel A through the gate | open (human) |
