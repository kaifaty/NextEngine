# NSR3-B4E2D7R19R65 face-restart PCG probe evidence

Date: `2026-08-25`

Status: `EXPLORATORY PASS / DEPTH REJECTED / BATCH FACE CHANGE REQUIRED`.

Implementation commit: `dfa5a2c3`.

The v2 probe preserves exact R63 parent stdout, completes all frozen work and
returns route `FACE_RESTART_PCG_DEPTH_REQUIRED`. It has no nonlinear, runtime,
timing or production authority.

## Result

| outer | face | restarts | maximum raw | projected-gradient | minimum fraction |
|---:|---:|---:|---:|---:|---:|
| 1 | 1,074 | 16 | `2.7049745509906773e-7` | `3.4286485995381830e-6` | `1.176707406220522e-3` |
| 2 | 2,223 | 16 | `1.6203138988666658e-7` | `2.9278852533390297e-6` | `1.3587279989674122e-3` |
| 4 | 4,333 | 16 | `5.8379503572167217e-8` | `1.2454292498968600e-6` | `1.116231448925810e-3` |
| 8 | 3,434 | 16 | `4.0103411166745259e-8` | `4.1085514295130992e-7` | `1.9211291923820038e-4` |
| 16 | 2,652 | 16 | `1.6124919674251280e-8` | `1.2961763856635220e-7` | `1.2108637398625618e-5` |

Stationarity remains between `3.86e-22` and `5.20e-24`; joint reprojection is
bit-exact and model reduction positive at every checkpoint. There are zero
curvature stops. Maintained-to-fresh `q` gaps are `8.95e-21..2.37e-20`, while
their conservative bounds are `5.70e-16..3.67e-15`.

Semantic result SHA-256:
`125ce5ce942897feeb1189bcfff5fb2640d6f89b5bf95f008eb725510218b4e6`.

## Exact structural work

```text
PCG Hessian products                 256
boundary hits / face restarts        256 / 256
boundary residual refreshes          256
A^T calls                            288
A calls                              544
boundary direct row terms         24,090
total structural terms       504,992,816
FISTA structural terms       596,971,680
dense Gram storage                      0
```

This is `15.41%` less counted sparse work than FISTA, but every available PCG
product removes only one blocking coordinate. Relative to the one-shot PCG
probe, final maximum raw improves only `1.26x` and projected gradient `1.20x`.
It still loses to FISTA by `13.64x` and `13.59x`, respectively.

## Classification

The active-set boundary policy, not PCG curvature or recurrence, is the
bottleneck. Increasing the HVP budget would scale one-row removal linearly and
is rejected. The next method must change many binding coordinates together.

Retain one Hildreth identification sweep, compute an approximate face-Newton
direction, project its full trial onto `lambda>=0` in batch, and minimize the
exact quadratic along the resulting feasible direction.
