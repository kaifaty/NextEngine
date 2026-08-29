# NSR3-B4E2D7R20R36 v4 failure-budget evidence

Status: `PASS / V4_FAILURE_BUDGET_IDENTIFIED / TWO INDEPENDENT BOUNDARIES`.

Implementation `cc1d40fc` reproduces only the frozen R35 torsion and
counterflow failures and emits byte-stable semantic:

```text
448a9b8b64a015d0dcbceac82555216ad6421e22a5b4a8b62a95fcfe42eab90f
```

## Torsion

The sole existing inverse audit fails as
`INVERSE_NORM_RHO_NOT_CONTRACTIVE`:

| field | value |
|---|---:|
| passive dimension / solved columns | `65 / 65` |
| maximum column residual | `3.901991471185238e-4` |
| maximum column arithmetic bound | `6.071173848328734e-1` |
| inverse residual norm `rho` | `3.753859524925144e+1` |
| cheap direction error | `7.109566050753423e+38` |
| inverse audit root | `5fe4d71e...5e7c` |
| norm root | `c2d53190...6dfd` |

The factor and every inverse column solve remain finite and pass their local
residual checks. Failure occurs only when the accumulated residual enclosure
must prove `rho < 1`; no inverse-norm or refined direction bound is therefore
issued.

## Counterflow

The iteration-10 natural face reproduces at 66 rows. Nominal slope and its
ordered bound both match the stored R35 binary128 values bit for bit:

| term | value | ratio to nominal slope |
|---|---:|---:|
| nominal slope | `1.849316406201201e-19` | `1` |
| residual-bound contribution | `6.976821282600160e-35` | `3.77265e-16` |
| direction-error contribution | `2.012390908441786e-14` | `1.08818e+5` |
| arithmetic-rounding contribution | `5.934356352612094e-50` | `3.20895e-31` |
| stored/recomposed bound | `2.012390908441786e-14` | `1.08818e+5` |

The NNQP-local model is still positive and certified (`slope 3.09903e-2`,
bound `2.04617e-3`), but its accumulated scalar direction error
`8.852949539615971e-3` is too wide after mapping the direction back to the
global dual slope. Residual evaluation and rounding are not the barrier.

The first observer build emitted invalid top semantic `855b660a...a651`: it
incorrectly applied the scalar direction error to rows outside the natural
face. The repaired observer reconstructs the 66-row face and requires its root
before recomposition; all parent case/step roots are unchanged. The invalid
observer result receives no scientific credit.

Two repeated R36 runs are exact. R35 remains
`59c32c19...37b3` and R32 remains `e7b9acaf...0f73`. R36 adds zero alternative
inverse solves, trials or state updates. The result selects separate bounded
certificate research; it does not authorize weaker bounds, solver changes,
timing, runtime or production promotion.
