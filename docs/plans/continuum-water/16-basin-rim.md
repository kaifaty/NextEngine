# Basin rim — authored walls around the reference basin

| Field | Value |
| --- | --- |
| Research ID | `WS4` |
| Status | `RUN` (2026-09-03) |
| Parent | plan `continuum-water/11` item 4; SPEC-26 bounded profile (static boxes); plan 13 (the shoreline that needs walls) |
| Purpose | the basin reads as a pool: a low authored rim around the water volume gives the shoreline fade and foam something to meet, blocks a casual walk into the water except through an opening, and frames the surface; the water authority is untouched |

## Frozen scope

- **Physics.** One static body `REFERENCE_WATER_BASIN_RIM_BODY_ID`
  (`0x89`) with five solid box shapes on the world layer: the west and
  east walls (`x 4.35..4.5` and `8.5..8.65`, `z 0.85..3.15`), the north
  wall (`z 3.0..3.15`, `x 4.35..8.65`) and the south wall split by an
  opening at `x 6.0..7.0` (segments `x 4.35..6.0` and `7.0..8.65`,
  `z 0.85..1.0`). Height `0.6 m` (`y 0..0.6`), above the authored `0.5 m`
  level, below the `1.5 m` level of the water checks. The opening keeps
  the reference walk of `water-volume` (`+x` to `6.5 m`, then `+z` into the
  basin) and the `--start-at-water` view free.
- **Presentation.** One authored compound mesh (`0x8e`, the five boxes,
  world coordinates, the base material) bound as an environment record to
  the rim body, like the R5b course.
- **Water.** No change to the volume, the level, the flow network or the
  buoyancy batch; the rim walls are ordinary static solids for the
  capsule and the boxes (the crate cannot leave the basin).
- **Pins.** Content roots, entries, meshes, scene records, bindings and
  rendered-object counts move by one; the pinned checks are refreshed from
  the failing runs once.

## Frozen gates

| Gate | Definition | Pass |
| --- | --- | --- |
| G1 checks | `water-volume`, `water-flow`, `water-present`, `water-buoyancy`, `physics-collision`, `play`, `persistence-replay`, `content-package`, `host-check` pass with the refreshed pins | pass |
| G2 walk | the `water-volume` walk still reaches `Wading` and `Swimming` (through the opening) | pass |
| G3 look (human) | the `--start-at-water` capture shows the rim around the water with the shoreline fade and foam along it | human |

Do not change the rim height, thickness or the opening after seeing the
captures; a change is a new revision with its own capture.

## Result (2026-09-03)

| Gate | Reading | Verdict |
| --- | --- | --- |
| G1 checks | `water-volume`, `water-flow`, `water-present`, `water-buoyancy`, `physics-collision`, `play`, `persistence-replay`, `content-package` and `host-check` all `PASS` on the tree with the rim (roots moved as authored content: 43 roots, 129 records, 17 meshes, 14 rendered objects; every pinned count refreshed) | pass |
| G2 walk | walked by the human on 2026-09-03 after the commit: the avatar enters the basin through the south opening, the "in water" HUD message appears; the rim body reaches the collision world through the same descriptor path as the push box | pass (human) |
| G3 look | release capture `--start-at-water` frame 120: four dark walls around the water, the south opening in front of the avatar, the crate and its reflection inside the rim | pass |

The capture stays outside Git.
