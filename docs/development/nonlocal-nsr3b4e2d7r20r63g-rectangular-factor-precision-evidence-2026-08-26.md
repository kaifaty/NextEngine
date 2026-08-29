# NSR3-B4E2D7R20R63G rectangular-factor precision evidence

Status: `PASS / BINARY64_RECTANGULAR_FACTOR_FLOOR`.

## Resolution

The complete no-drop binary128 Householder factor preserves the immutable
R63F weak direction. Rounding the rectangular tangent coefficients to
binary64 also preserves it. The complete strict-binary64 Householder factor
does not: its directional reconstruction error has greater squared norm than
the nonzero signal itself.

The precision loss is therefore attributable to binary64 factor arithmetic,
not binary64 coefficient storage and not an exact rank deficiency:

```text
exact tangent operator
        |
        | round coefficients to binary64
        v
binary64 stored operator              weak signal survives
        |
        | strict binary64 Householder QR
        v
binary64 Q,R                          weak signal lost
```

All 102 columns remain present in both factors. No pivot search, rank
threshold, row drop or regularization occurs.

## Factor correspondence

The fixed inherited permutation is a complete permutation of all 102 source
rows. The transposed operator is `315 x 102`.

| Item | binary128 reference | binary64 candidate |
|---|---:|---:|
| Nonzero diagonal entries | `102` | `102` |
| Minimum absolute diagonal | `2.5279575796217025e-16` | `3.5883071119473307e-16` |
| Maximum absolute diagonal | `2.1725910694727231e-1` | `2.1725910694727235e-1` |
| `Q^TQ-I` maximum residual | `1.0592614694129797e-33` | `1.2144192546396035e-15` |
| `QR-CP` maximum residual | `7.7384179336307070e-35` | `1.4802663002724091e-16` |
| `R^TR-P^TG_TP` maximum residual | `1.8055593228630336e-35` | `3.1931116039251514e-17` |

The global binary64 residuals look conventionally good. They are nevertheless
too large relative to this problem's weakest retained direction. This is why a
global backward-error gate alone would have selected the wrong arithmetic.

Operator roots:

```text
permutation 335235cbaf9a93c805a2bfdbd17c593eb3ae44c9a20ea8a752782fa10f15826f
transpose   8b5db34c925973b26d0761ea2b3e050ca9e665c42a8879716fe3b3fa17588061
q128 factor fde9aef5ee0233c920ab76c855c612cc722422c67f44d18598211ae901082ec4
f64 factor  7cfe979b17c53cf81f4e69fe4ecb51223bb87e97d0b149dcc86df31723818d5d
```

## Weak-direction attribution

The strict gate was frozen before execution:

```text
lane survives iff ||u_lane-u||^2 < ||u||^2.
```

| Lane | Squared error | Reference squared norm | Result |
|---|---:|---:|---|
| binary64 coefficient storage, binary128 audit | `9.314251576435118e-32` | `3.9495781330250565e-30` | survives |
| rounded binary64 `alpha`, exact operator | `3.178882727842237e-32` | same | survives; attribution only |
| binary128 Householder factor | `3.0045801361991741e-66` | same | survives |
| strict binary64 Householder factor | `8.727321233512254e-30` | same | fails |

Coefficient storage consumes only a small fraction of the signal energy. The
binary64 factor error is about 2.2 times the signal energy. Wider factor
arithmetic improves it by more than 36 orders of magnitude in this witness,
but this result does not yet select binary128 runtime arithmetic.

## Architectural consequence

Preserve binary64 storage and stop ordinary binary64 direct-factor work for
this face. The next stage must compare only bounded arithmetic remedies before
an NNQP RHS:

- factor the stored binary64 coefficients with wider accumulation/factor
  arithmetic, then return a binary64 candidate for independent certification;
- or use reorthogonalized Golub--Kahan/LSMR arithmetic on the stored
  rectangular operator, avoiding a materialized fragile factor.

The next discriminator must state its precision boundary explicitly. A full
binary128 world/runtime solver is neither required nor authorized by R63G.

## Reproduction

```bash
cmake --build /tmp/nextengine-r20r4-build \
  --target nonlocal-formula-reclosure -j 8

/tmp/nextengine-r20r4-build/nonlocal-formula-reclosure \
  --nonlocal-al-generalization-v5-rectangular-factor-precision
```

Semantic hash:

```text
d1b09efa20c088b60796c83b33d5870b9db49b26fa8e0f44477ec22c3825e1eb
```

Two independent stdout hashes:

```text
9de7752302bce06d9cf1cae7de8a7f8b83e3d87000f80f1ece09e87eb6881908
9de7752302bce06d9cf1cae7de8a7f8b83e3d87000f80f1ece09e87eb6881908
```

Parent byte regressions:

```text
R63B stdout 320d6ea1842d49fef30b6821586b189dc923ef5bcfe9e3c22e86f4a38bc24216
R63C stdout 0ace54f4baa595a931db4a7da14464fca22b1cdfe68d4869212b50c39b56581b
R63D stdout 11b8ddd92e5b4163f6122227e482aa64ec56a9926c6993c1293d03e74015e619
R63E stdout db3e61ad2955ccbedb1e98aaf0258a7aab0e8c5696f26c87f0ef09e0ba3c0710
R63F stdout c0c07b866bf73eeece8d8a66b6111d89e8d2383be3837dd6af0b7e6393de7304
```

Implementation commit: `f9adb354`.

## Invalid/apparatus-only executions

An initial implementation interleaved transpose construction with permutation
and could read a not-yet-written column. It stopped at
`RECTANGULAR_FACTOR_APPARATUS_REJECTED` after one nonzero factor diagonal and
has no scientific credit. The repaired implementation first roots the complete
transpose and only then applies the immutable permutation.

One subsequent same-route run preceded the final literal factor/permutation
control expansion. It was treated as preliminary. Only the two hashes above,
with final controls root
`a9bbfa525e3ea323e093bb7889283319a6080c32090b53177c675986f7b33a0f`,
are credited.

## Authority ceiling

No NNQP RHS, triangular solve, LSQR/LSMR/SVD, rank action, row drop,
regularization, inverse, center, replacement, state update or timing was
executed. The result is local to one immutable face and grants no runtime, GPU
or production authority. R64 and R65 remain blocked.
