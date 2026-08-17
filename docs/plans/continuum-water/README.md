# Continuum water — standalone implementation roadmap

Status: `PLANNED / NOT_ACTIVE`; post-v1 isolated program. Governing candidate
architecture: [SPEC-38](../../architecture/38-continuum-material-physics.md)
and [ADR-076](../../architecture/adr/076-continuum-material-physics-track.md).
No `CONTINUUM-*` ProductCheck has run.

This directory is the resume and execution surface for a dedicated water
worktree. The main [Next Engine roadmap](../../roadmap.md) keeps the track
inactive until `CONTINUUM-WATER-REF-P1 = PASS`. Progress here cannot change
the current PhysX, save/replay or public-contract baseline by implication.

## Selected product result

One sealed `4 × 2 × 1 m` basin contains `0.75 m` of clean water and one
`0.5 m`, `50 kg` PhysX crate. The nominal production fixture uses `48,000`
samples with a hard capacity of `50,000`. The player acts through the normal
production command path; deterministic headless consumes the recorded command
trace. Debug particles and diagnostic overlays are sufficient presentation.

If continuum capability cannot activate, the project loads its authored dry
basin variant. An active region never silently changes to dry, decorative or
frozen water.

## Stage graph

```text
W0 Specification closure                         COMPLETE / DOCUMENTATION
 └─ W1 Serial CPU oracle + external corpus       NOT_STARTED
     ├─ W2 Deterministic parallel CPU + benchmark NOT_STARTED
     │   └─ W3 One-pass PhysX coupling            NOT_STARTED
     │       └─ W4 Basin + crate + debug view     NOT_STARTED
     │           └─ W5 Exact active persistence   NOT_STARTED
     │               └─ W6 Production promotion   NOT_STARTED
     └─ WG Optional GPU correspondence mirror     NOT_STARTED / NON_BLOCKING
```

| Stage | Specification | Exit evidence | Blocks |
|---|---|---|---|
| W0 | [Product and evidence contract](00-product-and-evidence-contract.md) | Exact scenario, profile, metrics, limits and non-goals are frozen | W1 |
| W1 | [Serial CPU DFSPH oracle](01-serial-cpu-dfsph-oracle.md) | `CONTINUUM-WATER-REF-P1 = PASS` on the same-target reference profile | main-roadmap activation, W2, WG |
| W2 | [Deterministic parallel CPU and performance](02-deterministic-parallel-and-performance.md) | Worker/order exactness and `50k` THOTH budget PASS | W3 |
| W3 | [One-pass PhysX coupling](03-one-pass-physx-coupling.md) | `CONTINUUM-COUPLING-P1 = PASS` without a second rigid writer | W4 |
| W4 | [Basin, crate and debug presentation](04-basin-crate-debug-presentation.md) | Production command loop and presentation-independence pass | W5 |
| W5 | [Exact active persistence](05-exact-active-persistence.md) | `CONTINUUM-PERSISTENCE-P1 = PASS` for exact active state | W6 |
| W6 | [Production promotion](06-production-promotion.md) | Consumer-backed Accepted decision and all declared product checks | shipped claim |
| WG | [GPU correspondence mirror](wg-gpu-correspondence.md) | `CONTINUUM-MIRROR-P1` report for named devices | no authority or promotion stage |

## Program invariants

- CPU DFSPH is the only canonical water solver candidate for V1.
- The accepted state is stable sample ID plus fixed-point position/velocity;
  the next substep starts from that state.
- One physical substep produces one water candidate and one reaction batch;
  water and PhysX publish together or neither publishes.
- The first region is sealed and pinned active. Cross-region transfer, halos,
  streaming eviction and sleep conversion are not hidden implementation work.
- Public contracts appear only at W4/W6 with the runtime-bearing basin
  consumer and an Accepted promotion ADR.
- GPU completion, renderer cadence, wall time, worker count and cache warmth do
  not select an authoritative state.
- Heavy corpora, comparison trajectories, captures and performance reports
  stay outside Git; checked-in evidence is bounded hashes/summaries only.

## Worktree and main-roadmap protocol

1. Create the water worktree from the documentation checkpoint containing W0.
2. Record stage state and exact evidence links in this roadmap/task-state;
   never mark a PASS from a type, fixture, compilation or report-only run.
3. After W1 PASS, merge the evidence checkpoint and change the main R8 row from
   `PLANNED / NOT_ACTIVE` to an active integration track. Before that event the
   main roadmap contains only this experimental pointer.
4. Each stage is a coherent commit boundary. Failed evidence leaves the stage
   open and records the smallest discriminator; it does not relax thresholds.
5. If the 50k gate misses budget after two evidence-backed optimization
   cycles, stop the roadmap as `RESEARCH_ONLY`. GPU authority, a smaller
   production gate or a larger budget requires a new explicit decision.

## Whole-program non-goals

Ocean/weather simulation, fluid networks, cross-region particle transport,
adaptive particles, surface-quality rendering, foam/spray, generic plugin
solver ABI, arbitrary gameplay fluid queries, wet soil, terrain deformation,
full vehicles and production sleep/wake conversion.
