# NSR3-B4E2D7R20R63J original-residual refinement evidence

Status: `PASS / EXPORTED_FACTOR_REFINED_RHS_CANDIDATE`.

## Reproducible result

Implementation commit: `341a1169`.

Command:

```text
/tmp/nextengine-r20r4-build/nonlocal-formula-reclosure \
  --nonlocal-al-generalization-v5-original-residual-refinement
```

The command repeated byte-identically twice:

```text
stdout sha256   b4a2908e1cfb724354e16895221df688d27675de48de5da9717674d2fe99191d
semantic sha256 4a4d1798a88ba0adcc4fd869fefb0ddc536491f5bad1bd49cc6505d57aa78f96
route           EXPORTED_FACTOR_REFINED_RHS_CANDIDATE
```

Platform, controls, exact R63I reconstruction, common defect, fixed work and
lifecycle gates all pass. The common `C=I-XH` construction repeats
`rho_bound=0.0248854338484`, worst row 35, maximum Dot2 entry bound
`9.60e-34`, and root `85af6bc4...f839b`.

## Checkpoint result

All three lanes first certify at the earliest frozen checkpoint, depth 4:

| Initial lane | depth-4 certified error | depth-4 maximum difference from R60 center | depth-4 minimum sign separation | signs |
|---|---:|---:|---:|---:|
| retained-wide factor/consumption | `3.11077e-3` | `3.03609e-3` | `32.4202953` | `24/78/0` |
| exported factor, wide consumption | `1.98558e-3` | `1.93887e-3` | `32.4214205` | `24/78/0` |
| exported factor, strict-binary64 initial consumption | `4.51424e-3` | `4.40467e-3` | `32.4188918` | `24/78/0` |

The independently recertified depth-8 centers reach stable arithmetic floors:

```text
wide          error 5.08902e-4, reference difference 4.98897e-4
export/wide   error 5.98554e-4, reference difference 5.81029e-4
export/f64    error 1.96967e-3, reference difference 1.92336e-3
```

Depths 9, 10, 12 and 16 remain fully certified with the same componentwise
sign pattern. They are evaluated despite the earlier pass; no adaptive exit or
post-result ladder change exists.

## Why depth 4 beat the norm prediction

The frozen worst-case estimate used only
`||I-XH||inf<=0.0248854` and the initial radius `4.14e15`; it predicted that
nine ideal contractions might be needed to cross the sign margin. That bound
must cover every error direction. The actual stored-operator error is aligned
with a direction on which the defect acts much more strongly than its global
infinity norm, so four centered terms reduce the independently measured error
to order `1e-3`.

This does not invalidate the estimate or fit a new threshold. The depth ladder
was committed before execution, and every reported pass comes from a fresh
original `H,b` Dot2 certificate rather than the analytic prediction.

## Meaning

R63I's failure is recoverable: binary64 operator/factor storage did not destroy
the RHS information irreversibly. An original-system residual plus a
contractive correction operator can remove the order-`1e15` forward error and
recover all 102 solution signs.

However, the correcting operator in R63J is the existing dense verified R60
inverse `X`, and the correction recurrence uses binary128. `X` is an oracle
artifact, not the desired production representation. Therefore the result:

- validates original-residual correction as the right mathematical mechanism;
- preserves the exported factor as a viable preconditioner candidate;
- does not authorize integrating the R63I solution or R63J dense correction;
- does not establish runtime cost, GPU suitability or production readiness.

The next bounded experiment should use the retained/exported rectangular
factor itself to solve standard iterative-refinement corrections. It should
compute residuals against original `H,b`, use `X` only as an independent
certificate, and freeze a bounded iteration ladder. If standard refinement is
noncontractive, GMRES-IR/PCG or an explicit weak-direction low-rank correction
becomes the next branch.

## Regression evidence

All parent stdout hashes remain byte-identical:

```text
R63B 320d6ea1842d49fef30b6821586b189dc923ef5bcfe9e3c22e86f4a38bc24216
R63C 0ace54f4baa595a931db4a7da14464fca22b1cdfe68d4869212b50c39b56581b
R63D 11b8ddd92e5b4163f6122227e482aa64ec56a9926c6993c1293d03e74015e619
R63E db3e61ad2955ccbedb1e98aaf0258a7aab0e8c5696f26c87f0ef09e0ba3c0710
R63F c0c07b866bf73eeece8d8a66b6111d89e8d2383be3837dd6af0b7e6393de7304
R63G 9de7752302bce06d9cf1cae7de8a7f8b83e3d87000f80f1ece09e87eb6881908
R63H 2d56e2acba646946f53325553a586569815992b0ee7da9ec716c611d00f9176a
R63I 6aaf5889d5c15734db26a4ac7d6118896419f4d7c4027003790b91517d41bd7a
```

No timing was admitted. Shared-host command duration is not performance
evidence.
