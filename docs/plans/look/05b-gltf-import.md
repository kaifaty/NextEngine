# Look L5b — glTF import into the neutral schema (scene look item 5, second plan)

| Field | Value |
| --- | --- |
| Research ID | `SCENE-LOOK-L5B` (research report, not a product check) |
| Status | `RUN / G1-G5 PASS / G6 FAIL-AS-FROZEN (apparatus) / G7 HUMAN` (2026-09-05) |
| Parent | [`00-scene-look-roadmap.md`](00-scene-look-roadmap.md) item 5; plan [`05`](05-textured-materials.md) (the `texture-png` source record, the three-binding material, the mip chains); SPEC-24 (the neutral mesh record already carries normals, tangents and texcoord sets); SPEC-10 is untouched (it bounds the external Gothic importer over protected data; this plan reads open glTF files the project declares as its own sources) |
| Purpose | the reference scene's meshes are boxes written by hand in the authoring manifest, and no path brings a modelled asset in. A `mesh-gltf` source record decoded at build time from a declared glTF 2.0 file (positions, normals, tangents, UVs, indices, the node transform baked), a scaffold command that writes the authoring records for a glTF's images, materials and primitives, and one modelled asset in the reference scene to prove the path |

## Frozen scope

- **Authoring (project).** A `mesh-gltf` record: `asset_id`,
  `record_revision`, `relative_path` (a declared referenced source of the
  project, `.gltf` or `.glb`, so its bytes and license ride the
  composition lock), `mesh` (the glTF mesh index), `primitive` (the
  primitive index), optional `node` (a node index of the default scene
  whose global transform, through its parents, is baked into the
  vertices), `double_sided` (default `false`, the same index doubling as
  the `mesh` record). The loader reads glTF 2.0 JSON with external
  buffers among the declared referenced sources or `data:` base64 URIs,
  and the GLB container (`JSON` and `BIN` chunks); accessors `POSITION`
  (`float3`), `NORMAL` (`float3`), `TANGENT` (`float4`), `TEXCOORD_0`
  (`float2`, or normalized `u8`/`u16`), indices `u8`/`u16`/`u32` or a
  non-indexed primitive, `byteStride` honoured, `mode 4` (triangles)
  only. Positions become micrometres (glTF metres, rounded); normals go
  through the inverse transpose of the model's upper `3 x 3` and are
  renormalised to `snorm16`; tangents through the model matrix, the
  handedness flipped under a mirroring transform; UVs to `q16.16`;
  the bounds are the vertices' extent. Refused with a diagnostic:
  `extensionsRequired`, sparse accessors, other topologies, a missing
  `POSITION`, an index beyond the vertex count, a buffer or image file
  that is not a declared referenced source, a mesh or primitive index
  out of range.
- **Scaffold (xtask).** `cargo run -p xtask -- gltf-scaffold --project
  <dir> --source <relative path> --asset-base <hex byte>` prints a JSON
  fragment to paste into the authoring manifest: `referenced_sources`
  entries (the glTF, its external buffers and its images, with their
  `sha256`, the project's license and notice), `texture-png` records
  (base colour `srgb`, metallic-roughness and normal `linear`, `full`
  mips), `material` records (the glTF factors to `u16`, `doubleSided`,
  the map bindings, `uv_scale 1`), and one `mesh-gltf` record per
  primitive of every mesh node of the default scene; asset ids are
  `[base + n; 16]` in that order, and the command refuses a base whose
  ids collide with ids already in the manifest. Images that are not
  external PNG files (JPEG, embedded buffer views) are refused by name.
- **Content.** A checked-in synthetic asset written by
  `tools/gltf/generate.py`: a riveted steel water tank
  (`projects/reference-alpha/assets/models/water_tank.gltf` with its
  `.bin` and three `512 x 512` PNG maps): a smooth-shaded cylinder body
  with a domed lid as a child node under a rotation, four legs and a
  pipe with an elbow, a painted label on the `+x` side whose text reads
  upright so the UV orientation is checked by eye. Declared as
  referenced sources under the project's license; its records come from
  the scaffold; placed by a static environment binding west of the
  stream, south of the dam, in the falls camera's view (presentation
  only; no state root, checkpoint or command change).
- **Adapter.** No change: the neutral tangents are carried by the
  record and not consumed (the cotangent frame stands in, recorded);
  the interface contract stays at `b0.v6`; the roots move for the
  content only.
- **Not in scope (recorded).** JPEG and embedded images, Draco,
  meshopt and quantized extensions, skins, morph targets and animation,
  `KHR_materials_*`, occlusion and emissive textures (the emissive
  factor is carried), `TEXCOORD_1` and vertex colours, cameras and
  lights, the catalog decode limits (`16 MiB` total, `8 MiB` a field:
  the asset's maps stay `512 x 512`), block compression.

## Frozen gates

| Gate | Clause |
| --- | --- |
| G1 roots | `content-package` PASS with its counts recorded; `play`, `persistence-replay`, `water-present`, `audio-scene` PASS with roots recorded; `host-check` PASS |
| G2 unit (loader) | a hand-written glTF with a base64 buffer (one quad: positions, normals, tangents, UVs, `u16` indices) decodes to the expected micrometre positions, `snorm16` normals and `q16.16` UVs; the same content as GLB decodes identically; a node with a translation of `1 m`, a `90°` rotation about `y` and a scale of `2` bakes the positions and the normals; refused: `mode 1`, a missing `POSITION`, `extensionsRequired`, a sparse accessor, an index beyond the vertex count, an undeclared buffer file |
| G3 unit (scaffold) | the tank's scaffold names its texture, material and mesh records with sequential ids and the colour spaces by usage; a colliding base is refused; the printed fragment parses as the authoring records |
| G4 external | the Khronos `BoxTextured` sample (`.gltf`, `.bin`, one PNG) scaffolded into a scratch project passes `project validate` when the sample can be fetched; otherwise recorded as not run |
| G5 render | spawn and falls at `300` frames, capture at `60`: `PASS`, `MATERIAL_MAPS active` counting one more material with a normal map, water draws `8`, invalidations `0` |
| G6 cost | GPU frame mean after within `1.05 x` the before (`375 / 360 µs`, spawn and falls) |
| G7 look (human) | the tank stands on its legs west of the stream with its lid and pipe; the rivets read in the normal map; the label on the `+x` side reads upright; the sun-lit side faces the sun |

## Result (2026-09-05)

Implementation: the `mesh-gltf` authoring record and the glTF reader
(`authoring/gltf.rs`: JSON and GLB, external and `data:` buffers,
strided accessors, the node chain of the default scene baked into
positions, normals through the inverse transpose, tangents with the
handedness flipped under mirroring, `q16.16` UVs, half-open bounds);
the scaffold (`authoring/gltf_scaffold.rs`, `xtask gltf-scaffold`)
printing a parseable JSON document of `referenced_sources`,
`render_records` and `placements` with sequential ids from a base it
checks against the manifest; the generator `tools/gltf/generate.py` and
the water tank (`634` vertices, `760` triangles, seven nodes over four
meshes, three `512 x 512` maps) declared as five referenced sources and
imported through eleven scaffolded records (`0x50`-`0x5a`); seven static
environment bindings (`0x65`-`0x6b`) at `(-11, 0, 9) m`. No adapter or
contract change.

Findings during development, recorded: the neutral bounds are
half-open at their maximum, so the importer adds one micrometre (the
first validation failed with `PositionOutsideBounds`); the generator's
first ring parametrisation ran `u` with the angle, which reads the map
mirrored from outside (a modelling convention, not an importer fault;
the generator now runs `u` against the angle and the label reads left
to right); the counts pinned by the content-package check and the
render-performance test moved (`148 -> 159` records, `24 -> 31` meshes,
`13 -> 14` materials, `16 -> 19` textures, `22 -> 29` rendered objects
and draws, `24 -> 31` scene records and `23 -> 30` bindings in the
visual-presentation test, `22 -> 29` rendered objects in the platform
check) and the package pipeline's frozen source list gains the five
model files under a new `assets/models` directory of the frozen source.

| Gate | Reading | Verdict |
| --- | --- | --- |
| G1 roots | `content-package` PASS (`159` records, `31` meshes, `14` materials, `19` textures, `51` roots); play `99a30ffb261c`, persistence-replay `6eadd81024de`, water-present `cb728b6cd3de`, audio-scene `2809903d4941` (all PASS; the roots move for the content, as every catalog change does); `host-check` PASS | pass |
| G2 unit (loader) | `quad_decodes_from_a_data_uri`, `glb_decodes_identically`, `node_transform_bakes_positions_and_normals` (translation `1 m`, `90°` about `y`, scale `2`), `mirrored_transform_flips_the_tangent_handedness`, `refusals_name_their_reason` (mode `1`, missing `POSITION`, `extensionsRequired`, a sparse accessor, an index beyond the count, an undeclared buffer, a node out of range), `base64_and_percent_decoding`, `sibling_paths_stay_under_the_document_directory` | pass |
| G3 unit (scaffold) | `tank_scaffold_names_its_records_in_order` (three textures `srgb / linear / linear`, one material bound to them, seven `mesh-gltf` records with their nodes, ids `0x10`-`0x1a` in order, seven placements under one material; the fragment parses as JSON), `colliding_base_is_refused` (base `0x50` against the manifest) | pass |
| G4 external | Khronos `BoxTextured` (COLLADA2GLTF export: a `matrix` root node, a child mesh node, one `u16`-indexed primitive, one PNG) scaffolded into a scratch copy of `creator-smoke` (base `0x30` refused for a collision with its records, base `0x40` accepted: one texture, one material, one mesh) and `project validate` PASS | pass |
| G5 render | falls and spawn at `300` frames, capture at `60`: `PASS`, `MATERIAL_MAPS active materials=14 normal=4 metallic_roughness=4 mips=118`, water draws `8`, invalidations `0` | pass |
| G6 cost | the frozen before (`375 / 360 µs`) is not reproducible on this machine today: the HEAD build read `2863 / 3150 µs` (falls / spawn) in the same session. Two back-to-back pairs on the same conditions: `3064 / 3277` and `3108 / 3476 µs` against the HEAD build's `2863 / 3150`: `1.07 / 1.04 x` and `1.09 / 1.10 x`; the run-to-run spread of one binary is `1.5-6 %` | fail as frozen (apparatus); within `1.1 x` by the A/B |
| G7 look (human) | `l5b-falls-60.png`, `l5b-falls-tank.png` (a `4 x` crop, outside Git): the tank on its legs west of the pond with the domed lid, the plate seams and rivet rows from the normal map, the pipe with its elbow toward `+x`, the label on the `+x` side with the `H` at its left edge and the arrow upright; the sun-lit side toward the sun | human |

Recorded as open: the catalog decode limits (`16 MiB` total, `8 MiB` a
field) bound imported textures to about three `1k` maps per catalog;
JPEG and embedded images; Draco, meshopt and quantized extensions;
skins, morph targets and animation; `KHR_materials_*`, occlusion and
emissive textures; the tangent stream is carried by the record but not
consumed by the adapter; the GPU timing apparatus (the recorded means
depend on the machine's state by an order of magnitude, so a plan's
cost gate needs an A/B on the same session rather than a frozen
absolute).
