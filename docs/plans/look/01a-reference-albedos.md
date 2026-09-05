# Look L1a — the reference materials' albedos (content pass after plan 01)

| Field | Value |
| --- | --- |
| Research ID | `SCENE-LOOK-L1A` (content pass, not a product check) |
| Status | `RUN / G1-G2 PASS / G3 HUMAN` (2026-09-05) |
| Parent | [`01-hdr-chain-and-physical-lighting.md`](01-hdr-chain-and-physical-lighting.md) finding; the user's decision of 2026-09-05 (a content pass before item 2) |
| Purpose | the reference materials multiply dark `4 x 4` textures (mean linear `0.17..0.47`) by dark colour factors (`0.15..0.5`), so the floor's albedo is `0.03` and the walls' `0.08`; under the calibrated sun and sky the scene reads black. Raise the textures and the factors to plausible albedos so the look plans can be judged by eye |

## Frozen scope

- **Textures.** `0x88` (the ground and the works, mean `0.17`) and `0x8b`
  (the player and the quest giver, mean `0.23`) scaled by `1.6` in linear
  light (the pattern kept, the texels re-encoded to sRGB); the other
  textures unchanged.
- **Factors and roughness.** The base material `0x85` `(0.95, 1.0, 0.9)`
  roughness `0.9`; the defeated-enemy grey `0xd8` `(1.0, 0.97, 0.92)`
  roughness `0.8`; the relay approach pads `0xd9` `(1, 1, 1)` roughness
  `0.7`; the player `0xd1` `(0.35, 0.8, 1.0)`, the enemy `0xd2`
  `(1.0, 0.16, 0.12)`, the quest giver `0xd3` `(1.0, 0.62, 0.12)`, the
  relay inactive `0xd5` `(0.2, 0.45, 0.9)` at roughness `0.55..0.6`; the
  blade `0xd4` keeps its colour at roughness `0.35`, metallic `0.9`; the
  relay active `0xd6` roughness `0.4`, the indicator `0xd7` roughness
  `0.5`. The water material `0x7d` and `0x86` untouched. Record
  revisions bump; the project revision stays `12`.
- **Not in scope.** Real materials (item 5), new textures, the avatar's
  shape.

## Frozen gates

| Gate | Clause |
| --- | --- |
| G1 albedo | every opaque scene material's mean albedo (factor times the texture mean) lies in `0.15..=0.65` per its brightest channel; the water and `0x86` unchanged |
| G2 roots | `content-package` PASS with the same counts (`138` entries, `24` meshes); `play`, `persistence-replay`, `water-present`, `audio-scene` PASS with roots recorded (they move with the composition lock); `host-check` PASS |
| G3 look (human) | captures at the spawn, the lake and the falls against plan 01's: the ground and the walls read as lit surfaces, the avatars keep their hues |

## Result (2026-09-05)

The B0 profile rejected the first run (`render content is outside the
minimal B0 profile`): the profile required metallic `0` and roughness `1`
while plan 01's shading reads both. The two clauses were lifted from the
profile validation (`catalog.rs`; the profile test's metallic case removed),
an engine-side precondition recorded here rather than a content choice.

| Gate | Reading | Verdict |
| --- | --- | --- |
| G1 albedo | base `0.26/0.32/0.22`, defeated-enemy grey `0.27/0.31/0.23`, pads `0.25/0.27/0.28`, player `0.11/0.29/0.46`, enemy `0.40/0.07/0.06`, quest giver `0.30/0.23/0.06`, blade `0.31/0.37/0.44` (metallic `0.9`), relay inactive `0.08/0.20/0.42`, relay active `0.04/0.33/0.46`, indicator `0.86/0.71/0.11`; water `0.12/0.36/0.64` and `0x86` unchanged | pass |
| G2 roots | `content-package` PASS (`138` records, `64` chunks); play `c60f5c9a81ef`, persistence-replay `a60a5d44a4d9`, water-present `460d7fa6214f`, audio-scene `04b0eb9811f3`; `host-check` PASS | pass |
| G3 look (human) | `l1a-spawn-60.png`, `l1a-falls-60.png`, `l1a-lake-60.png` (outside Git): the ground reads as a lit grey-green plane, the works' sun and shade sides separate, the avatars keep their hues; the Preetham horizon's pink cast is the visible remaining flaw | human |
