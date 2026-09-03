# Research: can DLSS 5 make the water look good, and what does it take

| Field | Value |
| --- | --- |
| Date | 2026-09-03 |
| Question | The basin, the vessels and the crate render as flat blue quads (screenshot of 2026-09-03 12:24). Can NVIDIA DLSS 5 ("real-time neural rendering") be used to show beautiful water in this engine, and how would it be done correctly? |
| Scope | Desktop adapter only (`crates/desktop-sdl-ash`, SDL3 + ash, Linux-only v1 per ADR-090); the authoritative water (exact table and flow network, ADR-104) is untouched by anything here |
| Status | `RESEARCH / RECOMMENDATION` — no decision, no code |

## Short answer

DLSS 5 does not render water; it re-lights and re-materials an image that
the engine has already rendered, guided by the engine's own buffers. Flat
blue quads stay flat blue quads under it. The way to beautiful water is a
proper water render path (animated surface, reflection, refraction, depth
absorption, foam, caustics); DLSS then has two possible roles on top of
it: DLSS Super Resolution / Ray Reconstruction to make a ray-traced
reflection and refraction affordable, and DLSS 5 as a final "make the
materials photoreal" pass. Both are NVIDIA-only, proprietary binary SDKs;
DLSS 5 is RTX 50 only and, today, documented for Windows only. So: build
the water look with techniques that run on every Vulkan GPU on Linux, keep
the buffers DLSS needs (depth, motion vectors, albedo, normals, roughness,
HUD-less colour) as first-class outputs of the render path, and treat any
DLSS integration as an optional, out-of-workspace adapter feature that is
decided later, when the Linux status of DLSS 5 is known.

## What DLSS 5 is (facts, 2026-09-03)

- Announced at GTC 2026, released 2026-09-03 with NBA 2K27; NVIDIA calls
  it "a real-time neural rendering model that infuses pixels with
  photoreal lighting and materials" ([NVIDIA GTC news][nv-gtc],
  [NVIDIA DLSS 5 page][nv-dlss5], [NVIDIA newsroom][nv-news]).
- It is a generative stage that runs after the engine has rendered the
  frame: "the game engine's rendered frame, complete with its
  artist-designed geometry, textures, and lighting buffers" is the input,
  together with "motion vectors directly from the game engine"; the model
  takes "engine data such as color, surface albedo, detailed lighting, and
  surface normals". It is "trained to hold geometry, textures and the
  light-to-shadow relationships exactly as the artist built them"
  ([NVIDIA DLSS 5 page][nv-dlss5], [Back2Gaming preview][b2g]).
- What it changes: "natural skin subsurface scattering", "realistic light
  transmission through hair and foliage", "deeper contact shadows
  alongside global illumination", ambient occlusion and refined
  reflections. Controls: `Structure Intensity` (high-frequency detail:
  AO, reflections, subsurface), `Tone Intensity` (broad lighting and
  colour), semantic auto-masking and engine-level masks per object group,
  several model variants ([NVIDIA DLSS 5 page][nv-dlss5]).
- One frame in, one frame out; temporal stability comes from the engine
  motion vectors ([NVIDIA DLSS 5 page][nv-dlss5], [igor'sLAB][igor]).
- Integration: "the same NVIDIA Streamline framework already utilized"
  plus an Unreal Engine 5 plugin; inputs go through Streamline as
  "depth buffers, motion vectors, HUD-less color buffers"
  ([NVIDIA newsroom][nv-news], [NVIDIA DLSS 5 page][nv-dlss5]).
- Hardware: GeForce RTX 50 series only, up to 4K, single GPU; RTX 40 and
  older are not supported ([NVIDIA DLSS 5 page][nv-dlss5],
  [Back2Gaming][b2g]).
- Not published: the frame-time cost of the neural pass alone, VRAM, the
  model size, and its behaviour with "water, liquids, particles, and
  transparency"; the presentation "neither demonstrates actual temporal
  stability during motion nor behavior with hair, fine patterns,
  transparencies, particles or rapid disocclusions"
  ([Back2Gaming][b2g], [igor'sLAB][igor]). NVIDIA's own note: "DLSS 5 is
  not a shortcut around a weak renderer. More detailed materials, better
  assets and more accurate lighting all feed the foundation" ([b2g][b2g]).
- Platform: every DLSS 5 statement is about Windows and GeForce NOW; the
  Streamline README lists "Win10 20H1 (version 2004 - 10.0.19041) or
  newer" and "DirectX 11 and Vulkan 1.2 or higher", no Linux
  ([Streamline README][sl]). The DLSS-G (frame generation) plugin "is
  only available precompiled"; production builds must use "the original
  NVIDIA-signed SL DLLs" ([Streamline README][sl]).

## What DLSS can do on Linux today (the parts that exist)

- DLSS Super Resolution ships in the DLSS SDK for native Linux x86_64
  Vulkan applications (`libnvidia-ngx-dlss.so`); the SDK is under the
  NVIDIA RTX SDKs License, the SDK itself cannot be redistributed, a
  shipped application redistributes only the runtime library and the
  licence text ([NVIDIA DLSS SDK blog][nv-sdk], [Phoronix][phx],
  [DLSS repo, Linux libs][dlss-lin]).
- DLSS Ray Reconstruction (denoise + upscale of noisy ray-traced
  radiance) works for native Linux Vulkan applications through the NGX
  Vulkan functions; the CUDA path was enabled with SDK 310.5.3 in January
  2026 ([NVIDIA forum thread][rr-linux], [vk_denoise_dlssrr sample][rr]).
  Its inputs are noisy colour, diffuse/specular albedo, normal +
  roughness, motion vectors, linear depth and specular hit distance
  ([vk_denoise_dlssrr][rr]).
- Frame Generation and Multi Frame Generation reach Linux mainly through
  Proton; native use is documented as a Vulkan Streamline integration
  ([NVIDIA driver guide][nv-linux], [vk_streamline sample][vk-sl]).
- A Rust precedent exists: `dlss_wgpu` (Bevy) wraps Super Resolution and
  Ray Reconstruction over Vulkan on Windows and Linux; it does not vendor
  the SDK, requires `DLSS_SDK=/path/to/DLSS` at build time, and warns
  "Due to a bug in DLSS, you should expect to see Vulkan validation
  errors" ([dlss_wgpu][bevy]).

## Why DLSS 5 alone cannot fix the screenshot

The screenshot shows three flat quads with one unlit blue material and a
box on top. DLSS 5 is conditioned on the engine's albedo, normals,
lighting and motion; a flat quad with a constant normal and no lighting
response gives the model nothing to "make photoreal" except a blue plane.
NVIDIA's guidance is explicit that the quality of the input renderer sets
the ceiling. Water specifically is the case the vendor has not shown:
no published behaviour for liquids, transparency, refraction or
particles, and the academic record for neural rendering of strong
refraction is weak ([igor'sLAB][igor], [Back2Gaming][b2g]).

What DLSS 5 could plausibly add later, on top of a real water render
path: softer light response on wet surfaces, better-looking reflections of
lit characters and scenery in the water, contact shadows at the shore.
None of that is available on Linux today, and all of it needs an RTX 50.

## What actually makes water look like water

The engine's authoritative side already provides the exact state a good
water look needs: the level and the flux of every volume (ADR-100/103),
the presentation stage with a bounded ripple and a jet (WP1, plan 09),
and now a floating crate whose immersion is exact (WB1, plan 08). The
look is a renderer question. The standard, vendor-neutral ingredients
([vterrain survey][vt], [ocean survey][ocean], [Tessendorf FFT][fft],
[Vulkan water sample][vkw], [caustics][caus]):

1. **Surface shape.** A wave spectrum on the ADR-101 ring instead of a
   single sine: two FFT cascades (swell and ripple) or a sum of Gerstner
   waves, still driven only by the exact flux and level (SPEC-38 practice
   5 keeps waves presentation-only). The ring already carries per-vertex
   positions and normals; this replaces the ripple function, not the
   feed.
2. **Lighting of the surface.** A water material in the desktop adapter:
   Fresnel-weighted reflection, a sun/sky specular lobe, normal from the
   ring (plus a small detail normal map for sparkle), so the surface
   reads as a surface from any angle.
3. **Reflection.** Planar reflection of the scene for the basin and the
   vessels (three flat, bounded planes: render the scene mirrored once
   per plane, or once for the common level when levels coincide) or
   screen-space reflection. Planar is the cheapest exact answer for this
   scene and works on any GPU.
4. **Refraction and depth colour.** Render the water after the opaque
   pass, sample the scene colour behind the surface with a normal-based
   offset, and absorb it by the depth below the level (the ADR-102
   particle pass already does absorption and refraction for the jet with
   the same constants, `[1.2, 0.5, 0.25]` per metre and `0.08`
   refraction strength).
5. **Shore and object edges.** Soft foam and a depth fade where the
   surface meets the basin walls and the crate (depth difference between
   the surface and the scene depth buffer).
6. **Caustics.** A projected animated caustics texture on the basin floor
   and the submerged half of the crate, scaled by the level.
7. **Crate coupling.** The crate's wake and splash from the exact
   immersion change (an ADR-102 increment: spawn particles when
   `displaced volume` changes fast), and a local ring depression under
   the crate.

Every item runs on the Vulkan 1.2 baseline of the adapter on Linux,
needs no vendor SDK and no new authoritative state. Items 1, 2, 4 and 5
are enough to stop the "flat texture" impression.

## How to keep the door open for DLSS

If DLSS is wanted later, the render path should already produce what the
NGX features consume, so the integration is an adapter feature and not a
rewrite:

- per-pixel **motion vectors** for the water surface and the particles
  (both DLSS SR/RR and DLSS 5 require them; the ring has previous-frame
  positions, so they are computable);
- a **G-buffer with albedo, normal + roughness, linear depth** for the
  water and the scene (RR needs them; DLSS 5 consumes "surface albedo,
  detailed lighting, and surface normals");
- **HUD-less colour**: the UI overlay composited after the DLSS stage;
- **jitter** for the projection (SR/RR and DLAA are temporal);
- a **mask channel** per object group (DLSS 5 engine-level masking) so
  the water, the characters and the environment can be dosed
  separately.

Repository constraints (AGENTS.md, ADR-090): no vendor SDK in the Cargo
workspace; Linux-only v1; the desktop adapter is the only place with a
GPU. A DLSS adapter would therefore be an optional feature of
`next_desktop_sdl_ash` behind an environment-provided SDK path (the
`dlss_wgpu` pattern), disabled by default, never on the deterministic
path, and never a dependency of `game`/`headless` roots. DLSS 5 in
particular cannot be integrated on Linux today: no native Linux support
is documented and its SDK ships only inside Streamline for Windows.

## Recommendation

1. Build the water look with the seven vendor-neutral ingredients above,
   in this order: surface material with Fresnel and specular on the
   existing ring (2), shore fade and foam (5), refraction with depth
   absorption (4), wave spectrum on the ring (1), planar reflection (3),
   caustics (6), crate wake and splash (7). Each step is visible on the
   reference host with `--capture-frame`.
2. While doing it, emit motion vectors, a thin G-buffer and jitter from
   the water pass, and keep the HUD after the scene composite. This is
   cheap now and is the precondition for any DLSS feature.
3. Revisit DLSS when either of two things happens: NVIDIA documents DLSS
   5 for native Linux Vulkan, or the project decides to ray-trace the
   water reflection and refraction (then DLSS Ray Reconstruction on Linux
   is the denoiser of choice, with the `dlss_wgpu` integration pattern
   as the template).
4. Do not plan the look around DLSS 5: it is RTX 50 only, Windows only
   today, with unpublished cost and undocumented behaviour on water, and
   it improves what the renderer already computes rather than replacing
   it.

## Sources

- [NVIDIA: New DLSS 4 Games, Plus DLSS 5 Announced At GTC 2026][nv-gtc]
- [NVIDIA: DLSS 5 3D-Guided Neural Rendering Debuts in NBA 2K27][nv-dlss5]
- [NVIDIA Newsroom: DLSS 5 Delivers AI-Powered Breakthrough in Visual Fidelity][nv-news]
- [Back2Gaming: DLSS 5 Technical Preview][b2g]
- [igor'sLAB: DLSS 5 at Gamescom 2026, neural rendering explained][igor]
- [NVIDIA-RTX/Streamline README][sl]
- [NVIDIA: DLSS SDK now available for all developers with Linux support][nv-sdk]
- [Phoronix: DLSS Super Resolution SDK 3.1 with updated Linux demo][phx]
- [NVIDIA/DLSS: Linux x86_64 libraries][dlss-lin]
- [NVIDIA driver installation guide: DLSS / Smooth Motion / Reflex on Linux][nv-linux]
- [NVIDIA developer forum: NGX reports DLSS RR as unavailable on Linux (resolved with SDK 310.5.3)][rr-linux]
- [nvpro-samples/vk_denoise_dlssrr][rr]
- [nvpro-samples/vk_streamline][vk-sl]
- [bevyengine/dlss_wgpu][bevy]
- [vterrain.org: Water Rendering and Simulation][vt]
- [A Survey of Ocean Simulation and Rendering Techniques][ocean]
- [Tessendorf FFT ocean implementation (fftWater)][fft]
- [kentril0/WaterSurfaceRendering (Vulkan)][vkw]
- [Real-time rendering of water caustics][caus]

[nv-gtc]: https://www.nvidia.com/en-us/geforce/news/death-stranding-2-crimson-desert-dlss-4-multi-frame-gen/
[nv-dlss5]: https://www.nvidia.com/en-us/geforce/news/dlss-5-3d-guided-neural-rendering/
[nv-news]: https://nvidianews.nvidia.com/news/nvidia-dlss-5-delivers-ai-powered-breakthrough-in-visual-fidelity-for-games
[b2g]: https://www.back2gaming.com/features/nvidia-dlss-5-technical-preview-3d-guided-neural-rendering/
[igor]: https://www.igorslab.de/en/dlss-5-gamescom-2026-3d-guided-neural-rendering/
[sl]: https://github.com/NVIDIA-RTX/Streamline
[nv-sdk]: https://developer.nvidia.com/blog/nvidia-dlss-sdk-now-available-for-all-developers-with-linux-support-unreal-engine-5-plugin-and-new-customizable-options
[phx]: https://www.phoronix.com/news/NVIDIA-DLSS-SDK-3.1
[dlss-lin]: https://github.com/NVIDIA/DLSS/tree/main/lib/Linux_x86_64
[nv-linux]: https://docs.nvidia.com/datacenter/tesla/driver-installation-guide/gaming.html
[rr-linux]: https://forums.developer.nvidia.com/t/ngx-reports-dlss-rr-as-unavailable-on-linux/353770
[rr]: https://github.com/nvpro-samples/vk_denoise_dlssrr
[vk-sl]: https://github.com/nvpro-samples/vk_streamline
[bevy]: https://github.com/bevyengine/dlss_wgpu
[vt]: http://vterrain.org/Water/
[ocean]: https://arxiv.org/pdf/1109.6494
[fft]: https://github.com/iamyoukou/fftWater
[vkw]: https://github.com/kentril0/WaterSurfaceRendering
[caus]: https://medium.com/@martinRenou/real-time-rendering-of-water-caustics-59cda1d74aa
