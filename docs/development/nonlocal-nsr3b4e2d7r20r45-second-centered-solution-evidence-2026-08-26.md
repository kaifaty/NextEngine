# NSR3-B4E2D7R20R45 second centered-solution evidence

Status: `PASS / SECOND_CENTER_SOLUTION_CANDIDATE / 65 OF 65 RESOLVED`.

Implementation `6278cc01` emits reproducible semantic:

```text
4263411e0abf582cd45044635d26efa1f83e4d8d3be52170c18c480e87d46080
```

The second exact residual/correction roots are `612b7268...08b5` and
`6c25184c...8325`; every residual, `z`, centered residual and compensated
center contains its exact dyadic oracle without underflow.

| depth | certified radius | signs `+/-/?` | minimum separation |
|---:|---:|---:|---:|
| 1 | `3.8689438650e11` | `23/35/7` | negative |
| 2 | `2.3300302680e8` | `23/35/7` | negative |
| 4 | `8.4508390620e1` | `29/36/0` | `155.5235` |
| 8 | `1.1148371119e-11` | `29/36/0` | `240.0319` |
| 16 | `3.1658780995e-14` | `29/36/0` | `240.0319` |
| 32 | `3.1658780995e-14` | `29/36/0` | `240.0319` |

Signs first resolve at depth four, independently reproducing the qualitative
R41 behavior on a different matrix/RHS. The first arithmetic-floor checkpoint
is depth 16; depth 32 is identical. Maximum exact residual-evaluation error is
`1.58e-19`, versus an issued input-dominated bound `2.604e-14`.

Because R42 already proved that the next NNQP ratio consumes the error radius,
the next trajectory candidate selects depth 16 rather than the minimal sign-
resolving depth four. This selection is predeclared by R45's floor and does not
fit a new cap. Two R45 runs and R44 regression are exact. No second correction
or NNQP decision occurs in R45.
