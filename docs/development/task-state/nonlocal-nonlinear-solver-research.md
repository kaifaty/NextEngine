# Nonlocal nonlinear solver research -- current task state

| Field | Value |
|---|---|
| Status | `ACTIVE / NSR3B0R_PASS / NSR3B1_MULTISTEP_DESIGN` |
| Updated | `2026-08-20` |
| Task key | `nonlocal-nonlinear-solver-research` |
| Scope | Fundamental solver research over the verified Nonlocal variational objective, isolated from runtime and the stopped SISSM lineage |
| Definition of done | NSR0--NSR6 select a production-roadmap candidate or stop at an exact reproducible boundary |
| Authority | Working context only; Accepted architecture, SPEC-38/ADR-076 and the frozen stage contracts outrank this file |

## Resume in 60 seconds

- **Current conclusion:** NSR2-C2 passes all gates. At 512 particles the guarded
  floor stop reduces work from `26/13/153` outer/reject/HVP to `12/0/46` while
  retaining nanometric physical residual and exact oracle correspondence.
- **Current conclusion:** `outer-state-hessian-tape-v1` is bit-exact, remains
  inside its linear memory cap, improves build+HVP by `2.05x--2.43x` and total
  solve by `1.36x--1.55x` across 512--4096 particles.
- **Current conclusion:** the unscaled FCR cubic integrates to `1/8`; at
  `H=3dx` its infinite-lattice density is `0.125224*rho0`. The authors' code
  applies a missing fixed lattice normalization near `7.985668`.
- **Current conclusion:** the independent B0 diagnostic passes twice
  byte-identically and selects `FORMULA_RECLOSURE_REQUIRED`; all historical
  raw hashes remain unchanged.
- **Current conclusion:** `lattice-normalized-cubic-v1` passes density,
  gradient, HVP, dense, trust and A1/A2 tape correspondence under the new
  `nuv-variational-fcr2` identity; FCR1 hashes remain byte-identical.
- **Current action:** freeze manufactured multi-step state transitions,
  invariants and step-doubling gates before implementing them.
- **Next gate:** NSR3-B1 must prove time integration, translation/objectivity,
  conservation, exact repeat and convergence without any wall model.
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

### D-004 -- Reuse immutable outer-state curvature

- **Observation:** positions, active pressure centers and pair support do not
  change inside one trust-region outer state, but A1 recomputed their Hessian
  coefficients for every Krylov product.
- **Decision:** select `outer-state-hessian-tape-v1`; its construction is
  charged to total work and storage is bounded linearly in particles/pairs.
- **Evidence:** exact A1 correspondence, `2.05x--2.43x` combined build+HVP and
  `1.36x--1.55x` total speedup in two clean pinned campaigns.
- **Consequence:** use A2 for later report-only CPU research; retain A1 as the
  exact oracle. This grants no physical, GPU or runtime authority.

### D-005 -- Normalize the material kernel before selecting profiles

- **Observation:** the FCR1 raw cubic integrates to `1/8` and produces
  `0.125224338*rho0` on the `H=3dx` lattice; the authors' implementation uses
  a separate fixed lattice scale not present in FCR1.
- **Decision:** do not hide the deficit in `rho0`, mass or material strengths.
  Reclose one explicit common scale for `W`, `dW/dr` and `d2W/dr2` under the
  new `nuv-variational-fcr2` identity.
- **Consequence:** all FCR1 solver evidence remains valid for its synthetic
  objective but gives no physical-profile authority. NSR3-B1 stays blocked.

### D-006 -- Select the normalized FCR2 objective

- **Observation:** one common factor restores reference density while all six
  gradient/HVP/dense controls, both trust solves and A1/A2 correspondence pass.
- **Decision:** select `nuv-variational-fcr2` for later report-only CPU
  research; keep FCR1 available only under its old explicit commands.
- **Consequence:** coefficient anchors may now enter manufactured controls,
  but no hydro, boundary, CUDA or runtime claim is unlocked.

## Required context

1. `docs/architecture/agent-routing.md`, SPEC-38, ADR-076 and ADR-081.
2. `docs/plans/nonlocal-continuum-formula-reclosure/README.md` and
   `00-formula-contract.md`.
3. The stopped formula-reclosure task state and FCR3-B2 evidence.
4. `docs/development/nonlocal-nonlinear-solver-research-2026-08-20.md`.
5. `docs/plans/nonlocal-nonlinear-solver-research/README.md` and the current
   frozen stage contract.

## Exact next action

1. Freeze state ownership and exact recurrence for each B1 manufactured case.
2. Freeze mass, momentum, energy/work, objectivity and step-doubling metrics.
3. Implement short report-only multi-step execution with FCR2+A2.
4. Keep static boundaries and named physical trajectories blocked until B1
   passes.

## Reconsideration triggers

- Pairwise Descent paper/code becomes public: compare only after its exact
  formula and identity are reviewable.
- NSR0 HVP mismatch: fix one derivation/transcription defect under the same
  contract; a second mismatch stops the branch.
- NSR1 correct-model but penalty-dominated cost: draft an independent
  constrained primal-dual formula contract.
