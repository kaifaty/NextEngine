# NSR3-B4E2D7R20R48 third centered-solution evidence

Status: `PASS / THIRD_CENTER_SOLUTION_CANDIDATE / 65 OF 65 RESOLVED`.

Implementation `2de0791b` emits reproducible semantic:

```text
7d4d7a9e3765274338707855df4953d61c44b6a3d15c9362bf12b70eccbe7802
```

Every residual, correction, centered residual and compensated center contains
its exact dyadic oracle without underflow. The six frozen checkpoints are:

| depth | certified radius | signs `+/-/?` | minimum separation |
|---:|---:|---:|---:|
| 1 | `4.8495942212e10` | `28/30/7` | negative |
| 2 | `8.0796205515e6` | `28/30/7` | negative |
| 4 | `2.2426530571e-1` | `32/33/0` | `916.7713` |
| 8 | `6.5099484916e-14` | `32/33/0` | `916.9955` |
| 16 | `6.4926701057e-14` | `32/33/0` | `916.9955` |
| 32 | `6.4926701057e-14` | `32/33/0` | `916.9955` |

All signs first resolve at depth four with zero conflicts. Depth 16 is the
first arithmetic-floor checkpoint and depth 32 is identical. This independently
repeats R41/R45 on a third matrix/RHS. Two R48 executions and R47/R46
regressions are exact. No third correction or NNQP decision occurs in R48.
