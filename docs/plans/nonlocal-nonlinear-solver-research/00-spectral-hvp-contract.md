# NSR0 -- spectral Hessian and HVP contract

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / REPORT_ONLY`

Identity: `nuv-newton-krylov-r0`

Parent objective: `nuv-variational-fcr1`, exactly as specified by FCR0 and
evaluated by the passing FCR2 binary64 oracle.

## Purpose

NSR0 tests whether the stopped fast solvers omitted decisive global curvature.
It does not optimize a time step and grants no physical, performance or runtime
claim.

For every smooth fixed-active-set point, the pressure Hessian is

```text
H_pressure = kappa * sum_a (∇s_a ∇s_a^T + s_a ∇²s_a),
s_a = rho_a/rho0 - 1 > 0.
```

The first term contains cross-particle/off-diagonal coupling that a local
per-particle block cannot represent. The second term can be indefinite. NSR0
measures both through the full objective Hessian.

## Oracles

At a point `y` and direction `v`:

1. `H_analytic v` is evaluated without forming a global matrix, using exact
   inertia, fixed-reference viscosity, surface radial/tangential curvature and
   the two-pass pressure expression above.
2. `H_fd v = (g(y + eps*v) - g(y - eps*v)) / (2*eps)` uses the already verified
   FCR2 gradient.
3. A dense Hessian is formed only for tiny controls by applying `H_analytic` to
   canonical coordinate basis vectors.
4. A deterministic symmetric Jacobi eigensolver reports the eigenvalue range;
   it is diagnostic and is not used to alter the solver.

## Frozen numerical policy

- Normalize every test direction to Euclidean norm one.
- Directional finite-difference step:
  `eps = 2^-20 * max(1, ||y||_2)` metres.
- No sample may cross a kernel/surface branch, zero-distance guard or pressure
  active-set boundary under `y +/- eps*v`.
- Relative HVP error denominator is
  `max(||H_analytic v||, ||H_fd v||, 1)`.
- Dense symmetry error is
  `||H-H^T||_F / max(||H||_F, 1)`.
- Dense multiplication error uses the same relative vector norm convention.
- Jacobi performs at most `64*d*d` canonical largest-off-diagonal pivots and
  stops only when the largest off-diagonal magnitude is at most `1e-12` times
  `max(max_abs_diagonal, 1)`.

## Frozen controls

1. compressed two-particle pressure case, with rest density chosen for a
   density ratio of `1.1` at the initial point and separation `0.045 m`, away
   from every density/surface spline branch;
2. the existing four-particle combined P/V/S fixture at its predicted point;
3. repulsive two-particle surface case (`0.8 * spacing`);
4. attractive two-particle surface case (`1.7 * spacing`).

Each control uses one fixed canonical direction containing non-axis-aligned,
non-translational components. The report includes dimension, active pressure
count, minimum absolute density-ratio margin from one, symmetry error, HVP
finite-difference error, dense multiplication error, minimum/maximum
eigenvalue, negative-eigenvalue count and estimated positive condition number.

## Gates

All controls must satisfy:

- finite objective, gradient, HVP, dense Hessian and eigenvalues;
- unchanged active set under both finite-difference probes;
- HVP finite-difference relative error `<= 2e-6`;
- dense symmetry error `<= 2e-12`;
- analytic HVP versus dense multiplication relative error `<= 2e-12`;
- Jacobi converges inside its fixed budget;
- two executions emit byte-identical reports;
- FCR0, FCR1, FCR2 and the expected FCR3-B2 failure reports remain
  byte-identical to their frozen hashes.

`2e-6` is deliberately looser than the FCR0 first-derivative gate because NSR0
differentiates the gradient a second time across large pressure curvature. It
is frozen before seeing results and cannot be relaxed in this stage.

## Result selection

- PASS selects `NSR_HVP_CANDIDATE` and authorizes a separately frozen NSR1
  trust-region Newton-CG contract.
- Any mismatch first fixes only a derivation/transcription defect and reruns
  the same contract once. A second failure selects `NSR_STOP`.
- Eigenvalue indefiniteness does not fail NSR0; it is exactly the observation
  that determines NSR1 negative-curvature handling.
