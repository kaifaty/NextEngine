# NSR1 -- trust-region Newton-CG discriminator

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / REPORT_ONLY`

Identity: `nuv-newton-krylov-r0`

Parent evidence: NSR0 `NSR_HVP_CANDIDATE`; objective and gradient remain the
unchanged FCR2 `nuv-variational-fcr1` oracle.

## Algorithm

At accepted state `y`, form the quadratic model only through `g` and analytic
HVP calls:

```text
m(p) = E(y) + g^T p + 1/2 p^T H p,
||p||_2 <= Delta.
```

Use unpreconditioned Steihaug--Toint truncated CG from `p=0`, residual `r=g`
and direction `d=-r`. Stop the inner solve at the first of:

1. `d^T H d <= 0`: take the positive intersection with the trust boundary and
   classify `NEGATIVE_CURVATURE`;
2. the next CG point reaches/crosses the boundary: take the exact positive
   boundary intersection and classify `BOUNDARY`;
3. `||r_next|| <= eta_k * ||g||`, where
   `eta_k = min(0.5, sqrt(||g||))`: classify `RESIDUAL`;
4. `3*N` inner iterations: classify `DIMENSION_LIMIT`.

No Hessian projection, regularization, line search, coefficient adaptation or
warm state is allowed in NSR1.

## Trust policy

- Initial radius: `Delta_0 = spacing = 0.05 m`.
- Minimum/maximum radius: `2^-40 * spacing` and `4 * spacing`.
- Trial actual reduction: `ared = E(y) - E(y+p)`.
- Predicted reduction:
  `pred = -(g^T p + 0.5 p^T H p)`.
- Ratio: `rho = ared/pred`; nonfinite or non-positive `pred` rejects.
- Accept only when the trial is finite, `ared > 0` and `rho >= 0.1`.
- If `rho < 0.25` or invalid, set `Delta <- 0.25*Delta`.
- If `rho > 0.75` and the inner result touches the boundary, set
  `Delta <- min(2*Delta, Delta_max)`.
- Otherwise retain the radius.
- Maximum `64` outer trials; stop successfully at gradient norm `<= 1e-10`.
- Rejected trials do not change `y`, the objective or any future state other
  than the radius and diagnostic counters.

## Controls and comparison

Run the existing FCR2 compressed pair and combined tetrahedron with their exact
original fixtures, coefficients and `80`-iteration Armijo baselines. The
NSR1 solver starts from the same `y_star` and reports:

- accepted/rejected outer trials;
- objective/gradient evaluations and analytic HVP calls separately;
- inner stop reasons and negative-curvature count;
- trust-ratio minimum/maximum for accepted trials;
- radius minimum/maximum;
- pressure active-set changes and minimum observed density-ratio margin;
- initial/final objective, gradient norm and internal momentum residual.

The original FCR2 baseline evaluation count is
`1 + iterations + backtracks`: `1816` for compressed pair and `896` for the
combined tetrahedron. NSR1 counts one initial evaluation plus every trial
evaluation; HVP calls are reported separately and cannot be hidden in this
comparison.

## Gates

Each case must:

- remain finite and monotonically decrease objective at every accepted state;
- have no accepted trial with invalid/non-positive model reduction;
- finish with objective no greater than the FCR2 final objective plus
  `1e-10 * max(|E_fcr2|, 1)`;
- finish with gradient norm no greater than
  `max(2 * ||g_fcr2||, 1e-8)`;
- retain internal momentum residual `<= 1e-12`;
- use at most one quarter of the baseline objective/gradient evaluations:
  `<=454` for compressed pair and `<=224` for combined tetrahedron;
- not exhaust `64` outer trials or the minimum trust radius;
- emit byte-identical reports in two executions.

FCR0--FCR2 and the expected FCR3-B2 failure report hashes must remain unchanged.
There is deliberately no HVP-count performance gate in NSR1: the stage asks
whether full curvature fixes nonlinear convergence. NSR2 owns scalable cost.

## Result selection

- PASS selects `NSR_TRUST_REGION_CANDIDATE` and authorizes a measured NSR2
  preconditioner/scalability contract.
- Correct model agreement with excessive HVP/conditioning selects a
  preconditioning branch, not a physics-formula change.
- Persistent small-radius rejection or penalty-dominated conditioning selects
  an independently specified constrained primal-dual study.
- Failure of model agreement after one implementation-defect remediation
  selects `NSR_STOP`; no trust-parameter sweep is allowed.

