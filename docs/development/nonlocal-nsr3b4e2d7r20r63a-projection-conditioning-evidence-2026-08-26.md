# NSR3-B4E2D7R20R63A projection/conditioning evidence

Status: `SUPPORTED_BOUNDED / MIXED_PROJECTION_CONTRIBUTION`.

## Outcome

R63A closes both exact componentwise decompositions in both multiplication
orders. Neither matrix projection nor inverse projection dominates by the
predeclared factor 16. Both independently amplify to defects of order `1e15`,
so the valid classification is mixed.

```text
status: PASS
route: MIXED_PROJECTION_CONTRIBUTION
semantic SHA-256:
  c08b7abead8ce5623f78761be7af3f9546ed389256e691aefac95c355317639b
stdout byte SHA-256, both executions:
  b0dba1bcc42b2d70cc2f443da30db72f967ef6bfa8d592eb726468e997f5b4c9
```

The two executions are byte-identical. They run no new factorization, inverse
column, `Dot2Err` candidate, center iteration, state update or timing.

## Exact defect decomposition

| Order | Variant | Exact outward infinity norm |
|---|---|---:|
| right | original `Aq*Xq` | `9.80205440727151e-4` |
| right | projected `A`, original `X` | `1.57671012442314e15` |
| right | original `A`, projected `X` | `1.04064201665328e15` |
| right | projected `A*X` | `1.82144558866791e15` |
| left | original `Xq*Aq` | `2.48854338484085e-2` |
| left | projected `A`, original `X` | `1.62634904423698e15` |
| left | original `A`, projected `X` | `2.77155009716885e15` |
| left | projected `X*A` | `3.15136320374770e15` |

The maximum direct contribution norms are:

```text
right matrix  1.57671012442314e15
right inverse 1.04064201665328e15
left  matrix  1.62634904423698e15
left  inverse 2.77155009716885e15
```

Both exact identities
`base + matrix-first + inverse-after = combined` and
`base + inverse-first + matrix-after = combined` pass componentwise. Literal
matrix-dominant, inverse-dominant, mixed and mutated-closure controls also pass.

## Scale and conditioning facts

The matrix itself is symmetric with all 102 diagonal entries positive. Its
infinity norm is about `0.2536`; row-norm range is only `3.65x` and diagonal
range only `2.50x`. This is not the signature of a matrix whose main problem is
ordinary row or diagonal scale imbalance.

The certified binary128 inverse has infinity norm about `1.0409e33`. Its row
range is about `4.60e15`; the inverse projection delta alone has infinity norm
about `6.51e16`. A matrix projection delta of only about `1.00e-17` is enough
to create an order-`1e15` residual after multiplication by that inverse.

The inverse-norm proxy is:

```text
||Aq||inf * ||Xq||inf     ~= 2.63947091370256e32
u64 * proxy               ~= 2.93040138122098e16
```

Because the exact left defect satisfies `rho<1`, the relation

```text
||A^-1||inf >= ||X||inf / (1 + rho_left)
```

shows that the huge proxy reflects genuine conditioning rather than only a bad
inverse. Numerically the resulting condition lower estimate is about
`2.58e32`; a subsequent contract must compute any claimed directed bound
inside the exact harness before using it as a gate.

## Architectural interpretation

The captured principal matrix is formed as a symmetrized Gram matrix of active
projector rows. Extreme conditioning is therefore consistent with nearly
linearly dependent active constraints. Classical equilibration can reduce
scale imbalance but cannot be assumed to remove that dependence; the observed
row/diagonal ranges make blind scaling a weak next choice.

The next research should inspect the Gram/rank structure and certified
condition lower bound before selecting among:

- a rank-revealing active-set/pseudoinverse formulation over the underlying
  projected rows;
- a physically justified regularization/redundancy policy with a new
  mathematical identity;
- a rare software extended-precision certificate path;
- stopping binary64 certification for this degeneracy class.

No one of these is selected by R63A alone. R64 remains blocked.

## Parent regression

- R63 remains exact negative at stdout `b54cd01e...9d472`, semantic
  `70f298d4...11496`, route `BINARY64_RIGHT_NONCONTRACTIVE`.
- R60 remains exact positive at stdout `4b87849b...f83ef`, semantic
  `c8f11806...1ae2a`.

