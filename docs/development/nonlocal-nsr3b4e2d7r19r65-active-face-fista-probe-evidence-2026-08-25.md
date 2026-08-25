# NSR3-B4E2D7R19R65 active-face FISTA probe evidence

Date: `2026-08-25`

Status: `EXPLORATORY PASS / ACCELERATION REJECTED / NO SCIENTIFIC CREDIT`.

Implementation commit: `b04124f7`.

The probe reproduces exact R63 parent stdout
`38298214e352a87cfffb5c5432be90ef822c3f8ab00da5a2f7a3b1cb64565b62`,
builds the bounded-equivalent R64 sparse operator and completes the frozen
`16 outer x 16 projected-FISTA` schedule. It does not execute a nonlinear
trial, mutate runtime state, admit timing or claim production authority.

## Result

The working set grows from 1,080 to 4,680 rows and is closed by outer 4.
Every checkpoint has bit-exact joint reprojection. The reconstructed Dykstra
stationarity norm remains below `4.87e-22`, complementarity falls to
`1.95e-14`, and model reduction remains positive.

| outer | owned | positive raw | maximum raw | projected-gradient norm |
|---:|---:|---:|---:|---:|
| 1 | 2,520 | 1,792 | `2.4029312652998089e-7` | `1.5950169052523058e-7` |
| 2 | 3,960 | 2,184 | `1.7503190972249659e-7` | `3.2410139384141010e-7` |
| 4 | 4,680 | 1,516 | `1.9170743162266662e-8` | `1.6319308651021516e-7` |
| 8 | 4,680 | 1,272 | `3.9993404184649023e-9` | `6.6543577248530443e-8` |
| 16 | 4,680 | 1,196 | `1.1823169686944491e-9` | `9.5382120867736572e-9` |

Semantic result SHA-256:
`aba41214a36d4910a4d18c872c9bb170f7361b1e8d8142f0c77897ee794b192c`.

## Structural work

```text
FISTA iterations                         256
backtracking trials                      498
momentum restarts                          0
A^T calls                                770
A calls                                  288
joint projections                         16
fresh all-row audits                      17
dense Gram storage                         0
```

With 535,588 R64 aggregated entries and 605,144 directed slots, the explicit
operator work is:

```text
770 * 535,588 + 288 * 605,144 = 586,684,232 sparse terms
17 fresh audits * 605,144       =  10,287,448 directed slots
total excluding joint projection = 596,971,680 terms/slots
```

This is a structural comparison, not a wall-clock estimate.

## Classification

The frozen acceptance rule required a strong reduction at equal operator work.
It is not met. At outer 16, FISTA is only `1.18x` below the plain `omega=1`
Hildreth raw at cycle 64, while the already-recorded `omega=1.5` cycle-64 raw
is `2.99x` smaller than FISTA (`3.9595935263278441e-10`). Backtracking also
turns 256 nominal gradient iterations into 770 transpose calls.

Route: `ACTIVE_FACE_FISTA_REFERENCE_RETAINED`.

The negative result is algorithmic: scalar-Lipschitz projected acceleration
does not resolve the ill-conditioned correlated face efficiently. It does not
invalidate the dual model, sparse operator, monotone ownership or joint-set
Dykstra decomposition.

## Next research

Use a two-phase bound-QP method: one direct Hildreth/gradient-projection sweep
identifies the current nonnegative face, then Jacobi-preconditioned conjugate
gradients minimize the quadratic inside the strictly positive face. This is
the large-scale GPCG pattern of Moré and Toraldo:

- [Algorithms for bound constrained quadratic programming problems](https://doi.org/10.1007/BF01396045)
- [On the solution of large quadratic programming problems with bound constraints](https://doi.org/10.1137/0801008)

The next probe must remain matrix-free, truncate at the first nonnegative-dual
boundary, reconstruct before the joint projection and compare exact operator
work against this FISTA result.
