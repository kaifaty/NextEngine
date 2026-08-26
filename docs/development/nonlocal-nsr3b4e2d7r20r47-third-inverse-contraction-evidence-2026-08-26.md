# NSR3-B4E2D7R20R47 third inverse-contraction evidence

Status: `PASS / THIRD_INVERSE_CONTRACTIVE_CANDIDATE`.

Implementation `6165ebf5` emits reproducible semantic:

```text
7ce17f9542e0b024aa98b3102ae052b2da2e86797becbf4fbf841bd1d4091d83
```

The exact R46 two-replacement prefix and third tuple are reproduced. Both
orientation audits contain every exact dyadic defect entry without underflow:

| orientation | exact root | Dot2 root | exact/Dot2 `rho` | worst row |
|---|---|---|---:|---:|
| `I-A*X` | `79a5c348...5a5a` | `024cb61d...2b76` | `0.0059078521476` | 10 |
| `I-X*A` | `9454068d...e8b9` | `0992eca3...1c1b` | `0.2417586545923` | 37 |

The inherited certificate rejects the same represented inverse with
`rho=66.3570`, maximum column residual `4.3505e-4` and maximum issued column
bound `0.9399`. The compensated certificate is therefore contractive on both
sides; this is the third independent cancellation-dominated false rejection
in the torsion trajectory.

The first apparatus run used an R47-specific fail-closed provenance label and
correctly failed its frozen final-step parent root before either matrix audit.
It has no scientific credit. The corrected run changes only that label to the
exact R46 value. Two corrected R47 runs, R46 and R45 are exact. R47 applies no
third correction and evaluates no third RHS residual.
