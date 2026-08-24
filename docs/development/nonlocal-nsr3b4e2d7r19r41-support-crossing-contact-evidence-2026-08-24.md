# NSR3-B4E2D7R19R41 support-crossing and contact evidence

Date: `2026-08-24`

Status: `PASS / NONZERO_SUPPORT_CROSSING_REQUIRES_RELINEARIZATION / ROLLBACK EXACT`.

## Outcome

The R40 source and nonlinear trial differ by 1,282 ordered pair identities:
680 pairs leave compact support and 602 enter. These crossings have an almost
negligible density contribution, which explains the nearly exact nonlinear
feasibility ratio in R40, but they are not an exact zero-material shell. The
largest crossing second derivative is nonzero. The trial therefore changes
the curvature/operator graph and must be rebuilt and relinearized before it
can be considered for admission.

Contact attribution is independent and unambiguous: no new box face becomes
penetrating. All 1,268 worsened records are already active faces with an
inward normal step. A later composite step must respect their tangent cone,
but frozen precedence correctly selects support relinearization first.

## Exact support crossing ledger

| Quantity | Result |
|---|---:|
| Current/trial intersection | `339660` |
| Lost / entered | `680 / 602` |
| Lost fluid / support | `534 / 146` |
| Entered fluid / support | `418 / 184` |
| Maximum positive ULP distance from `H` | `69649155` |
| Maximum absolute horizon margin | `1.9331523881671586e-9` |
| Maximum absolute `W` | `2.3795696170601534e-21` |
| Maximum absolute `W'` | `4.0863428898433298e-12` |
| Maximum absolute `W''` | `0.0046782124222959215` |
| Maximum per-center crossing density delta | `3.8085980747601804e-22` |

The complete pair partition is exact:

```text
339660 + 680 = 340340 current pairs
339660 + 602 = 340262 trial pairs
```

The density contribution is extremely small, but the material-zero predicate
is deliberately exact and returns false. Compact support gives
`W(H)=W'(H)=W''(H)=0` at the horizon; it does not make finite interior
crossings exactly zero. In particular, the nonzero `W''` means that a frozen
current-state linearized operator does not own the trial state.

Ledger roots:

```text
crossing density a52e60d4f32d3f8ccbb2f7858bfec258813b84443526c8e8e0d743176bda5aac
crossing records a1385a76d824682cecb5dd827ad452c1873a4ec620a034dbcd12754a019be8ad
crossing audit   9c6ce07a6bb13d2e4334c9777b558d5295d6c8726837140cecb8f2784bd21142
```

## Exact contact ownership

| Quantity | Result |
|---|---:|
| Ordered particle-axis-side tests | `36000` |
| Source / trial penetrating | `1290 / 1290` |
| Newly penetrating / resolved | `0 / 0` |
| Worsened | `1268` |
| Existing inward-normal owners | `1268` |
| Source maximum penetration | `2.9985485743705409e-8` |
| Trial maximum penetration | `3.1322359945323841e-8` |
| Maximum individual increase | `2.0926122501485622e-9` |
| Worst owner | particle `200`, axis `0`, lower side |

The contact ledger root is
`1df2083d4096522f329cdb1ff2a70c5da448ead0d40cda2cdb02e520c88f1ef7`.
There is no contact-crossing event to solve. The missing rule is projection of
the normal step into the feasible tangent cone of contacts already active at
the source state; that work remains ordered after support relinearization.

## Work, rollback and reproducibility

- one exact R40 replay;
- two nonlinear workspace builds and releases, maximum live count two;
- one sorted pair symmetric-difference merge;
- exactly 36,000 direct contact-face tests;
- zero new JVP, VJP, HVP, model, trial, precision or outer operations;
- exact source, trial and endpoint rollback.

Two clean Release builds are binary-identical:

```text
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r41-a.PVaSn7
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r41-b.OBRll0
binary SHA-256 1347b3dd6b0fa3b13e7a0f4c1ae878e54a002c9652453e854937e1621f63b864
size           7579456
ELF build-id   9816585298b400fe4487ae47a5dbab4273e0961b
```

Fresh runs are byte-exact with empty stderr:

```text
/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r41-a.yHsMbS
/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r41-b.t45y1N
stdout bytes   2637
stdout SHA-256 0237a380e86732e1ef78c50ca390ef2f93b1a82ac47338488403d9c5bb00d81c
semantic       3462fdcbc39d821c51fd4fdfbf9d9141ca7fd192fa53e9fb2885162c110f2ad8
stderr bytes   0 / 0
```

The route ledger covers all 12 frozen cases and roots to
`859a360de669c840883bfca5b4fb819677109b0061f737a34018f75d5d2f9e41`.
No timing was measured or interpreted on the shared host.

## Decision

Preserve R40 and R41 exactly. Research and freeze a topology-stable neighbor
superset that contains both current and trial compact-support graphs while
evaluating the exact compact-support kernel at each state, then rebuild the
trial linearized operator on that stable ownership domain. Do not reinterpret
tiny density contribution as exact zero, expand the physical support horizon,
project contact yet, tune merit, commit the correction or claim production
readiness.
