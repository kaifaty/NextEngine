# Nonlocal corrected GPU objective assembly audit — current task state

| Field | Value |
| --- | --- |
| Status | `COMPLETE / NCGA7_MIXED_PRESSURE_STATIC_SOLVE_FAILED / PERFORMANCE_BLOCKED / REVIEW_NOT_RUN / NCGA2_IMMUTABLE` |
| Updated | `2026-08-30` |
| Task key | `nonlocal-corrected-gpu-assembly-audit` |
| Scope | Determine whether strict-f32 corrected CUDA assembly can complete both retained NSR1 static solves before any trajectory or performance claim |
| Definition of done | NCGA5 produces one hash-closed two-case author result with full continuous state ownership, negative controls and unchanged NCGA0--4/NSR1 regressions, or stops at the first frozen convergence boundary |
| Authority | Working context only; SPEC-38, ADR-076/081, FCR0 and the frozen NCGA2 contract outrank this file |

## Resume in 60 seconds

- **Current state:** NCGA7 is hash-closed
  `AUTHOR_REFUTED / MIXED_PRESSURE_STATIC_SOLVE_FAILED`. F64 pressure improves
  pair `R_x` to `1.064e-7` but misses the frozen `1e-7` gate and does not close
  the objective band; combined `R_x` regresses to `4.38e-7`. The arithmetic
  ladder is stopped. NCGA6 is hash-closed
  `AUTHOR_SUPPORTED_CAUSAL / PRESSURE_OPERATOR_PRECISION_REQUIRED`. GPU f64
  energy accepts one extra step per case and improves `R_x` by `4.77x/1.91x`,
  but still stops at `2.77e-6/3.00e-7` and misses the objective band. NCGA5 is
  hash-closed
  `AUTHOR_REFUTED / F32_STATIC_SOLVE_MATERIAL`. Both strict-f32 static solves
  approach the independent solution within `0.0353 um`, but f32 actual
  reduction loses sign/resolution and both runs collapse the trust radius.
  Compressed `R_x=1.323e-5` and both objective differences exceed the frozen
  candidate bands. NCGA4 revision 1 remains
  hash-closed
  `AUTHOR_SUPPORTED_BOUNDED / GPU_ASSEMBLED_TRUST_PREFIX_SUPPORTED`. Its
  GPU-assembled/host-controlled eight-trial Steihaug--Toint prefix exactly
  matches the reference controller route. NCGA3 revision 2 remains hash-closed
  `AUTHOR_SUPPORTED_BOUNDED / F32_CONSEQUENCE_NEGLIGIBLE_BOUNDED`. NCGA2 remains
  honestly refuted at `2.6247e-4`, but strict f32 reaches only `1.0643e-6` HVP
  and `2.0004e-6` regularized-step relative error; eight integer steps end with
  zero micrometre drift and identical objective at published precision.
- **Decision:** NCGA2 assembles objective energy, analytical gradient, exact
  dense Hessian/diagonal blocks and HVP on tiny immutable graphs. It does not
  port the stopped SISSM local matrix.
- **Why:** FCR3-B2 already rejected the pressure-bearing SISSM/Chebyshev
  recurrence. The nonlinear objective and its derivatives remain the valid
  mathematical boundary for a future separately selected solver.
- **Next action:** freeze a separate product-oriented physical acceptance
  contract for the scale-aware terminal state. Only if it accepts should work
  proceed to a local matrix-free GPU HVP/CG implementation and 16k/50k
  end-to-end benchmark; otherwise repair the solver/model first.
- **Current blocker:** neither strict nor selected mixed arithmetic closes the
  retained static solve package, and the dense Hessian requires roughly
  `90 GB` at 50k even in f32. Physical trajectory and full timing remain blocked;
  NCGA3--5 independent review is `NOT_RUN`.
- **Revision-1 discriminator:** all HVP probes completed and strict f32 reached
  only `1.0643e-6` maximum relative L2 error; reduction-only stayed outside the
  old element gate while f64 pressure products reached `4.7195e-5`. The fixed
  `H+7200I` reference system was not positive definite, so step/sequence fields
  were invalid and the run is `INCONCLUSIVE`.
- **Single apparatus repair:** revision 2 uses the reference symmetric
  infinity-norm bound plus `7200` as one common, guaranteed-positive shift.
  No threshold, fixture, arithmetic result or second repair is authorized.
- **Claim ceiling:** two tiny fixed-graph static-solve refutations only; no
  trajectory, performance, runtime or product-water claim.

## Competing hypotheses

| Hypothesis | Prediction | Discriminator | Status |
| --- | --- | --- | --- |
| H1 corrected GPU assembly corresponds | graphs, density, energies, gradient and Hessian pass the frozen mixed bounds | independent long-double oracle plus energy-only derivatives | refuted for frozen strict f32 |
| H2 historical mismatch was in accumulation/composition | isolated NCGA0 terms pass but active-pressure or combined assembly differs | active pressure and combined clusters | supported for naive f32 |
| H3 a wrong matrix can hide behind matching forces | gradient passes while exact Hessian or second derivative fails | dense matrix, HVP and Gauss-Newton/SISSM negatives | supported as a real audit risk |
| H4 reference/current graph roles are mixed | support-crossing viscosity or current terms differ | `reference_current_support_crossing` | refuted on frozen fixture |
| H5 compensated strict-f32 closes cancellation | exact same corpus passes without changing bounds | frozen Kahan-style recurrence plus naive control | refuted; symmetric closes, combined does not |
| H6 the remaining element miss changes local behavior materially | HVP, regularized step or short sequence exceeds the product screen | thirteen probes, common norm-bounded solve and eight steps | refuted on the frozen fixture |
| H7 binary64 reduction alone closes the miss | f32 products accumulated in f64 meet the old matrix gate | reduction-only arithmetic variant | refuted: `2.59425e-4` |
| H8 binary64 pressure products close the miss | promoted pressure coefficients/products meet the old gate | mixed-product arithmetic variant | supported: `4.71951e-5`, but not selected yet |
| H9 strict f32 preserves trust-region decisions | eight reference/CUDA outer signatures match and state drift stays below `5 um` | continuous-state fixed-graph Steihaug--Toint prefix plus sign-flipped HVP control | supported on NCGA4 |
| H10 strict f32 completes the retained static solves | both NSR1 cases reach the raw gradient stop while preserving state/objective/active-set bands | NCGA5 exact full solve with multi-HVP residual CG | refuted: energy/globalization floor |
| H11 f32 energy resolution is the first full-solve boundary | GPU f64 energy with unchanged f32 gradient/Hessian restores positive actual reduction and scale-aware convergence | NCGA6 energy-only mixed solve plus f32-energy negative | causal but insufficient |
| H12 pressure operator products are the remaining static boundary | f64 pressure density/coefficient/gradient/Hessian products close `R_x<=1e-7` with f32 storage | NCGA7 pressure-operator discriminator | refuted: pair marginal miss, combined regression |

## Decisions

### D-001 — Do not port the stopped SISSM split

- **Observation:** FCR3-B2 closed the existing SISSM/Chebyshev lineage after
  reproducible pressure non-descent/quality failures.
- **Evidence:** `docs/development/nonlocal-continuum-fcr3b2-chebyshev-evidence-2026-08-20.md`.
- **Conclusion:** CPU/GPU equality for that local split would not select a
  corrected solver.
- **Decision:** assemble derivatives of `nuv-variational-fcr1` directly and
  include the historical SISSM matrix only as a mandatory rejected identity.
- **Rejected alternatives:** port old `source/local_matrix`, or jump directly
  to a full GPU solve.
- **Consequences:** NCGA2 can feed a future matrix-free operator audit but
  cannot claim a solver step.
- **Remaining uncertainty:** whether strict-f32 active-pressure Hessian
  composition stays inside the frozen bound.
- **Reconsider when:** a separately reviewed solver chooses a different
  derivative/operator identity.

### D-002 — Keep product and research profiles separate

- **Observation:** SPEC-38 V1 selects CPU DFSPH with `0.1 m` support and no
  viscosity/surface model; NCGA0/FCR uses the Nonlocal `0.15 m` audit profile.
- **Decision:** NCGA2 stays on the latter and names it report-only. It does not
  inherit SPEC-38 product authority from NCGA1's exact integer cache test.
- **Consequence:** successful assembly is useful GPU-port evidence, not a claim
  that the V1 game water profile is implemented.

### D-003 — Preserve the naive-f32 pressure failure

- **Observation:** the first dense pressure fixture leaves `2.82e-4` gradient
  and `3.43e-3` Hessian residues where the host values are approximately zero;
  combined Hessian error is `2.274e-4` against the `2e-4` bound.
- **Evidence:**
  `docs/development/nonlocal-corrected-gpu-assembly-audit-evidence-2026-08-30.md`.
- **Conclusion:** revision 1 is refuted; graph/formula boundaries outside dense
  pressure remain supported only as local diagnostics.
- **Decision:** no tolerance/fixture change. The only admissible next
  discriminator is an exactly frozen compensated-binary32 accumulation path
  with naive f32 as a negative.
- **Remaining uncertainty:** whether compensation closes every gradient,
  Hessian and direct-HVP reduction, or strict f32 remains insufficient.

### D-004 — Freeze compensated f32 without changing the question

- **Observation:** cancellation, not graph or isolated term translation, is the
  first surviving explanation.
- **Decision:** revision 2 changes only the exact binary32 addition recurrence
  and work receipt. Profile, fixture bytes, bounds, products and oracles remain
  immutable; naive revision 1 becomes a common-comparator negative.
- **Rejected alternatives:** tolerance widening, deleting the symmetric case,
  using device binary64, switching to a new physical profile, or measuring a
  solver before assembly closes.
- **Reconsider when:** revision 2 fails an unchanged gate or independent review
  finds shared/hidden work.

### D-005 — Stop strict-f32 assembly after compensation fails

- **Observation:** compensation reduces the symmetric Hessian residue by about
  `75x`, but the combined scalar at index `51472` remains `2.6247e-4` relative
  error and fails the `2e-4` gate.
- **Evidence:**
  `docs/development/nonlocal-corrected-gpu-assembly-audit-revision-2-evidence-2026-08-30.md`.
- **Conclusion:** final reduction order is not the only source; strict-f32
  product/normal/coefficient composition remains outside the frozen bound.
- **Decision:** close NCGA2 without review or sanitizers because the positive
  candidate is already refuted. No third reduction identity and no full GPU
  solver port are authorized.
- **Rejected alternatives:** widen the bound, remove the combined fixture,
  accept HVP-only equality, or infer product water from neighborhood timing.
- **Reconsider when:** a new contract independently selects arithmetic from a
  solver/physics error budget or decomposes product-level rounding.

### D-006 — Retain f32 for the next solver discriminator

- **Observation:** the strict matrix still misses one elementwise gate, while
  all thirteen HVP probes, the stable regularized step and eight quantized
  steps remain far inside the separately frozen consequence bands.
- **Evidence:**
  `docs/development/nonlocal-corrected-gpu-consequence-evidence-2026-08-30.md`.
- **Conclusion:** the miss is numerically real but negligible for this bounded
  local response. Merely accumulating f32 products in f64 does not close it;
  f64 pressure products do close it but do not improve the observed step.
- **Decision:** begin any separately frozen solver experiment with strict f32;
  preserve mixed pressure products as a fallback/control rather than paying
  their cost before a solver-level failure requires them.
- **Rejected alternatives:** relabel NCGA2 as passing, declare mixed precision
  mandatory from one scalar, or infer full water stability/frame time from
  eight tiny steps.
- **Remaining uncertainty:** nonlinear globalization, long trajectory,
  conditioning across a corpus, 50k assembly/solve cost and independent review.
- **Reconsider when:** a solver/corpus changes descent, convergence or physical
  acceptance under strict f32.

### D-007 — Reuse the reviewed CPU globalization before GPU-resident optimization

- **Observation:** the repository already contains an NSR1 Steihaug--Toint
  trust-region Newton--CG candidate that solved the frozen static CPU controls;
  later research failures concern multistep accuracy, constraints and finite
  operator certificates rather than absence of a globalization algorithm.
- **Evidence:** `docs/development/nonlocal-nsr1-trust-region-evidence-2026-08-20.md`
  and the frozen NCGA4 contract.
- **Conclusion:** writing another solver or reviving SISSM would duplicate or
  contradict retained evidence. The smallest new uncertainty is whether the
  verified CUDA evaluator changes trust/Krylov decisions.
- **Decision:** NCGA4 keeps the controller on the host, adds continuous-state
  evaluator overloads, and compares exactly eight fixed-graph outer trials.
  GPU-resident CG, trajectory and timing remain later boundaries.
- **Rejected alternatives:** quantized micrometre pseudo-solve, immediate
  50k/full-frame timing, mixed precision before a strict-f32 solver failure,
  or importing the stopped SISSM recurrence.
- **Reconsider when:** NCGA4 closes its route/state gates or isolates a specific
  f32 globalization boundary.

### D-008 — Require a complete static solve before physical trajectories

- **Observation:** NCGA4 matches all eight trust decisions with only
  `0.000675 um` maximum state drift, but every inner path exits on negative
  curvature after one HVP and the gradient norm increases during descent.
- **Evidence:**
  `docs/development/nonlocal-corrected-gpu-trust-prefix-evidence-2026-08-30.md`.
- **Conclusion:** strict f32 is benign for this globalization prefix, but the
  run does not exercise residual-terminated multi-iteration CG or convergence.
- **Decision:** keep mixed pressure products unselected. Next reproduce the two
  retained NSR1 static solves to their frozen convergence gates on CUDA; only
  then reopen a boundary-free short trajectory.
- **Rejected alternatives:** call the prefix a full solver, infer water
  stability, time the dense 100-particle harness, or jump directly to contacts,
  dam break or 50k.
- **Reconsider when:** NCGA5 closes both negative-curvature and residual-CG
  routes or identifies a specific f32 convergence boundary.

### D-009 — Preserve the raw NSR1 stop and classify, rather than hide, an f32 floor

- **Observation:** NCGA4 establishes eight safe updates but does not approach
  convergence; binary32 energy may lose positive actual reduction before the
  raw `1e-10` gradient threshold becomes representable.
- **Decision:** NCGA5 keeps the exact NSR1 trust policy and raw success stop.
  It separately reports a scale-aware displacement residual and may classify a
  tightly bounded `STRICT_F32_CONVERGENCE_FLOOR`, but that classification is
  not solve success and cannot authorize a trajectory.
- **Rejected alternatives:** widen the convergence tolerance after the run,
  call a nearby state converged, introduce mixed precision inside revision 1,
  or proceed directly to performance.
- **Reconsider when:** NCGA5 reaches raw convergence or isolates the exact
  energy/globalization floor needed for a separately frozen repair.

### D-010 — Repair reference arithmetic-order equivalence, not candidate gates

- **Observation:** the independent dense reference and historical matrix-free
  NSR1 differ only at the terminal binary64-noise ratio; the former already has
  `R_x=9.54e-11`, far inside the pre-existing `1e-8` scale-aware criterion.
- **Conclusion:** exact terminal route equality was not portable across the two
  legitimate reference reduction orders.
- **Decision:** NCGA5 revision 2 accepts the independent combined reference at
  the pre-existing scale-aware criterion and recorded reduction floor. The
  compressed exact-count gate and every CUDA candidate threshold stay fixed.
- **Rejected alternatives:** tune the candidate bands from the observed GPU
  result, replace the independent reference with the candidate, or call
  revision 1 conclusive.
- **Reconsider when:** the single repaired run still lacks apparatus closure;
  then NCGA5 stops without a second repair.

### D-011 — Select energy/globalization precision before pressure-operator precision

- **Observation:** both strict candidates stay within `0.0353 um`, remain
  finite and keep the pressure active set, but their next predicted reductions
  are positive while f32 actual reductions become negative or zero.
- **Conclusion:** the first demonstrated solver failure is energy/globalization
  resolution. The evidence does not yet require paying for f64 gradient or
  Hessian products.
- **Decision:** NCGA6 changes only GPU energy evaluation to binary64 at the f32
  stored state/profile and uses the already frozen scale-aware residual as its
  convergence gate. Strict f32 remains the mandatory negative.
- **Rejected alternatives:** widen NCGA5, start trajectories despite the
  failed solve, promote the full operator before isolating energy, or time the
  dense tiny harness as if it were scalable.
- **Reconsider when:** mixed energy either closes both cases or fails while
  pressure/operator error remains the only surviving cause.

### D-012 — Promote the pressure operator only after energy causality

- **Observation:** f64 energy changes the exact failing acceptance decisions
  and improves residual/state, but leaves both cases outside convergence and
  objective bands.
- **Conclusion:** energy resolution is causal, while the remaining f32
  pressure gradient/Hessian/profile composition is now the smallest surviving
  numerical hypothesis.
- **Decision:** NCGA7 promotes pressure density/compression, coefficients,
  gradient and Hessian products to binary64, rounds final operator outputs to
  binary64 host values, and retains f32 state plus NCGA6 f64 energy.
- **Rejected alternatives:** start trajectory now, widen `R_x`, promote all
  terms indiscriminately, or infer performance from the serial diagnostic
  kernel.
- **Reconsider when:** NCGA7 closes both static cases or fails one frozen gate;
  no further arithmetic ladder is implicit.

### D-013 — Stop precision tuning and separate physical acceptance from performance

- **Observation:** NCGA7 leaves sub-micrometre state differences but cannot
  close the frozen solver residual/objective gates; the dense matrix would
  require `90 GB` in f32 for 50k particles.
- **Conclusion:** more precision tuning cannot answer the product performance
  question. Correctness acceptance and scalable representation are now
  separate blockers.
- **Decision:** stop the arithmetic ladder. Define physical/game tolerances
  independently, then — only on acceptance — replace dense assembly/host CG
  with a local matrix-free GPU HVP and resident solver before benchmarking.
- **Rejected alternatives:** widen the just-missed gate, time the serial tiny
  diagnostic, extrapolate neighbor-builder time to a full solver, or call the
  mixed state game-ready without a trajectory.
- **Reconsider when:** a frozen physical acceptance screen decides whether the
  current scale-aware terminal state is admissible.

## Required context

1. `docs/architecture/agent-routing.md`, SPEC-38 and ADR-076/081.
2. FCR0 formula contract and FCR3-B2 stop evidence.
3. NCGA0 and NCGA1 contracts, task states and independent reviews.
4. All five NCGA2/NCGA4 contracts in
   `docs/plans/nonlocal-corrected-gpu-assembly-audit/` and the NCGA3 evidence.

## Do not retry or infer

- do not use `variational_reference.cpp` or historical CPU/CUDA equality as
  the independent oracle;
- do not replace the exact Hessian with a clamped/Gauss-Newton/SISSM block;
- do not hide a host-built candidate graph or floating atomics;
- do not report NCGA2 work counts or the parallel NCGP0 neighborhood benchmark
  as full solver/game throughput.
