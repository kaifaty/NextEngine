# NSR3-B4E2D7R20R63ZF quadratic-step/sign-boundary research

Status: `RESEARCH_COMPLETE / CONTRACT_FROZEN / IMPLEMENTATION_NEXT`.

## Question

Reviewed R63ZE localizes the final-slot certificate loss to the selected
quadratic denominator:

```text
d_t = p1^T t  -> alpha_t -> x_t -> pass  24+/78-/0?
d_c = p1^T c  -> alpha_c -> x_c -> reject 12+/24-/66?
```

The reviewed executed values imply:

```text
(d_c-d_t)/d_t       about +0.900626%
(alpha_c-alpha_t)/alpha_t about -0.892587%
```

A sub-percent step change causing 66 unresolved certificate rows has two
materially different explanations:

1. the raw binary128 solution crosses component sign boundaries; or
2. raw signs remain unchanged, but the R63Y affine-image enclosure becomes too
   wide to prove them.

R63ZE aggregate sign counts cannot distinguish these explanations because an
unresolved certificate row does not state the sign of the raw solution.

## Minimal discriminator

Replay the common tangent prefix once through `p1`, compute both final products
and denominators once, and construct both final solution/residual endpoints:

```text
t = H_t p1                    c = H_c p1
d_t = Dot2(p1,t)              d_c = Dot2(p1,c)
alpha_t = rho1/d_t            alpha_c = rho1/d_c
x_t = x1 + alpha_t p1         x_c = x1 + alpha_c p1
r_t = r1 - alpha_t t          r_c = r1 - alpha_c c
```

Audit three layers without changing either endpoint:

1. **quadratic discrepancy:** compare the outward twofold interval for
   `d_c-d_t` with a direct `Dot2(p1,c-t)` interval including the propagated
   component-difference radii;
2. **step correspondence:** require `d_c>d_t`, `alpha_c<alpha_t` and contain
   every component of `x_c-x_t=(alpha_c-alpha_t)p1` under the fixed update
   order;
3. **raw-sign/certificate boundary:** count and root the 102 raw binary128
   signs for `x_t` and `x_c`, count changed/zero components, and compare them
   with the two immutable R63Y certificates and their `error_upper` values.

The probe needs the already-computed Dot2 error bound. Extend only the private
research `FormulaProbeScalar` DTO with that bound; do not alter the scalar
kernel, root, cache schema or recurrence arithmetic.

## Competing hypotheses

| ID | Hypothesis | Decisive observation |
|---|---|---|
| H0 | raw solution sign change contributes | one or more nonzero `x_t/x_c` raw signs differ |
| H1 | enclosure loss occurs without raw sign change | all 102 raw signs alias, tangent certificate passes, common certificate rejects, and the common `error_upper` is strictly larger |
| H2 | recurrence/algebra correspondence is defective | endpoint root, discrepancy interval or componentwise step identity fails |

H1 does not yet prove that the global error term alone is sufficient. It
selects a later `2 x 2` certificate-detail experiment crossing solution centers
with tangent/common error budgets. H0 selects an exact index-level sign/margin
audit instead.

## Expected controls

- R63ZE tangent/common denominator hex values, solution roots and certificate
  roots reproduce exactly.
- Both discrepancy constructions contain a common value with positive sign;
  reversed subtraction has negative sign.
- Mutating a Dot2 bound changes the audit/result identity without changing its
  center.
- A one-component raw-sign mutation changes the sign root and selected route.
- An injected raw zero takes a dedicated boundary rejection.
- Invalid dimension, nonfinite/overflow scalar and nonpositive denominator
  reject before endpoint publication with exact work prefixes.

## Boundaries

R63ZF is one fixed Linux x86-64 strict-binary128/twofold diagnostic. It changes
no operator, factor, input, scale, recurrence depth, certificate, tolerance or
nonlinear state. It performs no timing, corpus, CPU/GPU runtime or production
work. Even a clean result only selects the next certificate-boundary
discriminator; it cannot authorize an operator repair.
