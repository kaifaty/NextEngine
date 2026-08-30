# Nonlocal corrected GPU full step — current task state

| Field | Value |
| --- | --- |
| Status | `AUTHOR_COMPLETE / NCGP1_PHYSICS_REFUTED / REVIEW_PENDING` |
| Updated | `2026-08-30` |
| Task key | `nonlocal-gpu-full-step` |
| Scope | Implement and audit a standalone matrix-free corrected Nonlocal CUDA step for the frozen 50k physical/performance profile |
| Definition of done | NCGP1 ends `SUPPORTED_BOUNDED`, `PHYSICS_REFUTED`, `PERFORMANCE_REFUTED`, or `INCONCLUSIVE` with hash-closed evidence and the bounded independent review |
| Authority | Working context only; SPEC-38, ADR-076/081, FCR0 and the NCGP1 frozen contract outrank this file |

## Resume in 60 seconds

- **Current conclusion:** author result is `PHYSICS_REFUTED`. Exact graph and
  matrix-free HVP pass, but the first translated compressed-pair solver gate
  reaches minimum trust radius at `R_x=1.5036e-5 > 1e-5`; CPU succeeds.
- **Why:** the revision-2 firewall defines minimum radius as failure. A later
  diagnostic independently exhausts 128 HVP on hydrostatic step 12, so no
  authorized ceiling covers even that prefix.
- **Next action:** obtain the one bounded independent review of the frozen
  commit/evidence. Do not run performance or optimize a physically rejected package.
- **Current blocker:** independent review is pending; author implementation is closed.
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
| `docs/development/nonlocal-gpu-full-step-graph-evidence-2026-08-30.md` | `GRAPH_BASELINE_PASS` | 50k plus 43,056 ghosts fits 63.05 MB, exact permutation root and capacity controls |
| `docs/development/nonlocal-gpu-full-step-operator-evidence-2026-08-30.md` | `MATRIX_FREE_OPERATOR_PASS` | dense/CPU/CUDA HVP correspondence and six wrong identities close locally |
| `docs/plans/nonlocal-corrected-gpu-full-step/01-scale-aware-solver-terminal.md` | `FROZEN` | projected `R_x<=1e-5` is the only NCGP1 physical terminal; raw NCGA5 stop remains historical |
| `docs/plans/nonlocal-corrected-gpu-full-step/02-trajectory-corpus.md` | `FROZEN` | fixes exact tiny, 4k and 50k durations/initial states before trajectory code |
| `docs/development/nonlocal-gpu-full-step-evidence-2026-08-30.md` | `AUTHOR_PHYSICS_REFUTED` | exact identities, clean repeats, sanitizer/regression closure and first failing route |

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
| H1 f32 operator plus f64 reductions is physically adequate | tiny CUDA HVP error is `9.33e-7`; energy/gradient also pass | no nonlinear or dynamic trajectory exists | tiny matrix-free solve, then 4k trajectory |
| H2 pressure-product precision is the first physical boundary | NCGA7 changes residual behavior | combined static case still regresses | predeclared f64-pressure discriminator only after primary semantic pass |
| H3 representation/launch cost, not physics, blocks 4/6 ms | the larger support+ghost graph has one exploratory `3.49 ms` call | no percentile and full operator/solver unmeasured | stage-timed 50k full step after physical gate |
| H4 boundary/model behavior blocks before performance | no corrected dynamic sealed-basin evidence exists | FCR objective passes local term controls | independent 4k basin corpus and 50k 240-step run |

## Required context

1. `docs/architecture/agent-routing.md`, SPEC-38 and ADR-076/081.
2. `docs/plans/nonlocal-continuum-formula-reclosure/00-formula-contract.md`.
3. NCGA0 and NCGA1 contracts, task states and independent reviews.
4. `docs/development/task-state/nonlocal-corrected-gpu-assembly-audit.md`
   and every contract/evidence item it marks immutable.
5. `docs/plans/nonlocal-corrected-gpu-full-step/00-physical-performance-contract.md`.

## Next action

1. Run the single independent read-only review on the frozen author package.
2. If it finds an apparatus defect, allow at most one batched repair and one re-review.
3. Otherwise close NCGP1 `PHYSICS_REFUTED`; performance remains `NOT_RUN`.

## Do not retry

- Dense 50k Hessian or host Krylov — capacity and transfer boundary is already decisive.
- SISSM/Chebyshev pressure recurrence — FCR3-B2 reproducibly found non-descent.
- Timing before physical admission — NCGP0 is graph-only and cannot answer full-step cost.

## Handoff

- **Workspace state:** main worktree on `codex/water-research`; frozen author candidate is commit `c7c33e9b` and evidence is the only uncommitted transition.
- **Checks:** two clean Release builds/reports exact; three sanitizer tools zero-error; NCGA0--7 exact; tiny physical gate reproducibly refuted.
- **Remaining risk:** independent review may still find an apparatus defect. Performance and later trajectories are intentionally not run.
- **Promotion needed:** none; the track remains Proposed/report-only.
