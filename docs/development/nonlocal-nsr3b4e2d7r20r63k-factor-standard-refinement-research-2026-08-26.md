# NSR3-B4E2D7R20R63K factor standard-refinement research

Status: `IMPLEMENTED / WIDE_FACTOR_STANDARD_REFINEMENT_REJECTED`.

## Question

R63J recovers every R63I solution using an original residual and the dense
verified inverse `X` as correction provider. Can the retained/exported
rectangular factor itself provide the correction in standard mixed-precision
iterative refinement, leaving `X` only as an independent verifier?

This is the first correction algorithm in this lineage that does not consume a
dense inverse to generate its update.

## Standard refinement transaction

Let `B` be the Gram represented by a frozen R63H factor and `H` the original
captured Gram. Starting from the matching R63I solution `x0=B^-1 b`, repeat:

```text
r(k)       = b - H x(k)       // original residual
delta(k)   = B^-1 r(k)        // two triangular factor solves
x(k+1)     = x(k) + delta(k)
```

The exact error propagation is

```text
e(k+1) = (I - B^-1 H) e(k).
```

R63H weak-image fidelity does not prove this defect contractive, so R63K does
not infer convergence from factor quality. Every checkpoint is certified
against original `H,b` with the R60 verifier.

Standard and GMRES-based iterative-refinement theory separates factor, solve,
residual and working precisions. Standard refinement is the least complex
candidate; GMRES-IR is reserved for a later stage if the fixed factor solve
does not contract sufficiently.

Primary sources:

- [Carson and Higham, A New Analysis of Iterative Refinement](https://eprints.maths.manchester.ac.uk/2604/)
- [Carson and Higham, Iterative Refinement in Three Precisions](https://eprints.maths.manchester.ac.uk/2629/1/cahi18.pdf)
- [Higham, Error Analysis for Standard and GMRES-Based Iterative Refinement](https://eprints.maths.manchester.ac.uk/2735/1/paper.pdf)
- [Amestoy et al., Five-Precision GMRES-Based Iterative Refinement](https://eprints.maths.manchester.ac.uk/2852/1/paper.pdf)

## Three frozen lanes

1. `wide/wide`: retained binary128 `R`; original residual, triangular solves
   and update in binary128.
2. `export/wide`: exported binary64 `R` promoted exactly; original residual,
   triangular solves and update in binary128.
3. `export/binary64`: original residual computed with binary128 Dot2 then
   rounded once to binary64; exported `R`, substitutions and solution update
   use strict binary64. The iterate is promoted only for certification.

Each lane starts from its own exact R63I candidate and reuses no correction or
iterate from another lane.

## Frozen iteration ladder

Execute all 16 iterations and independently certify exactly these iterates:

```text
1, 2, 4, 8, 16
```

There is no adaptive retry or early exit. At every iteration bind original
residual, factor-solve intermediates, correction, update and work roots. At
every checkpoint use `al_r20_r63i_sign_certificate` against original `H,b,X`
and the R60 componentwise sign vector.

A lane passes only if a frozen checkpoint certifies all `24+/78-` signs.
Record its first passing checkpoint and continue to depth 16. Also report
whether certificate errors decrease monotonically, but monotonicity does not
replace sign certification.

## Routes

- apparatus, parent, baseline, arithmetic ledger, certificate or lifecycle
  failure: `FACTOR_STANDARD_REFINEMENT_APPARATUS_REJECTED`;
- retained-wide lane has no certified checkpoint through iteration 16:
  `WIDE_FACTOR_STANDARD_REFINEMENT_REJECTED`;
- retained-wide passes but export/wide does not:
  `EXPORTED_FACTOR_WIDE_STANDARD_REFINEMENT_REJECTED`;
- both wide lanes pass but strict binary64 does not:
  `EXPORTED_FACTOR_BINARY64_STANDARD_REFINEMENT_REJECTED`;
- all three pass: `EXPORTED_FACTOR_STANDARD_REFINEMENT_CANDIDATE`.

Nonfinite correction/update or zero diagonal is a closed lane rejection when
its exact iteration/failure location is recorded; it is not permission to
discard the whole experiment as an apparatus failure.

## Interpretation ceiling

A pass would select the exported factor plus original-residual standard
refinement as the first scalable correction candidate. `X` would remain only
in offline verification; later work must replace the captured dense `H`
residual with the equivalent matrix-free rectangular operator application and
then test a private following NNQP decision.

A retained-wide rejection selects GMRES-IR, symmetrically preconditioned
CG/MINRES or an explicit low-rank weak-direction correction for research. It
does not authorize dense `X`, weaker signs, rank 101, row deletion or
regularization.

R63K performs no replacement, following transition, trajectory, timing,
runtime/GPU or production inference. R64 and R65 remain blocked.

## Result

The frozen apparatus selects `WIDE_FACTOR_STANDARD_REFINEMENT_REJECTED` at
semantic `61c739a2...4ade`; stdout repeats at `7b13fa40...73c8`. Retained-wide
error falls monotonically to `144.035` at iteration 16 but still leaves four
unresolved signs. Export/wide reaches `5.06e4`; strict binary64 oscillates at
order `1e15..1e16`. See the
[evidence record](nonlocal-nsr3b4e2d7r20r63k-factor-standard-refinement-evidence-2026-08-26.md).

The next research stage may extend only the two wide lanes to a frozen
iteration-32 frontier. Strict binary64 standard refinement is rejected.
