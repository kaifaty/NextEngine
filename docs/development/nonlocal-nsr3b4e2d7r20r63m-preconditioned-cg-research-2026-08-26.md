# NSR3-B4E2D7R20R63M preconditioned-CG research

Status: `FROZEN FOR IMPLEMENTATION`.

Contract apparatus revision: `2` (the completion control uses eight distinct
positive diagonal eigenvalues so all eight frozen updates are exercised; the
scientific lanes, budget and routes are unchanged).

## Question

R63L proves that exported-factor standard refinement eventually certifies the
captured RHS, but it needs 21 sequential factor corrections. Can SPD
preconditioned conjugate gradients reach the same 102-sign certificate within
eight factor solves and fewer original-operator applications?

This stage isolates algorithmic acceleration. Factor/preconditioner,
residual, Krylov vectors and scalar reductions remain binary128; correction
precision is a later question.

## Why PCG is first

The original active Gram `H` is symmetric positive definite on the exact
represented rank-102 space. The retained/exported factor defines

```text
B = s P R^T R P^T,
M = B^-1,
```

with all 102 nonzero factor diagonals, so `B` is also SPD in exact arithmetic.
PCG exploits this structure and minimizes over a growing Krylov space, whereas
stationary refinement repeatedly applies the fixed polynomial
`I-B^-1 H`.

Primary sources:

- [Hestenes and Stiefel, Methods of Conjugate Gradients for Solving Linear Systems](https://nvlpubs.nist.gov/nistpubs/jres/049/jresv49n6p409_a1b.pdf)
- [Barrett et al., Templates for the Solution of Linear Systems](https://www.netlib.org/templates/templates.html)
- [Carson and Higham, A New Analysis of Iterative Refinement](https://eprints.maths.manchester.ac.uk/2604/)

Finite arithmetic can lose conjugacy or SPD observations, so every required
`r^T z` and `p^T H p` must be finite and strictly positive. No theorem alone
selects a route.

## Frozen PCG recurrence

Start each lane from its original rejected R63I candidate `x0`, not from an
R63K/R63L refined state:

```text
r0   = b - H x0
z0   = B^-1 r0
p0   = z0
rho0 = r0^T z0

qk      = H pk
alpha   = rhok / (pk^T qk)
x(k+1)  = xk + alpha pk
r(k+1)  = rk - alpha qk
z(k+1)  = B^-1 r(k+1)
beta    = rho(k+1) / rhok
p(k+1)  = z(k+1) + beta pk
```

Execute exactly eight iterate updates. The initial preconditioner application
plus updates after iterations 1--7 gives exactly eight factor solves; no unused
post-iteration-8 preconditioner solve is permitted.

Every vector/scalar reduction has fixed increasing-index binary128 Dot2 order.
The actual iterate is independently certified after every iteration. The
certificate's direct original residual is also compared with the PCG
recurrence residual to publish drift; it does not replace the algorithm
residual or restart PCG.

## Two frozen lanes

1. retained-wide `R` as the SPD preconditioner;
2. exported binary64 `R` promoted exactly, with otherwise identical
   binary128 PCG arithmetic.

The strict-binary64 lane remains outside R63M because R63K already rejects its
stationary correction. Precision recovery is researched only after an
accelerating algorithm is selected.

## Equal-work baseline

Publish algorithmic work separately from verification work:

```text
stationary retained-wide  17 factor solves / 17 original-H residual applies
stationary export/wide    21 factor solves / 21 original-H residual applies
PCG budget                 8 factor solves / 9 original-H applies
```

The ninth PCG `H` application is the initial residual; the eight others are
`H p`. Independent certificates are offline evidence and not counted as
runtime algorithm work. No wall timing is admitted.

## Routes

- apparatus, parent, SPD scalar, recurrence, drift, work, certificate or
  lifecycle failure: `PRECONDITIONED_CG_APPARATUS_REJECTED`;
- retained-wide PCG has no certified iterate through 8:
  `RETAINED_WIDE_PCG_REJECTED`;
- retained-wide passes but export/wide does not:
  `EXPORTED_FACTOR_WIDE_PCG_REJECTED`;
- both pass: `EXPORTED_FACTOR_WIDE_PCG_CANDIDATE`.

A deterministic nonpositive/nonfinite scalar is a recorded lane rejection,
not permission to modify `H`, regularize or switch algorithms inside R63M.

## Interpretation ceiling

A pass within eight iterations selects PCG as the lower-work correction
algorithm for the captured RHS. The next stages must prove dense-`H` versus
matrix-free `s C C^T` application correspondence, then bound a hardware-
available precision implementation. It does not yet authorize state
replacement or production.

A retained-wide rejection selects GMRES-IR or an explicit weak-direction
low-rank correction. An export-only rejection retains the wide factor state.

No adaptive restart, residual replacement, state update, following NNQP
transition, rank/row/regularization change, trajectory, timing, runtime/GPU or
production inference occurs. R64 and R65 remain blocked.
