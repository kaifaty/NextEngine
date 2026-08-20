# FCR3-A — local-curvature conditioning contract

Status: `BLOCK_V1_REJECTED / HYBRID_V2_FROZEN FOR IMPLEMENTATION / REPORT_ONLY`

## Candidate

`block-jacobi-gn-armijo-v1` keeps the exact FCR2 objective, gradient and Armijo
acceptance. It changes only the descent direction:

```text
p_i = -H_i^-1 * gradient_i
```

with one positive-definite `3x3` block per particle:

```text
H_i = m/dt^2 I
    + kappa * sum_active_k (d e_k / d y_i)(d e_k / d y_i)^T
    + sum_viscous_pairs m*omega/(rho0*dt) *
        (2*mu*P_t + lambda*P_n)
    + positive_part(surface radial/tangential curvature).
```

Here `e_k=max(rho_k/rho0-1,0)`. Compression uses a Gauss–Newton
approximation; viscosity is exact for fixed reference pairs; negative surface
curvature is omitted so every block stays positive definite. Inertia provides
the strict positive floor. The implementation performs no global matrix
assembly or solve.

## A/B corpus

The unchanged FCR2 inertial preconditioner and the block candidate run the
same fixed iteration caps and line-search rule on:

1. compressed pair;
2. surface-repulsive pair;
3. combined tetrahedron.

## Exit gate

- both paths remain finite, monotonic and internally momentum-closed;
- block final objective is no worse than baseline by more than relative
  `1e-10` at the same iteration cap;
- block final gradient is no worse than `2x` baseline (with `1e-8` absolute
  floor);
- aggregate backtracks fall by at least `4x`;
- each pressure/combined minimum accepted alpha improves by at least `16x`;
- the selected physical direction from FCR2 is preserved;
- two reports are byte-identical and FCR0–FCR2/frozen controls are unchanged.

If this fails, FCR3-B must not stack Chebyshev or SISSM acceleration on the
rejected preconditioner. The exact first case/metric selects a bounded redesign
or `FORMULA_RECLOSURE_STOP`.

Passing authorizes FCR3-B fast-iteration correspondence only. It grants no
profile, CUDA or runtime authority.

Pure block v1 fails fixed-budget final quality despite reducing aggregate
backtracks from `3788` to `7`. See the
[exact evidence](../../development/nonlocal-continuum-fcr3a-block-evidence-2026-08-20.md).
One bounded hybrid coarse-block/reference-polish discriminator is allowed;
the gates remain unchanged.

## Frozen hybrid v2 remediation

`block16-inertial64-warm-armijo-v2` keeps the same total cap of `80`:

1. iterations `0..15` use the block v1 direction;
2. iterations `16..79` use the unchanged FCR2 inertial direction;
3. Armijo starts at `min(1, 2*previous_accepted_alpha)` instead of resetting
   to one; the first iteration starts at one;
4. objective, gradient, graph, physical and failure rules remain unchanged.

The `16`-iteration switch is frozen because block v1 converged the surface
control in `14` iterations while the pressure/combined controls still needed
accurate polishing. No additional switch-point or relaxation sweep is allowed
inside FCR3-A.
