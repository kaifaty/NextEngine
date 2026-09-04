# The reference pond — a sunken body deep enough for the camera (plan 31 item 1, part A)

| Field | Value |
| --- | --- |
| Research ID | `CONTINUUM-WATER-POND-P1` (content increment with recorded roots) |
| Status | `RUN / G1-G5 PASS` (2026-09-04) |
| Parent | plan 31 item 1 (the camera under the surface); the user's choice of 2026-09-04 (a new deep pond rather than a longer basin or a closer camera) |
| Purpose | no water body of the reference scene can hold the third-person camera (4 m behind the avatar, focus 0.7 m up): the basin is 4 × 2 m and 0.5 m deep, the vessels smaller. A pond sunk into the ground, 10 × 4.5 m and 1.4 m deep with a broad stair, puts the camera under the level as soon as the avatar walks down, gives the first `Swimming` classification of the scene and the ground for plan 33 (the underwater view) |

## Frozen scope

- **Volume (authority).** `REFERENCE_WATER_POND_ID` (`0x9a`), bounds
  `x −9..1 m`, `y −1.5..0 m`, `z 5..9.5 m`, initial level `−0.1 m`
  (10 cm below the ground), the reference swimming depth (`1.2 m`), no
  ramp, profile revision 1. Not a node of the flow network; no buoyancy
  body of its own. The player's classification at the pond floor is
  `Swimming` (1.4 m over the foot point), on the top step `Wading`.
- **Ground (physics).** The one floor box becomes four strips around
  the hole (south `z −10..5`, north `z 9.5..10`, west `x −10..−9`, east
  `x 1..10`, each `y −1.7..0`), so the hole's sides are solid to the pond
  floor; a new static body `0x9b` carries the pond floor
  (`y −1.7..−1.5`) and five full-width steps at the west end (`0.5 m`
  deep, `0.25 m` rise, tops at `−0.25 … −1.25 m`, below the capsule's
  `0.3 m` step limit).
- **Presentation.** The floor mesh (`0x81`, revision 3) becomes the four
  strips at its `y 0.1 m`; a new pond interior mesh (`0x9d`: four walls,
  the floor, the five steps, the base material) bound to the pond body;
  a new surface quad (`0x9e`) over the pond plan with the water material,
  bound like the vessel quads; a fourth `WaterSurfaceBindingV1` so the
  ring, the ambient spectrum, the shoreline and the water pass apply; the
  particle bounds of the game include the pond.
- **Diagnostics.** `ReferenceSpawnOverrideV1::in_pond()` (the avatar on
  the pond floor at `x −4.5 m`, looking `+x` and 12 degrees up, the camera
  4 m west inside the pond `0.73 m` under the level) behind
  `--start-at-pond`.
- **Pins.** Content roots, entries, meshes, scene records, bindings,
  body counts, the `play` and `persistence-replay` roots and the
  `water-present` digest move; the pinned checks are refreshed from the
  failing runs once and the new values recorded here.
- **Not in scope.** Any render change for the camera under the level
  (plan 33); a lattice for the pond; a source or sink.

## Frozen gates

| Gate | Clause |
| --- | --- |
| G1 checks | `water-volume`, `water-flow`, `water-present`, `water-buoyancy`, `water-lattice`, `play`, `persistence-replay`, `content-package`, `host-check` PASS after the one refresh of pins |
| G2 classification | unit: the pond definition validates; the foot point at the pond floor classifies `Swimming`, on the top step `Wading`, on the ground beside the hole `Dry` |
| G3 geometry | unit: every step rise `≤ CAPSULE_MAX_STEP_HEIGHT_MICROMETRES`; the pond plan lies inside the floor strips' outer bound and overlaps neither the basin nor the vessels; the interior mesh's bounds equal the pond hole |
| G4 session | `--interactive --start-at-pond --maximum-frames 120`: exit 0, `PASS`; the HUD water label reads swimming (the `player_water` class in the UI records, checked through the capture's status line) |
| G5 look (human) | capture at frame 60 from `--start-at-pond`: the avatar on the pond floor, the stair behind, the surface plane above the camera; this is the "before" picture of plan 33 (no underwater treatment yet) |

## Result (2026-09-04)

| Gate | Reading | Verdict |
| --- | --- | --- |
| G1 checks | `water-volume`, `water-flow`, `water-present` (`surfaces_per_frame 4`), `water-buoyancy`, `water-lattice`, `physics-collision`, `platform`, `play`, `persistence-replay`, `content-package`, `host-check` PASS after the pin refresh below | pass |
| G2 classification | unit `pond_classifies_swimming_on_the_floor_and_wading_on_the_top_step`: floor `Swimming` (depth `1.4 m`, the pond's id), top step `Wading`, the ground beside the hole `Dry` | pass |
| G3 geometry | unit `pond_steps_and_plan_fit_the_ground_and_the_other_bodies`: rises of `0.25 m` under the `0.3 m` limit including the last step to the ground, the hole inside the strips, disjoint from the basin and the vessels, the strips solid from `−1.7 m` to `0` | pass |
| G4 session | `--interactive --start-at-pond --maximum-frames 120` on a fresh state root: exit 0, `PASS`; the HUD reads `Swimming` on the capture | pass |
| G5 look (human) | frame 60 (`p32-pond-60.png`, not in Git): the eye `0.73 m` under the level, the pond walls in checker to both sides, the avatar's head and the carried load above the water line, the sky above; no surface is drawn from below and nothing is fogged — the "before" picture of plan 33 | pass, recorded |

Pins refreshed once: in `crates/verification/src/content_package.rs`
root assets `43 → 45`, manifest entries and records `129 → 131`, catalog
and cooked meshes `17 → 19`, rendered objects and indexed draws
`14 → 16`; the same counts in the tests of `platform_check.rs`,
`render_performance.rs`, `crates/project/tests/content_pipeline.rs` (plus
the floor mesh's `16` normals) and
`crates/reference-game/tests/visual_presentation.rs` (scene records
`16 → 18`, bindings `15 → 17`).
New roots: `play` authoritative state root `255dec16218c…`,
`water-present` final state root `0abd67692c22…`, `persistence-replay`
final state root `aba2347c344b…` (correction of 2026-09-04 in plan 33:
the first record of this plan copied the `water-present` root into the
`persistence-replay` line; the value above was read on the same tree by
plan 33's chain, which changes no authority).

Apparatus notes:

- The project's `project_revision` stays `12`: the loader ties it to the
  world topology record's revision (`world_services.rs`), so a bump
  without a topology change is rejected as an invalid value.
- Mesh records must carry no zero normal; the pond interior mesh has no
  normal array (the derivative flat normal, like the rim).
- A surface quad is authored at `y 0` relative to its volume (the
  binding translates it to the level); the first pond quad at the level
  itself put the ring's vertices outside the catalog bounds.
- The camera focus sits `0.7 m` above the capsule centre (`−0.6 m` on the
  pond floor), so a level camera hovers `0.1 m` above the water; the
  start pitches `12°` up to put it `0.73 m` under.
- A state root saved before this change refuses to resume (`session
  recovery is incompatible`): the content lock moved. Sessions in this
  plan ran on a fresh `--state-root`.
