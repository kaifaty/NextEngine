# NSR3-B4E2D7R20R63M preconditioned-CG evidence

Status: `PASS / EXPORTED_FACTOR_WIDE_PCG_CANDIDATE`.

Claim status: `SUPPORTED_BOUNDED` for the one immutable R63I captured RHS and
the frozen Linux x86_64 binary128 arithmetic profile. Evidence classes:
`NUMERICAL`, `EXACT_CERTIFICATE`, `CORRESPONDENCE`.

## Reproducible result

Implementation commit: `6dcc3bc3`.

Command:

```text
/tmp/nextengine-r20r4-build/nonlocal-formula-reclosure \
  --nonlocal-al-generalization-v5-preconditioned-cg
```

Two independent executions are byte-identical:

```text
stdout sha256   9eebbd19938c95342bb4cb919d5648eaf05ea1ef1971bb0e4369f2225e8d09e7
semantic sha256 3fb7fdd65f887ae2a304dfdf8f1d10996c296d7a85c72ec79655be55e40ed424
route           EXPORTED_FACTOR_WIDE_PCG_CANDIDATE
controls root   7bb70b34fe6863c0faf93cc9831ae1743ee00fd59e0186d85ae4acaff2d5cfff
```

Platform, positive and negative controls, exact R63L reconstruction, both PCG
lanes, every-iterate original-system certificates, work and lifecycle gates
all pass. The two lane roots are:

```text
retained-wide  4a7686b2948b0951302e4f6c1d397681818ff26c89f1c22eadcfbdaff8ce2220
export/wide    ed31e5f6f933a56e487b275be3beae4a0e22cd53af76fd5e7b4001d4a7dbb0fd
```

## Exact PCG frontier

Both lanes start from their original rejected R63I solutions, not from an
R63K/R63L refined solution. Their first update still leaves the inherited 66
weak signs unresolved. Their second update certifies the complete immutable
R60 sign pattern:

```text
lane             iteration 1                   iteration 2
retained-wide    error 1.71319e15, 66 open     error 8.10567e-4, 24+/78-/0
export/wide      error 3.52719e15, 66 open     error 3.27724e-4, 24+/78-/0
```

The frozen experiment continues through iteration 8 after first success. All
later iterates remain `24+/78-/0`. The maximum recurrence-versus-direct
residual drifts over all eight updates are bounded by:

```text
retained-wide  5.602329131865594e-19
export/wide    3.859294325780417e-19
```

No nonpositive `r^T z` or `p^T H p`, nonfinite update, loss of certificate or
PCG breakdown occurs.

## Deterministic work result

The full predeclared eight-update experiment performs, across both lanes:

```text
                         factor solves   algorithm H applications
stationary R63L baseline      38                    38
R63M PCG                      16                    18
```

This is a `57.9%` reduction in factor solves and a `52.6%` reduction in
original-operator applications under the frozen equal-work accounting. The
iteration-2 certificate exposes a possible smaller stopping policy, but the
current verifier uses the dense R60 inverse and is offline evidence; it does
not authorize a two-update runtime algorithm or a production tolerance.

Each completed lane has exactly eight factor solves, nine algorithmic
`H` applications, eight positive `r^T z` reductions, eight positive
`p^T H p` reductions and eight independent sign certificates. Certificate
work is published separately and excluded from algorithmic work.

## Meaning and ceiling

For the captured dimension-102 system, PCG is the selected lower-work
correction algorithm. The exported binary64 factor remains usable when it is
consumed with binary128 PCG arithmetic. GMRES-IR and an explicit weak-direction
correction are not selected at this boundary.

This result does not establish matrix-free correspondence, hardware-available
mixed precision, an admissible runtime stop rule, state replacement, a
following NNQP transition, trajectory robustness, GPU throughput or production
readiness. Dense `H` and the dense verified inverse remain research-only
objects in R63M. No wall timing was admitted.

The smallest next discriminator is dense `H p` versus the direct rectangular
application

```text
H p = (1 / (1 + eta)) T (T^T p),
```

using the immutable `102 x 315` tangent operator and the same fixed binary128
Dot2 order. It must bind product error and replay both PCG lanes without
changing preconditioner, start, budget or sign semantics.

## Regression evidence

R63B--R63L public stdout remains byte-identical:

```text
R63B 320d6ea1842d49fef30b6821586b189dc923ef5bcfe9e3c22e86f4a38bc24216
R63C 0ace54f4baa595a931db4a7da14464fca22b1cdfe68d4869212b50c39b56581b
R63D 11b8ddd92e5b4163f6122227e482aa64ec56a9926c6993c1293d03e74015e619
R63E db3e61ad2955ccbedb1e98aaf0258a7aab0e8c5696f26c87f0ef09e0ba3c0710
R63F c0c07b866bf73eeece8d8a66b6111d89e8d2383be3837dd6af0b7e6393de7304
R63G 9de7752302bce06d9cf1cae7de8a7f8b83e3d87000f80f1ece09e87eb6881908
R63H 2d56e2acba646946f53325553a586569815992b0ee7da9ec716c611d00f9176a
R63I 6aaf5889d5c15734db26a4ac7d6118896419f4d7c4027003790b91517d41bd7a
R63J b4a2908e1cfb724354e16895221df688d27675de48de5da9717674d2fe99191d
R63K 7b13fa402665836d36f3442942c34c2c611df942df0fff582034f487224773c8
R63L bf59f1edd265db77fb993fbb6e366e93514085aa63dd7e864909e7bec5f96de5
```

Build passed for target `nonlocal-formula-reclosure`. No CPU/wall performance
comparison was run on the shared host.
