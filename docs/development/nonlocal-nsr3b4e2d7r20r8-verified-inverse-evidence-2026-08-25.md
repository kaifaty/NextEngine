# NSR3-B4E2D7R20R8 verified inverse evidence

Status: `PASS / DEVELOPMENT CORPUS CERTIFIED`.

## Frozen result

Implementation `301b3a85` ran twice with stable semantic:

```text
afca1c777053172169e227bcf3625ad7c828892520340db5eb45c98d53b4374a
```

Route:

```text
VERIFIED_INVERSE_DEVELOPMENT_CERTIFIED
```

The unchanged R7 executable retains its frozen negative semantic
`8ca318ee...1c58`. The refinement is therefore isolated rather than a rewrite
of the negative result.

## On-demand audit

Only filled corner invokes the verified inverse, during nonlinear iteration
six. As the active NNQP grows, three passive systems require audits:

| dimension | inverse column solves | `rho=||I-HQ||_inf` upper | cheap error | refined error |
|---:|---:|---:|---:|---:|
| 60 | 60 | `1.41e-24` | `2.13e-1` | `1.15e-23` |
| 61 | 61 | `1.70e-24` | `8.92` | `1.31e-23` |
| 62 | 62 | `2.37e-24` | `1.71e2` | `1.57e-23` |

All `rho` bounds are vastly below one, so their Neumann inverse enclosures are
strict. The solution values, pivots and passive sequence are unchanged. The
old recursive bound is demonstrated to be conservative rather than unsafe.

## Complete development result

- four quiet transfer/negative cases: iteration zero;
- supported: one accepted iteration;
- filled edge: eight accepted iterations;
- filled corner: six accepted iterations.

Both filled terminal tuples are below `2^-70` in every frozen KKT component.
Corner ends near `1.29e-32` primal, `1.82e-32` dual mapping,
`5.45e-34` complementarity, zero stationarity and `2.61e-30` scaled gap.

This is the first strict end-to-end development certificate for the
cone-aware relinearized semismooth solver. It is not generalization evidence:
both v2 filled cases have influenced the method since R20R3.

## Decision

Freeze a new source-only v3 holdout manifest before constructing operators or
running preflight/solver code. Generalization remains blocked until the
unchanged R8 algorithm certifies those new cases.

No runtime, GPU or production authority exists.
