# The lake, the dam and the stream — a water showcase in the reference scene (plan 31 item 7)

| Field | Value |
| --- | --- |
| Research ID | `CONTINUUM-WATER-SHOWCASE-P1` (content and authority increment with recorded roots) |
| Status | `RUN / G1-G5 PASS` (2026-09-04) |
| Parent | plan 31 item 7; the user's decision of 2026-09-04 (a lake now, with a dam and a small stream running downhill, one scene that shows the fluid scenarios); ADR-103 (the network), plans 32 (the pond), 34 (sound), 35 (wetness), 36 (the lever), 37 (drift), 38 (waves) |
| Purpose | one place where every water scenario of V1 is visible on foot: a big still body (the lake), a weir over a dam crest, a stream stepping down three terraces with sills and falls, a fall into the pond, a spring and a drain that hold the levels, and the pond, basin and vessels as before. The reference floor grows so the scene fits |

## Frozen scope

- **The ground.** The floor strips extend to `x, z ∈ [−30, 30] m`
  (`60 × 60 m`) around the pond's hole; the floor mesh follows. The
  vessels, until now past the floor's east edge, stand on it.
- **The lake (authority).** Volume `LAKE` (`0xe0`) `x −20..0 m`,
  `z 14..28 m`, `y 0..4 m`, initial level `2.25 m`, the reference
  swimming depth. Banks: a static body of boxes `1 m` thick and `3 m`
  high around it (west, north, east, and the dam as the south bank in
  two pieces) with the spillway between `x −4.75..−3.25 m`: a sill
  piece `y 0..2.2 m`. A staircase of `12` steps (`0.25 m` rise, `1 m`
  tread) climbs the west bank's outside from `z 17 m` to the bank top,
  so the player walks the bank ring and looks down at the lake.
- **The stream (authority).** A lattice region (`0xe3`) of one column
  and three rows along `z` from `z 10 m` to `13 m`, `1.5 m` wide
  (`x −4.75..−3.25 m`), floors `0.5`, `1.2`, `2.0 m` (south to north),
  initial levels `5 cm` over the floors, sills between rows at the
  higher floor (coefficient `600 ‰`). Terraces under the cells and low
  side walls (`0.3 m` over each floor) as static boxes.
- **The network.** The region's two sills plus: `LAKE → row 2` an
  `Open` weir at the dam crest (`sill 2.2 m`, width `1.5 m`,
  `600 ‰`); `row 0 → POND` an `Open` sill at `0.5 m` (the fall of
  `0.6 m` into the pond); a `Source` into the lake (the spring,
  `3 L/s`); a `Sink` from the pond (the drain, `3 L/s`). The vessels'
  edges unchanged. The lake settles a few centimetres over the crest
  where the weir carries the spring's rate; the pond stays at `−0.1 m`.
- **Presentation.** Four new surface quads (the lake, three cells) and
  four new ring bindings (eight rings, the adapter's bound); one
  authored works mesh (banks, dam, sill, stairs, terraces, walls, the
  base material) bound to the works body. Sound, wet band, waves and
  the lever's path apply as they are; the game's particle bounds cover
  the new bodies.
- **Diagnostics.** `ReferenceSpawnOverrideV1::at_falls()` (south-west of
  the pond at `x −7.5 m`, looking `+z`, the stream and the dam right of
  the avatar) behind `--start-at-falls`.
- **Not in scope.** A large-ring profile or LOD (item 10: the lake's
  ring keeps `32 × 16` vertices, `0.65 × 0.93 m` cells), a lever on the
  spillway, the player in the lake (banks keep it out), erosion.

## Frozen gates

| Gate | Clause |
| --- | --- |
| G1 checks | every water check, `physics-collision`, `audio-scene`, `play`, `persistence-replay`, `content-package`, `host-check` PASS after the one refresh of pins; the new roots recorded |
| G2 unit | the ten volumes and the network validate; over `1 800` ticks the lake's level ends within `2.20..2.35 m` and the pond's within `−0.15..−0.05 m`; every stream sill, the dam weir and the fall carry flux at the end; the total volume changes by `source − sink` only |
| G3 stage | the stage's frame carries eight surfaces and, once the stream runs, edge records for the weir, the two sills and the fall (a `Fall` kind for the drop into the pond) |
| G4 session | `--interactive --start-at-falls --maximum-frames 362`: exit 0, `PASS`, no bounds rejection; frames per tick `≥ 5.0` |
| G5 look (human) | captures at frame 60 and 360 from `--start-at-falls`: the pond in front with the fall's droplets at its north edge, the three terraces stepping up to the dam with water on each, the weir's jet at the crest; ripples on the pond from the fall (plan 38), the wet band along the terraces (plan 35); a second capture from the bank top over the lake |

## Result (2026-09-04)

Implementation: the lake definition, the stream's lattice region (three
cells, its own sills), the weir, the fall, the spring and the drain as
edges of the reference network, the works body (banks, dam, sill, twelve
steps, terraces, walls) and its mesh, the larger floor strips and mesh,
four surface quads and four more ring bindings (eight, the adapter's
bound), the game's particle bounds and wave grids over the new rings,
`--start-at-falls` and `--start-at-lake`.

| Gate | Reading | Verdict |
| --- | --- | --- |
| G1 checks | `water-volume` (`815ea59e…`), `water-flow` (`ce81f1d5…`, `drained_by_tick 868`), `water-present` (`f1c48305…`, `surfaces_per_frame 8`), `water-buoyancy` (`8b4e3f21…`), `water-lattice`, `physics-collision`, `audio-scene` (`8950864d…`), `play` (`933982a0…`), `persistence-replay` (`d8a6bf2a…`), `content-package`, `host-check` PASS after the pin refresh | pass |
| G2 unit | `the_lake_feeds_the_stream_and_the_pond_holds_its_level`: eight volumes and the network validate; after `1 800` ticks the lake's level lies in `2.20..2.35 m`, the pond's in `−0.15..−0.05 m`, the weir, both sills and the fall carry flux; the network's exact cell volumes move only by the vessels' bounded source (`≤ 30 L`) — the level-derived total is not exact for a `280 m²` cell (a micrometre of its level is `0.28 L`), recorded | pass |
| G3 stage | `the_stage_shows_the_showcase`: eight surfaces, records for the weir, the two sills and the fall, the fall's kind `Fall` | pass |
| G4 session | `--start-at-falls --maximum-frames 362`: `PASS`, `362` frames over `68` ticks (`5.3` per tick), no bounds rejection; the lake start `PASS` | pass |
| G5 look (human) | the falls start (frame 60): the pond in front, the three terraces stepping up to the dam with water on each and the falls' droplet streaks between them, the dam wall behind; the first start faced the stream and the avatar hid it (a third-person camera hides whatever the avatar faces), moved aside. The lake start: the avatar on the west bank's top, the lake's refracted floor, the banks and the stair at the right | pass |

Pins refreshed once: root assets `45 → 50`, entries `133 → 138`, meshes
`19 → 24`, rendered objects and draws `17 → 22`, scene records
`19 → 24`, bindings `18 → 23`.

Observations: the stage's cost in the debug check rose from `934` to
`2 040 µs` mean with eight rings (the release gate holds); the lake's
ring at `32 × 16` vertices has `0.65 × 0.93 m` cells, coarse for its
ripples — the large-ring profile is item 10.
