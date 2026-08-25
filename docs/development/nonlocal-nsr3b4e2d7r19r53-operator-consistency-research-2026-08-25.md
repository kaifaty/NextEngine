# NSR3-B4E2D7R19R53 operator-consistency research

Date: `2026-08-25`

Status: `RESEARCH COMPLETE / ROWWISE BINARY128 DECOMPOSITION SELECTED`.

## Question

Why does R52's exact 494-row pair-once Gram model reach predicted maximum
`1.8707447849842052e-20`, while a fresh directed evaluation of the same
64-sweep witness still has raw `h=3.9830932122808307e-13` and maximum certified
upper `7.7539986039447153e-14`?

## Observed boundary

All raw and directed-positive R52 rows are inside the master. Capacity and
constraint generation are closed for this witness. Additional coordinate or CG
work would only solve the pair-once model more accurately. It cannot explain a
six-order cross-evaluation gap.

The two evaluations are algebraically equivalent but not operationally
identical:

- Gram construction visits each pair once and scatters its binary64 term;
- certification revisits each row and recomputes/folds directed terms in row
  order;
- the existing upper adds one conservative gamma bound using the global maximum
  degree.

At ordinary residual scale these differences are negligible. Around `1e-13`
they can own the sign.

## Numerical research

Error-free transformations can compute accurate sums/dot products and verified
error bounds using only working-precision operations; see
[Ogita--Rump--Oishi](https://doi.org/10.1137/030601818). Binned accumulators can
make reductions order-independent with much tighter error than conventional
summation; see the primary
[Demmel--Ahrens--Nguyen report](https://www2.eecs.berkeley.edu/Pubs/TechRpts/2016/EECS-2016-121.html)
and [Ahrens--Demmel--Nguyen](https://doi.org/10.1145/3389360). These are
production candidates, but choosing one before measuring the present error
decomposition would be premature.

## Selected discriminator

At the exact R52 64-sweep record, and without changing its witness:

1. reuse the captured directed binary64 image;
2. evaluate one fresh pair-once binary64 JVP;
3. traverse the stable directed rows once, recomputing each ordinary binary64
   term and accumulating it with compensated binary128. This isolates fold
   order while preserving the binary64 term values;
4. in the same traversal recompute the algebra from frozen binary64 workspace
   coefficients/positions in binary128 and compensate its row sum. This
   separates local arithmetic from reduction order;
5. report per representation: raw-positive count, maximum raw value and row;
6. report pair/direct, direct/term128 and term128/full128 maximum differences,
   sign-disagreement counts and worst rows;
7. decompose the current directed upper into raw value and gamma envelope,
   including maximum envelope, worst certified row and bound-only count.

Binary128 is an offline oracle only. It does not replace the current directed
certificate or authorize a new production arithmetic path.

## Frozen classification

- Any pair-versus-full128 sign disagreement selects operator alignment required.
- Otherwise a positive, binary128-resolved raw row selects high-precision raw
  residual confirmed.
- Otherwise current certified positives select enclosure tightening required.
- Only zero current certified positives can select the existing compatibility
  route.

This precedence deliberately treats cross-operator disagreement before raw
physics or bound tightness. No tolerance is fitted; signs are resolved only
outside a conservative binary128 forward-error envelope.

## Alternatives not selected

- **More Hildreth/CG:** rejected until both solver and certificate evaluate the
  same numerical operator.
- **Lower gamma:** rejected; current bound remains authority until replaced by a
  proof.
- **Long double only:** useful but architecture-dependent; GCC binary128 already
  exists in this reference and provides a wider oracle.
- **Immediate ReproBLAS/ExBLAS integration:** postponed until the decomposition
  identifies whether local products, fold order or the envelope dominates.
- **Another nonlinear outer:** rejected; it would move the witness and hide the
  numerical cause.

## Recommendation

Freeze and implement one rollback-only decomposition with one fresh pair-once
JVP and one binary128 directed traversal. Preserve the 64-sweep witness, master,
topology and existing certificate bit-for-bit. Use the result to choose exactly
one of operator unification, high-precision residual correction or proved
enclosure tightening.
