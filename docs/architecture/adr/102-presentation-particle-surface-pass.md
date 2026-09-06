# ADR-102: Presentation-only particle surface pass (screen-space fluid)

| Field | Value |
|---|---|
| ID | ADR-102 |
| Status | Proposed |
| Version | 0.3 |
| Proposal date | 2026-09-02 |
| Last verified | 2026-09-02 (NGQ10 prototype evidence: `docs/development/nonlocal-gpu-screen-space-fluid-evidence-2026-09-02.md`) |
| Normative dependencies | [SPEC-04](../04-rendering-and-platform.md), [SPEC-30](../30-presentation-extraction-and-render-content.md), [SPEC-38](../38-continuum-material-physics.md), [ADR-003](003-vulkan-renderer-and-shader-toolchain.md), [ADR-028](028-platform-session-and-presentation-authority.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-100](100-authoritative-water-volume-and-presentation-only-gpu-water.md), [ADR-101](101-presentation-only-dynamic-surface-ring.md) |
| Supersedes | none |
| Superseded by | none |

## Context

ADR-100 makes water dynamics presentation-only and ADR-101 gives the
renderer a per-frame vertex ring for a catalog mesh. The only surface
producer today is a top-down height field (NGQ5/NGQ6/NGQ9): one height per
pixel of a fixed grid, opaque B0 shading with base colour only. It cannot
show overhangs, falling streams seen from the side, spray, stacked bodies,
transparency, reflection or thickness, and its mask stages are the source
of every flicker class fixed so far. The research note
`docs/development/water-rendering-research-2026-09-02.md` reviews the
alternatives; screen-space fluid rendering (van der Laan, Green, Sainz 2009;
narrow-range filter, Truong and Yuksel 2018; Flex anisotropic splats) renders
the particle set directly with view-dependent detail at low-millisecond cost
and needs no mask, mesh or grid.

## Decision

### One bounded particle set is a presentation-private input

- A run may declare at most one `ParticleSurfaceProfileV1`: particle
  capacity bounded by `65,536`, a fixed particle radius in micrometres, a
  colour/absorption tint and a half- or full-resolution flag. Capacity
  never grows at runtime.
- A `ParticleSurfaceUpdateV1` replaces the whole set for one frame slot:
  positions in micrometres inside the declared bounds, strictly increasing
  sequence, content-only canonical hash. Undeclared, over-capacity,
  out-of-bounds or regressed updates fail closed before exposure; a
  publication batch commits atomically together with ADR-101 updates.
- Updates never enter `PresentationSnapshotV3`, the frame-plan hash, any
  content, save, replay or gameplay root. Device loss rebuilds the pass
  from the profile; the next frame republishes.

### The pass is a fixed sequence of presentation-private stages

After the opaque world pass and before the UI overlay:

1. **Particle depth**: every particle is a depth-replaced sphere sprite
   into a private depth target at the declared resolution; the scene depth
   occludes it.
2. **Thickness**: the same sprites accumulate additively into a private
   single-channel target, occluded by the scene depth.
3. **Smoothing**: the depth target is filtered with the narrow-range
   filter (bilateral Gaussian as the documented cheap fallback); iteration
   count and radius are profile constants, not per-frame inputs.
4. **Composite**: a full-screen pass reconstructs the view-space normal
   from the smoothed depth, shades with Fresnel against the sky gradient,
   refracts a copy of the opaque scene colour perturbed by the normal,
   attenuates by thickness with the profile tint, and writes only where
   the fluid is nearer than the scene.

The closed B0 shader interface, its descriptor layouts and every existing
suite are untouched; the pass is a separate shader suite with its own
pinned provenance. It runs only when a profile is declared and the device
reports the required formats; otherwise the run renders the ADR-100 still
surface and reports the pass as unavailable.

### Boundary

No type from this decision enters `crates/contracts`; the profile and
update types are adapter-owned presentation API consumed by composition
roots and developer tools, like ADR-101. Each particle of the one set
carries a producer-side neighbour count; the profile's spray threshold
splits that set into surface splats and spray discs drawn after the
composite (0.2, NGQ10 revision 2), still one bounded set. Each particle
may also carry a producer-side smoothed position and a symmetric
anisotropic kernel matrix; the splat then renders the kernel ellipsoid
(0.3, NGQ10 revision 4, after Yu and Turk 2010). Foam, a second particle
set and full-resolution refraction are later increments under the same
ADR. Nothing here changes the ADR-100 authority
split: gameplay reads `WaterVolume`, never this pass.

## Product checks

| ID | Scenario | Expected behavior | Fallback |
|---|---|---|---|
| `RENDER-PARTICLE-SURFACE-P1` | Publish bounded particle sets for one declared profile over a bounded run with frame capture on the reference host. | One catalog/snapshot/frame plan; pass GPU time within the declared budget at 1080p; silhouette coverage change per published particle set under a fixed camera below the frozen threshold (the per-stream-frame apparatus of plan 24, revision 3); rejected batches leave the prior set; capture diagnostic only. | Still ADR-100 surface; pass reported unavailable. |

## Considered alternatives

- World-space reconstruction (anisotropic kernels plus marching cubes):
  rejected for the live path (grid per frame, frame-to-frame grid
  artifacts, large vertex payload, same shading work still needed).
- Extending the height field inside B0: rejected (no transparency,
  overhangs or streams by construction).
- Hybrid height field plus splat spray: deferred until this pass exists.
