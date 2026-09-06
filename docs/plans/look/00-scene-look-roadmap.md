# Scene look — the remaining work, ordered by effect per effort

| Field | Value |
| --- | --- |
| Status | `ROADMAP / NOT_FROZEN` (an ordering document; each item is frozen as its own plan with gates before anything runs) |
| Date | 2026-09-05 |
| Parent | task-state [`scene-look.md`](../../development/task-state/scene-look.md); SPEC-04 ([rendering and platform](../../architecture/04-rendering-and-platform.md)), SPEC-30 ([presentation extraction](../../architecture/30-presentation-extraction-and-render-content.md)), SPEC-24 ([neutral asset schemas](../../architecture/24-content-catalog-bundle-and-neutral-asset-schemas.md)); the water look series (plans [`continuum-water/12`](../continuum-water/12-water-look-l1-surface-material.md) to [`18`](../continuum-water/18-water-look-l8-dlss-ready-outputs.md)) as the pattern |
| Purpose | the user's reading of the reference scene on 2026-09-05: "a bare Minecraft". One ordered list of what the renderer and the content lack for a modern look, from the item that changes the impression most per unit of work to the one that changes it least, so the next look plans are picked in that order |

## Why the scene reads as blocks

- **Geometry.** All `24` meshes are script-generated boxes and quads; no
  terrain, no props, no vegetation; the avatar is boxes.
- **Materials.** Flat colours and a checker floor; no normal or
  roughness maps, no tangents in the meshes, although the G-buffer
  already stores roughness.
- **Lighting.** Lambert diffuse plus a constant ambient, one shadow map
  without cascades or a soft filter, a gradient sky, no GGX specular, no
  ambient occlusion, no HDR target, no tone mapping.
- **Post.** No anti-aliasing (TAA), no bloom, no grading, no volumetric
  light; the motion vectors for TAA exist since plan 18.
- **Environment.** `60 x 60 m` floor strips as the ground; fog is the
  only atmosphere.

The checker floor and the hard Lambert carry most of the "blocks"
impression; the water looks better than its surroundings because it is
the only part with physically motivated shading.

## Invariants

Every item is presentation only: no state root, checkpoint or command
changes; the frame plan roots may move only where a plan says so.
Evidence per plan: captures from fixed cameras (`--start-at-*`,
`--capture-frame`), the frame cost at `960 x 540` through the
render-performance check, `host-check`, and the B0 fallback path kept
(a feature that cannot be created falls back to the current look and
prints its reason once, as the water pass does).

## The list

| # | Item | What | Scale | Depends on |
| --- | --- | --- | --- | --- |
| 1 | HDR chain and physical lighting | an `R16G16B16A16` scene target; GGX specular from the G-buffer's roughness; an analytic sky (Hosek-Wilkie) whose spherical-harmonic ambient replaces the constant; exposure and ACES tone mapping; sRGB-correct output | S-M | — |
| 2 | Shadows | cascaded shadow maps for the sun, PCF or PCSS soft edges, a contact shadow; the planar reflection of the water uses the same map | S | 1 (the exposure sets how dark a shadow reads) |
| 3 | Ambient occlusion | GTAO from the G-buffer's depth and normals, applied to the ambient term | S-M | 1 |
| 4 | Temporal anti-aliasing | TAA on the plan 18 motion vectors (jitter, history clamp), DLSS where available; removes the edge shimmer and the far-water flicker | M | 1 |
| 5 | PBR materials | triplanar mapping for the generated boxes first (no UV work), then albedo, normal and roughness maps through SPEC-24 with tangents in the mesh records | M | 1, 4 (normal maps alias without TAA) |
| 6 | Environment | a height-field terrain with splat materials in place of the floor strips; imported prop meshes; vegetation (SPEC-40); a real skinned avatar | L | 5 |
| 7 | Post | bloom, volumetric light shafts and height fog, a colour-grading LUT | S-M | 1, 4 |

Items 1 to 4 touch only the desktop adapter and its shaders and need no
content; together they are about the size of plans 32 to 39 of the
water series. Item 6 is where the real cost sits and looks best once 1
to 5 are in.

## Engine work and scene work (2026-09-05)

The material contract is already PBR-shaped (base colour, metallic,
roughness, emissive, normal scale, occlusion, texture bindings) and the
mesh record carries tangents; what is missing is on the engine side
first, the scene second.

| Subsystem | Needed | Items |
| --- | --- | --- |
| Desktop adapter and shaders | done in plans [`01`](01-hdr-chain-and-physical-lighting.md) to [`04`](04-temporal-anti-aliasing.md) and [`07`](07-post-chain.md): HDR target and tone mapping; GGX from roughness; analytic sky with SH ambient; cascaded shadows with PCF; GTAO; TAA with jitter; bloom, volumetric height fog with shadowed in-scattering, grading LUT | 1, 2, 3, 4, 7 |
| B0 content profile | done in plans [`05`](05-textured-materials.md) and [`06a`](06a-terrain-presentation.md): mip chains, linear normal and roughness maps, texture arrays with a splat control slot (`b0.v7`); open: BC5/BC7 compression, emissive and occlusion maps | 5, 6 |
| Asset import | done in plan [`05b`](05b-gltf-import.md): the `mesh-gltf` record and `xtask gltf-scaffold` bring glTF meshes (with tangents), materials and PNG textures into the neutral schema; open there: compressed extensions, embedded and JPEG images, skins, the `16 MiB` catalog decode limit | 5, 6 |
| World systems | the height-field terrain's presentation (the `mesh-heightfield` record and the splat material) is done in plan [`06a`](06a-terrain-presentation.md); open: its physics collider (SPEC-26 `HeightField`, a state change), vegetation per SPEC-40, local lights (only the sun exists), frustum culling and LOD (everything draws today) | 6 |
| Checks | the render-performance budget at `960 x 540` for each new pass; fixed-camera captures as evidence; the fallback path when a feature cannot be created | all |

Scene work proper is item 6 and half of item 5: terrain authoring,
props, vegetation, a real avatar, textures for the existing boxes. It
waits on the importer and the fuller B0 profile.

## Technology verdicts (2026-09-06)

[The modern engine technology research](../../reviews/modern-engine-technology-research-2026-09-06.md)
records, per technology, whether it is implemented, deferred or left out
under this roadmap's invariants, the implementation shape each one takes
(adapter pass, content and profile revision, offline tool, authority
series) and a proposed order after the terrain: instanced foliage, the
skinned avatar, block compression, culling and LOD, a render graph,
local lights with baked probes, then the small lighting plans.

## How to proceed

Freeze item 1 as `look/01-hdr-chain-and-physical-lighting.md` with
gates (captures at the spawn, the lake and the falls; the frame cost;
roots unchanged; the fallback) before anything runs; then 2, 3, 4 in
order; re-read this list after item 4 with fresh captures before
committing to the content work of items 5 and 6.
