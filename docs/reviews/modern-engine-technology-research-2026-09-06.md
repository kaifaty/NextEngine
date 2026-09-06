# Modern engine technology: what to implement, what to leave out, and how

Date: **2026-09-06**. Checkpoint: `20bcbf2c` (plan `look/06a`). A research
report on the presentation and content side of the engine; it changes no
Accepted contract, roadmap status or chosen technology. It complements the
[architecture feasibility review](architecture-feasibility-and-technology-review-2026-09-05.md),
which covers the authority side (command archive, PhysX continuation,
cognition, motor learning) and is not repeated here.

## 1. Where the engine stands

The scene look series (`docs/plans/look/`, items 1 to 5 and 7 plus the
terrain of item 6) took the desktop adapter from Lambert light over flat
boxes to: an HDR chain with a middle-grey exposure and the ACES curve, GGX
with metallic-roughness, a Preetham sky projected to SH2 irradiance,
three cascaded shadow maps with PCF, GTAO at half resolution, TAA on the
plan 18 jitter, mip-mapped PBR maps with a screen-space cotangent frame,
a glTF importer into the neutral schema, a post chain (bloom, a
shadow-marched height fog, a grading LUT), and a height-field terrain
under a four-layer splat material with texture arrays. Everything drawn
goes through one forward world program over a G-buffer prepass; there is
no culling, no LOD, no local light, no texture compression and no
skinned imported avatar yet.

The constraints that decide every verdict below:

- **Presentation never owns state.** Render quality, culling and effects
  must not change commands, events, replay or authoritative roots
  ([SPEC-04](../architecture/04-rendering-and-platform.md) "Technical
  authority boundary"). Anything that changes gameplay truth (a physics
  height field, destructible vegetation) is a state series with its own
  roots, not a look plan.
- **One shipping target.** Linux x86_64 with a production NVIDIA Vulkan
  adapter ([ADR-090](../architecture/adr/090-linux-only-v1-and-indefinitely-deferred-windows.md),
  [ADR-091](../architecture/adr/091-linux-release-performance-authority.md)).
  There is no second backend to keep portable, but every optional tier
  needs the B0 fallback: `RENDER-02` forces no RT and no mesh shaders and
  the scene must stay complete.
- **Capability tiers exist already.** B0 (Vulkan 1.3, dynamic rendering,
  synchronization2, indirect indexed draws), E1 (descriptor indexing,
  draw-indirect-count, mesh shaders), E2 (ray query). The cooker already
  emits meshlets next to conventional geometry.
- **No vendor types in contracts.** Vulkan, SPIR-V and any SDK type stay
  inside the adapter; a vendor SDK that must link into the shipped binary
  is a product decision, not a renderer detail.
- **A measured frame budget.** `1920 x 1080`: `p95 <= 14 ms`,
  `p99 <= 16.7 ms` on the reference RTX 3080 host; the look series spends
  about `0.55 ms` at `960 x 540` today. Every feature is judged by a
  same-session release A/B (plan 05b's apparatus finding).
- **One maintainer.** A feature is worth its upkeep only when its
  fallback is the current path and its evidence is a fixed-camera capture
  plus a cost ratio; no feature may require a second toolchain to stay
  alive.

## 2. The decision rule

Implement a technology when all four hold: it fits a declared tier with
the B0 path as its fallback; it is presentation-only (an adapter pass) or
content-only (a SPEC-24 revision with roots re-pinned); its benefit is
visible from the three fixed cameras of the reference scene; one person
can own it with the existing shader toolchain and the plan-with-gates
process. Defer it when a prerequisite is missing (local lights before
clustered shading, an avatar mesh before skinning work). Leave it out when
it needs a vendor runtime in the product, a second geometry or lighting
representation to maintain, or a research budget with no consumer.

Four implementation shapes recur, and every "how" below names one:

| Shape | Contract | Precedent |
| --- | --- | --- |
| **A. Adapter pass** | none; a suite in the shader manifest, a boxed optional pass with a printed fallback | plans `look/02`, `03`, `04`, `07` |
| **B. Content and profile** | a SPEC-24 revision, `b0.vN`, roots move and are re-pinned | plans `look/05`, `06a` |
| **C. Offline cooker or tool** | project-crate code and a manifest record; the runtime sees neutral records only | plan `look/05b` (glTF), `tools/textures/generate.py` |
| **D. Authority series** | a SPEC-26 or SPEC-40 vertical with state roots, checkpoints and product checks | continuum-water plans, vegetation physics |

## 3. Verdicts

### 3.1 Geometry and submission

| Technology | Verdict | How, or why not |
| --- | --- | --- |
| Frustum and occlusion culling (Hi-Z from the prepass depth) | **Implement, next** | Shape A on B0: a compute pass builds the visible list into the indirect buffer SPEC-04 already requires; occlusion from a Hi-Z pyramid of the previous frame's depth with a reprojection test. The roadmap notes "everything draws today". Evidence: draw counts and the cost ratio at the lake camera. |
| Mesh LOD | **Implement, with the importer** | Shape B and C: the `mesh-gltf` record gains `lod` levels (the importer takes glTF node names or a decimation pass in the project crate); the neutral mesh record already allows several primitives; the adapter picks a level by screen size in the same culling pass. |
| GPU-driven indirect count and meshlet culling | **Implement on E1, after culling** | Shape A: draw-indirect-count and per-meshlet cone and bounds culling; the cooker already produces meshlets (`B0CookedMeshV1`). Task and mesh shaders stay optional: enable only when a measured scene exceeds the vertex path, keep the cooked B0 pipeline as fallback (SPEC-04 already mandates this). |
| Virtualized geometry (Nanite-style clusters, software rasterization) | **Do not implement** | A second geometry representation, a streaming format and a software rasterizer to maintain; the reference scene is under a million triangles and the terrain is one mesh. LOD plus meshlets plus culling reaches the same frame budget at this scale. |
| Bindless materials (descriptor indexing) | **Later, with material count** | SPEC-04 defines the logical model and the bounded fallback. Today thirteen material sets are fine; move to one descriptor array when materials exceed a few hundred or when decals and instanced foliage need per-instance materials. |
| Instanced foliage (grass, bushes) with wind | **Implement, next of item 6** | Shape A plus C: a `foliage-scatter` cooker step samples the splat control map (grass weight) into instance transforms; instanced indirect draws from the culling pass; vertex-shader wind from the lighting block; alpha-tested cards through the prepass. Presentation only; SPEC-40 (destructible trees) is the separate authority track. |
| Decals | **Later** | Shape A: deferred decals over the prepass (albedo and normal), for wear on the concrete and the paths; cheap once the G-buffer holds what they modify. |

### 3.2 Lighting and shading

| Technology | Verdict | How, or why not |
| --- | --- | --- |
| Local lights (point, spot) with clustered forward shading | **Implement, prerequisite for interiors and night** | Shape B and A: a neutral light record (SPEC-24 revision) placed like presentation bindings; a compute pass bins lights into a froxel cluster grid; the world programs loop the cluster's lights. Shadows for a bounded number of lights from a shadow atlas. The current sun-only block stays the fallback. |
| Deferred shading | **Do not switch** | The forward path with a thin prepass already feeds AO, TAA, fog and future decals; clustered forward gives local lights without a fat G-buffer and keeps the material variants in one program. |
| Global illumination | **Later: baked probes; never Lumen-style** | Shape C and A: irradiance probes (SH2) baked at cook time from the sky and the sun over the static scene into a probe-volume record, sampled by the world programs in place of the single sky SH. A dynamic DDGI update on E2 ray query can follow for time of day. Screen-space or SDF-traced GI (Lumen, SDFGI) is a second scene representation and a maintenance program on its own. |
| Screen-space reflections | **Implement, small** | Shape A: a ray march over the prepass depth with the resolved previous frame, fallback to the sky SH; the planar reflection stays for the water. |
| Ray-traced shadows or reflections (E2) | **Later, optional** | Only as a replacement of an existing effect (the sun's contact shadows or the water reflection) with the raster path as fallback; the target GPU allows it, the scene does not need it yet. Ray-traced GI: no (see above). |
| Virtual shadow maps | **Do not implement** | Three cascades at `2048²` with PCF fit the budget; PCSS or contact shadows (a short screen-space ray toward the sun) close the remaining softness at a fraction of the cost. |
| Atmosphere: Hosek-Wilkie or precomputed scattering with aerial perspective | **Later, with time of day** | Shape A: replace the Preetham evaluation by a precomputed transmittance and scattering LUT (Bruneton) so the fog's in-scatter and the sky share one model; only worth it when the sun moves. |
| Materials: emissive and occlusion maps, clear coat, anisotropy | **Emissive and occlusion: implement small; the rest: no** | Shape B (`b0.v8`): the two slots exist in the neutral material already; the shading reads them. Clear coat and anisotropic BRDFs have no consumer in the scene. |
| Triplanar mapping for the terrain's slopes | **Implement, small** | Shape A: a triplanar branch of the splat sampling weighted by the normal; the rim's rock stretches today. |
| Block compression (BC7, BC5, BC4) | **Implement, offline** | Shape C and B: an encoder in the project crate (a reasonable-quality BC7 encoder is a few hundred lines; BC5 for normals, BC4 for masks), the texture record's `texel_encoding` gains the block formats (SPEC-24), the adapter uploads them as is. Cuts VRAM and bandwidth by four to six and lets the maps return to `512²` and above. |
| Virtual texturing | **Do not implement** | A page cache, feedback pass and cooker on top of streaming that the world does not need at this size; BC plus mips plus chunk-level texture streaming (SPEC-03) suffice. |

### 3.3 Anti-aliasing, resolution and post

| Technology | Verdict | How, or why not |
| --- | --- | --- |
| TAA | **Done** | Keep; add a sharpening pass only if captures read soft after the foliage lands. |
| DLSS | **Do not ship** | A vendor runtime in the product and a device-specific path with no B0 equivalent; the adapter already exposes DLSS-ready inputs (jitter, motion, depth) so an experiment costs little if it is ever wanted. |
| FSR 3 upscaling (shader-based, open source) | **Later, if the budget asks** | Shape A: a temporal upscaler in place of the TAA resolve when a scene misses `p95` at `1080p`; today the frame sits far under budget. Frame generation: no (latency and a second interpolation pipeline). |
| Auto-exposure | **Implement, small** | Shape A: a luminance histogram over the resolved frame, an exposure that adapts within bounds; the middle-grey rule of plan 01 becomes the target instead of the value. |
| Depth of field, motion blur, lens dirt and flares | **Do not implement now** | Taste features with real cost in TAA interaction; revisit for cinematics only. |
| HDR display output | **Later** | Needs an HDR swapchain through SDL and a second tone map; no display in the reference cohort. |

### 3.4 Characters and animation

| Technology | Verdict | How, or why not |
| --- | --- | --- |
| A real skinned avatar through the importer | **Implement, next of item 6** | Shape C and B: the glTF importer gains skins (joints, weights, inverse binds) into the base skinning profile that already exists (`NeutralBaseSkinningProfileV1`, the humanoid catalog); the neutral skeleton must match the engine's body schema. Content over engine work. |
| GPU skinning (compute) | **Later** | The current CPU skinning stream is fine for a few characters; move it to compute when crowds exist. |
| Motion matching, learned animation | **Do not implement** | The engine's animation track is the deterministic skeletal pipeline plus the physical motor policies ([SPEC-28](../architecture/28-skeletal-animation-retargeting-and-ik.md), [SPEC-14](../architecture/14-physical-archetypes-motor-skills-and-policy-lifecycle.md)); a motion database is a third animation system. |
| Hair, subsurface skin | **Do not implement** | No consumer; the avatar is a proof of the skinning path, not a character showcase. |

### 3.5 World, physics and effects

| Technology | Verdict | How, or why not |
| --- | --- | --- |
| Physics height field | **Later, shape D** | SPEC-26 names the geometry; the collision asset and the reference solver plus PhysX parity are missing. Needed once gameplay leaves the flat play area; a state series with its own roots. |
| Destructible vegetation | **Follow SPEC-40 as written** | The Proposed track has its gates and calibration blockers; nothing in the look series pre-empts it. |
| GPU particle systems (smoke, dust, sparks) | **Later** | Shape A: the water lane's particle surface already owns a compute path; generalise it to a presentation-only emitter record when an effect needs it. |
| Water: SSR on the surface, caustics, foam masks | **Later, small** | Shape A over the existing ring grids; the reflection pass stays. |
| Weather and time of day | **Later, with the atmosphere** | The sun direction is a constant today; a presentation-only clock driving the lighting block is cheap once the sky model handles any elevation. |

### 3.6 Tooling and the adapter itself

| Technology | Verdict | How, or why not |
| --- | --- | --- |
| A render graph (the `RenderGraph` SPEC-04 names) | **Implement as a refactor, before local lights** | The adapter's passes hand-write barriers and layouts (plans 01 to 07 each added their own); a small graph that declares reads and writes and derives transitions removes the most error-prone code and makes cluster, probe and foliage passes cheap to add. Internal; no contract. |
| Shader hot reload and RenderDoc markers | **Implement, small** | Debug names on images and passes and a development-only reload of the checked-in SPIR-V; the pins stay the release truth. |
| An in-engine editor | **Do not implement** | Authoring is JSON plus generators plus the importer by design (ADR-083 to 085 give the creator CLI); an editor is a product on its own. |
| A second graphics backend (DX12, Metal, WebGPU) | **Do not implement** | ADR-090 and ADR-001 settle the target; the engine-owned render API keeps the door open without paying for it now. |
| Neural rendering (neural materials, neural radiance caches, neural upscaling) | **Do not implement** | Research with vendor dependencies and no B0 path; the engine's ML budget belongs to the motor and cognition tracks. |

## 4. A proposed order after the terrain

Not a plan; an ordering by benefit per unit of work under the rule above.

1. **Instanced foliage with wind** (A, C): the largest visible change to an
   outdoor scene; cooked from the splat control map.
2. **The skinned avatar through the importer** (B, C): the boxes are the
   last unmodern object in every capture.
3. **Block compression** (B, C): removes the texture budget that has
   already shaped two plans.
4. **Culling and LOD** (A, B): the frame budget headroom for everything
   after; the roadmap's "everything draws today".
5. **A render graph refactor** (internal): before the passes multiply.
6. **Local lights with clustered shading, then baked probes** (B, A):
   interiors, night and time of day become possible.
7. **Atmosphere, auto-exposure, SSR, triplanar, emissive and occlusion
   maps, decals, PCSS** (A, B): small plans, each one capture and a ratio.

Left out by decision, not by delay: virtualized geometry, Lumen-style or
SDF GI, virtual shadow maps, virtual texturing, DLSS and frame generation,
motion matching, neural rendering, an editor, a second backend.

## 5. Sources

- [SPEC-04](../architecture/04-rendering-and-platform.md) capability tiers,
  submission rules, the B0 performance profile; [SPEC-24](../architecture/24-content-catalog-bundle-and-neutral-asset-schemas.md)
  neutral records; [SPEC-26](../architecture/26-physics-world-collision-constraints-queries-and-canonical-snapshots.md)
  `HeightField`; [SPEC-40](../architecture/40-structural-vegetation-physics.md).
- [ADR-090](../architecture/adr/090-linux-only-v1-and-indefinitely-deferred-windows.md),
  [ADR-091](../architecture/adr/091-linux-release-performance-authority.md),
  [ADR-001](../architecture/adr/001-product-repository-license-and-platforms.md).
- The scene look roadmap and plans [`00`](../plans/look/00-scene-look-roadmap.md)
  to [`07`](../plans/look/07-post-chain.md) with their recorded costs.
- Public method references: Jimenez 2014 (bloom), Jimenez et al. 2016
  (GTAO), Karis 2014 (PBR and TAA), Bruneton and Neyret 2008 (atmosphere),
  Majercik et al. 2019 (DDGI), Haar and Aaltonen 2015 (GPU-driven
  rendering), Olsson et al. 2012 (clustered shading).
