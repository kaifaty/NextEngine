# Nonlocal nonlinear solver research -- current task state

| Field | Value |
|---|---|
| Status | `ACTIVE / NSR3A1_PASS / NSR3A2_HESSIAN_TAPE` |
| Updated | `2026-08-20` |
| Task key | `nonlocal-nonlinear-solver-research` |
| Scope | Fundamental solver research over the verified Nonlocal variational objective, isolated from runtime and the stopped SISSM lineage |
| Definition of done | NSR0--NSR6 select a production-roadmap candidate or stop at an exact reproducible boundary |
| Authority | Working context only; Accepted architecture, SPEC-38/ADR-076 and the frozen stage contracts outrank this file |

## Resume in 60 seconds

- **Current conclusion:** NSR2-C2 passes all gates. At 512 particles the guarded
  floor stop reduces work from `26/13/153` outer/reject/HVP to `12/0/46` while
  retaining nanometric physical residual and exact oracle correspondence.
- **Current conclusion:** `hvp-workspace-stream-v1` is bit-exact and improves
  serial total median by `1.18x` across 512--4096 particles.
- **Current action:** Implement the frozen per-outer-state Hessian coefficient
  tape and charge its construction to the measured solve.
- **Next gate:** NSR3-A2 must improve combined tape+HVP and large total time
  without dense storage or arithmetic/order changes.
- **Do not retry:** old profile tuning, block/hybrid maps, Chebyshev radius or
  iteration sweeps, product-scale/CUDA work.
- **Runtime authority:** none.

## Decisions

### D-001 -- Separate identity

- **Observation:** FCR2 objective passes, while all admitted fast-SISSM paths
  stopped.
- **Decision:** Use `nuv-newton-krylov-r0`; preserve `nuv-variational-fcr1` as
  objective parent and historical report identity.
- **Consequence:** Solver reports and future performance roots are new; old
  timing grants no credit.

### D-002 -- Diagnose curvature before selecting a fast solver

- **Observation:** Pressure Hessian contains global `J^T J` coupling and a
  possibly indefinite geometric term not represented by FCR3-A blocks.
- **Decision:** NSR0 must prove an analytic matrix-free HVP against the verified
  gradient and a dense tiny oracle before NSR1.
- **Rejected:** implementing trust-region logic on an unverified Hessian;
  treating an approximate block as ground truth.

### D-003 -- Trust-region first, constrained reformulation conditional

- **Observation:** fixed Chebyshev becomes non-descent under changing
  pressure-bearing states.
- **Decision:** use full-HVP Steihaug--Toint trust-region Newton-CG as the first
  bounded optimizer discriminator. Consider primal-dual inequality pressure
  only if NSR1 evidence isolates penalty stiffness.
- **Rejected:** simultaneous solver/model changes, which would make a result
  uninterpretable.

## Required context

1. `docs/architecture/agent-routing.md`, SPEC-38, ADR-076 and ADR-081.
2. `docs/plans/nonlocal-continuum-formula-reclosure/README.md` and
   `00-formula-contract.md`.
3. The stopped formula-reclosure task state and FCR3-B2 evidence.
4. `docs/development/nonlocal-nonlinear-solver-research-2026-08-20.md`.
5. `docs/plans/nonlocal-nonlinear-solver-research/README.md` and the current
   frozen stage contract.

## Exact next action

1. Assemble exact pressure/viscosity/surface HVP coefficients once per outer
   state under the frozen memory cap.
2. Prove every tape HVP and final solve bit-exact against A1.
3. Run the frozen alternating A1/A2 tournament on logical CPU 4.
4. Select/reject A2, then return to NSR3-B0 profile derivation before any long
   physical trajectory or runtime proposal.

## Reconsideration triggers

- Pairwise Descent paper/code becomes public: compare only after its exact
  formula and identity are reviewable.
- NSR0 HVP mismatch: fix one derivation/transcription defect under the same
  contract; a second mismatch stops the branch.
- NSR1 correct-model but penalty-dominated cost: draft an independent
  constrained primal-dual formula contract.
