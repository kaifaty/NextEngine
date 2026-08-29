# NSR2-A1 -- scale-aware nonlinear stopping contract

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / REPORT_ONLY`

Identity: `nuv-newton-krylov-r0`

Parent result: NSR2-A
`FAIL / SCALE_DEPENDENT_STOPPING_CRITERION / PRECONDITIONER_NOT_SELECTED`.

## Single remediation

For the incremental objective, inertia maps a position residual `delta_y` to
gradient `(mass/dt^2) * delta_y`. Define the dimensionless maximum residual:

```text
r_y = max_i ||(dt^2/mass) * gradient_i||_2 / spacing.
```

An outer solve is converged when either the historical NSR1 raw gate
`||g||_2 <= 1e-10` or `r_y <= 1e-8`. This corresponds to at most `5e-10 m`
at the unchanged `0.05 m` spacing, well below the proposed canonical
micrometre publication quantum. The report must retain raw gradient norms and
identify which stop fired.

This criterion is independent of particle count and has position meaning. It
is frozen before the remediation run. It does not relax objective acceptance,
trust ratios, momentum closure or the preconditioner selection gate.

## Unchanged experiment and gates

Re-run the exact NSR2-A 8/27/64 fixtures, unpreconditioned baseline and
`block-gn-metric-v1` candidate. Retain:

- exact objective, gradient and full HVP;
- block construction and `M`-norm inner solve;
- NSR1 trust policy and maximum `64` outer trials;
- candidate final objective within `1e-10` relative of baseline;
- candidate dimensionless `r_y <= max(2*r_y_baseline,1e-8)`; raw gradient is
  still reported but is no longer compared across particle-scaled solves;
- momentum residual `<=1e-12`;
- no more per-case objective evaluations or HVPs than baseline;
- candidate aggregate 27+64 HVP count `<=75%` of baseline;
- byte-identical double execution and unchanged historical hashes.

## Selection

- PASS selects `block-gn-metric-v1` for NSR2-B.
- Failure rejects the block candidate and selects the unpreconditioned solver
  for NSR2-B. No second convergence or preconditioner remediation is allowed.
- Either result leaves constrained pressure blocked; NSR1 model agreement is
  unchanged.
