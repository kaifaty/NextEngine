# Continuum material physics — umbrella specification series

Status: `Proposed`; architecture and evidence-gated package specifications,
not one linear implementation roadmap. Governing candidate architecture:
[SPEC-38](../../architecture/38-continuum-material-physics.md),
[ADR-076](../../architecture/adr/076-continuum-material-physics-track.md) and
the [research report](../../development/continuum-material-physics-research-2026-08-16.md).

Water execution has its own
[standalone roadmap](../continuum-water/README.md) for a dedicated worktree.
This directory owns shared gates, the independent terrain/wet-material lane
and later cross-lane lifecycle specifications. No package changes the current
PhysX baseline or public schemas without a consumer-backed Accepted ADR.

## Dependency graph

```text
00 Shared product/evidence gates
 ├─ Water: ../continuum-water W0A → W0B → W1 → W2 → W3 → W4 → W5 → W6
 │                                           └──────── optional WG mirror
 └─ Terrain:
     10T dry-sand evidence contract
       → 11T serial MLS-MPM dry-sand reference
       → 12T exclusive wheel/terrain coupling
       → 13T exact active terrain persistence
       → 14T saturation and drainage
       → 15T closed free-water/terrain flux
       → 30T production promotion

Exact active persistence (water W5 or terrain 13T)
 ├─ 20X optional lossy sleep/wake conversion
 └─ 21X optional cross-region transfer
```

| Package | Specification | Starts only after |
|---|---|---|
| 00 | [Shared product and evidence gates](00-product-and-evidence-gates.md) | none |
| 10T | [Dry-sand product and evidence contract](10-dry-sand-product-and-evidence-contract.md) | 00 |
| 11T | [Serial MLS-MPM dry-sand reference](11-mls-mpm-dry-sand-reference.md) | 10T profile/curves frozen |
| 12T | [Exclusive wheel/terrain coupling](12-exclusive-wheel-terrain-coupling.md) | 11T PASS |
| 13T | [Exact active terrain persistence](13-exact-active-terrain-persistence.md) | 12T PASS |
| 14T | [Saturation and drainage](14-saturation-and-drainage.md) | 13T PASS |
| 15T | [Free-water/terrain flux](15-free-water-terrain-flux.md) | water W5 plus 14T PASS |
| 20X | [Lossy sleep/wake conversion](20-lossy-sleep-wake-conversion.md) | exact active persistence for that material |
| 21X | [Cross-region transfer](21-cross-region-transfer.md) | one-region production evidence plus explicit consumer |
| 30T | [Terrain validation and promotion](30-terrain-validation-and-promotion.md) | selected terrain lane packages |

## Program invariants

- one Physical Embodiment ownership umbrella, material-specific numerical
  state and no solver monoculture;
- one representative scenario and explicit non-goals per package under B-10;
- exact profile, corpus, thresholds, capacities and failure codes before code;
- fixed cadence/resolution and finite memory/iteration profiles before
  adaptivity or broader materials;
- CPU reference before parallel/accelerated implementation;
- no public schema before a demonstrated production consumer under ADR-046;
- exact active persistence before lossy sleep or cross-region lifecycle;
- renderer, GPU cache, MPM grid and SPH neighbor state never become authority;
- whole-owner-step atomicity with no retry-to-green or partial publication;
- first-party consumers use the same validated public path later available to
  community content.

## Branch boundaries

Water and terrain do not block one another after Package 00. A passing water
oracle does not validate MPM, and a passing dry-sand corpus does not authorize
wet soil. Free-water/terrain flux is the first package that joins both owner
lanes and therefore requires both exact active persistence gates.

Package 20X and 21X are future optional branches. They are never implicit work
inside a solver, save or streaming package and receive no roadmap credit from
an exact pinned-active checkpoint.
