# NSR3-B4E2D7R20R44 second inverse-contraction evidence

Status: `PASS / SECOND_INVERSE_CONTRACTIVE_CANDIDATE`.

Implementation `84c09a12` emits reproducible semantic:

```text
cdb334f4981ad13c0bcd5a337bd39054e21a6e76b832543141785c02c53ebc08
```

The exact R43 trajectory and both target cardinalities reproduce. The inherited
audit rejects the second candidate with `rho=47.8627`, maximum column residual
`3.34969e-4` and accumulated bound `0.664291`.

The independent exact/Dot2 results are instead:

| orientation | exact outward `rho` | Dot2 bound | worst row |
|---|---:|---:|---:|
| `I-A*X` | `0.005021730191573352` | `0.005021730191573352` | 48 |
| `I-X*A` | `0.17755905186287393` | `0.17755905186287393` | 25 |

All `4225+4225` entries are contained without underflow. Maximum actual entry
errors are `3.41e-37` right and `5.41e-37` left, while issued Dot2 bounds are
about `1.44e-33`. The old right enclosure is therefore roughly four orders of
magnitude too wide and gives the wrong contractivity verdict again.

Both orientations are safely below one. This proves the second represented
inverse is suitable for a centered fixed-point certificate, but says nothing
yet about the signs of its particular RHS solution. Two R44 runs and R43/R42
regressions are exact. R44 applies no second correction and adds no solve,
inverse column, trial or state change.
