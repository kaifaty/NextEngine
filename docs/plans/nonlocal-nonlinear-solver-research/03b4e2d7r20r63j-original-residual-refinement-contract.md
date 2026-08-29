# NSR3-B4E2D7R20R63J original-residual refinement contract -- revision 1

| Field | Value |
|---|---|
| Research ID | `NSR3-B4E2D7R20R63J` |
| Architecture snapshot | R63I semantic `4055d0b0...6415`; all stored-operator RHS lanes rejected |
| Engineering consumer | Decide mathematical recoverability before a scalable correction-solver design |
| Claim class | Fixed-depth centered original-residual correction plus independent original-system sign certificates |
| Claim status target | apparatus boundary, per-lane refinement rejection or refined RHS candidate |
| Budget | One capture-only parent replay; exact R63I reconstruction; one common left defect; three depth-16 recurrences; six frozen certificates per lane; no replacement, following transition or timing |

## Exact claim

R63I repeats at semantic `4055d0b0...6415`, route
`WIDE_RHS_SIGN_CERTIFICATE_REJECTED`, with exact initial solution roots:

```text
wide          e76e676b410b0a315013d94645e78af598497f1ea8f074c3e16e1bcdaa57c4a2
export/wide   c9e3c16966c9ed31d513af678ae7edec369448d49d600ea301230daa9319db5f
export/f64    a15db84826bb3ace61015b14e855910818f2b0c41ef7816899c031dfbdc1da79
```

Their baseline certificate roots repeat:

```text
wide          04767e5592f87f615a28d6b7d1a1a693f50bad534a1bb1fec91a94c8db2d23bb
export/wide   bda9f523326080e9025de277cc89916ed9e80041a6ad1968847a48bd6b90c64b
export/f64    190e71c0327119b093c6caa97c3ea0c60f96a479b67488a76dd12c2af0208a21
```

All three are exact, rejected and have 66 unresolved signs. The R60 reference
repeats at root `454dfcc38e41a82cbd81e9b50fcdad2435aaf5d3c2a5512e3fddc0c4604cd935`
with `24/78/0` signs and `rho<1`.

## Common left defect

Construct every entry of `C=I-XH` once with `al_r20_r38_dot2`, using the
identity as leading term and `-XH` thereafter. Require 10,404 exact,
no-underflow dots, finite values/bounds and an outward row-sum maximum exactly
equal to the frozen R60 `rho_bound`.

Bind nominal/bound arrays, worst row, maximum entry bound and root. Mutating
one matrix, inverse, value, bound or row identity bit must change the root.

## Per-lane generation

For each initial candidate independently:

1. Reuse its already bound R63I original residual values/bounds.
2. Reuse its already bound `z=Xr` values/bounds.
3. Set `d0=0` and compute 16 fixed updates
   `d(k+1)=z+C d(k)` with increasing-column binary128 Dot2.
4. At depths `{4,8,9,10,12,16}`, compute every center component
   `lambda0+d(k)` with binary128 Dot2 and bind its rounding bound/root.
5. Run a fresh `al_r20_r63i_sign_certificate` on the actual center against
   original `H,b,X,rho` and the R60 componentwise sign vector.

Generation finiteness and Dot2 exact/no-underflow are apparatus gates. The
analytic contraction prediction is report-only; only the independent
checkpoint certificate passes a depth.

Record error, residual/image bounds, sign counts, minimum separation,
reference-center difference and root at every checkpoint. Record the first
passing frozen depth or zero if none. Later depths are still evaluated and
bound; there is no adaptive early exit.

## Fixed work

Require:

- one common left-defect construction: 10,404 Dot2 reductions;
- per lane: 16 x 102 recurrence reductions;
- per lane: 6 x 102 center reductions;
- per lane: 6 x 102 original residual and 6 x 102 inverse-image reductions;
- exactly three initial two-triangular solves inherited from R63I;
- zero new inverse construction, factorization, iterative factor solve,
  GMRES/LSQR/LSMR/SVD, rank action, row drop or regularization.

The term `refinement` names the bounded centered correction; count exactly one
such construction per lane and zero public refinement/replacement callbacks.

## Controls

1. A one-dimensional literal with `C=1/4` verifies the recurrence and a known
   first-passing checkpoint.
2. A literal nonidentity permutation reproduces all three R63I initial solves.
3. `rho>=1`, nonfinite `C/z`, a wrong recurrence orientation and a missing
   checkpoint fail closed.
4. A center that crosses zero is rejected until its independent interval
   excludes zero with the reference sign.
5. Mutating a recurrence value, center, certificate sign or first-pass depth
   changes the lane root.
6. Classifier precedence covers all five routes.
7. R63B--R63I byte regressions pass.

## Resolution precedence

1. `ORIGINAL_RESIDUAL_REFINEMENT_APPARATUS_REJECTED`.
2. `WIDE_ORIGINAL_RESIDUAL_REFINEMENT_REJECTED`.
3. `EXPORTED_FACTOR_WIDE_REFINEMENT_REJECTED`.
4. `EXPORTED_FACTOR_BINARY64_REFINEMENT_REJECTED`.
5. `EXPORTED_FACTOR_REFINED_RHS_CANDIDATE`.

## Does not count

Replacing the parent solution; executing the following NNQP decision; proving
a scalable correction method; accepting dense `X` or binary128 as production
state; changing factor input, rank, rows or regularization; trajectory, timing,
runtime/GPU or production inference.

## Stop and reconsider

- All lanes pass: freeze a scalable correction-solver discriminator; do not
  integrate the dense-inverse oracle.
- Binary64-consumption lane alone rejects: preserve export storage and
  research bounded wider initial consumption.
- Export/wide rejects: preserve retained-wide state for scalable correction
  research.
- Retained-wide rejects through depth 16: compare original-operator wide QR
  with a preconditioned Krylov correction; do not weaken the certificate.
- R64 and R65 remain blocked for every route.
