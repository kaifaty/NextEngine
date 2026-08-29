# NSR3-B4E2D7R20R63N matrix-free correspondence evidence

Status: `PASS / DIRECT_RECTANGULAR_PRODUCT_NOT_CONTAINED`.

Claim status: `REFUTED` for the frozen raw product-bound correspondence claim.
Evidence classes: `NUMERICAL`, `EXACT_CERTIFICATE`, `CORRESPONDENCE`.

## Reproducible result

Implementation commit: `793be511`.

Command:

```text
/tmp/nextengine-r20r4-build/nonlocal-formula-reclosure \
  --nonlocal-al-generalization-v5-matrix-free-correspondence
```

Two independent executions are byte-identical:

```text
stdout sha256   db8cf95704401b3b6a1a498b5fad54ea6f1aea2b9d46eada9f9a46e2042fdd70
semantic sha256 b8d0c7081646d94aaeb441982ac22afee4477f5df426e6311d4ce03079f02772
route           DIRECT_RECTANGULAR_PRODUCT_NOT_CONTAINED
controls root   ac6c11bdd53f380b8ee8cfb0d89ba6e6e54520797f3308ffdfd4546f6ba38615
```

Platform, controls, exact R63M reconstruction, tangent identity, work and
lifecycle gates pass. The route is a scientific negative, not an apparatus
failure.

## Canonical basis succeeds

The immutable physical tangent operator is reconstructed exactly:

```text
dimensions       102 x 315
nonzero values   17,748
value root       114a73ea34534ae25c0ea33a708944a434987374e18b703df542de8130aa73f1
```

All 102 canonical basis products pass. Every one of 10,404 dense/direct
component intervals overlaps:

```text
contained components       10,404 / 10,404
maximum center residual    3.009265538105056e-36
maximum combined bound     1.094244902126629e-35
minimum slack              9.103626697467437e-49
canonical root             3f13e864147e022d47a12c88a8e0911c78b5f8f11fd5ba15123e07470802c28e
```

This supports the finite-profile basis identity between captured dense `H`
and `sigma T T^T`. It does not by itself certify arbitrary linear
combinations of those columns.

## First arbitrary-vector boundary

Both matrix-free lanes stop before their initial residual and before any
factor solve. The direct product of each original rejected R63I start does not
overlap the corresponding dense product under the frozen local arithmetic
bounds:

```text
retained-wide  initial minimum slack  -3.336645036110384e-19
export/wide    initial minimum slack  -3.336046945995463e-19
```

The failure roots are:

```text
retained-wide  623feaec490e9f3c424eb5665fbe77f0b3244815db5bdd15adc1f4d70f27e6bb
export/wide    1acc1fcee6e90faba50ca8192fd30f2985ee469fc5e7845ba6c941f82e86c4f1
```

Therefore R63N does not execute PCG and does not refute matrix-free PCG
convergence. It refutes the narrower premise that independent arithmetic
bounds for the two product evaluations are sufficient after canonical basis
agreement.

## Causal interpretation

The frozen direct-product bound transports Dot2 and scale rounding internal to
`T(T^T p)`. It does not transport the already observed finite representation
difference between each direct basis column and the corresponding stored dense
`H` column through the coefficients of an arbitrary vector.

The next falsifiable hypothesis is:

```text
E_ij >= |H_ij - (sigma T T^T)_ij|

model_bound_i(p) = sum_j E_ij |p_j|
```

with every operation rounded upward. Adding this derived term is not a looser
empirical tolerance: it is the missing linear image of the exhaustive R63N
canonical certificate. R63N does not yet prove that the transported bound will
close the two R63I starts or later Krylov vectors; that requires a separately
frozen R63O discriminator.

## Work and ceiling

The failure occurs after two initial rectangular products across the two
lanes. No preconditioner solve, PCG update, state update, sparse zero elision,
CSR construction or timing occurs. Canonical audit work remains valid evidence.

Do not retry R63N with a fitted decimal tolerance, remove the initial vectors,
use only random/small vectors, or call this a PCG failure. The result gives no
runtime, GPU or production authority.

## Regression evidence

R63B--R63M public stdout remains byte-identical. The immediate parent is:

```text
R63M 9eebbd19938c95342bb4cb919d5648eaf05ea1ef1971bb0e4369f2225e8d09e7
```

All earlier R63B--R63L hashes equal the values recorded by R63M evidence. Build
passed for target `nonlocal-formula-reclosure`. No CPU/wall performance
comparison was run on the shared host.
