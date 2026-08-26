# NSR3-B4E2D7R20R33 v4 manifest evidence

Status: `PASS / V4_HOLDOUT_MANIFEST_FROZEN`.

Implementation `d788afbc` emits twice:

```text
33ff1cbf1a8ae753fb84a607d728e0e82ef487d44875ee7615e1d4aa19fcee63
```

Exactly five immutable `BLIND_HOLDOUT_V4` sources contain 404 fluid samples:

| source | samples | source root |
|---|---:|---|
| upper-x/lower-z torsion `8x3x3` | 72 | `2d61415e...332f` |
| upper-x/lower-y/lower-z corner `4x6x4` | 96 | `45c6a6b1...aaf1` |
| biaxial counterflow `6x4x3` | 72 | `bb6dde92...bcc5` |
| helical compression `4x4x5` | 80 | `656e4d66...2dc1` |
| alternating y layer `3x7x4` | 84 | `32ec8e36...9d7f` |

All finite ownership and exact count gates pass. Source roots and geometry
roots are each pairwise distinct, including the two 72-sample controls.
Operator, projection, preflight, solver and timing flags are all false. R32
remains exact at `e7b9acaf...0f73` after the source addition.

This freezes sources only. No excitation, operator validity, solver
generalization, runtime, GPU, performance or production conclusion exists.
The sources must not change in response to later R34/R35 observations.
