# NSR3-B4E2D7R20R63M preconditioned-CG contract -- revision 1

| Field | Value |
|---|---|
| Research ID | `NSR3-B4E2D7R20R63M` |
| Architecture snapshot | R63L semantic `16633e07...7333`; stationary wide/export candidates at 17/21 corrections |
| Engineering consumer | Select PCG versus GMRES-IR/low-rank acceleration |
| Claim class | Eight-step SPD preconditioned Krylov solve with every-iterate original-system certificates |
| Claim status target | apparatus/SPD boundary, retained/export rejection or two-lane PCG candidate |
| Budget | One capture-only replay; exact R63L reconstruction; two eight-update PCG lanes; eight preconditioner solves and nine algorithmic H applications per lane; no strict lane, restart, replacement or timing |

## Exact parent

Require R63L semantic `16633e07...7333`, route
`EXPORTED_FACTOR_WIDE_STANDARD_REFINEMENT_CANDIDATE` and frontier roots:

```text
retained-wide  ca308376d80d1df349e03b4b58e5f681d0692e56f621426c2ccf4ad7d9a34136
export/wide    16ac04bff852fe13a7c974d9bcb9be1a774bccb12f250f4c55e86eb2c2582f63
```

Reconstruct exact R63I initial solutions and R63K/R63L parent lanes. PCG must
start from the R63I roots, never a refined parent solution. Bind this start
correspondence explicitly.

## Frozen arithmetic

For each lane:

1. Compute initial `r=b-Hx` with 102 original-system Dot2 reductions.
2. Apply the lane factor preconditioner once through two triangular solves.
3. Compute `rho=r^Tz`; require exact/no-underflow, finite and positive.
4. For iterations `1..8`:
   - compute `q=Hp` with 102 Dot2 reductions;
   - compute positive `denominator=p^Tq` and `alpha=rho/denominator`;
   - update all 102 components of `x` and recurrence `r` with Dot2;
   - independently certify actual `x` against original `H,b,X`;
   - compute and bind recurrence/direct-residual infinity drift;
   - unless iteration 8, apply the factor to recurrence `r`, compute positive
     `rho_new`, `beta=rho_new/rho`, and update `p=z+beta p` with Dot2.

All scalar divisions use binary128 round-to-nearest after positive finite
denominator checks. No residual replacement, restart, reorthogonalization or
look-ahead exists.

## Lane success and breakdown

Record all scalars, vector roots, first passing iteration, minimum certified
error, maximum residual drift and exact breakdown iteration/stage. A lane
passes only if it completes all eight updates and at least one frozen iterate
matches all 102 R60 signs. Continue after pass.

Nonpositive/nonfinite `rho`, `p^THp`, alpha or beta, nonfinite factor solve or
update is a deterministic lane rejection. Apparatus remains exact when the
failure location/root and skipped work are exact.

## Fixed algorithmic work per completed lane

- 8 factor preconditioner solves: 41,208 forward and 41,208 backward terms,
  1,632 divisions;
- 9 original-`H` applications: one initial residual plus eight `Hp`, totaling
  918 row Dot2 reductions;
- 8 `r^Tz` products and 8 `p^THp` products;
- 816 `x` updates, 816 recurrence-residual updates and 714 direction updates;
- 8 independent certificates: 816 residual plus 816 inverse-image Dot2
  reductions, excluded from algorithmic equal-work counts.

The `r^Tz` count includes initial `rho0` and seven `rho_new`; no unused eighth
post-update preconditioner/rho exists. Direction updates occur after iterations
1--7 only.

Across both lanes count 16 factor solves and 18 algorithmic `H` applications,
versus stationary parent 38 solves/applications. Count zero strict-binary64
work, dense-inverse corrections, new factor/inverse construction,
GMRES/MINRES, rank action, row drop, regularization or state update.

## Controls

1. Scalar SPD `H=1`, preconditioner inverse `3/4`, `b=1`, `x0=-3` certifies
   exactly at PCG iteration 1.
2. Negative scalar `H` rejects at nonpositive `p^THp` with exact stage.
3. A passed/refined parent start is rejected; only the R63I start is valid.
4. A nonidentity permutation verifies preconditioner orientation.
5. Mutating one scalar, vector, drift, certificate, breakdown or first-pass
   iteration changes the lane root.
6. Classifier precedence covers all four routes.
7. R63B--R63L byte regressions pass.

## Resolution precedence

1. `PRECONDITIONED_CG_APPARATUS_REJECTED`.
2. `RETAINED_WIDE_PCG_REJECTED`.
3. `EXPORTED_FACTOR_WIDE_PCG_REJECTED`.
4. `EXPORTED_FACTOR_WIDE_PCG_CANDIDATE`.

## Does not count

Wall-time speedup; strict/compensated binary64; residual replacement or PCG
restart; GMRES/MINRES; matrix-free correspondence; state replacement or
following decision; rank/row/regularization change; trajectory, GPU or
production inference.

## Stop and reconsider

- Both pass: freeze dense-versus-rectangular matrix-free `H` application
  correspondence, then precision engineering.
- Export rejects: retain wide factor state and research export error carrier.
- Retained rejects/breaks down: research GMRES-IR or certified weak-direction
  low-rank correction.
- R64 and R65 remain blocked for every route.
