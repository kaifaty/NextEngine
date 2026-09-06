# NCGP4 corrected full-step solver diagnosis contract

| Field | Value |
| --- | --- |
| Research ID | `NCGP4` revision 1 |
| Status | `FROZEN / DIAGNOSIS_FIRST / REPORT_ONLY` |
| Architecture snapshot | commit `c0134a66a2891625a2416b7b2b077a10dd73dfbe`, tree `8da10968b71f30c65ac8d9de3e0e9b7e0a0c9000`; SPEC-38/ADR-076 Proposed; ADR-081 Accepted guardrails |
| Frozen parent | NCGP3 `FINAL INCONCLUSIVE`, corrected candidate `b74688b296cc6adfda1e975cbed45f31a0d9d983` |
| Engineering consumer | decide the smallest admissible Newton-CG repair before the full 4k corpus and 50k timing |
| Claim class | finite/profile-bound causal diagnosis, then bounded correctness and performance measurement |
| Budget | one diagnostic implementation, one selected solver repair, then the already-authorized two performance optimisation cycles |

## Exact question and claim ceiling

The first frozen NCGP3 failure is deterministic: corrected and ID-permuted
GPU routes complete 38 hydrostatic steps and fail step 39 with
`WorkBudgetExceeded`, using 126 of 128 HVP before the next three-HVP Jacobi
outer trial; the independent CPU route succeeds. NCGP3 did not establish why.

NCGP4 first asks which of the frozen competing hypotheses explains that work
ceiling on the exact step-39 input. It may then implement only the repair
selected by that trace. It may run 50k timing only after the complete corrected
corpus passes. A timing PASS is standalone Proposed evidence, not runtime
promotion, integrated frame time or a change to the CPU DFSPH fallback.

## Retained physical and numerical profile

NCGP4 retains without tuning:

```text
profile_id       = nonlocal-water-50k-v1
dt               = 1/240 s
spacing          = 0.05 m
horizon          = 0.15 m
mass             = 0.125 kg
ghost_layers     = 3
maximum_neighbors= 256
kernel_scale     = 7.985668078772472
kappa            = 1226.25 J
lambda           = 1.4138231728735551e-5 kg m/s
mu               = 0
gamma            = 0.010664424039285813 m/(kg s^2)
```

All NCGP3 corrected FCR2 formulas, canonical `(hi,lo)` binary32 state,
pair-aware graph, swept analytic contacts, owner-gather reductions,
Steihaug-Toint radius/curvature/acceptance/`rho` rules and public-state
publication remain fixed. Scalar reductions remain binary64. The total HVP
ceiling remains selected from `{32,64,128}`; diagnosis may execute at most four
extra HVP on the frozen failing state, labelled counterfactual and never counted
as a correctness PASS.

## Apparatus repairs required before causal selection

The successor must close the four independent NCGP3 review defects:

1. run all three 4k scenarios for exactly 240 steps in the frozen order;
2. replace the forward-only energy label with an actual reversible control:
   execute the declared conservative free-fall leg, reverse velocity, execute
   the equal reverse leg under the same potential, and compare the final
   position and sign-reversed velocity with the initial state; seal both legs;
3. express the selected `mu=0` viscosity gate as relative kinetic-energy change
   with denominator `max(abs(E0), 1 J)` and limit `1e-6`;
4. seal every gated observable, identity and work field, the complete
   compensated transaction state, and every rollback operation; a failed copy
   or synchronization during restore returns typed `DeviceFailure` and cannot
   be reported as successful restoration.

These repairs change the evidence apparatus, not the physical model or a prior
NCGP3 result.

## Competing hypotheses

| ID | Causal hypothesis | Prediction on exact step 39 | Falsifier |
| --- | --- | --- | --- |
| H1 | Scalar Jacobi leaves a badly conditioned coupled particle operator | most work is inner CG; true residual decreases slowly or stalls while the diagonal has a large spread/floor population; unpreconditioned is no better and a symmetric positive-definite 3x3 block-Jacobi materially reduces HVP without changing the accepted result | few inner iterations, or block-Jacobi does not improve the residual/HVP trace |
| H2 | Trust-region globalization, not the linear solve, consumes the budget | most outer trials terminate cheaply but are rejected, `rho < 0.25` repeatedly shrinks radius, and inner residuals satisfy forcing quickly | HVP are concentrated in long inner solves with few rejects/shrinks |
| H3 | Binary32 operator cancellation creates a convergence floor | inner solves converge, but the outer scaled residual stalls near `1e-5`; an independent same-state long-double gradient/HVP disagrees above the retained HVP gates, with one formula component dominating | independent operator correspondence remains inside the frozen gates and residual is not near a finite-precision plateau |
| H4 | Work accounting or stopping implementation rejects an otherwise admissible step | the trace shows the frozen convergence predicate already true, an incorrectly reserved HVP, a mismatched true/preconditioned residual, or executable work different from the sealed count | executed and sealed work agree and every convergence predicate is false at the ceiling |

## Frozen diagnostic

Reproduce hydrostatic steps 1--38 through the corrected production path and
seal the complete compensated pre-step-39 state. Run corrected and permuted
step 39 once each with budget 128 and record, for every outer and inner event:

- radius before/after, active-center and directed-pair counts;
- gradient norm, maximum-gradient scaled residual and acceptance predicate;
- diagonal component minimum/maximum, inertia-floor count and nonfinite count;
- initial true and preconditioned residuals, forcing term, per-HVP true and
  preconditioned residuals, curvature, alpha/beta and step norm;
- negative-curvature/radius/forcing/work termination reason;
- predicted/actual reduction, `rho`, accepted/rejected result and cumulative
  HVP count.

The trace is a diagnostic D2H stream outside the future hot timing window. Its
schema, event order, values, work and root are closed. A deliberate event
mutation and a work-count mutation must be rejected by the trace checker.

On the identical pre-step state, run the existing unpreconditioned profile and
an independent long-double gradient/HVP comparison. Only if H1 is selected may
one symmetric-positive-definite per-particle 3x3 block-Jacobi be implemented;
only if H3 isolates pressure-operator error may the already authorized f64
pressure coefficients/products discriminator be selected. H2 without an
implementation defect and an unresolved H1/H3/H4 stop solver repair as
`INCONCLUSIVE`; radius/acceptance/tolerance tuning is forbidden.

## Correctness gate after the selected repair

The sequence stops at the first failure:

1. all retained NCGP1/NCGP2 and NCGP3 graph/boundary/transaction controls;
2. repaired reversible-energy, relative-viscosity, free-fall,
   translation/rotation, surface-relaxation and identical-state HVP controls;
3. 240-step 4k hydrostatic hold, dam-break and orifice, corrected and
   ID-permuted;
4. 16k and exact 50k coherent/permuted/advected correctness and capacity;
5. 240-step 50k sealed-basin run.

The unchanged gates are position RMSE/max `2.5/5 mm`, compression-density
RMSE/max `5/10%`, normalized momentum residual `1%`, positive energy excess
and reversible drift `1%`, penetration `2.5 mm`, exact count/mass and no NaN,
overflow, permutation loss, state drift or hidden work. HVP relative L2 remains
`1e-3`, cosine loss `1e-6`, with exact active signature on identical state.

If no member of `{32,64,128}` passes the complete 4k corpus after the selected
repair, correctness is `REFUTED_BOUNDED` and performance is `NOT_RUN`.

## Performance protocol after correctness

The measured 50k hot step includes graph rebuild, density/active set,
energy/gradient/all HVP, trust-region control, boundary handling and state
integration. It excludes startup, initial allocation, CPU oracle, JSON and
rendering. No allocation or full-vector transfer is permitted in the hot step.

Use two fresh processes, 256 conditioning + 32 warmup + 128 measured steps,
CUDA events, synchronization per sample and nearest-rank p50/p95/p99 without
outlier removal or retry-to-green. Both processes must satisfy `p95 <= 4 ms`
and `p99 <= 6 ms`. Publish graph, energy/gradient, HVP-total, reductions,
solver-control and boundary/integration timings plus transfers and allocation.

At most two optimization cycles are permitted without changing physics,
workload or tolerances: (1) single-pass neighbor construction and compatible
owner-row fusion; (2) launch/synchronization reduction, scratch reuse and
compatible HVP fusion. A miss after cycle two is `PERFORMANCE_REFUTED`.

## Primary sources and bounded implications

- Trond Steihaug, 1983, DOI `10.1137/0720042`: preconditioned CG is a valid
  approximate trust-region subproblem method; this does not show our
  preconditioner is effective.
- PETSc `KSPGLTR` documentation: the trust-region preconditioner must be
  symmetric positive definite and termination reasons distinguish negative
  curvature and constrained steps; this motivates, but does not validate, the
  recorded trace fields.
- Teschner et al., *SPH Techniques for the Physics Based Simulation of Fluids
  and Solids*, 2019: matrix-free CG convergence for implicit SPH viscosity can
  improve with block Jacobi; this is prior art, not evidence for our pressure,
  surface or full-step operator.

## Stop and promotion boundary

- Stop `INCONCLUSIVE` if trace identity/work is incomplete or does not
  discriminate the hypotheses.
- Stop correctness at the first frozen failure; do not time a failed solver.
- Do not retry NCGP3, raise HVP above 128, loosen a tolerance, localize host
  coordinates, use the raw FCR profile or report neighbor-only cost as a full
  solver result.
- Even a `4/6 ms` PASS establishes only standalone RTX-3080 feasibility.
  Runtime integration, PhysX coupling, renderer/frame time and roadmap
  promotion require a later consumer-backed package.
