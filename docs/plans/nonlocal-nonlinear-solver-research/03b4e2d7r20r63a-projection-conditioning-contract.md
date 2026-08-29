# NSR3-B4E2D7R20R63A research contract -- revision 1

| Field | Value |
|---|---|
| Research ID | `NSR3-B4E2D7R20R63A` |
| Parent | R63 semantic `70f298d4...11496`, immutable R60 dimension-102 tuple |
| Engineering consumer | Select the next representation/scaling discriminator without constructing another inverse |
| Claim class | Exact finite projection-error decomposition |
| Claim status target | `SUPPORTED_BOUNDED` classification or exact apparatus boundary |
| Budget | One capture-only replay; exactly eight 102x102 exact residual products plus exact vector/matrix norm folds; zero solves, Dot2 candidates, centers or timing |

## Exact claim

For both right and left multiplication order, exact dyadic arithmetic closes
both decompositions of the combined projected defect from the original,
matrix-only and inverse-only defects. The original tuple remains two-sided
contractive and the combined projected tuple remains two-sided
noncontractive. Predeclared contribution norms then classify inverse,
matrix or mixed projection dominance.

## Exact negation

The first capture/root, exact-conversion, original-contraction,
combined-noncontraction, componentwise decomposition or work/lifecycle gate
fails. No dominance claim is then admitted.

## Frozen variants

Evaluate these exact products once in each order:

| ID | Left operand | Right operand |
|---|---|---|
| `qq` | original binary128 representation | original binary128 representation |
| `dq` | projected matrix / original inverse | as named by right or left order |
| `qd` | original matrix / projected inverse | as named by right or left order |
| `dd` | projected binary64 representations | projected binary64 representations |

For right order these mean `A*X`; for left order `X*A`. Require roots of
`qq` to reproduce R60 exact residuals and roots of `dd` to reproduce R63 exact
residuals.

## Contribution definitions

For every residual entry, exact subtraction constructs:

```text
right matrix first:  Rdq-Rqq
right inverse after: Rdd-Rdq
right inverse first: Rqd-Rqq
right matrix after:  Rdd-Rqd

left matrix first:   Rqd_left-Rqq_left
left inverse after:  Rdd_left-Rqd_left
left inverse first:  Rdq_left-Rqq_left
left matrix after:   Rdd_left-Rdq_left
```

The naming follows which physical source (`A` or `X`) was projected, not the
operand position. Require both exact componentwise sums to equal `Rdd`.

Compute exact infinity norms for all eight defect matrices and all eight
contribution matrices. Use the larger of the two order-dependent contribution
norms for each source. Classify with exact comparisons:

```text
inverse-dominant iff inverse >= 16*matrix on right and left
matrix-dominant  iff matrix  >= 16*inverse on right and left
mixed otherwise
```

Zero/zero is mixed, not dominant.

## Scale and structure facts

For `Aq,Ad,Xq,Xd` and the two projection differences report:

- exact/outward infinity norm;
- maximum absolute and minimum nonzero absolute entry;
- nonzero range ratio;
- maximum row norm, minimum nonzero row norm and ratio;
- diagonal minimum/maximum absolute value and positivity count;
- exact symmetry flag and infinity norm of `M-M^T`;
- `||A||inf*||X||inf` proxy and `u*proxy` for `qq` and `dd`.

All names must say `proxy`; no singular value or true condition number is
claimed.

## Controls and evidence

1. Literal 2x2 diagonal matrix/inverse: exact decomposition with matrix-only
   perturbation classification.
2. Literal 2x2 inverse-only perturbation classification.
3. Literal mixed perturbation classification.
4. Deliberately alter one contribution entry; exact closure must reject.
5. Bind all tuple, projected, eight residual, contribution and control roots.
6. Execute twice byte-identically and regress R63/R60.

## Resolution firewall

- Positive routes: `INVERSE_PROJECTION_DOMINATES`,
  `MATRIX_PROJECTION_DOMINATES` or `MIXED_PROJECTION_CONTRIBUTION`.
- Negative routes name the first parent, exact, decomposition, structure, work
  or lifecycle boundary.
- Does not count: a floating residual, norm-only closure, choosing 16 after
  observation, computing a scaling, factorizing a matrix, refining an inverse
  or running R64.
- Ceiling: one exact decomposition and representation diagnosis only.

## Stop and reconsider

- Stop at the first exact identity failure; do not infer dominance from norms.
- Preserve the measured dynamic range even when one contribution dominates.
- Freeze any equilibration/direct-inverse experiment separately after R63A.

