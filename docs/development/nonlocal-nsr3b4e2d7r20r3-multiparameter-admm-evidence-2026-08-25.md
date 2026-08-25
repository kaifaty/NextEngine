# NSR3-B4E2D7R20R3 multiparameter ADMM evidence

Status: `PASS HARNESS / M3 INSTABILITY SUPPORTED / EMITTED ROUTE DEFECT`.

## Frozen result

Implementation `58132477` ran once with exact parent semantic
`929a6676182dc7702a8950343833e56faad4ac46e4784fda1c67f169f6eff08c`.
All binary128 factors, linear audits, finite checks and workspace lifecycle
passed. Result semantic:

```text
cd094b7fe5cb7ca5a713f60185493250fb9d66ed3221038eb1c8a0f0a25db486
```

The supported transfer case certified at cycle 1,024, versus 16,384 for fixed
`rho=1`. Its KKT tuple is between approximately `1e-23` and `1e-24`, with
signed gap `1.881e-37` inside a `6.837e-33` bound. Thus independent block
penalties can dramatically accelerate a regular regime.

The filled development cases did not certify at the fixed 16,384 cap. Their
penalty histories repeatedly traverse several orders of magnitude and hit the
upper `2^20` safeguard. The visible filled-edge terminal tuple regresses from
the fixed-penalty R20R2 state:

| metric | R20R2 fixed rho | R20R3 MpSRA |
|---|---:|---:|
| primal KKT | `3.345e-5` | `4.885e-5` |
| projected dual | `3.376e-5` | `4.145e-2` |
| complementarity | `1.122e-6` | `8.801e-4` |
| stationarity | `1.626e-6` | `7.733e-2` |

The final edge penalties at the selected checkpoint are approximately
`rho_density=1.43e2`, `rho_domain=4.71e3`; subsequent history continues to
large oscillations. The corner also reaches the cap without certification and
repeatedly saturates the safeguard.

## Reporting defect

The executable emitted
`MULTIPARAMETER_ADMM_ACTIVE_SET_REFINEMENT_REQUIRED` because its `stable` bit
only represented finite arithmetic and valid Cholesky factors. The frozen M3
hypothesis explicitly included oscillatory cap/safeguard behaviour and KKT
regression as instability. Therefore the scientific classification is M3
supported; the emitted route classifier is incomplete and has no selection
credit.

Do not repair the label and rerun the same development probe. Preserve the
semantic and add an oscillation/KKT-regression gate to future adaptive-solver
reports. MpSRA is stopped for this nonsmooth changing-active-set problem; no
interval, bound or penalty grid follows.

## Next boundary

The useful result is not an adaptive penalty policy but a warm-start lesson:
global coupling gets close and can identify structure, while first-order
consensus updates struggle to finish. Research a direct semismooth/active-set
polish with exact projector generalized derivatives and dual globalization.
Validate the derivative and degeneracy profile before executing a Newton step.

The v2 filled cases remain development data. No generalization, runtime, GPU or
production authority exists.
