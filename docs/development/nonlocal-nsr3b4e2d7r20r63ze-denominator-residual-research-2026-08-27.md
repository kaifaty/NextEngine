# NSR3-B4E2D7R20R63ZE denominator/residual research

Status: `SUPPORTED_BOUNDED / REVIEWED / DENOMINATOR_STEP_SUFFICIENT`.

## Question

Reviewed R63ZD shows that a common-operator substitution at any one of the
three PCG products is sufficient to lose the state-2 R63Y certificate. The
cleanest singleton is `TTC`: its state through `p1` is exactly the passing
`TTT` baseline, and only the final `Hp1` product changes.

At that final slot the product has two distinct consumers:

```text
denominator d = p1^T Hp1  -> alpha = rho1 / d -> x2 = x1 + alpha p1
residual vector Hp1       -> r2 = r1 - alpha Hp1
```

R63Y certifies `x2`, not `r2`. Therefore the last-slot certificate can change
through the product's scalar denominator/step length, while the vector used
only in the terminal residual should be observationally isolated from that
certificate. This is a structural derivation, not yet executable evidence.

## Minimal discriminator

Freeze the passing tangent trajectory through state 1. Compute both final
products on the identical `p1`:

```text
t = H_tangent p1
c = H_common  p1
d_t = p1^T t
d_c = p1^T c
```

Run the complete `2 x 2` factorial:

| Lane | Denominator | Residual update |
|---|---|---|
| `TT` | `d_t` | `t` |
| `TC` | `d_t` | `c` |
| `CT` | `d_c` | `t` |
| `CC` | `d_c` | `c` |

All lanes execute both final products and both denominator dots before
selecting consumers, so work is equal. The selector cannot change `x0`,
`r0`, `p0`, `x1`, `r1`, `z1`, `rho1` or `p1`.

## Classified outcome and diagnostic failures

| ID | Meaning | Required pass pattern in `TT,TC,CT,CC` order |
|---|---|---|
| H0 | final-slot loss is denominator/step-length sufficient | `1100` |
| D1 | certificate/state isolation is residual-vector dependent | `1010` |
| D2 | certificate/state isolation is independently denominator- and residual-dependent | `1000` |
| D3 | certificate/state isolation has a denominator/residual interaction | `1110` |

Only H0 is an admissible positive result. Any D1--D3 result rejects the
solution-only certificate-isolation premise and stops the research for an
implementation, state-publication or verifier-dependence audit. Any other
pattern rejects the apparatus or endpoint premise.

## Expected exact aliases

- `TT` and `TC` must have the same state-2 solution root as R63ZD `TTT`,
  `38f8d0b2...0baea`; a valid solution-only certificate must also share its
  root and `24+/78-/0?` result.
- `CT` and `CC` must have the same state-2 solution root as R63ZD `TTC`,
  `f7e28b44...f59e8`; a valid solution-only certificate must also share its
  root and `12+/24-/66?` result.
- Within each denominator pair, the two state-2 residual roots must differ.
  This proves that the residual selector was executed even though it cannot
  alter the solution-only certificate.

## Why not a continuous homotopy yet

A blend `H(lambda)=(1-lambda)H_t+lambda H_c` would immediately introduce a
grid, threshold and interpolation identity. R63ZE first determines which
already-executed scalar/vector consumer carries the exact endpoint change.
Only if the denominator is sufficient should a later stage audit the exact
quadratic discrepancy and certificate margin; no fitted transition point is
needed here.

## Conditional next research

- `DENOMINATOR_STEP_SUFFICIENT` selects R63ZF: an exact/twofold audit of
  `p1^T(H_common-H_tangent)p1`, the induced `alpha` difference and the
  solution/certificate boundary.
- Any residual-dependent result stops R63ZE and audits state/certificate
  isolation before further operator mathematics.

## Boundaries

R63ZE changes no operator value, factor, RHS, scale, precision, recurrence
depth, certificate or tolerance. It is one fixed Linux x86-64 strict-binary128
causal discriminator. It performs no timing, corpus, nonlinear state,
CPU/GPU runtime or production work and authorizes no operator repair.

## Reviewed outcome

The [captured evidence](nonlocal-nsr3b4e2d7r20r63ze-denominator-residual-evidence-2026-08-27.md)
returns byte-exact pattern `1100` and route
`DENOMINATOR_STEP_SUFFICIENT`. Solution/certificate roots alias within each
denominator pair while residual and full state roots differ, so the terminal
residual consumer is observably active but cannot explain the certificate
change. Initial review found unsafe incomplete-lane precedence; one batched
repair closed it and the single re-review returned `GO`.
