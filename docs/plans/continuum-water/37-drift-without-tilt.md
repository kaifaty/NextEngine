# Drift without tilt — floating boxes follow the water (plan 31 item 5, first cut)

| Field | Value |
| --- | --- |
| Research ID | `CONTINUUM-WATER-DRIFT-P1` (authority increment with recorded roots) |
| Status | `RUN / G1-G5 PASS` (2026-09-04) |
| Parent | plan 31 item 5; ADR-105 revision 1.1 and SPEC-26 2.9 (this plan's revisions, agreed by the user 2026-09-04); D-008 (vertical free bodies, amended here); ADR-103 (the flow network) |
| Purpose | boxes rise and fall on a vertical line: the canonical world integrates dynamic boxes along `y` only and the batch's drag has no horizontal effect. After this plan a floating box drifts with the water's current and is braked by the water in every axis, while a box resting on a floor keeps the push-only behaviour of WR1. Tilt and rotation stay out (a rotational rigid body in the exact world is an ADR of its own) |

## Frozen scope

- **The law (ADR-105 1.1).** The drag impulse acts on the relative
  velocity: `J_drag = −k ρ V (v − u) / (1000 hz)` per axis, `v` the
  committed linear velocity of the body, `u` the water velocity of the
  cell that holds the largest clipped volume; the buoyancy term is
  unchanged. `u` is zero for a cell that is not a node of the flow
  network. For a node: every edge of the network with two cells and a
  non-zero flux `Q` (cubic millimetres per tick, positive from `a` to
  `b`) contributes to both of its cells the vector
  `Q · hz · 10⁹ / (depth · width)` micrometres per second along the
  unit plan direction from `a`'s plan centre to `b`'s plan centre
  (q15), where `depth` is the cell's effective level over its floor and
  `width` the cell's plan extent projected across that direction
  (`|d_z| · extent_x + |d_x| · extent_z`); one-cell edges (sources,
  sinks, pumps to the outside) contribute nothing; contributions sum per
  cell in edge-id order; `i128` intermediates, truncation toward zero.
  No profile field changes; the batch's `compute` takes the network.
- **The world (SPEC-26 2.9, D-008 amended).** Every dynamic box applies
  all three components of its impulse at the first substep (`J / m` per
  axis), adds gravity to `v_y`, sweeps along `y`; when the downward sweep
  is cut the box is supported and its horizontal velocity is zeroed for
  the substep (push-only as before); otherwise it sweeps along `x` then
  `z` against the same obstacles and zeroes the cut axis's velocity.
  Rotation stays identity; the box count and the push rule are as WR1.
- **Roots.** The reference basin has no current and its crate rests on
  the floor or floats still, so the reference roots are expected to
  hold; if they move, they are re-recorded.
- **Not in scope.** Tilt, rotation, a current from the level gradient
  inside one cell, the lattice's cell-to-cell currents as a field, the
  player.

## Frozen gates

| Gate | Clause |
| --- | --- |
| G1 checks | `water-buoyancy` (`CONTINUUM-WATER-BUOYANCY-P1`, the immersion pins unchanged), `water-flow`, `water-volume`, `water-present`, `physics-collision`, `play`, `persistence-replay`, `content-package`, `host-check` PASS; roots recorded |
| G2 unit (contract) | still water reproduces the WB1 impulses byte for byte (the existing test); a two-cell lattice with a flowing sill gives both cells a current along `+x` of `Q·hz·10⁹/(depth·width)` and a still box in the source cell an impulse along `+x` of `k ρ V u / (1000 hz)`; a one-cell edge gives no current |
| G3 unit (world) | an unsupported box with a horizontal velocity moves by `v/hz` per substep and stops with zero velocity at a wall; a box resting on the floor with the same horizontal velocity stays put with its horizontal velocity zeroed; the vertical results of the WR1 tests unchanged |
| G4 drift | a physics-api scenario: one floating box in the source cell of a two-cell flowing lattice, the batch computed per tick with the network and fed as the step's impulses: the box's `x` increases monotonically over `60` ticks and its horizontal speed stays below the current; the same box on the floor (a sunk, heavier box) does not move in `x` |
| G5 cost | the batch's cost gate of plan 08 (`batch_cost_mean_us`) holds within `+20 %` on the water-buoyancy check |

## Result (2026-09-04)

Implementation: `water_currents` and the relative-velocity drag in
`crates/contracts/src/physics/buoyancy.rs` (`compute` takes the network;
the runtime passes the checkpoint's), the three-axis integration with the
support rule in `crates/physics-api/src/reference_world/locomotion.rs`
(`prepare_dynamic_states` no longer zeroes horizontal velocity), ADR-105
1.1, SPEC-26 2.9, D-008 amended, the index rows.

| Gate | Reading | Verdict |
| --- | --- | --- |
| G1 checks | `water-buoyancy` PASS with `immersion_settled 196368` (unchanged), `water-flow` (`drained_by_tick 868`), `water-lattice`, `water-present` (`5a376a0d…`), `physics-collision`, `audio-scene` (`3130730a…`), `play` (`1af7bbc8…`), `persistence-replay` (`c7cb4302…`), `content-package`, `host-check` PASS. Roots unchanged except `water-volume` (`82270ec3… → 0dcdb5de…`): its walk pushes the floating crate, and a pushed floating box now keeps its horizontal velocity and is braked by the water instead of stopping at once — the new rule, recorded | pass |
| G2 unit (contract) | `buoyancy_batch_is_exact_and_rides_the_step_input` unchanged (still water reproduces WB1 byte for byte); `flowing_lattice_gives_both_cells_a_current_and_a_still_box_drifts_with_it`: both cells `Q·hz·10⁹/(depth·width)` along `+x`, a still box gets `k ρ V u / (1000 hz)` along `+x`, without the network `0` | pass |
| G3 unit (world) | `unsupported_boxes_drift_and_resting_boxes_keep_the_push_only_rule` (a `0.3 m/s` push moves an unsupported box `0.3 m` in a second while the same push leaves a resting box in place), `a_drifting_box_stops_at_a_wall` (rests against the wall's face with zero horizontal velocity); the WR1 tests unchanged | pass |
| G4 drift | `a_floating_box_drifts_with_the_lattice_current`: over `60` ticks the box in the source cell moves along `+x` and never outruns the current, floating within `0.3 m` of its start height | pass |
| G5 cost | `water-buoyancy` in release: `batch_cost_mean_us 13`, `batch_cost_max_us 22` (plan 08 recorded mean `13`, maxima `18`–`25`); the debug run reads `177` mean, as debug runs do (the check gates cost in release only) | pass |
