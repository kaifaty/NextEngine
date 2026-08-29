# NSR3-B4E2D7R20R6 NNQP representative evidence

Status: `PASS / Q1 AND Q2 SUPPORTED / RELINEARIZATION REQUIRED`.

## Frozen result

Implementation `1854c327` ran twice with stable semantic:

```text
bfa10141c0f50658548205e44d397a8999f36a0d0c0e00da0cb9d3d7ebc01500
```

Route:

```text
NNQP_RELINEARIZATION_REQUIRED
```

All parent roots, active-set controls, exact-ratio ties, principal-factor
audits, NNQP complementarity bounds, dual line-search bounds and lifecycle
checks pass. No diagonal, epsilon sign tolerance or block pivot is used.

## Local representatives

| case | initial face | selected support | transitions | principal solves | local KKT |
|---|---:|---:|---:|---:|---|
| supported | 16 | 16 | 18 | 18 | certified |
| filled edge | 38 | 21 | 31 | 31 | certified |
| filled corner | 42 | 28 | 38 | 38 | certified |

The filled supports are strict subsets of the R20R5 full faces, exactly as its
negative unconstrained components predicted. Their local primal and dual
violations are zero at the outward bounds; normalized complementarity is
`2.65e-35` and `7.55e-35`.

## Globalization

The supported case reproduces the R20R5 unit step and strict full KKT
certificate. The filled cases accept certified exact-dual ascent:

| case | accepted alpha | dual increase lower | projector mask changes | full KKT after one step |
|---|---:|---:|---:|---|
| filled edge | `1/4` | `1.173e-2` | 39 | not certified |
| filled corner | `1/8` | `5.188e-3` | 42 | not certified |

The post-step primal/dual-mapping components are approximately `0.32/0.40`
and `0.37/0.45`. This is not a failed direction: the exact dual increases, but
the step crosses dozens of projector faces, invalidating the old local model.
It is direct evidence for nonlinear relinearization.

## Decision

Iterate the same construction. At every accepted multiplier state, rebuild the
joint projector derivative and the natural complementarity face, solve the
cone-aware quadratic Newton model, and reuse the certified dual Armijo search.
The first iterative experiment remains binary128, cold-starts each local NNQP,
and has a fixed nonlinear cap. Runtime optimization is still premature.

No generalization, runtime, GPU or production authority exists.
