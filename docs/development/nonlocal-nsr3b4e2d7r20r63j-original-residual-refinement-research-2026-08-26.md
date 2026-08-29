# NSR3-B4E2D7R20R63J original-residual refinement research

Status: `IMPLEMENTED / EXPORTED_FACTOR_REFINED_RHS_CANDIDATE`.

## Question

R63I shows that solving the Gram of the binary64-stored tangent operator gives
an unacceptable solution of the original captured system even when the factor
and substitutions remain binary128. Can one bounded correction constructed
from the original-system residual recover all 102 certified signs without
changing the factor representation, rank, rows or state?

This is a mathematical recoverability experiment. It is not a scalable runtime
algorithm: it is allowed to consume the already verified dense R60 left
inverse and binary128 Dot2 arithmetic only inside the shadow certificate.

## Why a small residual was insufficient

R63I retained-wide consumption has original residual infinity bound only
`6.86e-2`, but the verified inverse maps it to an order-`4e15` correction. The
system is extremely ill-conditioned, so backward accuracy does not imply
forward or sign accuracy.

This is consistent with standard iterative-refinement analysis: residual,
factor/solve, working and update precisions are separate, and convergence
requires the correction equation to reduce the error sufficiently. For very
ill-conditioned systems GMRES-based refinement can extend the useful range of
a low-precision factor, but that adds a new iterative method and stopping
policy. R63J first tests the stronger application-specific fact already in
hand: the exact R60 left defect is contractive.

Primary sources:

- [Carson and Higham, A New Analysis of Iterative Refinement](https://eprints.maths.manchester.ac.uk/2604/)
- [Carson and Higham, Iterative Refinement in Three Precisions](https://eprints.maths.manchester.ac.uk/2629/1/cahi18.pdf)
- [Higham, Error Analysis for Standard and GMRES-Based Iterative Refinement](https://eprints.maths.manchester.ac.uk/2735/1/paper.pdf)
- [Amestoy et al., Five-Precision GMRES-Based Iterative Refinement](https://eprints.maths.manchester.ac.uk/2852/1/paper.pdf)

## Frozen centered correction

For original `H lambda*=b`, initial R63I candidate `lambda0`, verified
approximate inverse `X` and

```text
C = I - X H,
rho >= ||C||inf,
rho = 0.0248854338484... < 1,
r = b - H lambda0,
z = X r,
```

the exact error satisfies `e=z+C e`. Starting with `d0=0`, construct once:

```text
d(k+1) = z + C d(k)
lambda(k) = lambda0 + d(k).
```

Thus `d(k)` is the first `k` terms of the Neumann correction. It uses one
original residual, not `k` factor solves and not `k` accepted state updates.
All matrix-vector reductions and center sums are fixed-order binary128 Dot2.

The nominal contraction estimate from the largest R63I radius gives:

```text
depth 8   <= about 609
depth 9   <= about 15.2
depth 16  <= about 8.96e-11
```

against the immutable R60 minimum sign separation `32.4234`. This predicts a
boundary between depths 8 and 9 in ideal arithmetic, but it is not an
acceptance assumption.

## Frozen checkpoint ladder

Evaluate exactly these depths and no adaptive retry:

```text
4, 8, 9, 10, 12, 16
```

For every depth independently recompute `b-H lambda(k)` and `Xr` using the
R63I outward Dot2 certificate. A checkpoint passes only when its certified
error intervals resolve all 102 components and match the exact R60
`24 positive / 78 negative` sign vector component by component.

The independent residual certificate is intentional. It accepts the actual
computed center, including all recurrence and cancellation error; no analytic
`rho^k` estimate can select a route.

## Three frozen lanes

Replay the three exact R63I initial candidates:

1. stored operator, retained-wide `R`, wide triangular consumption;
2. stored operator, exported `R`, wide triangular consumption;
3. stored operator, exported `R`, strict-binary64 consumption promoted once
   for the common binary128 correction.

Build the common `C` once. Each lane receives its own initial residual, `z`,
recurrence, checkpoint centers and certificates. Record the first passing
frozen depth or `none`; never reuse another lane's correction.

## Routes

- apparatus, parent, baseline, contraction, generation, containment, work or
  lifecycle failure: `ORIGINAL_RESIDUAL_REFINEMENT_APPARATUS_REJECTED`;
- retained-wide lane has no passing checkpoint through depth 16:
  `WIDE_ORIGINAL_RESIDUAL_REFINEMENT_REJECTED`;
- retained-wide passes but export/wide does not:
  `EXPORTED_FACTOR_WIDE_REFINEMENT_REJECTED`;
- both wide-consumption lanes pass but the binary64-consumption lane does not:
  `EXPORTED_FACTOR_BINARY64_REFINEMENT_REJECTED`;
- all three lanes pass by a frozen checkpoint:
  `EXPORTED_FACTOR_REFINED_RHS_CANDIDATE`.

No expected first-passing depth is placed in the classifier.

## Interpretation ceiling

A pass proves that high-precision original-residual correction can recover the
captured solution semantics from the stored-operator factor. It does not make
the dense `X` construction or binary128 correction production-suitable. The
next research question would be whether a scalable rectangular preconditioned
correction (for example GMRES-IR) reproduces the same certificate without a
dense inverse.

A depth-16 rejection sends research to an original-operator factor or
preconditioned Krylov correction. It does not authorize weaker signs, rank 101,
row deletion or regularization.

R63J performs no replacement, following NNQP transition, trajectory, timing,
runtime/GPU or production inference. R64 and R65 remain blocked.

## Result

The frozen apparatus selects `EXPORTED_FACTOR_REFINED_RHS_CANDIDATE` at
semantic `4a4d1798...8f96`; stdout repeats at `b4a2908e...191d`. All three
lanes independently recover the exact R60 `24+/78-` sign pattern at the first
frozen checkpoint, depth 4. Their certified errors are
`3.11e-3 / 1.99e-3 / 4.51e-3`. See the
[evidence record](nonlocal-nsr3b4e2d7r20r63j-original-residual-refinement-evidence-2026-08-26.md).

This proves mathematical recoverability with the dense verified `X` oracle.
The next research stage must replace `X` as correction provider with the
retained/exported rectangular factor while keeping `X` only for independent
certification.
