# Nonlocal continuum — performance reclosure roadmap

Status: `ACTIVE / NP0_COMPLETE / NP1_P1_P2_RETAINED / P3_NEXT / REPORT_ONLY / NO_W2_CREDIT`

This roadmap follows the closed
[NR4 `NONLOCAL_48K_RECLOSURE_CANDIDATE` decision](../../development/nonlocal-continuum-nr4-decision-2026-08-20.md).
It does not reopen NR0–NR4, replace DFSPH, amend SPEC-38/ADR-076, authorize GPU
runtime authority or claim that the existing 48k result passes the exact 50k
`4/6 ms` p95/p99 target.

The research basis and ranked algorithm choices are recorded in the
[dated source audit](../../development/nonlocal-continuum-performance-roadmap-research-2026-08-20.md).
The frozen hypotheses, profile families, measurement rules and terminal states
are in the [performance research contract](00-performance-research-contract.md).
The exact generator, trace and persistent-runner contract is in the
[NP0 corpus specification](01-np0-corpus-and-baseline.md).
P1 arithmetic, timing and rollback rules are frozen in the
[fused owner-term specification](02-np1-p1-fused-owner-terms.md).
P2 representation, fallback and capacity rules are frozen in the
[compact CSR specification](03-np1-p2-compact-csr.md).

## Outcome

Produce one evidence-backed answer to this bounded question:

> Can a separately rooted Nonlocal GPU profile meet the standalone exact-50k
> Linux RTX 3080 `4/6 ms` p95/p99 stop target on both coherent and
> dynamically representative states, or should the method remain a smaller
> local-domain/research candidate?

Even a positive answer is not W2 PASS. Production still requires the full
SPEC-38 correctness corpus, CPU/canonical authority decision, Windows/Linux
root work, PhysX coupling, persistence and the ADR-081 combined
`world-dynamics-step` budget. Windows is outside this roadmap.

## Stage graph

```text
NR4 48k feasibility decision                    COMPLETE
  └─ NP0 exact-50k/dynamic workload reclosure   COMPLETE
      └─ NP1 exact-work GPU tournament          IN_PROGRESS
          ├─ P1 pair-term traversal fusion       COMPLETE / RETAINED
          ├─ P2 compact CSR and memory path      COMPLETE / RETAINED
          ├─ P3 dynamic locality tournament      NEXT
          └─ P4 certified neighbor reuse
              ├─ NP4 fixed-work decision path   BLOCKED_BY_EVIDENCE
              ├─ NP2 algorithmic work reduction CONDITIONAL
              │   ├─ residual/adaptive exit
              │   ├─ safeguarded Anderson probe
              │   └─ Pairwise Descent watch gate
              │       └─ NP4 algorithm decision path
              └─ NP3 scale reduction            CONDITIONAL / SEPARATE_CREDIT
```

| Stage | Required output | Exit gate | Credit |
|---|---|---|---|
| NP0 | exact 50k coherent/permuted/advected profiles, multi-substep runner, p99-capable retained baseline | all input/topology/oracle hashes repeat; stage timing and dynamic pair distributions exist | corpus only |
| NP1 | ordered exact-work tournaments with adjacent rollback identities | retained stack improves the exact 50k dynamic and coherent totals without numeric/capacity regression | fixed-work performance only |
| NP2 | separately identified residual/adaptive/acceleration candidates | stationary-result and physical-quality corpus passes; end-to-end time includes convergence checks | algorithm candidate only |
| NP3 | active-domain or adaptive-resolution feasibility plan | conservation/identity/error receipts and coarse/fine comparison pass | scale-research only; never 50k credit |
| NP4 | exactly one terminal state from the research contract | complete reproducible evidence and architecture review | may authorize a later Proposed reclosure only |

## Immediate queue

NP0 is complete. Its implementation and hashes are frozen in the
[NP0 corpus specification](01-np0-corpus-and-baseline.md), and its timings,
negative result and raw-report hashes are in the
[dated evidence](../../development/nonlocal-continuum-np0-evidence-2026-08-20.md).
P1 is retained by its
[exact-work evidence](../../development/nonlocal-continuum-np1-p1-evidence-2026-08-20.md);
P2 is retained by its
[exact-work evidence](../../development/nonlocal-continuum-np1-p2-evidence-2026-08-20.md);
P3 now uses fused traversal plus compact CSR as its adjacent denominator.

### NP0 — Reclose the measured workload

1. Add exact `50,000` coherent water and 16k coupled controls without changing
   the closed v0 profiles.
2. Add a stable-ID-permuted version of the same geometry to isolate memory
   order from physics.
3. Add a hash-bound advected multi-substep corpus whose neighbor list is built
   from each prior substep state and frozen only within that nonlinear solve.
4. Extend reports with p99-ready raw totals, pair/degree distributions,
   storage distance, neighbor rebuild reason and amortized per-substep cost.
5. Run the retained NR4 stack in one persistent same-process benchmark to
   establish the new denominator.

NP0 is documentation/corpus/measurement work, not an optimization. If the new
retained baseline fails correctness or cannot be reproduced, NP1 does not
start.

### NP1 — Exact-work GPU tournament

Execute candidates in this order; retain at most one result per adjacent gate:

1. `fused-owner-terms-p1`: traverse the directed CSR once for compatible active
   terms, keep per-term accumulators separate, combine in the retained stage
   order and preserve owner-only writes.
2. `compact-csr-u16-p2`: checked `u16` neighbor IDs for the `<= 50k` profiles,
   `u32` offsets, aligned SoA loads and compile-time block-size variants. The
   100k stress profile remains `u32`.
3. `dynamic-cell-local-p3`: retest stable cell/radix storage only on permuted
   and advected inputs; coherent O4 remains the negative control and sort/map
   cost is included.
4. `verlet-skin-p4`: reuse a canonical superset CSR under a fixed displacement
   certificate, exact current-horizon filtering and canonical active-neighbor
   order.

Shared-memory tiles, a persistent cooperative kernel or CUDA Graphs are
admitted only after an NP1 profiler proves reusable data or a non-pair stage
owns at least `15%` of the retained total. Density is not fused across its
global dependency. Atomics, unique-pair pre-addition and altered reduction
trees do not return without a new numeric specification.

### NP2 — Reduce nonlinear work

NP2 first derives and freezes the residual. Fixed and adaptive identities
remain distinct.

1. high-iteration CPU `f64` stationary-result corpus;
2. normalized fixed-point/KKT residual, energy safeguard, check cadence,
   min/max iteration and typed stagnation/divergence failure;
3. adaptive exit with the complete reduction/synchronization cost;
4. guarded Anderson acceleration against the same stationary corpus;
5. Pairwise Descent audit only after a public primary paper or code release.

Warm start remains deferred unless its future-affecting state is either
reconstructed solely from canonical input or added through a separate state,
persistence and replay decision.

### NP3 — Reduce represented world work

NP3 is conditional and separately credited. It may study a high-fidelity local
Nonlocal domain coupled to a coarse far field, followed later by true adaptive
material samples. It cannot make the 50k benchmark pass by executing fewer
samples.

Before code, an NP3 profile closes stable identity, mass, linear/angular
momentum, energy/error, split/merge/handoff receipts and repeated transition
bounds. The Dual-SPH fission-fusion implementation is not a portable solution:
it adapts auxiliary pressure samples, not Nonlocal material state.

## Stop discipline

- Tiny, stiff-surface i2 and dynamic topology gates precede timing.
- One exact-work candidate failing numeric correspondence stops that candidate
  family; tolerances are not widened after failure.
- Two consecutive retained NP1 candidates below `5%` adjacent total gain stop
  exact-work tuning unless a new profiler stage owns at least `15%`.
- A candidate regressing either coherent or advected exact-50k p95 by more than
  `2%` is rejected even if another fixture improves.
- Temporary memory above the frozen per-profile capacity or any unbounded pair
  list rejects before a long run.
- Only retained finalists receive the `>= 512`-sample p95/p99 campaign.
- NP3, ML, multi-GPU and runtime integration cannot be invoked as a rescue for
  an NP0/NP1 correctness failure.

## Fallback and authority

DFSPH remains the correctness reference and fallback. The standalone lab may
produce bounded JSON and profiler reports only. It cannot publish water state,
rigid reactions, commands, events, saves or ProductCheck results.
