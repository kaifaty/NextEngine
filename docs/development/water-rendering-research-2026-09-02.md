# Water rendering research note — 2026-09-02

Bounded research cycle for the question "how should NextEngine render the
presentation water?" under ADR-100/ADR-101: the solver is presentation-only,
the renderer is the SDL3/Ash B0 path (Lambert diffuse, hemisphere ambient,
shadow, fog; base colour only, closed shader interface), and the only
dynamic path today is the ADR-101 vertex/index ring fed by a top-down
height-field extraction (NGQ5/NGQ6/NGQ9). Sources are primary where
available; claims are bounded to what each source states.

## What the current path can and cannot do

- Height field over a fixed top-down pixel grid: one height per pixel, so
  overhangs, falling sheets seen from the side, spray and stacked bodies
  cannot be represented; the spill jet renders as a ridge on the floor
  grid, not as a falling stream.
- Opaque diffuse shading: no Fresnel, refraction, absorption by thickness
  or specular, because B0 exposes base colour only.
- What it does well: exact CPU/GPU equivalence, bounded vertex budgets, a
  static catalog identity, and the calm free surface of a basin.

## Options

### A. Keep the height field, polish within B0

Cheapest. Improvements are limited to mesh quality (smoothing, edge
heights) and colour; none of the visual cues of water (transparency,
reflection, thin streams) become available. Suitable for the still
gameplay fallback (ADR-100), not for the presentation solver's output.

### B. Screen-space fluid rendering (SSF)

Van der Laan, Green and Sainz (I3D 2009) render the particle set directly:
particles are drawn as depth-replaced sphere sprites into a depth target
(nearest particle wins), the depth image is smoothed, view-space normals
follow from the smoothed depth by finite differences, a second additive
pass accumulates thickness, and a full-screen composite applies Fresnel,
reflection, refraction of the scene colour perturbed by the normal, and
absorption by thickness (Beer-Lambert style, per channel); optional
Perlin noise advected with the particles adds foam. It is not based on
polygonisation, renders only what is visible with inherent
view-dependent level of detail, and consists of a few fragment-shader
passes with intermediate render targets. Their benchmark: 64,000 SPH
particles at 1024x768 on a GeForce 8800 GTS, whole frame 17.5 ms with
curvature flow at quarter resolution and 15 iterations, 50 ms at full
resolution and 100 iterations, 18.1 ms with a separable bilateral blur;
noise and foam add a pass (30 ms at quarter resolution). Limitation
stated by the authors: only the nearest surface is rendered, so several
fluid layers with air between them are not physically correct, and the
explicit curvature-flow integration needs many iterations at high
resolution to stay stable.

Later work replaces the smoothing filter: the narrow-range filter (Truong
and Yuksel, PACMCGIT/I3D 2018) is designed to avoid blending distinct
nearby surfaces while still smoothing grazing-angle variation; Kitware's
VTK implementation (2019 article) uses it as the default over the bilateral
Gaussian ("simpler, lower surface quality"), keeps the four passes (depth,
thickness with blending, filtering, composition) and reports interactive
rates with about 675k particles at 1920x1080 and real-time rendering of
over 10 million particles on a 1080 Ti. NVIDIA Flex's UE4 fluid surface
renders the same way and adds per-particle anisotropy so particles are
splatted as oriented ellipsoids (smoother thin sheets; scale about 1.0,
min/max about 0.1..1.0), a depth-edge falloff that stops foreground and
background layers from blending, and thickness-based attenuation.

Fit for NextEngine: the solver already publishes particle positions every
frame; a 48k particle set is 576 KB of positions per frame, less than the
16k height-field mesh today. Spray, the spill jet, thin sheets and stacked
bodies are handled by construction; the flicker classes we fixed in the
extractor do not exist because there is no mask. Cost on an RTX 3080 at
1080p should sit in the low milliseconds by scaling from the 2009 and 2019
figures, which fits the presentation budget next to a 4 ms solver step.
What it needs: new renderer passes and targets (particle depth, thickness,
a scene-colour copy for refraction, a composite), a fragment shader set
outside the closed B0 interface, and a presentation-snapshot record for a
bounded particle set (positions plus radius, optional anisotropy) instead
of a mesh. That is a SPEC-04/ADR-003 extension and therefore its own ADR,
in the same presentation-private, non-authoritative frame as ADR-101.

### C. World-space surface reconstruction (marching cubes)

Yu and Turk (TOG 2013) build an implicit surface from anisotropic kernels
(PCA over neighbours) and polygonise it; it represents smooth surfaces,
thin streams and sharp features better than isotropic kernels. GPU
two-level grids (Visual Computer 2016) make reconstruction interactive.
Against it for a game renderer: a grid per frame, frame-to-frame grid
artifacts noted by van der Laan et al. for low-resolution real-time grids,
a large vertex payload per frame and no transparency without the same
shading work as B. It remains the right tool for offline or capture-grade
surfaces, not the live path.

### D. Hybrid height field plus SSF spray

Possible later (bulk as height field, spray as splats), but it doubles the
pipelines and reintroduces the two-representation seams; not before B
exists.

## Recommendation

Adopt B as the presentation water path for the game renderer, staged:

1. ADR (Proposed) "presentation particle surface pass": bounded particle
   set record (`<= 65,536` particles, positions in micrometres, radius,
   optional per-particle anisotropy), renderer passes at half resolution
   with the narrow-range filter as the default smoother (bilateral as the
   cheap fallback), thickness attenuation and Fresnel/refraction against a
   scene-colour copy; foam by advected noise as an optional later stage.
   Everything stays outside every root; the ADR-100 still surface is the
   fallback when the pass is unavailable.
2. Prototype in the existing water-preview bridge first (the stream
   already carries positions), with frozen gates: cost `<= 2 ms` GPU at
   1080p on the RTX 3080 witness, no temporal flicker of the silhouette
   under a fixed camera (per-pixel coverage difference between consecutive
   frames below a frozen threshold), and captures of the spill scene as
   human evidence.
3. Only then move the pass into the game root under the ADR-101 lineage.

## Sources

- W. J. van der Laan, S. Green, M. Sainz, "Screen space fluid rendering
  with curvature flow", I3D 2009 (ACM DOI 10.1145/1507149.1507164; PDF
  read in full).
- D. Truong, C. Yuksel, "A Narrow-Range Filter for Screen-Space Fluid
  Rendering", Proc. ACM CGIT 2018 (DOI 10.1145/3203201; abstract via
  search, full text not retrieved).
- Kitware, "Screen-Space Fluid Rendering in VTK" (kitware.com article,
  read: four passes, narrow-range default, 1080 Ti figures).
- NVIDIA Flex artist tools, "Fluid Rendering" (UE4 documentation, read:
  anisotropy, depth edge falloff, thickness).
- J. Yu, G. Turk, "Reconstructing surfaces of particle-based fluids using
  anisotropic kernels", ACM TOG 2013 (DOI 10.1145/2421636.2421641; abstract
  via search).
- "Anisotropic screen space rendering for particle-based fluid
  simulation", Computers & Graphics 2023 (DOI 10.1016/j.cag.2022.12.007;
  listing only, full text not retrieved).
