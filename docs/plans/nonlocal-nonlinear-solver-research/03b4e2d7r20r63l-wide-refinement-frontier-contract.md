# NSR3-B4E2D7R20R63L wide-refinement frontier contract -- revision 1

| Field | Value |
|---|---|
| Research ID | `NSR3-B4E2D7R20R63L` |
| Architecture snapshot | R63K semantic `61c739a2...4ade`; wide lanes contract but miss 16-step certificate |
| Engineering consumer | Distinguish slow stationary convergence from need for Krylov/low-rank acceleration |
| Claim class | Exact continuation of two wide standard-refinement lanes through iteration 32 |
| Claim status target | apparatus boundary, retained/export frontier exhaustion or two-lane candidate |
| Budget | One capture-only replay; exact R63K reconstruction; 16 additional residual/factor/update transactions and certificates per wide lane; no strict lane, replacement, following transition or timing |

## Exact parent

Require R63K semantic `61c739a2...4ade`, route
`WIDE_FACTOR_STANDARD_REFINEMENT_REJECTED` and lane roots:

```text
wide          c7301c0cf5b78d0a18c12a6b0e55f2ab268b1bc9619ee7a431305d21b5d156e7
export/wide   16bcae3027caa2fd10a8c6244d904e3c0bb793b622ecc9227749c99f4b512eaa
strict/f64    49954a7ea4289bd3e01f42863839158a7c823f54ccb565a2c709b7f2997933a7
```

Require both wide parent lanes finite, completed at 16, not passed and carrying
their exact final solution roots. Retain the strict lane only as a frozen
negative root; execute zero strict continuation work.

## Continuation arithmetic

For each wide lane and iteration `17..32`:

1. Original `b-Hx` via exact/no-underflow Dot2.
2. One `al_r20_r63i_q_solve` through the lane's existing `R`, permutation and
   scale; require 5,151 forward terms, 5,151 backward terms and 204 divisions.
3. Binary128 Dot2 update `x+delta` for all 102 components.
4. Fresh `al_r20_r63i_sign_certificate` for the actual updated iterate.
5. Bind all residual, solve, update, iterate and certificate roots.

There is no early exit. A nonfinite value or zero diagonal is a deterministic
lane frontier rejection when its exact iteration/stage is recorded.

## Fixed work

Per completed continuation lane require:

- 1,632 algorithm residual Dot2 reductions;
- 82,416 forward and 82,416 backward triangular terms;
- 3,264 divisions;
- 1,632 update Dot2 reductions;
- 1,632 certificate residual and 1,632 certificate inverse-image reductions.

Across both lanes require 32 added factor corrections and 32 certificates.
Count zero strict-binary64 corrections, dense-inverse-generated corrections,
new factorizations/inverses, GMRES/PCG/MINRES, rank actions or state updates.

## Controls

1. Scalar `H=1`, `M=3/4` continuation from an uncertified iteration-16-like
   state crosses at a known first continuation iteration.
2. A lane already passed at 16 is rejected as invalid parent input.
3. Missing iteration 17 or 32, early exit after pass and any duplicate
   iteration fail the work/root ledger.
4. Zero diagonal/nonfinite update records exact frontier rejection.
5. Mutating one parent solution, continuation residual, correction, iterate,
   certificate or first-pass iteration changes the lane root.
6. Classifier precedence covers all four routes.
7. R63B--R63K byte regressions pass.

## Resolution precedence

1. `WIDE_REFINEMENT_FRONTIER_APPARATUS_REJECTED`.
2. `RETAINED_WIDE_REFINEMENT_FRONTIER_EXHAUSTED`.
3. `EXPORTED_FACTOR_WIDE_REFINEMENT_FRONTIER_EXHAUSTED`.
4. `EXPORTED_FACTOR_WIDE_STANDARD_REFINEMENT_CANDIDATE`.

## Does not count

Changing the closed R63K budget; continuing strict binary64; adaptive stopping;
GMRES/PCG/MINRES; dense-inverse correction; state replacement or following
decision; matrix-free runtime closure; rank/row/regularization change;
trajectory, timing, GPU or production inference.

## Stop and reconsider

- Both pass: compare convergence acceleration/correction arithmetic before any
  integration.
- Export frontier exhausts: retain wide factor state and research accelerator.
- Retained frontier exhausts: stop stationary refinement and research
  GMRES-IR/PCG or certified weak-direction correction.
- R64 and R65 remain blocked for every route.
