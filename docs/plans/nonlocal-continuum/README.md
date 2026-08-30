# Nonlocal continuum — bounded research roadmap

Status: `NR4_COMPLETE / NONLOCAL_48K_RECLOSURE_CANDIDATE / NO_W2_CREDIT`

This directory specifies a report-only evaluation of the method described in
*A Nonlocal Unified Variational Framework for Free Surface Flows*. The work is
an isolated numerical candidate. It does not replace the frozen DFSPH water
profile, amend SPEC-38 or ADR-076, add a runtime/backend dependency, or make a
general material-solver claim.

The governing constraints remain [SPEC-38](../../architecture/38-continuum-material-physics.md),
[ADR-076](../../architecture/adr/076-continuum-material-physics-track.md) and
[ADR-081](../../architecture/adr/081-world-dynamics-gap-closure-and-promotion-guardrails.md).
The passing DFSPH Linux corpus remains the correctness reference and fallback.
Nonlocal results cannot inherit its roots or ProductCheck credit.

## Research question

Can a source-faithful, fixed-iteration Nonlocal/SISSM implementation be made
fast enough on the current Linux RTX 3080 host to justify a separately rooted
continuum profile, while retaining finite, conservative and independently
reproduced pair/iteration behavior?

The first discriminator uses clean water because that provides the existing
48k comparison scale. Viscosity and surface tension are secondary profiles
used to test the actual advantage of the unified formulation. Plasticity,
temperature, phase change, solids and a universal particle record are not part
of this experiment.

## Stage graph

```text
NR0 Research contract and source audit       SPECIFIED / DOCUMENTATION
 └─ NR1 Source-faithful baseline + CPU oracle  BASELINE_MISMATCH
     └─ NR1-RC1 owner-only gather reclosure    RECLOSED / PASS
         └─ NR2 Fixed-iteration optimization COMPLETE / 3.27688x HN-3
             └─ NR3 Algorithm-changing probes SKIPPED / NOT_REQUIRED_FOR_NR4
                 └─ NR4 Architecture decision COMPLETE / 48K_RECLOSURE_CANDIDATE
```

| Stage | Specification | Exit evidence | Credit |
|---|---|---|---|
| NR0 | [Research contract](00-research-contract.md) and [source audit](../../development/nonlocal-unified-continuum-source-audit-2026-08-19.md) | hypotheses, provenance, fixtures, metrics and stop states are explicit | documentation only |
| NR1 | [Baseline and oracle](01-source-faithful-baseline-and-oracle.md) and [evidence](../../development/nonlocal-continuum-nr1-baseline-evidence-2026-08-19.md) | tiny, water and viscous controls pass; stiff surface repeated-output control fails | report only |
| NR1-RC1 | [Deterministic accumulation reclosure](03-nr1-deterministic-accumulation-reclosure.md), [candidate research](../../development/nonlocal-continuum-accumulation-reclosure-research-2026-08-19.md) and [execution evidence](../../development/nonlocal-continuum-nr1-rc1-evidence-2026-08-19.md) | `nuv-gather-directed-r0` passes CPU algebra, 11/11 CUDA tiny cases, exact two-/twenty-iteration surface repeats and all full controls | report only |
| NR2 | [GPU optimization discriminators](02-gpu-optimization-discriminators.md), [O1 pointer-swap contract](04-nr2-o1-pointer-swap.md), [O1 evidence](../../development/nonlocal-continuum-nr2-o1-evidence-2026-08-20.md), [O2 term-specialization contract](05-nr2-o2-term-specialization.md), [O2 evidence](../../development/nonlocal-continuum-nr2-o2-evidence-2026-08-20.md), [O3 layout contract](06-nr2-o3-accumulation-layout-tournament.md), [O3 evidence](../../development/nonlocal-continuum-nr2-o3-evidence-2026-08-20.md), [O4 locality contract](07-nr2-o4-cell-sorted-locality.md) and [O4/final NR2 evidence](../../development/nonlocal-continuum-nr2-o4-evidence-2026-08-20.md) | O3 fails stiff-surface correspondence. O4 is exact but slower, so stable-sample remains retained. Final gather/swap/specialized/stable reaches `3.27688x` HN-3 geometric mean, water-48k `4.019520 ms` p95 and complete final profiler attribution | report only |
| NR3 | [Algorithm-changing probes](02-gpu-optimization-discriminators.md#nr3-algorithm-changing-probes) | a separately labelled convergence/algorithm candidate passes its own oracle | no fixed-iteration credit |
| NR4 | [Decision contract](00-research-contract.md#decision-states) and [decision evidence](../../development/nonlocal-continuum-nr4-decision-2026-08-20.md) | `NONLOCAL_48K_RECLOSURE_CANDIDATE` selected from complete fixed-work evidence | authorizes a later Proposed reclosure and fresh corpus only |

## Relationship to water W2

NR0–NR4 are one alternative research branch after the measured CPU and direct
GPU W2 misses. They do not close W2. NR4 selected
`NONLOCAL_48K_RECLOSURE_CANDIDATE`, which justifies drafting a new Proposed
solver/profile decision with fresh roots and a fresh correctness corpus. It
does not itself land that decision. W3 coupling, public contracts, persistence
and production promotion remain blocked.

The separately rooted follow-up is the
[Nonlocal performance reclosure roadmap](../nonlocal-continuum-performance/README.md).
It begins at exact-50k/dynamic corpus closure and cannot relabel this 48k
feasibility result as the production `4/6 ms` gate.

The NR2 `8 ms` threshold is a feasibility cutoff, not the existing `4/6 ms`
standalone performance PASS. The eventual integrated consumer would still
need the ADR-081 successor `world-dynamics-step` budget.

## Corrected GPU correspondence follow-up

A later report-only audit now separates the corrected FCR objective from the
historical stopped SISSM solver. NCGA0 reviews corrected scalar/full-pair terms
`GO`; NCGA1 reviews exact integer GPU neighborhood construction `GO`; NCGA2
keeps a real strict-f32 combined-Hessian element failure; and NCGA3 author
evidence finds that miss negligible across thirteen Hessian actions, one
norm-regularized response and eight `50 um`-capped integer steps. Binary64
pressure products close the old element gate but are retained only as a
fallback because they do not improve the bounded response materially.

See the [NCGA3 evidence](../../development/nonlocal-corrected-gpu-consequence-evidence-2026-08-30.md).
Independent NCGA3 review, a corrected nonlinear solver, physical trajectories
and full 50k assembly/solve timing remain open. This follow-up grants no NR4,
W2 or ProductCheck credit and does not change CPU DFSPH product authority.

## Implementation boundary

The intended implementation path is a standalone tool under
`crates/continuum-water/tools/nonlocal-feasibility`. It is not a Cargo workspace
member, does not link into `game`, `headless`, runtime or contracts, owns no
canonical state and writes bounded JSON reports only to stdout. Generated
binaries, profiles and raw reports remain outside Git.

PeriDyno is an external reference, not a repository dependency or production
backend. The lab reimplements the published equations and records provenance.
If any upstream source is copied rather than independently reimplemented, the
Apache-2.0 notice and exact copied-file provenance must be added before that
change is committed.

## Invariants

- DFSPH W0F/G/H/I roots and `CONTINUUM-WATER-REF-P1=PASS` remain unchanged.
- GPU execution is report-only and cannot publish water, impulses, commands,
  events or checkpoints.
- Timers, convergence telemetry and device completion order never enter an
  authoritative result.
- Fixed-iteration and adaptive-iteration results have different profile IDs
  and cannot share a performance claim.
- No run continues after nonfinite state, bound overflow, invalid neighbors or
  failed oracle comparison.
- No long corpus or percentile campaign starts before its smaller control and
  stop gate pass.
- Pairwise Descent is not implemented from a title or announcement; it enters
  the candidate set only after a public paper or code release can be audited.

## Non-goals

Production integration, PhysX coupling, save/replay schemas, Windows execution,
multi-GPU, adaptive particles, ML warm starts, rendering, fluid/solid topology,
plasticity, thermochemistry and changing current ProductCheck results.
