# NSR3-B4E2D7R20R63F representative-injectivity evidence

Status: `PASS / LOCAL_REPRESENTATIVE_NULLSPACE_REFUTED`.

## Resolution

The frozen 102-row tangent operator has represented row rank 102 under both
independent prime fields. The inherited R63E near-null row combination has a
strictly nonzero direct tangent image. Therefore the selected projector face
does not admit a nontrivial local dual perturbation which leaves the primal
tangent image unchanged.

The apparent rank 101 in the square Gram profile is numerical rank loss, not
an exact projection-equivalent representative:

```text
rectangular tangent rows A     exact represented rank 102
             |
             | form A A^T
             v
square tangent Gram            numerical rank 101
```

Deleting a row, choosing effective rank 101 or applying the polyhedral
extreme-representative construction would therefore change the frozen local
problem without its required premise.

## Direct rectangular certificate

| Item | Observed |
|---|---:|
| Tangent matrix | `102 x 315` |
| Nonzero represented values | `17,748` |
| Rank modulo `2^61-1` | `102` |
| Rank modulo `2^31-1` | `102` |
| Mapped scalars per prime | `32,130` |
| Row root | `6788422f80d918f39abfb7f9eb3f839e07f4598e408127fce67303314fcf5635` |
| Value root | `114a73ea34534ae25c0ea33a708944a434987374e18b703df542de8130aa73f1` |

Both modular eliminations are exact under the already frozen binary-to-field
mapping. They are independent represented-rank certificates; neither is a
floating-point pivot threshold.

## Square-root and witness correspondence

The independently accumulated direct image Gram agrees with the R63E tangent
Gram for all 10,404 entries:

| Quantity | Observed |
|---|---:|
| Products | `3,277,260` |
| Contained entries | `10,404 / 10,404` |
| Maximum residual | `6.0185310762101120e-36` |
| Maximum frozen bound | `3.5190591943843574e-30` |

The R63E normalized-pivot coefficients were also applied directly to all 315
tangent scalars:

| Quantity | Observed |
|---|---:|
| Combination terms | `32,130` |
| Contained scalars | `315 / 315` |
| Direct image norm squared | `3.949578133025056496992440349871504351e-30` |
| R63E image norm squared | `3.949578133025056497004133199170471796e-30` |
| Maximum component residual | `1.1386641207153209e-34` |
| Maximum frozen component bound | `5.1581527437634508e-30` |

The direct image is tiny but strictly nonzero. This is the predeclared
discriminator between severe conditioning and an exact local kernel.

## Architectural consequence

The projection-equivalent-representative path stops for this face. The next
research target is a no-drop rectangular formulation which works with `A` and
`A^T` directly and avoids explicitly solving through `A A^T`.

The first candidate comparison must remain pre-RHS and report-only:

- dense Householder QR/QRCP as a small exact-correspondence oracle;
- matrix-free Golub--Kahan/LSQR-style arithmetic as the scalable direction;
- explicit binary64 versus bounded wider-accumulation feasibility;
- all 102 rows retained, with no fitted rank threshold or regularization.

This result does not yet prove that a binary64 rectangular solve can certify
the required NNQP RHS. It only proves that exact representative freedom is not
the remedy and that normal-equation arithmetic is the next abstraction to
replace.

## Reproduction

```bash
cmake --build /tmp/nextengine-r20r4-build \
  --target nonlocal-formula-reclosure -j 8

/tmp/nextengine-r20r4-build/nonlocal-formula-reclosure \
  --nonlocal-al-generalization-v5-representative-injectivity
```

Semantic hash:

```text
048601d394e75ae42fc84554cc08b1f3012817824ebb994e990e1b25dd7e9acf
```

Two independent stdout hashes:

```text
c0c07b866bf73eeece8d8a66b6111d89e8d2383be3837dd6af0b7e6393de7304
c0c07b866bf73eeece8d8a66b6111d89e8d2383be3837dd6af0b7e6393de7304
```

Parent byte regressions:

```text
R63B stdout 320d6ea1842d49fef30b6821586b189dc923ef5bcfe9e3c22e86f4a38bc24216
R63C stdout 0ace54f4baa595a931db4a7da14464fca22b1cdfe68d4869212b50c39b56581b
R63D stdout 11b8ddd92e5b4163f6122227e482aa64ec56a9926c6993c1293d03e74015e619
R63E stdout db3e61ad2955ccbedb1e98aaf0258a7aab0e8c5696f26c87f0ef09e0ba3c0710
```

Implementation commit: `4e90627c`.

## Authority ceiling

No QR/SVD, RHS, rank action, row drop, center, replacement, state update or
timing was executed. The result is local to one immutable projector face and
does not grant runtime, GPU or production authority. R64 and R65 remain
blocked.
