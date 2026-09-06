# Task state: scene look (the reference scene's modern look)

| Field | Value |
| --- | --- |
| Task | Bring the reference scene from script-generated boxes under Lambert light to a modern look, presentation only |
| Status | `ACTIVE / ENGINE_FIRST (D-L01) / PLANS_01_TO_05B_07_06A_DONE / ITEM_6_PROPS_VEGETATION_AVATAR_NEXT` |
| Branch | `codex/water-research` |
| Last updated | 2026-09-06 |

## Resume in 60 seconds

- **The roadmap** is [`docs/plans/look/00-scene-look-roadmap.md`](../../plans/look/00-scene-look-roadmap.md):
  seven items ordered by effect per unit of work (HDR chain and physical
  lighting; shadows; ambient occlusion; TAA; PBR materials; environment;
  post). Items 1 to 4 are adapter and shader work without content.
- **Plan 01 done (2026-09-05):** the HDR chain (`R16G16B16A16_SFLOAT`
  scene target, ACES tone map), the `LightingUniforms` block (set 0,
  binding 1) from the Preetham sky (SH2 irradiance, horizon fog, middle-grey
  exposure), `sky_analytic`, GGX with the catalog's metallic and roughness,
  the interface contract at `b0.v3`. Gates G2-G6 pass; G1 fails as frozen: the contract hash rides the render content profile into the project lock, so every root moved (re-pinned, creator-smoke scenario refreshed); GPU frame time
  `1.10 x`. Finding: the reference materials' albedos are `0.08..0.12`,
  so the calibrated scene reads dark until item 5 relights the content.
- **Plan 01a done (2026-09-05, user: a content pass before item 2):** the
  reference materials' albedos raised to `0.2..0.5` (two textures scaled
  `1.6x`, the factors and roughness per material, the blade metallic); the
  B0 profile now admits metallic and roughness. Roots re-pinned in the plan.
- **Plan 02 done (2026-09-05):** three shadow cascades (`12 / 36 / 108 m`)
  in a `2048²` depth array with linear compare taps, a normal-offset
  receiver and a shared `sun_visibility`; the contract at `b0.v4`; GPU
  frame time `1.20 x`. Roots re-pinned in the plan.
- **Plan 03 done (2026-09-05, revision 1):** GTAO at half resolution over
  the G-buffer prepass (now before every scene pass), a depth-aware blur,
  the world material scaling its sky irradiance by it; the contract at
  `b0.v5`; GPU frame time `1.30 x` after the half-resolution revision
  (full resolution read `1.7 x`, recorded); the frozen contact-band metric
  at the pad's base read `4.9 %` (fail as frozen), the avatars' contact
  `33 %`.
- **Plan 04 done (2026-09-05):** TAA on the plan 18 jitter and motion
  vectors (variance clipping, history `0.9`, the sky through the previous
  view-projection), `--no-temporal-aa`; no contract change, roots unchanged;
  GPU frame time `1.10 x`; frame-to-frame difference `8.6 %` of the
  jitter-only value.
- **Plan 05 done (2026-09-05, item 5 first plan):** textured materials:
  the B0 profile admits mip chains, linear maps and three bindings under a
  uniform UV scale; `texture-png` sources with a PNG decoder and box-filter
  mips; normal and metallic-roughness maps in the world programs (cotangent
  frame); the contract at `b0.v6`; procedural ground, concrete and pad sets
  (`tools/textures/generate.py`); the works mesh with box-projected UVs.
  Open: BC compression, anisotropy, the `8 MiB` catalog field limit, the
  glTF importer (next plan of item 5).
- **Plan 05b done (2026-09-05, item 5 second plan):** the glTF importer:
  the `mesh-gltf` authoring record (JSON and GLB, buffers and images as
  declared referenced sources, the default scene's node transform baked,
  normals, tangents and UVs into the neutral mesh), `xtask gltf-scaffold`
  (sources, `texture-png`, `material` and `mesh-gltf` records with
  sequential ids, a placements table), the Khronos `BoxTextured` sample
  validated in a scratch project, and a synthetic riveted water tank
  (`tools/gltf/generate.py`) in the reference scene west of the pond
  (seven static bindings `0x65`-`0x6b`). No contract change. The cost
  gate fails as frozen because the machine's GPU means moved by an order
  of magnitude between sessions; the same-session A/B reads `1.04-1.10 x`.
  Open: the `16 MiB` catalog decode limit for imported textures, JPEG
  and embedded images, compressed extensions, skins.
- **Plan 07 done (2026-09-06, item 7, three revisions):** the post chain
  in place of the plan 01 resolve: a five-mip bloom (Jimenez downsample
  with the Karis average, tent upsample, `0.04` mix), an exponential
  height fog marched at half resolution through the cascades with the
  sky's radiance along the ray partitioned into a shadowed sun share,
  exposure, ACES and a `32³` grading LUT; `--no-post`. Revisions: the
  grade's endpoints, the in-scatter from the sky model (the frozen sum
  double-counted the aureole and washed the falls view), the march at
  half resolution with one tap (the frozen chain cost `2.2 x`). Cost
  `1.23 / 1.28 / 1.25 x` in a release build (spawn / lake / falls);
  roots unchanged. Open: the shafts do not read at this density.
- **Plan 06a done (2026-09-06, item 6 first plan, revision 1):** the
  terrain presentation: the `mesh-heightfield` record from a 16-bit PNG
  (flat `0.1 m` across the play area, a `2.5 m` rise at the rim, the pond
  hole), the splat material (SPEC-24 3.4: the `SplatControl` slot, B0
  `v7` texture arrays of up to four layers, the fourth weight as the
  remainder), the render catalog decode limits raised to `64 / 48 MiB`
  (the plan 05 `8 MiB` finding closes), the ground as grass, dirt, rock
  and sand layers under a painted control map. Physics unchanged (the
  four floor boxes). Cost `1.20 / 1.17 / 1.19 x` (release A/B). Open:
  the tiling repetition, the pale rim, the physics floor under the rises.
- **Item 5 is done in two plans (05, 05b).** Item 7 is done; item 6 has its terrain (06a). Next: props through
  the importer, vegetation (SPEC-40), a real avatar.
- **Items 1-4 (engine first, D-L01) are done.** Next by the roadmap: item 5
  (PBR materials: mip chains, linear textures, BC compression, normal maps,
  a glTF importer) and item 7 (post: bloom, volumetric fog, grading); item 6
  (environment) after them.
  and 4 (TAA).
- **Invariants.** Presentation only: no state root, checkpoint or
  command changes; captures from fixed cameras, the frame cost at
  `960 x 540`, `host-check`, and a fallback to the current look when a
  feature cannot be created.

## Decisions

- **2026-09-05 (user):** the reference scene reads as "a bare Minecraft";
  the look work is recorded as its own roadmap so it is not lost behind
  the water series. Order of the first plans: 1, 2, 3, 4.

- **2026-09-05 (user, D-L01):** the engine systems come first: items 1 to 4
  (HDR chain and physical lighting, shadows, ambient occlusion, TAA) on
  the current scene before any content work; the scene itself (items 5
  and 6) waits for them.

## Related

- Water look series: task-state [`water-volume-authority.md`](water-volume-authority.md), plans `continuum-water/12` to `18`, `33`, `35`, `42`.
