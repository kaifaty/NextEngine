# Structural vegetation physics — research report, 2026-08-16

## Question and method

The source brief proposes a physical vegetation stack spanning tree elasticity,
cutting, fracture, foliage, wind, fire, moisture, roots, soil and forest LOD.
This report treats it as a research hypothesis, not as repository instructions.
It compares the proposal with current Next Engine ownership, determinism,
persistence and product-scope constraints and with primary technical sources.

Input reviewed in full:
`/home/kaifaty/Downloads/vegetation_physics_destructible_trees_research.md`,
SHA-256
`7e6be975caf83dd096ae84ff8aeacdc12c1bb34f43365ff35755cb95083bef51`.

The candidate architecture is recorded in
[SPEC-39](../architecture/39-layered-physical-world.md),
[SPEC-40](../architecture/40-structural-vegetation-physics.md) and
[ADR-077](../architecture/adr/077-layered-physical-world-and-living-structures-track.md).
The implementation gates are in the
[standalone vegetation roadmap](../plans/vegetation-physics/README.md).

## Executive conclusion

Retain the brief's strongest idea: a tree is a sparse rooted structural graph
whose visual mesh follows beam/rod state, while local section damage controls
stiffness and fracture. This can support a physical notch, hinge and fall
without simulating the render mesh or reducing the tree to hit points.

Change the execution strategy:

1. use ownership/transaction layers rather than one effect stack;
2. keep PhysX as the sole rigid writer and add one living-structure owner;
3. select the exact rod/beam formulation through a CPU serial corpus rather
   than treating “Cosserat” as an implementation specification;
4. use a bounded polar section lattice at one felling zone before arbitrary
   fibres/cuts;
5. hand the detached component to one PhysX compound body instead of running a
   flexible falling crown in V1;
6. prove exact active persistence before modal/sleep/forest streaming;
7. make GPU correspondence-only and LOD independent of camera/timing;
8. keep fire, roots and continuum soil as later peer-owner lanes.

The architecture is closed, but implementation is not ready: Package V0 still
needs one exact tree, calibrated wood/anchor profile, cut-work mapping, numeric
state scales, curves, thresholds, capacities and a forest performance fixture.

## Evidence matrix

| Question | Evidence | Bounded conclusion |
|---|---|---|
| Is a rod/beam skeleton credible? | Bergou et al., [Discrete Elastic Rods](https://doi.org/10.1145/1360612.1360662), model thin rods with arbitrary cross-section/rest shape, dynamic bending and bending/twist coupling and validate buckling/stability cases. | Yes as a formulation candidate and corpus source. Its quasi-static material-frame and inextensibility choices are not automatically the right tree V1. |
| Is a shearable Cosserat comparator available? | [PyElastica](https://github.com/GazzolaLab/PyElastica) is an MIT-licensed Cosserat-rod implementation; its documentation includes [dynamic cantilever and catenary validation examples](https://docs.cosseratrods.org/en/v1.0.0/_gallery/index.html). Tag [v1.0.0](https://github.com/GazzolaLab/PyElastica/tree/v1.0.0), commit `b087f1399f9be2fdd2fcf3768689f7735a96f7ab`, is pinned for external evidence. | Use aggregate curves as an independent comparator. Do not copy/link it or make its node identity authoritative. |
| Can one isotropic wood default be used? | The US Forest Service [Wood Handbook 2021](https://research.fs.usda.gov/fpl/wood-handbook) separates longitudinal/radial/tangential elastic behavior, strength, vibration and moisture relations. It also distinguishes clear straight-grained specimens from full structural members. | No. V0 needs one calibrated orthotropic/reduced profile and must state whether it is synthetic or species-specific. Handbook tables alone do not calibrate a living tree, junction or root anchor. |
| Are tapered cantilever and modal tests meaningful? | Gardiner, Peltola and Kellomäki's [dynamic Sitka spruce model](https://pubmed.ncbi.nlm.nih.gov/10527716/) uses a tapered cantilever, canopy mass and time-varying wind to predict primary natural frequency and failure. Zanotto et al. [compare measured and FE tree modes](https://doi.org/10.1016/j.foreco.2024.122188) and report first-mode differences up to 16% under unmeasured geometry/material assumptions. | Use taper, pull/release, gust and modal curves as validation families. Do not accept “plausible sway” or one generic frequency as proof. |
| Should fracture follow fibre direction? | Hädrich et al., [Interactive Wood Fracture](https://doi.org/10.2312/sca.20201215), combines shape matching and Cosserat-like fibres for anisotropic interactive fracture. Ennos and van Casteren's [branch failure analysis](https://pubmed.ncbi.nlm.nih.gov/20018786/) relates transverse stresses and low transverse strength to splitting/greenstick fracture. | Directional section/fibre response is credible, but the graphics poster is not a calibrated production oracle. Begin with section geometry and directional strength; detailed strands are later. |
| Can wind and foliage be one visual effect? | Tree dynamics literature models wind load, canopy mass/drag and structural modes as coupled physical inputs; visual leaf motion is not the measured structural state. | No. V1 uses a hash-closed analytical forcing and bounded drag clusters. Shader flutter remains presentation. |
| Is GPU-first state safe? | Current Next Engine SPEC-21/26 and ADR-058 require CPU/canonical roots; parallel floating reductions and final quantization do not prove identical trajectory history. | No. CPU serial then deterministic parallel authority; optional GPU aggregate correspondence only. |
| Can distance-only LOD control tree physics? | Current SPEC-05 already forbids renderer camera/frustum and measured timing from selecting physical LOD. Tree damage/contact adds durable state that a visual tier cannot reconstruct. | No. LOD uses canonical residency/distance/importance/contact/damage facts and exact transition receipts. |

## Corrections to the source brief

### Retained

- sparse rooted graph for trunk and important branches;
- beam/rod mechanics rather than rigid-joint chain or visual-mesh simulation;
- orthotropic/reduced wood response and directional section mechanics;
- persistent local damage with no tree HP;
- local refinement near a cut rather than full-tree fibre resolution;
- fracture as a graph topology change;
- analytical/spectral wind and bounded foliage drag clusters;
- modal/coarse/active tiers and immutable presentation extraction;
- separate moisture, thermal, roots and continuum-soil concerns.

### Changed

- “Cosserat” becomes a V1 candidate, not a preselected exact solver;
- GPU SoA becomes an optional mirror after the CPU oracle, not authority;
- relevance LOD excludes camera, frame time and completion-order signals;
- first cutting is one felling zone and polar section lattice, not arbitrary
  saw/axe/fibre fracture;
- first detached crown becomes one PhysX compound body, avoiding two owners of
  the same mass and a flexible falling-tree solver in V1;
- collision uses a fixed staged contact-load profile with one-substep stagger;
- exact active save/restart precedes modal/sleep or forest streaming;
- fire and root/soil coupling become independent later DAG branches;
- the broad crate/module list is reduced to one living-structures lane and
  consumer-driven implementation boundaries.

## Layered physical-world finding

The useful top-level decomposition is not:

```text
water -> trees -> wind -> fire -> rigid -> gameplay
```

Those phenomena are not a single ownership chain. The safe model is:

```text
activation/profile closure
        -> canonical schedule and revisions
        -> immutable environmental forcing projections
        -> peer physical owners (PhysX | continuum | living structures | future energy)
        -> canonical exchange batches and composite PhysicalStep
        -> committed outcomes plus composite persistence
        -> read-only presentation
```

This form answers four implementation questions that the effect diagram leaves
open: who writes each field, which state is complete, when coupling becomes
visible, and what fails atomically. It also lets a project instantiate only
the owners needed by its product scenario.

## First product boundary

The trail-tree vertical deliberately proves one gameplay result:

```text
production axe commands
  -> directional section-cell loss
  -> computed hinge/failure
  -> atomic graph split and PhysX handoff
  -> rigid fallen trunk can block the trail
```

It excludes fire, soil, vehicle/tree, tree/tree and full forest streaming. The
first visual requirement is debugging, because cut meshes and foliage can hide
an incorrect structural result. An authored static tree is the only
pre-activation fallback; once active, failure stops the physical run.

## Open decisions before code

| Decision | Why it is still open | Smallest closure action |
|---|---|---|
| Reference tree geometry | Brief gives only a `5..20` branch range and no exact taper/mass graph | Freeze one canonical procedural tree and capacities |
| Wood/root-anchor profile | Published clear-wood tables do not determine a living tree/junction/root | Choose synthetic vs species-specific goal and calibrate exact reduced constants/curves |
| Formulation/integrator | Stiff axial modes make naive explicit 240 Hz execution unsafe; DER/Cosserat states differ | Run V1 bake-off after V0 thresholds are frozen |
| Cut-work and section cells | Tool energy, grain, ring/sector count and failure law are unspecified | Freeze one axe/contact trace, polar lattice and measured/synthetic notched-beam curves |
| Canonical scales/state | Rotation, strain, damage and topology continuation widths are absent | Define the full SPEC-21 numeric profile before solver implementation |
| Forest gate | “Large forest” has no counts or THOTH budget | Freeze visible/modal/active/refined counts and p95/p99/memory limits |

## Non-goals and claim boundary

This report proves no solver accuracy, real-time budget, material realism,
cutting behavior, cross-target determinism, persistence, LOD or production
support. All `VEGETATION-*` checks are `NOT_RUN`; SPEC-39/SPEC-40/ADR-077 remain
Proposed and current PhysX/runtime/schema/save behavior is unchanged.
