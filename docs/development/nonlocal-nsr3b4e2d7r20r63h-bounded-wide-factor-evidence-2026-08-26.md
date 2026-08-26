# NSR3-B4E2D7R20R63H bounded-wide-factor evidence

Status: `PASS / WIDE_BUILD_BINARY64_EXPORT_CANDIDATE`.

## Resolution

The tangent operator can remain stored in binary64. Promoting only the private
active block during Householder construction preserves the stored and original
weak directions. The completed `Q/R` factor can then be rounded once back to
binary64 and still preserve both signals.

```text
binary64 operator storage
        |
        | promote active 315 x 102 block
        v
binary128 Householder build
        |
        | one-time factor export
        v
binary64 Q/R                    weak direction still preserved
```

This rejects the need to retain a binary128 factor state solely on the basis of
the R63G witness. It does not yet prove that binary64 triangular consumption of
the exported `R` is accurate enough for the real RHS.

## Stored-operator factor

The promoted factor is constructed from exactly the already rounded binary64
coefficients, not from the original binary128 oracle.

| Item | Observed |
|---|---:|
| Stored input root | `7d51d221849ceb463711c6c137eaba63a1398848207c1e513cead179cba98dda` |
| Permuted input root | `ca4c10b374377201d55e4641334ea980646b1eb155af11b3da8262c415b0e1c7` |
| Stored Gram root | `37335288e99332b132ab8786a62d62737662fbe4bf60f54685e2b49859a06c10` |
| Nonzero factor diagonals | `102` |
| Minimum absolute diagonal | `2.4941674948483164e-16` |
| Maximum absolute diagonal | `2.1725910694727231e-1` |
| Wide factor root | `bf0a9e1b9e2fd4525d267ec63d63640675479ab55c5b6cd3bb3b0c4ea4779b14` |

The binary128 audit closes:

| Identity | Maximum residual |
|---|---:|
| `Q^TQ-I` | `9.6296497219361793e-34` |
| `QR-C64P` | `9.1594644365247390e-35` |
| `R^TR-P^T(C64^TC64)P` | `2.4074124304840448e-35` |

## Binary64 export

The completed factor is cast once; there is no second factorization.

```text
Q export root 4d5d79d8491ad608aa7e1de41637264161936c12caeba0142be143e175dbc763
R export root a7a85789364f5af91aaf71e1531759713c95ec085e48b7ca614daa524c3a4f85
```

Its global binary128 audit gives:

| Identity | Maximum residual |
|---|---:|
| `Q^TQ-I` | `6.1051890617539245e-17` |
| `QR-C64P` | `8.5543835285284728e-18` |
| `R^TR-P^T(C64^TC64)P` | `4.5647770039003339e-18` |

These values are observations only; the strict weak-direction gates decide the
route.

## Weak-direction gates

| Quantity | Squared norm/error |
|---|---:|
| Original R63F signal | `3.9495781330250565e-30` |
| Stored-operator signal | `3.8945511341735747e-30` |
| Wide factor error versus original | `9.314251576435118e-32` |
| Wide factor error versus stored | `3.3456465068437846e-66` |
| Export factor error versus original | `1.4559446713113880e-31` |
| Export factor error versus stored | `6.3627239351863151e-32` |

All four errors lie strictly inside their corresponding nonzero signal balls.
The exported factor's original-image error is about 27 times smaller than the
original signal energy. Thus the damaging R63G error came from performing the
factorization in binary64, not from representing a well-built factor in
binary64.

## Architectural consequence

The retained candidate boundary is:

```text
binary64 operator -> bounded wide QR build -> binary64 exported R
```

The next stage may execute exactly one immutable NNQP RHS through
`R^T R`, with three separately audited consumption lanes:

1. retained-wide `R` and wide triangular arithmetic;
2. exported-binary64 `R` promoted for wide triangular arithmetic;
3. exported-binary64 `R` with strict-binary64 triangular arithmetic.

All candidates must be checked against the original captured Gram and RHS,
including residual and sign semantics. No lane may update solver state.

## Reproduction

```bash
cmake --build /tmp/nextengine-r20r4-build \
  --target nonlocal-formula-reclosure -j 8

/tmp/nextengine-r20r4-build/nonlocal-formula-reclosure \
  --nonlocal-al-generalization-v5-bounded-wide-factor
```

Semantic hash:

```text
34452faf4640780badf441f0f3d22e12d0e5797194e568cda22c1c0a79156708
```

Two independent stdout hashes:

```text
2d56e2acba646946f53325553a586569815992b0ee7da9ec716c611d00f9176a
2d56e2acba646946f53325553a586569815992b0ee7da9ec716c611d00f9176a
```

Parent byte regressions:

```text
R63B stdout 320d6ea1842d49fef30b6821586b189dc923ef5bcfe9e3c22e86f4a38bc24216
R63C stdout 0ace54f4baa595a931db4a7da14464fca22b1cdfe68d4869212b50c39b56581b
R63D stdout 11b8ddd92e5b4163f6122227e482aa64ec56a9926c6993c1293d03e74015e619
R63E stdout db3e61ad2955ccbedb1e98aaf0258a7aab0e8c5696f26c87f0ef09e0ba3c0710
R63F stdout c0c07b866bf73eeece8d8a66b6111d89e8d2383be3837dd6af0b7e6393de7304
R63G stdout 9de7752302bce06d9cf1cae7de8a7f8b83e3d87000f80f1ece09e87eb6881908
```

Implementation commit: `da07c05b`.

## Authority ceiling

No RHS, triangular solve, iterative refinement, LSQR/LSMR/SVD, rank action,
row drop, regularization, center, replacement, state update or timing was
executed. The result selects a research factor boundary only and grants no
runtime, GPU or production authority. R64 and R65 remain blocked.
