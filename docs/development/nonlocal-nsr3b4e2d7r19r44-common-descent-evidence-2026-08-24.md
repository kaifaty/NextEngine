# NSR3-B4E2D7R19R44 contact-feasible common-descent evidence

Date: `2026-08-24`

Status: `PASS / CONTACT_FEASIBLE_COMMON_DESCENT_CANDIDATE / ROLLBACK EXACT`.

## Outcome

At the exact R43 projected trial, the negative complete-merit gradient remains
almost unchanged after projection into all source-active contact tangent
half-spaces and is a strict first-order descent direction for both complete
merit and the density hinge. A density-null-space QP is therefore not required
for this state. The next experiment is bounded nonlinear line globalization,
not a full composite SQP solve.

## Direction and operator

| Quantity | Result |
|---|---:|
| Dimensionless gradient/raw norm | `5.4651187820645202e-9` |
| Contact-projected direction norm | `5.4585174082622525e-9` |
| Clamped gradient components | `730` |
| Density image norm `||A d||` | `2.4669167087420759e-9` |

```text
gradient root da86a85d98bce2f77cc281249d63f3f1d27d7a735ec67ed85393adb4183422e4
raw root      0f3e558dc0ec23be03aec5be8fba63ea85d53a259cc9a701a83f2c56230edc45
contact root  71765e02ba9965489e79d86f3399d1c6401f25fa873d04d664146e4308d45b93
image root    1dbda950816d37ef2ffab831bd3df5345fefbd15a5dba05301abf7aa045dab20
```

Projection is idempotent, contact-feasible, nonzero and norm-nonincreasing.

## Strict directional signs

| Slope | Raw | Normalized |
|---|---:|---:|
| Complete merit | `-2.9795412296302061e-17` | `-0.99879208960216359` |
| Density hinge | `-2.8973468185798968e-17` | `-0.31721833979269115` |

Both dots are accumulated in long double and selected only by strict sign;
no observed epsilon is fitted. This is first-order existence evidence only.
It does not establish a finite accepted step across nonlinear active-set and
topology changes.

## Reproducibility

Implementation commit: `cc95b777`.

```text
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r44-a.nMWKVR
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r44-b.wlCIdp
binary SHA-256 f8701068622bedc12c9010799898dc3b8422834400d55ef7e678b9a64d092d12
size           7673384
ELF build-id   1e7dc3060a34e8c6d462d558965bad28a59e8af0

/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r44-a.jeed51
/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r44-b.67oeNQ
stdout bytes   1446
stdout SHA-256 2fb4299081627e36752d13b4d3ca53046b3e19e8b72596bc318bb4ab85355292
semantic       b1485fd07b22ce57c291be9842cfcab9e3993c1ddb5aec730b732c3836c7a9a1
stderr bytes   0 / 0
route cases    10 / 10
route root     525e44d51a58f0bb82833e942fcdbc4c98845da0330671f1f467e4cd09ac2de6
```

No timing was measured or interpreted on the shared host.

## Decision

Preserve the R44 direction as a rollback-only common-descent candidate.
Research its scale parameterization and a bounded nonlinear line acceptance
that requires contact feasibility, stable-superset coverage, nonlinear hinge
progress and precision-resolved positive complete merit. Do not infer a usable
`alpha` from the raw gradient magnitude or commit a state.
