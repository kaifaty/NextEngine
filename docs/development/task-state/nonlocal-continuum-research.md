# Nonlocal continuum research — completed task state

| Field | Value |
|---|---|
| Status | `NR4_COMPLETE / NONLOCAL_48K_RECLOSURE_CANDIDATE / REPORT_ONLY` |
| Updated | `2026-08-20` |
| Task key | `nonlocal-continuum-research` |
| Scope | Source-faithful Nonlocal/SISSM baseline and bounded fixed-work GPU optimization research |
| Definition of done | Complete: one frozen NR4 state selected without runtime/public-contract claim |
| Authority | Working context only; Accepted SPEC/ADR and current DFSPH roots outrank this file |

## Resume in 60 seconds

- Branch/worktree: `codex/nonlocal-continuum-n0` at
  `/home/kaifaty/Documents/NextEngine-nonlocal-continuum-n0`.
- NR4 selected `NONLOCAL_48K_RECLOSURE_CANDIDATE`. This authorizes a fresh
  Proposed solver/profile reclosure and independent corpus only; it is not W2,
  runtime, GPU-authority or production credit.
- Retained identity: `nuv-gather-directed-r0 + pointer-swap-o1 +
  nuv-terms-specialized-o2 + stable-sample-v0`.
- Final retained p95: `2.075040 ms` water-16k, `4.019520 ms` water-48k and
  `7.084416 ms` viscous-16k. HN-3 geometric-mean fixed-work speedup is
  `3.27688x`.
- Final profiler attribution assigns about `84.0%` of water-48k and `94.3%` of
  viscous-16k GPU kernel time to density, incompressibility and viscosity pair
  kernels. Nsight Compute counters remain unavailable under
  `ERR_NVGPUCTRPERM`; no counter-derived claim exists.
- Negative evidence remains negative: source-atomic stiff surface is a
  baseline mismatch, O3 endpoint pre-addition is a numeric mismatch and O4
  cell-sorted storage is exact but slower on the coherent frozen lattice.
- DFSPH W0F/G/H/I roots and `CONTINUUM-WATER-REF-P1=PASS` remain unchanged.
- The separate [performance reclosure](nonlocal-continuum-performance.md) is
  now complete at `NONLOCAL_50K_FIXED_WORK_RECLOSURE_CANDIDATE`; it does not
  reopen this completed NR0–NR4 record or grant W2/runtime credit.

## Decisions

### D-NR-001 — Quarantined standalone tool

Keep the experiment in `crates/continuum-water/tools/nonlocal-feasibility`.
PeriDyno is a pinned external reference, not an engine dependency. The tool
publishes reports only and owns no canonical state.

### D-NR-002 — Fixed work before algorithm changes

NR1/NR2 use fixed terms and iteration counts. Adaptive stopping, warm starts
and alternative optimizers require separate identities and evidence.

### D-NR-003 — Preserve failed identities

`source-atomic-v0` remains failed on stiff surface and is an HN-3 denominator
only on its passing profiles. `nuv-unique-pair-segmented-o3` remains rejected
because altered floating association exceeds the frozen surface bounds.

### D-NR-004 — Retain directed owner gather

`nuv-gather-directed-r0` is the correctness baseline. It passes independent CPU
algebra, CUDA tiny `11/11`, exact stiff-surface repeats and full controls.

### D-NR-005 — Retain O1 and O2

`pointer-swap-o1` and `nuv-terms-specialized-o2` preserve exact results and
their adjacent performance gates. Copy handoff and runtime dispatch remain
selectable rollback comparators.

### D-NR-006 — Reject O4 for the frozen lattice

`cell-sorted-o4` passes map, CSR, capacity and exact-output gates but regresses
all three measured profiles. The stable fixture is already spatially coherent;
do not generalize this result to dynamically disordered particles.

### D-NR-007 — Close fixed-work optimization

The final profiler leaves no untested non-pair stage owning the required 20%,
so O5 is not admitted. O6 has no bounded packed-field hypothesis. NR3 is not
needed for the fixed-work NR4 decision.

### D-NR-008 — Select the 48k reclosure state

NR4 selects `NONLOCAL_48K_RECLOSURE_CANDIDATE`: correctness passes, HN-3 is
`3.27688x`, and water-48k p95 is `4.019520 ms` against the `8 ms` research
cutoff. The stronger state wins over the also-satisfied local-domain cutoff.
Unavailable optional hardware counters do not make the evidence incomplete.

## Evidence

| Evidence | Result |
|---|---|
| [Research contract](../../plans/nonlocal-continuum/00-research-contract.md) | `EXECUTED / 48K_RECLOSURE_CANDIDATE` |
| [Source audit](../nonlocal-unified-continuum-source-audit-2026-08-19.md) | primary paper and pinned implementation inspected |
| [NR1 evidence](../nonlocal-continuum-nr1-baseline-evidence-2026-08-19.md) | source-atomic stiff-surface mismatch retained |
| [NR1-RC1 evidence](../nonlocal-continuum-nr1-rc1-evidence-2026-08-19.md) | directed-gather reclosure PASS |
| [O1 evidence](../nonlocal-continuum-nr2-o1-evidence-2026-08-20.md) | pointer swap retained |
| [O2 evidence](../nonlocal-continuum-nr2-o2-evidence-2026-08-20.md) | term specialization retained |
| [O3 evidence](../nonlocal-continuum-nr2-o3-evidence-2026-08-20.md) | segmented endpoint pre-addition rejected |
| [O4/final NR2 evidence](../nonlocal-continuum-nr2-o4-evidence-2026-08-20.md) | stable storage retained; fixed work complete |
| [NR4 decision](../nonlocal-continuum-nr4-decision-2026-08-20.md) | `NONLOCAL_48K_RECLOSURE_CANDIDATE` selected |

## Do not retry or infer

- do not relabel 48k/`8 ms` evidence as the exact 50k `4/6 ms` gate;
- do not widen O3 tolerances or rerun source atomics as a correctness remedy;
- do not infer occupancy/throughput from unavailable Nsight counters;
- do not implement Pairwise Descent without an auditable public source;
- do not infer plasticity, thermochemistry, solids or a universal solver;
- do not start W3, persistence, public contracts or runtime integration.

## Handoff

- **Closure:** NR0–NR4 are complete; the fixed-work research question has one
  frozen answer and no remaining action in this task.
- **Verification:** all executable checks are recorded in the linked evidence;
  this closure changes documentation only.
- **Next owner:** the separately rooted performance reclosure roadmap must
  establish exact 50k/dynamic profiles before any new optimization claim.
