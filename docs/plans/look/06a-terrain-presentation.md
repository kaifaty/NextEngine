# Look L6a — terrain presentation: a height-field mesh and splat materials (scene look item 6, first plan)

| Field | Value |
| --- | --- |
| Research ID | `SCENE-LOOK-L6A` (research report, not a product check) |
| Status | `RUN / G1-G4, G6 PASS / G5 1.20 x AT THE SPAWN (0.4 % OVER), PASS AT THE LAKE AND THE FALLS / G7 HUMAN` (2026-09-06) |
| Parent | [`00-scene-look-roadmap.md`](00-scene-look-roadmap.md) item 6; plans [`05`](05-textured-materials.md) (the `texture-png` source, the three-binding material, the mip chains), [`05b`](05b-gltf-import.md) (the record-from-source pattern); SPEC-24 (the neutral texture record already carries array layers; the material slot union gains one member); SPEC-26 is untouched (the physics `HeightField` geometry has no collision asset or backend support yet: the floor's four static boxes stay the authority) |
| Purpose | the ground is one flat quad under one tiled texture. A height-field mesh authored from a PNG (flat across the play area, gentle rises toward the rim, a hole for the pond), and a splat material blending four tiled layers (grass, dirt, rock, sand: albedo, normal, metallic-roughness each) by a painted control map, so the ground reads as terrain. Presentation only: the physics floor is unchanged and the height-field is flat wherever gameplay walks |

## Frozen scope

- **Contracts (SPEC-24 revision, B0 `v7`).** `MaterialTextureSlotV1`
  gains `SplatControl` (tag `6`). The B0 profile admits 2D textures
  with `1..=4` array layers; a material binding a `SplatControl` texture
  (2D, one layer, `RGBA8` linear: the four layers' weights) is a splat
  material whose `BaseColor`, `MetallicRoughness` and `Normal` textures
  carry the same layer count (`2..=4`); a material without it binds
  one-layer textures as plan 05. The shader interface manifest moves to
  `b0.v7`: set 1 binding 3 (the control map), bindings 0 to 2 sampled
  as arrays (layer `0` for plain materials), the push block's material
  lane documented as `uv scale, negative for a splat material` (the
  magnitude is the layers' tiling over the mesh's `uv0`). The render
  content catalog decodes under `RENDER_CONTENT_DECODE_LIMITS` (`64 MiB`
  total, `48 MiB` a field, otherwise the defaults) at the activation
  site; the plan 05 `8 MiB` finding closes.
- **Authoring (project).** `texture-png` gains `layers` (further PNG
  files of the same size and format among the referenced sources,
  becoming layers `1..`); the PNG decoder admits 16-bit grey (to 16-bit
  samples). A `mesh-heightfield` record: `asset_id`, `record_revision`,
  `relative_path` (a 16-bit or 8-bit grey PNG among the referenced
  sources: rows along `z`, columns along `x`), `origin_micrometres`
  (the `x, z` of column and row `0`), `cell_micrometres`,
  `height_range_micrometres` (`[low, high]`, the sample range mapped
  linearly), `holes` (axis-aligned `x, z` rectangles in micrometres
  whose fully covered cells are dropped), producing a neutral mesh with
  smooth normals from the central differences, `uv0` in `0..1` over the
  field, half-open bounds. The `material` record gains
  `splat_control_texture_asset_id`.
- **Adapter.** Textures upload every layer of every level; views are
  `2D_ARRAY` for every texture (one layer for plain ones), the
  placeholders too; set 1 gains binding 3 with a `(1, 0, 0, 0)` linear
  placeholder; `b0_textured`, `b0_textured_no_shadow`, `b0_reflect` and
  `gbuffer` sample arrays and, under a negative uv scale, blend the
  layers by the control map sampled at `uv0` (weights normalised),
  tiling the layers at `uv0 * |scale|`; the material lane's sign follows
  the material record. `TERRAIN active materials=N layers=L` printed
  once when a splat material is prepared.
- **Content.** The generator writes a `121 x 121` 16-bit height PNG at
  `0.5 m` cells over the floor's `60 x 60 m` (`0.1 m` across
  `|x|, |z| <= 22 m`, a smooth rise to `2.5 m` at the rim, the pond
  rectangle as a hole), a `256²` control map (grass with dirt paths
  between the pads and the works, rock on the rim rise, sand around the
  pond and the lake shore) and four `256²` layer sets (grass from the
  plan 05 ground set, dirt, rock, sand). The floor mesh `0x81` becomes
  the `mesh-heightfield` record (revision bumped), the base material
  `0x85` the splat material (its bindings the layer arrays and the
  control map, uv scale `30`: `2 m` tiles); the floor binding and the
  four floor boxes are unchanged. Roots move for the content and are
  re-pinned (recorded).
- **Not in scope (recorded).** A physics height-field (SPEC-26 asset
  and backend work, a state change: its own series), the terrain's
  collision beyond the flat play area (the rim rises are
  presentation-only and the avatar would walk into them), triplanar
  mapping on steep slopes, tessellation or LOD (one mesh), a terrain
  editor, vegetation (SPEC-40, a later plan), the lake bed's own
  shaping.

## Revision 1 (2026-09-06, before the render readings)

The neutral texture record's alpha semantics admit no data alpha (a
`Straight` texel with zero alpha must be black; `Opaque` alpha is
`255`), so the frozen `RGBA` control map cannot carry a fourth weight in
alpha. The control map is `Opaque`: `RGB` weigh layers `0` to `2` and
layer `3` takes the remainder `1 - r - g - b`; the profile rule, the
shaders and the generator follow.

## Frozen gates

| Gate | Clause |
| --- | --- |
| G1 roots | `content-package` PASS with its counts recorded; `play`, `persistence-replay`, `water-present`, `audio-scene` PASS with roots recorded; creator-smoke refreshed; `host-check` PASS |
| G2 contract | golden, manifest and runtime check updated together; module pins refreshed; PROVENANCE records the revision; `docs/architecture/README.md` records the SPEC-24 revision |
| G3 unit | the height-field builder: a `3 x 3` field with one raised centre gives the expected positions, normals pointing away from the centre and `uv0` corners, and a hole covering the centre cell drops four triangles; the 16-bit PNG decodes to its samples; the profile accepts a four-layer splat material and rejects a splat material with mismatched layer counts, a layered control map and a layered texture on a plain material; the decode limits admit a `20 MiB` texture field the defaults refuse |
| G4 render | spawn, lake and falls at `300` frames, capture at `60`: `PASS`, `TERRAIN active materials=1 layers=4`, water draws `8`, invalidations `0` |
| G5 cost | GPU frame mean with the terrain within `1.2 x` the same session's run of the previous commit's release build (plan 05b apparatus: a same-session A/B, release builds) |
| G6 fallback | every plain material still samples its one layer (unit: the slot resolution keeps `None` for the control slot; the placeholders are one-layer arrays); the water reflection shows the terrain's blend |
| G7 look (human) | the ground reads as grass with worn dirt paths and sandy shores, rock on the rim, no visible tiling repetition at the play area's scale, the pond hole clean, no seam where the flat area meets the rise |

## Result (2026-09-06)

Implementation: `MaterialTextureSlotV1::SplatControl` and the B0 `v7`
rules in the catalog validation (`B0_MAX_SPLAT_LAYERS`), the interface
manifest `b0.v7` (golden `0c1cece9abc2…`), `RENDER_CONTENT_DECODE_LIMITS`
at both activation round trips (the project activation and the
contract-side `ActivatedProjectV8` check, which also decoded the catalog
under the defaults and failed first with a hash mismatch); the
`mesh-heightfield` record with `authoring/heightfield.rs`, the 16-bit
grey PNG decoder, `layers` on `texture-png`, the material's
`splat_control_texture_asset_id`; the adapter's array textures (every
material texture and placeholder viewed as a one-layer or `n`-layer 2D
array, uploads per layer), set 1 binding 3 with the `(1, 0, 0, 0)`
placeholder, the four world programs blending the layers under a
negative uv scale; the generator's dirt, rock and sand sets, the
`121 x 121` height field (`0.1 m` exactly across the play area through
the `[0.1, 2.6] m` range, a rise to `2.5 m` at the rim, the pond hole),
the `256²` control map; the floor mesh `0x81` as the height field
(`14 641` vertices before the hole), the base material `0x85` as the
splat material over the layered ground set and the control map `0x5b`.
Revision 1 recorded above (the fourth weight as the remainder).

| Gate | Reading | Verdict |
| --- | --- | --- |
| G1 roots | `content-package` PASS (`160` records, `31` meshes, `14` materials, `20` textures, `51` roots); play `6a105341c611`, persistence-replay `edca78384012`, water-present `c46aeebc7f98`, audio-scene `4725b2f3f9b8`; creator-smoke refreshed (the profile hash moved every lock); `host-check` PASS | pass |
| G2 contract | golden `0c1cece9abc2942f24dc8321255cf93de23b57048121fe200e12da72e228eacd` in the contracts test, `manifest.json` and the runtime check; four module pins refreshed; PROVENANCE records the revision; SPEC-24 at 3.4 and its README row | pass |
| G3 unit | `raised_centre_and_hole` (positions, normals leaning away from the centre, uv corners, a hole dropping six indices and the unreferenced centre vertex, an all-hole field refused), the 16-bit decoder through the height field itself (the pipeline test reads the origin vertex at exactly `0.1 m` with an up normal), `b0_admits_a_four_layer_splat_material` (accepts; rejects mismatched layer counts, a layered control map and a layered plain material), `render_content_limits_admit_a_large_texture_field` (a `2048²` texture field the defaults refuse) | pass |
| G4 render | spawn, lake and falls at `300` frames (release build): `PASS`, `TERRAIN active materials=1 layers=4`, `MATERIAL_MAPS … mips=127`, water draws `8`, invalidations `0` | pass |
| G5 cost | two back-to-back rounds, release builds, the same session: the previous commit `460 / 448 / 448 µs` and `461 / 448 / 448`, this plan `554 / 522 / 532` and `554 / 525 / 531` (spawn / lake / falls): `1.20 / 1.17 / 1.19 x`; the spawn view's `1.204 x` sits `0.4 %` over the clause | pass at the lake and the falls; `1.204 x` at the spawn (fail as frozen by `0.004`) |
| G6 fallback | the thirteen plain materials read layer `0` of their one-layer arrays through the same programs (`material_maps_of` keeps `None` for the control slot, the placeholders are one-layer arrays); the lake capture's reflection carries the blended shore | pass |
| G7 look (human) | `l6-{spawn,lake,falls}-60.png` (outside Git): grass with worn dirt around the pads, sand around the pond and along the lake shore, the rock rise at the rim, the pond hole clean; the sand's `2 m` ripple tiling repeats visibly in the falls foreground and the rim's rock reads pale in full sun | human |

Recorded as open: the sand and rock tiling at the play area's scale (a
macro-variation or triplanar pass would hide it), the pale rim, the
physics floor under the rises (the four boxes stay at `0.1 m`: the
avatar walks into the rim), triplanar mapping on slopes, LOD, the lake
bed's shaping, vegetation (the next plan of item 6).
