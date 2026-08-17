# Continuum water — standalone implementation roadmap

Status: `PLANNED / NOT_ACTIVE`; post-v1 isolated program. Governing candidate
architecture: [SPEC-38](../../architecture/38-continuum-material-physics.md)
and [ADR-076](../../architecture/adr/076-continuum-material-physics-track.md),
with [ADR-081](../../architecture/adr/081-world-dynamics-gap-closure-and-promotion-guardrails.md)
as the promotion guardrail.
The original W0B numeric/profile/corpus closure remains hash-frozen evidence,
but W1-RC1 independently reproduced its hydro non-convergence and requires a
W0C calibration reclosure. W0C diagnostics reject both a ceiling-only repair
and the first cell-centred ghost-boundary candidate; deterministic
pre-equilibrated initialization is next. The W1 serial oracle is implemented
and blocked. No `CONTINUUM-*` ProductCheck has run.

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
W0A Product and evidence scope                    COMPLETE / DOCUMENTATION
 └─ W0B Original numeric/corpus closure           INVALIDATED_BY_RC1 / ROOTS_RETAINED
     └─ W0C Hydro calibration reclosure            IN_PROGRESS / GHOST_CANDIDATE_REJECTED
         └─ W1 Serial CPU oracle + external corpus IMPLEMENTED / BLOCKED_ON_W0C
         ├─ W2 Deterministic parallel CPU + benchmark NOT_STARTED
         │   └─ W3 One-pass PhysX coupling            NOT_STARTED
         │       └─ W4 Basin + crate + debug view     NOT_STARTED
         │           └─ W5 Exact active persistence   NOT_STARTED
         │               └─ W6 Production promotion   NOT_STARTED
         └─ WG Optional GPU correspondence mirror     NOT_STARTED / NON_BLOCKING
```

| Stage | Specification | Exit evidence | Blocks |
|---|---|---|---|
| W0A | [Product and evidence contract](00-product-and-evidence-contract.md) | Product fixture, evidence categories, authority and non-goals are selected | W0B |
| W0B | [Original numeric execution and corpus closure](00b-numeric-execution-and-corpus-closure.md) | Immutable rejected-profile evidence; RC1 invalidates promotion use | W0C |
| W0C | [Hydro calibration reclosure](00c-hydro-calibration-reclosure.md) | A justified successor profile, new roots and passing independent hydro discriminator | W1 resume |
| W1 | [Serial CPU DFSPH oracle](01-serial-cpu-dfsph-oracle.md) | `CONTINUUM-WATER-REF-P1 = PASS` on the same-target reference profile | main-roadmap activation, W2, WG |
| W2 | [Deterministic parallel CPU and performance](02-deterministic-parallel-and-performance.md) | Worker/order exactness and standalone `50k` THOTH stop-target PASS | W3 |
| W3 | [One-pass PhysX coupling](03-one-pass-physx-coupling.md) | `CONTINUUM-COUPLING-P1 = PASS` under one composition DAG, exact exchange tuple and one PhysX integration | W4 |
| W4 | [Basin, crate and debug presentation](04-basin-crate-debug-presentation.md) | Production command loop and presentation-independence pass | W5 |
| W5 | [Exact active persistence](05-exact-active-persistence.md) | `CONTINUUM-PERSISTENCE-P1 = PASS` with scheduled checkpoint epochs | W6 |
| W6 | [Production promotion](06-production-promotion.md) | Consumer-backed Accepted decision, explicit fault/capacity profiles and successor combined budget PASS | shipped claim |
| WG | [GPU correspondence mirror](wg-gpu-correspondence.md) | `CONTINUUM-MIRROR-P1` report for named devices | no authority or promotion stage |

## Program invariants

- CPU DFSPH is the only canonical water solver candidate for V1.
- Authoritative private `f64` requires the exact ADR-081 execution profile and
  adversarial Windows/Linux root gate before solver code can promote.
- The accepted state is stable sample ID plus fixed-point position/velocity;
  the next substep starts from that state.
- One physical substep produces one water candidate and one reaction batch;
  water and PhysX publish together or neither publishes.
- Exchange keys use namespace/owners/world/revisions/roots/participants and an
  operation slot, never a generic world or checkpoint generation.
- The production profile pre-admits worst-case capacities, binds the primary
  session fault domain and rebuilds PhysX at fixed checkpoint epochs.
- The first region is sealed and pinned active. Cross-region transfer, halos,
  streaming eviction and sleep conversion are not hidden implementation work.
- Public contracts appear only at W4/W6 with the runtime-bearing basin
  consumer and an Accepted promotion ADR.
- GPU completion, renderer cadence, wall time, worker count and cache warmth do
  not select an authoritative state.
- Heavy corpora, comparison trajectories, captures and performance reports
  stay outside Git; checked-in evidence is bounded hashes/summaries only.

## Frozen W0B inputs

The authoritative W0B document SHA-256 is
`d357bca64983fbd2074961a462743a4fb5d3fedc2af09631d32ec178bd299550`.
Its machine-facing float-profile and corpus roots are recorded inside
[W0B](00b-numeric-execution-and-corpus-closure.md). W1 preflight binds all
three roots. W1-RC1 invalidates their promotion use but does not rewrite this
evidence. W0C must issue successor roots rather than editing history in place.

## Current W1 discriminator

Commit `74730e208cfeb70b05a3ec44b2bb9c2f5002fe97` implements the serial oracle.
On a clean exact-profile run, `CW-FREEFALL-001` passes all 97 canonical frames
and two repeats produce the same trajectory root. `CW-HYDRO-001` then stops at
its first density solve with `WATER_DENSITY_NONCONVERGENCE`: iteration 20 ends
at `74,482,699 ppb` against the `100,000 ppb` threshold. The bounded
[evidence report](../../development/continuum-water-w1-evidence-2026-08-17.md)
records roots and artifact hashes.

Commit `d54e10e55bb20528c4cf485bc3bc6be5a0a6b5c3` adds a clean-tree independent
brute-force audit. It reports `EXACT_MATCH` for all inputs, boundary volumes,
the 20-value global residual curve and four representative complete row
traces. The bounded [RC1 report](../../development/continuum-water-w1-rc1-audit-2026-08-17.md)
therefore routes the next work to W0C rather than a W1 repair.

Commit `328d01c70004bd0c9572ca4dc49891d141f6ba7f` adds exact W0C contribution,
extended-curve and boundary-candidate diagnostics. The bounded
[W0C report](../../development/continuum-water-w0c-hydro-calibration-2026-08-17.md)
rejects a larger iteration ceiling and `ghost-cell-shell-v1`: the candidate
fixes the initial partition and passes step 1, but fails the unchanged-ceiling
soak on step 2; ceilings 100 and 160 only defer failure. The next discriminator
is deterministic pre-equilibrated initialization, not another ceiling or
boundary-scale variant.

W1 remains open and the main roadmap remains inactive. W2, WG, PhysX and GPU
work remain blocked.

## Worktree and main-roadmap protocol

1. Create the water worktree from the documentation checkpoint containing W0A
   and the hash-frozen W0B closure.
2. Record stage state and exact evidence links in this roadmap/task-state;
   never mark a PASS from a type, fixture, compilation or report-only run.
3. After W1 PASS, merge the evidence checkpoint and change the main R8 row from
   `PLANNED / NOT_ACTIVE` to an active integration track. Before that event the
   main roadmap contains only this experimental pointer.
4. Each stage is a coherent commit boundary. Failed evidence leaves the stage
   open and records the smallest discriminator; it does not relax thresholds.
5. Treat the `4/6 ms` 50k number as a standalone stop target. Production also
   requires the mutually exclusive successor `world-dynamics-step` row across
   every substep in one gameplay tick. If that combined gate misses after two
   evidence-backed optimization
   cycles, stop the roadmap as `RESEARCH_ONLY`. GPU authority, a smaller
   production gate or a larger budget requires a new explicit decision.

## Whole-program non-goals

Ocean/weather simulation, fluid networks, cross-region particle transport,
adaptive particles, surface-quality rendering, foam/spray, generic plugin
solver ABI, arbitrary gameplay fluid queries, wet soil, terrain deformation,
full vehicles and production sleep/wake conversion.
