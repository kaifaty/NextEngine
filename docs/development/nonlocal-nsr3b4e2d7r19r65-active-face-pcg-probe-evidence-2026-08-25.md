# NSR3-B4E2D7R19R65 active-face PCG probe evidence

Date: `2026-08-25`

Status: `EXPLORATORY PASS / FACE IDENTIFICATION REQUIRED / NO SCIENTIFIC CREDIT`.

Implementation commit: `78794e91`.

## Invalid first execution

The first execution incorrectly sent `target-p_C-A^T lambda` to the joint-set
projector after the fixed-`p_C` density block. Its stationarity rose to about
`1e-6`. The result is invalid and has no credit. The corrected implementation
projects `target-A^T lambda`, exactly as the two-set Dykstra decomposition
requires. This restores stationarity to rounding scale.

## Valid corrected result

The corrected probe returns `PASS` with route
`ACTIVE_FACE_PCG_FACE_IDENTIFICATION_REQUIRED`. It reproduces exact R63 parent
stdout `38298214e352a87cfffb5c5432be90ef822c3f8ab00da5a2f7a3b1cb64565b62`,
keeps every joint reprojection bit-exact and retains positive model reduction.

| outer | owned | face | maximum raw | projected-gradient | face alpha |
|---:|---:|---:|---:|---:|---:|
| 1 | 2,520 | 1,074 | `2.8744963569865389e-7` | `4.3077354368871887e-6` | `2.145751449322521e-3` |
| 2 | 3,960 | 2,211 | `1.9911554750701692e-7` | `4.0639253245835338e-6` | `3.7125181765438483e-3` |
| 4 | 4,680 | 4,342 | `1.1748281177051966e-7` | `2.3894648177847395e-6` | `7.4825461891324534e-5` |
| 8 | 4,680 | 3,458 | `3.8165833616780880e-8` | `4.4041732631363254e-7` | `9.0406275243539282e-4` |
| 16 | 4,680 | 2,694 | `2.0319667027316305e-8` | `1.5534832416361686e-7` | `1.0905061088671182e-3` |

Checkpoint stationarity ranges from `5.93e-22` down to `1.33e-23`.
There are no nonpositive-curvature stops. Every local PCG direction is descent.
The final block reduces its face residual from `1.5555e-7` to `9.8880e-8`
before truncation.

Semantic result SHA-256:
`5cf99f77b4df7d5d58913c7acebb6fd6efc2265ec0c3c91a4cf0b3a10ee5a597`.

## Structural work

```text
identification sweeps                 16
coordinate visits                 68,400
coordinate updates                52,616
coordinate residual terms      6,395,200
coordinate update terms        4,900,667
PCG iterations                       256
A^T calls                            288
A calls                              288
fresh audits                          17
structural terms             350,114,131
FISTA structural terms       596,971,680
dense Gram storage                      0
```

The PCG route therefore removes `41.35%` of the counted sparse work relative
to FISTA. This is structural evidence, not timing evidence.

## Why it is not the selected accelerator

All 16 outer blocks are boundary-limited. A single tiny positive multiplier
limits the accumulated face-Newton direction, so each outer applies only
`7.48e-5` to `3.71e-3` of that direction and discards the remaining useful
Krylov work. Final maximum raw is `17.2x` larger than FISTA, and projected
gradient is `16.3x` larger. The operator is cheaper and well behaved, but the
one-truncation-per-outer active-set policy is wrong.

Do not add PCG depth. Restart the face inside the same Hessian-product budget:
when a direction hits `lambda=0`, fix that coordinate, refresh the exact raw
residual, restart PCG on the reduced positive face and continue with the
remaining budget.
