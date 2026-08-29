# NSR3-B4E2D7R20R63I exported-factor RHS evidence

Status: `PASS / WIDE_RHS_SIGN_CERTIFICATE_REJECTED`.

## Reproducible result

Implementation commit: `20fc8021`.

Command:

```text
/tmp/nextengine-r20r4-build/nonlocal-formula-reclosure \
  --nonlocal-al-generalization-v5-exported-factor-rhs
```

The command repeated byte-identically twice:

```text
stdout sha256  6aaf5889d5c15734db26a4ac7d6118896419f4d7c4027003790b91517d41bd7a
semantic sha256 4055d0b043bd14c7c65a28f052bef02d39a55bc9625a9c2d861f41fdf0466415
route          WIDE_RHS_SIGN_CERTIFICATE_REJECTED
```

The frozen platform, controls, parent reconstruction, lifecycle and exact
work ledgers all pass. The immutable R60 certificate repeats with
`rho=0.0248854338484`, error `2.7127883e-6`, minimum separation `32.4234033`
and the exact `24 positive / 78 negative / 0 unresolved` component pattern.

## Three-lane result

| Lane | original residual infinity bound | certified forward-error radius | signs `+/-/?` | maximum difference from R60 center |
|---|---:|---:|---:|---:|
| stored operator, retained-wide `R`, wide consumption | `6.86363e-2` | `4.14356e15` | `12/24/66` | `4.04056e15` |
| stored operator, exported `R`, wide consumption | `1.38572e-1` | `4.11531e15` | `12/24/66` | `4.01301e15` |
| stored operator, exported `R`, strict binary64 consumption | `7.02779e-1` | `4.05230e15` | `12/24/66` | `3.95157e15` |

All six triangular substitutions complete with the frozen `5,151 + 5,151`
terms and 204 divisions per lane. Every residual and left-inverse image is
enclosed by Dot2 without underflow. Therefore this is not an apparatus,
finiteness, zero-diagonal, orientation or incomplete-work failure.

## Meaning

R63H established a forward operator claim: rounding the rectangular tangent
operator to binary64, building its factor in binary128 and exporting the factor
once still preserves one immutable near-null image. R63I asks the strictly
stronger inverse question for an actual RHS.

The stronger claim fails even before factor export. The retained-wide factor
solves the Gram of the already rounded binary64 operator, while acceptance is
against the original captured Gram. This system is so ill-conditioned that the
small coefficient perturbation is amplified into an order-`1e15` solution
change. A small residual is not a forward-accuracy certificate here: applying
the verified approximate inverse to that residual produces an order-`1e15`
correction, and 66 component intervals still contain zero.

Consequently:

- R63H remains valid; factor fidelity to the stored operator was proved;
- binary64 triangular arithmetic is not the primary boundary, because the
  retained-wide lane already fails;
- binary64 storage is adequate for forward application of the tested weak
  direction but is not adequate by itself for this inverse problem;
- neither exported factor is an admissible NNQP replacement;
- there is no runtime, GPU or production authority.

This is exactly the frozen `WIDE_RHS_SIGN_CERTIFICATE_REJECTED` route. It does
not authorize rank 101, row deletion, regularization or a weaker sign test.

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
```

No timing was admitted. Build or command duration on the shared host is not
performance evidence.

## Next bounded question

Research one original-system residual refinement with the already verified
R60 left inverse. The observed contraction bound predicts that reducing an
initial radius `4.14e15` below the immutable sign margin `32.42` needs at least
nine ideal contractions:

```text
ceil(log(32.42 / 4.14e15) / log(0.0248854)) = 9.
```

Therefore the next contract must not assume that one scalar correction step is
enough. It should freeze a bounded centered/refinement depth before execution,
retain original-system Dot2 residuals, compare wide/export lanes separately,
and stop without publishing state. GMRES-IR or a different operator
representation remains a later branch only if this fixed contraction sequence
cannot recover the sign certificate.
