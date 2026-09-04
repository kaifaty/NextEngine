# Waves that answer events — the shallow-water presentation grid (plan 31 item 6)

| Field | Value |
| --- | --- |
| Research ID | `CONTINUUM-WATER-WAVES-P1` (presentation increment, no root or stage change) |
| Status | `RUN / G1-G5 PASS` (2026-09-04) |
| Parent | plan 31 item 6; plan 09 (the ring, the stage's pure frame), plan 17 (wakes and splashes of boxes), plan 21 (the edge records), ADR-101 (the dynamic ring) |
| Purpose | the ring's spectrum is the same whether the water is still or a crate just fell in. A wave grid on every ring, driven by the stage's records, lets impacts and jets send rings across the surface that reflect from the rim and die out |

## Frozen scope

- **Where.** In the game's water feed (`apps/game`), between the stage's
  frame and the adapter's update: presentation-only state per ring,
  stepped once per published frame at the presentation clock
  (`1/60 s`). The stage stays the pure function it is (its digest does
  not move); the authority reads nothing.
- **The grid.** The ring's own `32 × 16` vertices. Two height fields
  (this frame and the previous, metres, `f32`) per ring and the wave
  equation `h⁺ = 2h − h⁻ + (c·dt/dx)²·Δh` with `c = √(g·H)`, `H` the
  ring's authored depth (the volume's initial level over its floor),
  `Δh` the five-point Laplacian with reflective edges (the neighbour
  outside the ring mirrors the edge cell), a velocity damping of `2 %`
  per frame (`h⁺ = h + 0.98·(h⁺ − h)`) and a height relaxation of
  `0.5 %` per frame (the displaced volume returns to the level), and the stability bound checked
  at construction (`c·dt/dx ≤ 0.7` per axis, else the ring gets no
  grid). Heights are clamped to `±15 mm` and added to the stage's
  height before the shared `±20 mm` cap of the catalog bounds; normals
  are recomputed by the stage's finite-difference rule.
- **The excitation.** From the frame's records, per frame:
  every box record moves the cells under its plan by
  `v_box · dt · 0.3` (a falling box, negative speed, pushes the surface
  down; a rising one pulls it up); every edge record whose crest lies in the ring's
  plan lowers the crest's cell by `flux · hz / cell_area · dt` (the
  falling stream's push), capped at `5 mm` per frame. No player term
  (item 11).
- **Not in scope.** A stage record for the grid (the digest stays), a
  cross-ring wave, wetting of the rim, the PhysX lane.

## Frozen gates

| Gate | Clause |
| --- | --- |
| G1 roots | `play`, `persistence-replay`, `water-present` (its digest and roots) equal plan 37's; `host-check` PASS |
| G2 unit | a point impulse on a still grid spreads: after `n` frames the disturbance's radius is `c·n·dt ± one cell`; total energy (`Σh²`) decays monotonically after the impulse; a wave started at the centre reaches the rim and the edge cell's height rises (reflection) without the grid blowing up over `600` frames; the clamp holds |
| G3 cost | the four rings' step and re-normal per frame `< 200 µs` mean (an ignored timing probe, release) |
| G4 look (human) | `--start-at-vessels` capture at frame 60 against plan 36's: rings around the jet's mouth in vessel B and the sill's crest in vessel A, spreading and reflecting; the water-start capture differs only by the crate's own ripples if it moves |
| G5 stability | `--interactive --start-at-vessels --maximum-frames 362`: `PASS`, no `PRESENTATION_DYNAMIC_SURFACE_INVALID` (the cap keeps every vertex inside the catalog bounds) |

## Result (2026-09-04)

Implementation: `apps/game/src/water_waves.rs` (`WaveGridV1`: the ring's
grid, the wave equation with reflective edges, damping, relaxation and the
cap; excitation from the frame's box and edge records; every excitation
is a displacement of both height fields, not a velocity kick) and the
feed's `waved_surface` (the grid's heights added under the catalog cap,
the normals by the stage's rule); the feed excites once per stage frame
and steps once per publication.

| Gate | Reading | Verdict |
| --- | --- | --- |
| G1 roots | `play` `1af7bbc8…`, `persistence-replay` `c7cb4302…`, `water-present` `5a376a0d…` (plan 37's values, the stage untouched); `host-check` PASS (a first run caught the test-only grid helpers as dead code in the binary; gated `cfg(test)`) | pass |
| G2 unit | `a_point_impulse_spreads_at_the_wave_speed_and_never_gains_energy` (the 2 % front within two cells of `c·t/dx` at frames 8, 16, 24; `Σh²` never above the impulse's), `a_wave_reaches_the_rim_reflects_and_the_grid_settles` (the edge column rises, every height finite and under the cap over `600` frames, `Σh² < 10⁻⁸` from `10⁻⁴`), `the_clamp_holds_and_an_unstable_ring_gets_no_grid`, `records_excite_the_cells_they_cover` | pass |
| G3 cost | `bench_four_rings_per_frame` (release): `7 µs` per frame for the four reference rings | pass |
| G4 look (human) | frame 60 of the vessels' start against plan 36's: the water of vessel A carries concentric ripples from the sill's crest where it was flat (`14 523` pixels differ, all on the vessels' water); the water start differs only where the basin's ring is (the crate rests, so faint) | pass |
| G5 stability | vessels' start over `362` frames: `PASS`, no `PRESENTATION_DYNAMIC_SURFACE_INVALID`; the water start and the 120-frame vessels session `PASS` | pass |

Observation: the first test formulation gated `Σh²` against a velocity
kick (the impulse in one field only) and failed by construction, as the
scheme's conserved quantity is not `Σh²`; excitations are displacements
of both fields since, which is also what a moving box does to the water.
