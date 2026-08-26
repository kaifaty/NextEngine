# NSR3-B4E2D7R20R63O canonical-defect transport evidence

Status: `PASS / RETAINED_WIDE_TRANSPORTED_MATRIX_FREE_PCG_REJECTED`.

Claim status: the defect-transport subclaim is `SUPPORTED_BOUNDED`; the frozen
matrix-free PCG preservation claim is `REFUTED`. Evidence classes:
`NUMERICAL`, `EXACT_CERTIFICATE`, `CORRESPONDENCE`.

## Reproducible result

Implementation commit: `00ddea89`.

Command:

```text
/tmp/nextengine-r20r4-build/nonlocal-formula-reclosure \
  --nonlocal-al-generalization-v5-canonical-defect-transport
```

Two independent executions are byte-identical:

```text
stdout sha256   97011922a1efc9b502627c2dc6cb20c1d90a7ebb855f3e38a743686bdf1c131f
semantic sha256 4b2434fa485f6d3f5362169587da44ffc744d0b448a9384ebe769f3f3dc2f270
route           RETAINED_WIDE_TRANSPORTED_MATRIX_FREE_PCG_REJECTED
controls root   e6fa87f3298215ecd2515197918d4f1a082afeb0fd53c29b964a3760feb9c863
```

Platform, positive/negative transport controls, exact R63N reconstruction,
defect construction, all product transports, work and lifecycle gates pass.
The route is a scientific PCG rejection for the direct represented operator,
not an apparatus or bound failure.

## Defect transport closes R63N exactly

The immutable mechanically derived `102 x 102` enclosure has:

```text
entries           10,404
maximum entry     1.360997889306316e-35
maximum row sum   1.751222360313530e-34
root              e531f06788e134c1fa65e39dd8e4f0be8e6350b23b781c990d6e7d72ac666f94
inflation factor  1
```

R63N's raw initial slacks reproduce exactly near `-3.34e-19`. The fixed
`sum_j E_ij |p_j|` transport contains both starts and every later Krylov
product:

```text
lane             minimum transported slack   maximum model bound
retained-wide    3.639738065792041e-64         6.536106735682792e-18
export/wide      3.970400059071765e-62         6.534935145421805e-18
```

All 187,272 predeclared transport terms are consumed. The small positive
slacks are accepted because `E` is not fitted: every entry follows the frozen
triangle formula, and the control that zeroes one required entry fails.

## Matrix-free PCG does not preserve the dense solution

Both direct-operator lanes remain finite, keep positive PCG scalars, complete
all eight updates and contain all nine products. Neither ever certifies the
original dense-system signs:

```text
lane             best original-H error       signs at iterations 2..8
retained-wide    3.746371335672538e13         12+/24-/66 unresolved
export/wide      3.746371335672538e13         12+/24-/66 unresolved
```

The first update remains order `1e15`; the second drops to the order-`1e13`
plateau, and subsequent updates no longer improve the original-system error.
All direct iterates are independently checked with original dense `H,b,X`.

Lane roots:

```text
retained-wide  7cf542379a194e3cf9830b62a95aaed8a74313ab92f29ee6cc76b76fc84df6fc
export/wide    562382d435a48b761016ec7675ff90cb7b132c3a95539f81fa4e89add7be0b78
```

This establishes that raw product containment was not the only obstacle.
Although stored dense `H` and direct `K=sigma T T^T` differ by only
order-`1e-35` per entry, the near-null direction amplifies that perturbation
enough to change the inverse solution materially.

## Meaning and next discriminator

Do not add a larger transport bound: the executed direct centers themselves
converge to a solution that fails original `H` semantics. Do not add a
rank-one patch before deciding which finite operator is the intended numerical
model.

The next research must arbitrate operator semantics against a higher-precision
or exact-dyadic evaluation of the frozen source rows and projector derivative:

```text
H* = A DPi A^T.
```

It should compare stored dense `H`, stored-tangent Gram `K`, their quadratic
forms on the inherited weak witness and their RHS solutions against `H*`.
Only then can the roadmap choose dense-block reproduction, a certified
low-rank correction, or direct `K` as the canonical discretization.

## Work and ceiling

Across two lanes the candidate path performs 16 factor solves, 18 rectangular
products, 7,506 direct Dot2 reductions and 1,156,680 direct terms. Transport
and dense comparison are offline verification. These counts are not a speed
claim; the dense rectangular reference remains more arithmetic than dense `H`
at dimension 102.

R63N stdout remains byte-identical at
`db8cf95704401b3b6a1a498b5fad54ea6f1aea2b9d46eada9f9a46e2042fdd70`;
R63B--R63M regressions remain those recorded by R63N. Build passed. No
CPU/wall performance comparison, state update, runtime, GPU or production
inference occurred.
