# Nonlocal continuum — bounded research roadmap

Status: `NR2_O2_RETAINED_TERM_SPECIALIZATION / O3_NEXT / NO_W2_CREDIT`

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
         └─ NR2 Fixed-iteration optimization O2 RETAINED / O3 NEXT
             └─ NR3 Algorithm-changing probes NOT_STARTED / CONDITIONAL
                 └─ NR4 Architecture decision NOT_STARTED
```

| Stage | Specification | Exit evidence | Credit |
|---|---|---|---|
| NR0 | [Research contract](00-research-contract.md) and [source audit](../../development/nonlocal-unified-continuum-source-audit-2026-08-19.md) | hypotheses, provenance, fixtures, metrics and stop states are explicit | documentation only |
| NR1 | [Baseline and oracle](01-source-faithful-baseline-and-oracle.md) and [evidence](../../development/nonlocal-continuum-nr1-baseline-evidence-2026-08-19.md) | tiny, water and viscous controls pass; stiff surface repeated-output control fails | report only |
| NR1-RC1 | [Deterministic accumulation reclosure](03-nr1-deterministic-accumulation-reclosure.md), [candidate research](../../development/nonlocal-continuum-accumulation-reclosure-research-2026-08-19.md) and [execution evidence](../../development/nonlocal-continuum-nr1-rc1-evidence-2026-08-19.md) | `nuv-gather-directed-r0` passes CPU algebra, 11/11 CUDA tiny cases, exact two-/twenty-iteration surface repeats and all full controls | report only |
| NR2 | [GPU optimization discriminators](02-gpu-optimization-discriminators.md), [O1 pointer-swap contract](04-nr2-o1-pointer-swap.md), [O1 evidence](../../development/nonlocal-continuum-nr2-o1-evidence-2026-08-20.md), [O2 term-specialization contract](05-nr2-o2-term-specialization.md) and [O2 evidence](../../development/nonlocal-continuum-nr2-o2-evidence-2026-08-20.md) | O2 retains exact compile-time viscosity specialization with unchanged memory, same-process profiler attribution and `1.0635x` O2-only HN-3 adjacent total-p95 geometric speedup; O3 is next | report only |
| NR3 | [Algorithm-changing probes](02-gpu-optimization-discriminators.md#nr3-algorithm-changing-probes) | a separately labelled convergence/algorithm candidate passes its own oracle | no fixed-iteration credit |
| NR4 | [Decision contract](00-research-contract.md#decision-states) | exactly one predeclared decision state is selected from complete evidence | may authorize a later Proposed reclosure only |

## Relationship to water W2

NR0–NR4 are one alternative research branch after the measured CPU and direct
GPU W2 misses. They do not close W2. A successful NR4 result may justify a new
Proposed solver/profile decision with fresh roots and a fresh correctness
corpus. Until that decision lands, W3 coupling, public contracts, persistence
and production promotion remain blocked.

The NR2 `8 ms` threshold is a feasibility cutoff, not the existing `4/6 ms`
standalone performance PASS. The eventual integrated consumer would still
need the ADR-081 successor `world-dynamics-step` budget.

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
