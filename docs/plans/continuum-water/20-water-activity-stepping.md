# Water activity stepping — only active water steps (SPEC-38 practice 2)

| Field | Value |
| --- | --- |
| Research ID | `WL-ACTIVITY` |
| Status | `RUN / G1 G3 G4 PASS / G2 G5 FAIL (readings recorded) / CONTINUUM-WATER-LATTICE-P1 = PASS` (2026-09-03) |
| Parent | plan `continuum-water/11` section 3 (after the lattice tier, plan 19); SPEC-38 2.2 practice 2 ("only active water steps"); ADR-103; plan 07 revision 2 (`step_in_place`) |
| Purpose | the exact flow step skips the edges that cannot move water this tick, with a derived activity set that is rebuilt on restore and never saved, so a large lattice at rest costs one comparison per edge and the roots of the always-stepped run are reproduced byte-for-byte |

## Frozen scope

- **Rest rule (contracts).** An edge is at rest for a tick when its
  state (`WaterFlowEdgeStateV1`, including `last_flux`) equals the state
  after the previous step, its last flux is `0`, and every endpoint
  cell's state (`WaterFlowCellStateV1`: volume and synced revision) after
  the authored-override sync equals the state after the previous step.
  The flux of an edge is a pure function of those inputs, so a resting
  edge would compute `0` again: skipping it and recording flux `0`
  changes nothing. Cells touched by no active edge keep their volume and
  level. A level command bumps the table's record revision, which the
  sync turns into a changed cell state (wake); a flow command changes
  the edge state (wake); a restore starts with an empty activity set
  (everything active for one step).
- **Derived state (contracts).** `WaterFlowActivityV1 { cell_states,
  edge_states }`: the states after the last step, keyed by id; `Default`
  is empty. `WaterFlowNetworkV1::step_in_place_with_activity(&mut self,
  volumes, activity) -> WaterFlowStepStatsV1 { edges, active_edges,
  skipped_edges }` uses and refreshes it; `step_in_place` stays the
  always-active path (an empty activity every step) and `step` is
  unchanged. The activity is not part of any canonical record or
  checkpoint.
- **Runtime.** `GroundedCapsuleWorld` holds one `WaterFlowActivityV1`
  beside its checkpoint (not canonical, not restored, reset to empty
  when the checkpoint is replaced) and `step_water_flow_in_place` steps
  with it; the backend trait default stays the cloning step.
- **Check (plan 19 check, revision 2).** `water-lattice` runs the
  `8 x 8` region three ways: always active (`step_in_place`), with
  activity, and with activity plus a wake at tick `1,200` (the west
  column's level record raised to `1.0 m` through the table's record
  revision, in both the always-active and the activity run). It reports
  the mean skipped-edge fraction over the run, the skipped fraction at
  tick `1,800`, and the tick range of the wake.
- **Not in scope.** Sleeping cells in presentation, streaming of
  regions, gameplay commands.

## Frozen gates

| Gate | Definition | Pass |
| --- | --- | --- |
| G1 roots identical | the activity run reproduces the always-active run's network and table hashes on every tick, with and without the wake | pass |
| G2 skipping happens | at tick `1,800` the skipped-edge fraction is `>= 25` percent (the drained west region's sills) and every skipped edge records flux `0` | pass |
| G3 wake | after the wake at tick `1,200` the west column's sills are active on tick `1,201` and the run returns to the same hashes as the always-active wake run | pass |
| G4 restore | the runtime path: `water-flow` G3 (restored run identical) and `persistence-replay` PASS with the activity in the world; `play` roots unchanged | pass |
| G5 cost | the activity step over the `64 x 112` lattice at rest (the last `100` ticks of a run continued to `3,600` ticks) is cheaper than the always-active step by at least a factor `2` in the mean, both reported | pass |

Rules, the wake tick and level, and the thresholds are frozen; a change
after the run is a new revision with its own evidence.

## Result (2026-09-03)

Implementation: `WaterFlowActivityV1`, `WaterFlowStepStatsV1`,
`WaterFlowNetworkV1::step_in_place_with_activity` (contracts), the
activity field of `GroundedCapsuleWorld` (reset with the table or the
network, empty after a fork or restore), the `water-lattice` check
revision 2. One rule correction found by the first unit test and fixed
before any reading: the activity must hold the *inputs* of the previous
step (the cell states after the authored sync at its start, the command
state of every edge), not the states after it — compared with the
states after the step, every cell looks unchanged at the start of the
next tick.

| Gate | Reading | Verdict |
| --- | --- | --- |
| G1 roots identical | the activity run reproduces the always-active run's network and table hashes on all `1,800` ticks and ends with equal networks and tables | pass |
| G2 skipping | at tick `1,800` no sill rests (`0` percent; mean `0.4` percent over the run, the initially dry east region until the water arrives); every skipped edge records flux `0` (exactness clause holds). The frozen `>= 25` percent did not hold because the exact weir law with integer truncation reaches flux `0` only when the head is under about `6 um` and drains the last micrometres at one cubic millimetre per tick: the first resting sill after the east column is wet appears at tick `12,221` (`6.8` minutes), `59` percent of the sills rest at tick `36,000` (`20` minutes) | FAIL against the frozen threshold; readings recorded |
| G3 wake | the wake at tick `1,200` activates all `112` edges on that tick and the activity wake run equals the always-active wake run byte-for-byte | pass |
| G4 restore | the runtime path with the activity in the world: `water-flow` (restored run identical) and `persistence-replay` PASS, `play` roots unchanged — see the task-state entry | pass |
| G5 cost | over the last `100` ticks of the `36,000`-tick run: always active `17 us` mean, with activity `14 us` mean (`59` percent of the sills skipped) — a factor `1.2`, not `2`: the resting edges still pay the comparison and the active `41` percent the full law | FAIL against the frozen factor; readings recorded |

Decision needed (task-state D-010 candidate): the practice is exact
and correct but rests late under the exact weir tail. A quiescence
clause in the law — an edge whose computed per-tick flux would be below
a frozen threshold (for example `1,000 mm^3`, one micrometre over a
square metre) records `0` — would let settled water rest within seconds
instead of minutes, but it changes ADR-103's flux law and therefore
every recorded root; it is not made here.
