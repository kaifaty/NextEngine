# Nonlocal corrected GPU full step — current task state

| Field | Value |
| --- | --- |
| Status | `ACTIVE / NCGP1_CONTRACT_FROZEN / IMPLEMENTATION_PENDING` |
| Updated | `2026-08-30` |
| Task key | `nonlocal-gpu-full-step` |
| Scope | Implement and audit a standalone matrix-free corrected Nonlocal CUDA step for the frozen 50k physical/performance profile |
| Definition of done | NCGP1 ends `SUPPORTED_BOUNDED`, `PHYSICS_REFUTED`, `PERFORMANCE_REFUTED`, or `INCONCLUSIVE` with hash-closed evidence and the bounded independent review |
| Authority | Working context only; SPEC-38, ADR-076/081, FCR0 and the NCGP1 frozen contract outrank this file |

## Resume in 60 seconds

- **Current conclusion:** the product-oriented physical gate is now separate
  from the retained NCGA2 elementwise failure; implementation may replace the
  90 GB dense Hessian with a matrix-free GPU operator.
- **Why:** NCGA3 bounds the old element miss at `1.0643e-6` HVP relative error
  and sub-micrometre local drift, while NCGA5--7 show that raw static residual
  precision is not a useful 50k representation selector.
- **Next action:** implement the tool-only profile, independent CPU oracle and
  preallocated scalable GPU workspace without changing NCGA0--7.
- **Current blocker:** matrix-free full-step code and trajectory evidence do
  not yet exist.
- **Do not retry:** dense Hessian, source-shaped SISSM/Chebyshev, tolerance
  widening or adding NCGP0 neighbor time to an unmeasured solve.
- **Reconsider when:** NCGP1 physical evidence selects a specific operator,
  solver, boundary or precision failure.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| `docs/plans/nonlocal-corrected-gpu-full-step/00-physical-performance-contract.md` | `FROZEN` | fixes profile, physical gates, performance window and stop rules before code |
| `docs/development/task-state/nonlocal-corrected-gpu-audit.md` | `NCGA0_REVIEWED_GO` | corrected scalar/full-pair terms may be reused as immutable formula boundary |
| `docs/development/task-state/nonlocal-corrected-gpu-neighborhood-audit.md` | `NCGA1_REVIEWED_GO` | exact integer device graph may be reused as immutable indexing boundary |
| `docs/development/task-state/nonlocal-corrected-gpu-assembly-audit.md` | `NCGA7_FAILED / NCGA2_IMMUTABLE` | old dense/static precision ladder stays closed; matrix-free physical claim is new |
| `docs/development/nonlocal-corrected-gpu-neighborhood-performance-evidence-2026-08-30.md` | `50K_SUPPORTED_BOUNDED` | graph-only p95 is `1.02..1.18 ms`; duplicate traversal is the first measured graph bottleneck |

## Decisions that still constrain the work

### D-001 — Keep the campaign tool-only

- **Observation:** SPEC-38 and ADR-076 remain Proposed and CPU DFSPH is the
  product candidate.
- **Decision:** implement only under the standalone Nonlocal feasibility tool;
  no Rust contracts, runtime, PhysX, renderer or canonical state changes.
- **Reconsider when:** a reviewed NCGP1 result informs a separately accepted
  promotion decision.

### D-002 — Select physical consequences before raw static residuals

- **Observation:** strict f32 misses one old Hessian scalar and raw static
  convergence, but bounded HVP/state consequences remain tiny.
- **Decision:** retain old failures and gate NCGP1 on independent HVP, state,
  balance, boundary and trajectory observables.
- **Rejected alternatives:** relabel NCGA2/5--7, widen their gates, or declare
  the near miss harmless without trajectory evidence.

### D-003 — Matrix-free owner gather is the scalable identity

- **Observation:** 50k dense f32 Hessian would require roughly 90 GB and the
  current solver copies it to a CPU controller.
- **Decision:** implement CSR energy/gradient/diagonal/HVP with owner gathers,
  reverse adjacency and no floating endpoint atomics.
- **Rejected alternatives:** dense blocks, host matvec, stopped SISSM split or
  unordered endpoint scatter.

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1 f32 operator plus f64 reductions is physically adequate | NCGA3 HVP/state consequence is tiny | no dynamic trajectory exists | tiny matrix-free correspondence, then 4k trajectory |
| H2 pressure-product precision is the first physical boundary | NCGA7 changes residual behavior | combined static case still regresses | predeclared f64-pressure discriminator only after primary semantic pass |
| H3 representation/launch cost, not physics, blocks 4/6 ms | exact graph already costs 1.02--1.18 ms p95 and duplicate traversal dominates | full operator/solver unmeasured | stage-timed 50k full step after physical gate |
| H4 boundary/model behavior blocks before performance | no corrected dynamic sealed-basin evidence exists | FCR objective passes local term controls | independent 4k basin corpus and 50k 240-step run |

## Required context

1. `docs/architecture/agent-routing.md`, SPEC-38 and ADR-076/081.
2. `docs/plans/nonlocal-continuum-formula-reclosure/00-formula-contract.md`.
3. NCGA0 and NCGA1 contracts, task states and independent reviews.
4. `docs/development/task-state/nonlocal-corrected-gpu-assembly-audit.md`
   and every contract/evidence item it marks immutable.
5. `docs/plans/nonlocal-corrected-gpu-full-step/00-physical-performance-contract.md`.

## Next action

1. Add the exact profile and independent CPU reference/corpus.
2. Add the preallocated device workspace and reviewed two-pass graph baseline.
3. Prove the matrix-free HVP on tiny cases before any trajectory or timing.

## Do not retry

- Dense 50k Hessian or host Krylov — capacity and transfer boundary is already decisive.
- SISSM/Chebyshev pressure recurrence — FCR3-B2 reproducibly found non-descent.
- Timing before physical admission — NCGP0 is graph-only and cannot answer full-step cost.

## Handoff

- **Workspace state:** main worktree on `codex/water-research`; contract-only checkpoint pending.
- **Checks:** not run; implementation has not started.
- **Remaining risk:** boundary model, dynamic convergence, work budget and complete 50k cost are untested.
- **Promotion needed:** none; the track remains Proposed/report-only.
