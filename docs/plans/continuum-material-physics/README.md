# Continuum material physics — implementation specification series

Status: `Proposed`; research/implementation planning only. Governing candidate
architecture: [SPEC-36](../../architecture/36-continuum-material-physics.md),
[ADR-072](../../architecture/adr/072-continuum-material-physics-track.md), and
the [research report](../../development/continuum-material-physics-research-2026-08-16.md).

The files below are ordered work packages, not one large implementation plan.
Each package must produce its own evidence before the next package relies on
it. No package adds public contracts or changes the current PhysX production
baseline unless a later consumer-backed Accepted ADR explicitly says so.

| Order | Specification | Output | Depends on |
|---:|---|---|---|
| 01 | [CPU DFSPH water reference](01-cpu-dfsph-water-reference.md) | Deterministic offline water oracle and corpus | none |
| 02 | [Boundaries and rigid coupling](02-boundaries-and-rigid-coupling.md) | Closed reaction-impulse vertical | 01 |
| 03 | [Water presentation](03-water-presentation.md) | Read-only visible water | 01; 02 for moving bodies |
| 04 | [GPU water correspondence](04-gpu-water-correspondence.md) | Accelerated non-authoritative mirror | 01–03 |
| 05 | [MLS-MPM dry terrain](05-mls-mpm-dry-terrain.md) | Sand/soil deformation and patch coupling | 02 concepts |
| 06 | [Wet soil and mud](06-wet-soil-and-mud.md) | Saturation-dependent terrain | 05 |
| 07 | [Streaming and persistence](07-streaming-and-persistence.md) | Active/sleep owner-state lifecycle | the material lane being persisted |
| 08 | [Validation, performance and promotion](08-validation-performance-and-promotion.md) | Product scenario and promotion dossier | 01–07 as applicable |

Program invariants:

- one mutable owner per field;
- fixed cadence and finite memory/iteration profiles;
- stable sample/body/region identity and canonical reductions;
- whole-step atomicity with typed failure;
- CPU reference precedes accelerated mirror;
- dry terrain precedes wet terrain;
- fixed resolution precedes adaptivity;
- renderer and GPU caches never feed authoritative state;
- production schemas follow a demonstrated consumer under ADR-046;
- v1 gameplay remains correct when the entire optional track is absent.
