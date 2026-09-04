# Water and the ground — seepage into a ground-water cell (plan 31 item 9)

| Field | Value |
| --- | --- |
| Research ID | `CONTINUUM-WATER-GROUND-R1` (research report, not a product check) |
| Status | `RUN / G1-G4 PASS` (2026-09-05) |
| Parent | plan 31 item 9; ADR-103 (Accepted 1.0, revised here to 1.1); plans 07 (the network), 39 (the showcase) |
| Purpose | the lattice sits on the terrain and water neither soaks nor seeps; the showcase's spring is an unbounded source and the pond's drain a sink to nowhere. A `Seep` edge kind carries water from a surface cell into a ground-water cell at an infiltration rate per plan area, the ground-water cell is an ordinary volume (its level is the water table), and a well is a pump out of it. The showcase becomes a closed cycle: pond and lake seep into the ground, the spring pumps the ground into the lake |

## Frozen scope

- **Contract (ADR-103 1.1).** `WaterFlowEdgeKindV1::Seep {
  rate_micrometres_per_second }` (tag `7`, two cells: `cell_a` the
  wetted cell, `cell_b` the ground-water cell). Flux per second is
  `rate · area_a` (the infiltration capacity of the bed times the cell's
  plan area) while `cell_a`'s level is above `cell_b`'s, else `0`; it is
  limited like an `Open` edge by the wetted cell's whole volume and half
  the equalising volume, so seepage stops exactly when the water table
  reaches the surface. The rate validates in `0..=1 mm/s`; no command
  addresses a seep (`WrongKind`). Canonical record: field 2 the rate,
  the other slots canonical zeros; an older decoder rejects tag `7` with
  `UnknownTag` (a checkpoint holding a seep does not load on the old
  build; the checkpoint schema version is unchanged since no field
  changes). `water_currents` (ADR-105) ignores seeps: the transfer is
  vertical and makes no plan current. The presentation and the water
  audio make no record and no emitter of a seep.
- **Ground water.** A ground-water cell is an ordinary
  `WaterVolumeDefinitionV1` under the terrain, disjoint from the surface
  volumes; its level is the water table. A well is a `Pump` out of it
  (already expressible), bounded by its maximum head.
- **Showcase.** A hidden ground cell `0xf4` (`60 x 60 m` in plan,
  `y ∈ [-40, -10] m`, table at `-12 m`, no surface object, no ring);
  the pond seeps into it (`0xf3`, `48 µm/s` over `45 m²` = `2.16 L/s`)
  and the lake (`0xf5`, `3 µm/s` over `280 m²` = `0.84 L/s`); the
  spring `0xf2` becomes a pump ground → lake at `3 L/s` with a `20 m`
  maximum head. Seeps and spring balance exactly per tick, so the
  cycle lake → weir → stream → fall → pond → ground → spring is closed
  and the total over its six cells is conserved to the cubic
  millimetre. The vessels keep their outside source and sink.
- **Not in scope (recorded).** A soak rate per surface material read
  from a terrain material catalog (none exists; the rate is authored per
  edge), a well in the scene (a pump out of the ground is exercised in
  the contract's tests), ground water that flows between ground cells
  (a `Pipe` between two ground cells already expresses it).

## Frozen gates

| Gate | Clause |
| --- | --- |
| G1 roots | `play`, `persistence-replay`, `water-present`, `audio-scene` PASS with their roots recorded (they move: the showcase's network changes); `host-check` PASS |
| G2 contract | a seep's first-tick flux equals `rate · area / hz`; it is `0` when the table is at or above the wetted level; a ground cell filled to the wetted level by seepage stops the seep with the two levels equal; total volume across the two cells is conserved exactly over `1 000` ticks; tag `7` round-trips through the canonical record and tag `8` is rejected; a well pump out of the ground cell draws the table and stops when its head exceeds the maximum head; `water_currents` yields no current in a cell whose only edge is a seep; `SetSource` on a seep is rejected `WrongKind` |
| G3 showcase | nine volumes; after `1 800` ticks the lake in `2.20..=2.35 m`, the pond in `-0.15..=-0.05 m`, every stream edge and both seeps carry water, and the total over lake, stream cells, pond and ground is exactly unchanged (`delta == 0`) |
| G4 stage | the stage still carries eight surfaces, and no edge record nor audio emitter is made for a seep |

## Records to update

ADR-103 → 1.1 (the `Seep` kind, the revision section), the README row,
the task-state entry, plan 31 item 9 marked done.

## Result (2026-09-05)

Implementation: `WaterFlowEdgeKindV1::Seep` (tag `7`, capacity in
`0..=1 mm/s`, flux `k · A_a / hz` downwards, capped like an open sill);
`water_currents` skips seeps; the presentation makes no record of one;
the showcase's ground cell `0xf4`, the pond seep `0xf3` (the drain's
id), the lake seep `0xf5`, the spring `0xf2` as a pump ground → lake.

| Gate | Reading | Verdict |
| --- | --- | --- |
| G1 roots | `play 64ac5dac81d8`, `persistence-replay 7fb9966adcbc`, `water-present 6fbf15e511fc` (eight surfaces), `audio-scene ed0da7bedc3a`; `host-check` PASS | pass |
| G2 contract | `a_seep_soaks_the_pond_into_the_ground_until_the_table_reaches_it` (first-tick flux `10 000 mm³` for `100 µm/s` over `3 m²` at `30 Hz`; the total exact over `1 000` ticks; flux `0` once the table is set to the wetted level), `a_well_pumps_the_ground_until_its_head_is_exceeded`, `a_seep_is_the_seventh_kind_and_takes_no_command` (round trip, tag `8` unknown, capacity bound, `SetSource` → `WrongKind`) | pass |
| G3 showcase | nine volumes; after `1 800` ticks the lake and the pond inside their bands, every stream edge, the spring and both seeps carry water, the six-cell cycle's total exactly unchanged (`the_lake_feeds_the_stream_and_the_pond_holds_its_level`) | pass |
| G4 stage | eight surfaces; the spring is a `Mouth` record in the lake, the seeps have no record (`the_stage_shows_the_showcase`); a frame at the lake at frame `60` shows the lake held at its level | pass |

Apparatus note: the first draft of the contract tests put the ground
cell under the pond with its ceiling at the pond's floor, where the
table can never reach the wetted level (a stacked ground cell stops at
the floor); the test's ground cell now stands beside the pond and
reaches above it. The well's head margin was set to `2 cm` so the head
is exceeded inside the run (the pond rises `33 µm` per tick).

Recorded as open: a soak rate per surface material from a terrain
material catalog (none exists), a well in the scene, ground water that
moves between ground cells (a `Pipe` between them already expresses it).
