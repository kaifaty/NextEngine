# NSR3-B4E2D7R20R63K factor standard-refinement contract -- revision 1

| Field | Value |
|---|---|
| Research ID | `NSR3-B4E2D7R20R63K` |
| Architecture snapshot | R63J semantic `4a4d1798...8f96`; dense-oracle correction recovers all lanes at depth 4 |
| Engineering consumer | Choose standard factor refinement versus GMRES-IR/PCG/low-rank correction |
| Claim class | Sixteen fixed original-residual/factor-correction iterations with independent sign certificates |
| Claim status target | apparatus boundary, per-lane rejection or exported-factor standard-refinement candidate |
| Budget | One capture-only replay; exact R63J reconstruction; 16 residual/factor/update iterations per lane; five certificates per lane; no dense-inverse correction, replacement, following transition or timing |

## Exact parent

Require R63J semantic `4a4d1798...8f96`, route
`EXPORTED_FACTOR_REFINED_RHS_CANDIDATE`, stdout regression and exact depth-4
pass roots for all three oracle lanes. Reconstruct the immutable R63I initial
solutions, baseline rejection certificates, factor/permutation/scale, R60
reference and original `H,b,X` roots exactly.

The R63J dense-oracle correction is replayed only to bind the parent result. It
must not supply any R63K correction or iterate.

## Per-iteration arithmetic

For each lane and iteration `k=0..15`:

1. Compute all 102 components of `r=b-Hx` using `al_r20_r38_dot2`; require
   exact/no-underflow and bind values/bounds/root.
2. Solve `B delta=r` with the inherited permutation, scale and two frozen
   triangular substitutions. Wide lanes use `al_r20_r63i_q_solve`; the strict
   lane uses `al_r20_r63i_d_solve` after the one required residual cast.
3. Wide lanes update each component with binary128 Dot2. The strict lane uses
   one ordinary binary64 addition and retains binary64 iterate storage.
4. Bind residual, scaled/permuted RHS, both triangular intermediates,
   correction, update and cumulative work roots.
5. At iterations `{1,2,4,8,16}`, promote the actual iterate if necessary and
   run a fresh R63I original-system sign certificate.

No residual interval is fed as an exact correction RHS. The algorithm consumes
the nominal computed residual; the independent certificate accepts or rejects
the actual resulting iterate, including all residual, solve and update error.

## Lane observation and nonfinite semantics

Record `finite`, `completed_iterations`, first failure iteration/stage and all
completed roots. A zero diagonal or nonfinite result fails that lane. The
apparatus remains exact when the frozen failure is recorded deterministically,
all later work in that lane is skipped and the classifier selects the
corresponding rejection route.

For a finite completed lane require:

- 16 original residual evaluations;
- 16 two-triangular corrections;
- 16 updates;
- all five independent certificates;
- first passing checkpoint or zero;
- per-checkpoint error ratios relative to the preceding checkpoint/baseline;
- monotonic-error observation and minimum achieved error.

Only an exact passing sign certificate selects lane success.

## Fixed work

For every completed dimension-102 wide correction solve require `5,151`
forward terms, `5,151` backward terms and 204 divisions. Require the same
counts for strict binary64. Per completed lane require 1,632 original residual
Dot2 reductions, 1,632 updates, 82,416 forward and 82,416 backward terms, and
3,264 divisions. Each completed lane adds five times 102 residual and 102
inverse-image verification reductions.

Count zero dense-inverse correction multiplies. `X` may appear only inside the
read-only certificate and parent reconstruction.

## Controls

1. Scalar `H=1`, correction inverse `M=3/4`, `b=1`, `x0=-3` verifies
   original-residual refinement and first certifies at iteration 2.
2. A nonidentity permutation verifies factor correction orientation.
3. Zero diagonal, nonfinite correction and nonfinite update record the exact
   lane failure without apparatus rejection.
4. Reversing the update sign fails the scalar sign certificate.
5. Mutating one residual, factor intermediate, correction, iterate,
   certificate or failure stage changes the lane root.
6. Classifier precedence covers all five routes.
7. R63B--R63J byte regressions pass.

## Resolution precedence

1. `FACTOR_STANDARD_REFINEMENT_APPARATUS_REJECTED`.
2. `WIDE_FACTOR_STANDARD_REFINEMENT_REJECTED`.
3. `EXPORTED_FACTOR_WIDE_STANDARD_REFINEMENT_REJECTED`.
4. `EXPORTED_FACTOR_BINARY64_STANDARD_REFINEMENT_REJECTED`.
5. `EXPORTED_FACTOR_STANDARD_REFINEMENT_CANDIDATE`.

## Does not count

Dense-inverse correction; GMRES/PCG/MINRES; replacement or following NNQP
decision; matrix-free residual closure; runtime integration; rank/row/
regularization change; trajectory, timing, GPU or production inference.

## Stop and reconsider

- All lanes pass: freeze matrix-free residual correspondence before any state
  replacement.
- Strict lane rejects: preserve exported storage with wide/compensated
  correction arithmetic.
- Export/wide rejects: retain wide factor state for the next correction
  method.
- Retained-wide rejects: research preconditioned GMRES/CG/MINRES or a certified
  weak-direction low-rank correction.
- R64 and R65 remain blocked for every route.
