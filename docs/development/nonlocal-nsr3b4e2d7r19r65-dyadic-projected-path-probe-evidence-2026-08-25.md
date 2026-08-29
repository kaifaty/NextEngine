# NSR3-B4E2D7R19R65 dyadic projected-path probe evidence

Date: `2026-08-25`

Status: `EXPLORATORY PASS / MODEL REJECTED / MODEL-AWARE PATH REQUIRED`.

Implementation commit: `17709733`.

The v4 probe reproduces exact R63 parent stdout and returns route
`DYADIC_PROJECTED_PATH_MODEL_REDUCTION_REJECTED`. Sparse recurrence,
curvature, KKT, joint reprojection, rollback and frozen work accounting pass.
The result has no nonlinear, timing, runtime or production authority.

## Result

| outer | face | dyadic trials | alpha | predicted zeros | applied zeros | maximum raw | projected gradient | model reduction |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 1,074 | 1 | `1` | 240 | 240 | `3.1108357693353815e-7` | `2.6650423810233326e-6` | `1.7759100325346879e-13` |
| 2 | 2,111 | 1 | `1` | 492 | 492 | `2.4050968851511664e-7` | `3.1482632133173886e-6` | `-4.2683422480279146e-15` |
| 4 | 3,512 | 2 | `1/2` | 960 | 699 | `9.5384986723051893e-8` | `1.7392245618662511e-6` | `2.8634005653348768e-14` |
| 8 | 2,431 | 4 | `1/8` | 306 | 178 | `1.5469168048356014e-8` | `2.1171244505011781e-7` | `1.6297255019194601e-14` |
| 16 | 2,262 | 1 | `1` | 47 | 47 | `1.4715292272917613e-9` | `2.1891803600714008e-8` | `1.8038227136697871e-14` |

All 240 face-PCG products have positive curvature. Stationarity is
`3.67e-22..6.65e-24`; recurrence gaps remain `9.81e-21..2.21e-20` against
conservative bounds `6.92e-16..2.05e-15`. Parent stdout SHA-256 is
`38298214e352a87cfffb5c5432be90ef822c3f8ab00da5a2f7a3b1cb64565b62`.
Semantic result SHA-256 is
`096d3ae80d5dcc27269d50a2658b27ead3e354ca7d9fc302282ba02bf3a1c67b`.

## Work and comparison

```text
face PCG products                    240
projected-path candidates             39
predicted / applied zeros     5,260 / 4,065
A^T / A calls                   311 / 272
joint projections                     16
structural terms              351,545,778
FISTA structural terms        596,971,680
dense Gram storage                      0
```

The true projected path applies the face changes that the v3 endpoint chord
could only predict. It uses `41.11%` less counted sparse work than FISTA and
ends near its residual, but does not strictly dominate it: terminal maximum
raw is `1.24x` higher and projected gradient is `2.30x` higher.

## Why the valid dual step is rejected

Candidate selection minimizes the fixed-correction dual quadratic. The
accepted outer-2 candidate has strict negative dual change, but after the
subsequent exact projection onto the joint contact-box/trust-ball set the
high-level inertia model increases by `4.2683422480279146e-15` relative to
the frozen normal reference. Therefore fixed-block dual descent is necessary
but not sufficient for the composed alternating step.

Do not weaken the positive-model gate or discard the useful projected-path
geometry. The next probe must evaluate each dyadic candidate through the
same exact joint projection and accept it only when both the dual objective
and final inertia model strictly improve.
