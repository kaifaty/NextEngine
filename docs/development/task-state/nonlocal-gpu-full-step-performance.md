# Nonlocal GPU full-step performance — current task state

| Field | Value |
| --- | --- |
| Status | `ACTIVE / NCGP4_UNPRECONDITIONED_SELECTED` |
| Updated | `2026-08-31` |
| Task key | `nonlocal-gpu-full-step-performance` |
| Scope | Diagnose the corrected compensated solver work ceiling, close the correctness corpus, then measure the 50k full GPU step |
| Definition of done | complete frozen correctness followed by two-process 50k p95/p99 evidence, or the first honest bounded refutation |
| Authority | Working context only; SPEC-38, ADR-076/081, frozen NCGP1--NCGP4 contracts and exact evidence outrank this file |

## Resume in 60 seconds

- **Goal:** measure the complete corrected Nonlocal GPU water step on 50,000
  particles against `p95 <= 4 ms`, `p99 <= 6 ms` on RTX 3080.
- **Current boundary:** performance is still `NOT_RUN`; only the neighbor stage
  has prior `~1.0--1.18 ms p95` evidence.
- **First failing fact:** exact corrected NCGP3 hydrostatic hold fails GPU step
  39 at 126/128 HVP; CPU succeeds. NCGP3 is closed `INCONCLUSIVE` because its
  240-step ordering, reverse-energy apparatus, result closure and rollback
  handling were incomplete.
- **Current action:** NCGP4 revision 2 selects the retained unpreconditioned
  solver after the exact trace showed it completes step 39 in 69 HVP while
  scalar Jacobi fails at 126 HVP. Run the complete 240-step 4k corpus at
  budgets `{32,64,128}` before any 50k timing.
- **Product ceiling:** tool-only Proposed benchmark. CPU DFSPH remains fallback;
  no Rust/public/runtime/PhysX/renderer contract changes.

## Required context

- `docs/plans/nonlocal-gpu-full-step-performance/00-ncgp4-solver-diagnosis-contract.md`
- `docs/plans/nonlocal-gpu-full-step-performance/01-unpreconditioned-selection.md`
- `docs/development/task-state/nonlocal-gpu-compensated-scale.md`
- `docs/development/nonlocal-gpu-compensated-scale-evidence-2026-08-30.md`
- `docs/plans/nonlocal-gpu-compensated-scale/00-compensated-scale-contract.md`
- `docs/plans/nonlocal-gpu-compensated-scale/01-graph-control-corrigendum.md`
- `docs/plans/nonlocal-gpu-compensated-scale/02-boundary-transaction-fixtures.md`
- `docs/plans/nonlocal-gpu-compensated-scale/03-corrected-fcr-profile-corrigendum.md`
- `docs/plans/nonlocal-gpu-compensated-scale/04-review-repair-contract.md`

## Material evidence and decisions

### D-001 — Diagnose before changing solver work

- **Observation:** the deterministic work ceiling gives only aggregate HVP and
  outer counts; it does not show whether inner conditioning, outer rejection,
  numerical cancellation or accounting caused the failure.
- **Evidence:** reviewer hydro240 stdout SHA-256
  `313d5eb78babfd34b407876248c90881dca0a5c75f63308e707e781e8184c834`;
  work root `0687e23dc524a83858a04d58b9cfd524a0e639c3ba282bf4cb7429a383c2d128`.
- **Decision:** freeze NCGP4 and instrument the exact pre-step-39 state before
  any preconditioner, precision, budget or tolerance change.
- **Rejected:** increasing 128 HVP, timing the failed route, treating dam step 6
  as first failure or guessing that scalar Jacobi is the cause.
- **Reconsider when:** a root-closed trace and independent same-state operator
  comparison select one frozen hypothesis.

### D-002 — Remove harmful scalar Jacobi from the scalable path

- **Observation:** on the exact compensated pre-step-39 state, scalar Jacobi
  fails after 126 HVP and 28 outer trials; the retained unpreconditioned path
  succeeds after 69 HVP and 19 outer trials.
- **Evidence:** diagnostic stdout SHA-256
  `78b40392e0981eec77e37c514ce39cae78172c642edfa9ead3de0d737e342f47`,
  result root `2f376da73d4edb8a09d5a223b43870415cfdd3e685601cb5e639b4fff52e4cce`;
  HVP relative L2 against long double `1.4262815435566996e-6`, cosine loss
  `9.8629643948550116e-13` and exact active signature.
- **Conclusion:** scalar diagonal construction is correctly counted but harms
  this coupled operator. The failure is not evidence for pressure-f64 or a
  changed trust-region tolerance.
- **Decision:** freeze revision 2 with the existing unpreconditioned
  Steihaug--Toint profile as the one selected solver repair. Keep physics,
  tolerances and the 128-HVP ceiling unchanged.
- **Rejected:** block-Jacobi, pressure-f64, higher HVP ceiling, radius or
  acceptance tuning.
- **Reconsider when:** only if the complete 4k corpus exposes a different
  first failure with root-closed evidence; no second solver repair is allowed
  by NCGP4.

## Hypothesis ledger

| ID | Hypothesis | Current evidence | Next discriminator |
| --- | --- | --- | --- |
| H1 | scalar Jacobi leaves the coupled operator ill-conditioned | falsified as frozen: unpreconditioned succeeds with 69 HVP; no block-Jacobi | closed |
| H2 | repeated trust-region rejects/radius shrink consume work | present in Jacobi trace, but unchanged unpreconditioned rules succeed | no tuning; observe full corpus |
| H3 | binary32 operator cancellation creates a residual floor | falsified on witness by `1.43e-6` HVP relative L2 and exact active signature | closed; no pressure-f64 |
| H4 | stopping/work accounting rejects admissible execution | no count mismatch; real three-HVP diagonal work is counterproductive | remove Jacobi from selected path |

## Do not retry

- NCGP3 repair/re-review; its allowance is exhausted.
- raw FCR1 profile as physical evidence;
- global one-part f32 state, host-origin localization or high-only graph/contact;
- tolerance widening, HVP above 128 or pressure-f64 without isolated pressure
  operator error;
- neighbor-only timing as water/frame performance.

## Next action

1. Bind revision 2 into the NCGP4 target and rebuild from a clean tree.
2. Run retained controls, then the complete 4k corpus for budgets 32, 64 and
   128 in ascending order, selecting the smallest full-corpus PASS.
3. Run 16k/50k correctness and the 240-step 50k sealed basin.
4. Measure two fresh processes only if every correctness gate passes.
