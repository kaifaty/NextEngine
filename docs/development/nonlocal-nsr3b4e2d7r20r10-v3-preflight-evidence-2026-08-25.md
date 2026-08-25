# NSR3-B4E2D7R20R10 v3 preflight evidence

Status: `PASS / ALL BLIND HOLDOUTS EXCITED`.

Implementation `10d2ac7e` ran twice with stable semantic:

```text
c2328bfb4c6b0ed90a66274a03b5abe51535918c513b3f56693d82ff4c87b577
```

Route `V3_PREFLIGHT_PASS`. All workspace, R64 slot/entry/incidence,
scale-invariance, distinct-problem and lifecycle gates pass. Solver,
candidate and inverse iterations remain zero.

| holdout | rows | certified positive rows | maximum certified residual | problem root |
|---|---:|---:|---:|---|
| lower-x/upper-z edge | 72 | 43 | `4.431e-3` | `3dc846e5...dee3` |
| lower-x/lower-y/upper-z corner | 75 | 50 | `5.072e-3` | `9911ae71...8c51` |
| upper-y shear layer | 70 | 65 | `3.585e-3` | `08c636f4...8b62` |
| radial compression | 60 | 47 | `7.742e-3` | `948f5abd...ab22` |

Every source has zero positive source constraints, so the zero-step feasibility
witness remains valid. All four projected targets are strongly nontrivial; no
holdout is reclassified or replaced.

The exact R8 solver may now run once on these four immutable problem roots.
No generalization result exists yet, and no runtime/GPU/production authority
is implied.
