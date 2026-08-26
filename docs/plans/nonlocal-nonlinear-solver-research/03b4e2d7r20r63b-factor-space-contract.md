# NSR3-B4E2D7R20R63B research contract -- revision 2

| Field | Value |
|---|---|
| Research ID | `NSR3-B4E2D7R20R63B` |
| Parent | R63A semantic `c08b7abe...7639b`; immutable R60 tuple/factor lineage |
| Engineering consumer | Decide whether a binary64 factor-space/QR reformulation remains arithmetically plausible |
| Claim class | Bounded factor-space arithmetic correspondence |
| Claim status target | `SUPPORTED_BOUNDED` factor candidate or exact factor boundary |
| Budget | One capture-only replay; existing factor only; 102 forward triangular columns; four exact and two strict-binary64 defect audits; no RHS solve, center, retry or timing |

## Exact claim

The unique failed inverse audit that invokes the R60 capture callback also
exposes the same 102-dimensional matrix, represented inverse and existing lower
Cholesky factor. The factor reconstructs the captured matrix within its frozen
binary128 bound. A newly constructed represented triangular inverse `Z`
contracts on both sides in exact binary128 arithmetic. After independent
binary64 projection, every strict `Dot2Err` interval contains its exact-dyadic
factor defect and both outward infinity norms remain `<1`.

## Exact negation

The first capture, factor reconstruction, triangular column, exact contraction,
binary64 profile/EFT, containment or projected contraction gate fails. The
first named boundary is preserved; no scaling, pivoting, refinement or wider
candidate arithmetic is permitted.

## Frozen capture

- Replay only v5 case index 3 under the exact R60 capture refiner.
- Simultaneously install the existing verified-inverse capture. The parent
  replay may expose other inverse audits; select only the unique audit whose
  matrix root equals the already frozen R63 matrix root. Require one selected
  audit and 102 selected column solves, while reporting the total observed
  audit count.
- Require dimension 102 and exact R60 matrix/inverse/RHS/solution roots.
- Require captured inverse root equal R60's inverse and the factor's matrix
  root equal the same captured matrix.
- Record the inherited lower-factor root as an observed output.

Revision 2 is an apparatus-only correction made after the first execution
stopped at `FACTOR_SPACE_CAPTURE_REJECTED`: the hook observed 125 parent audits
rather than one. The immutable R63 matrix root was frozen before R63B and does
not depend on factor results. The rejected execution has no scientific credit;
no arithmetic result was available before this selection was corrected.

## Triangular inverse

For columns `j=0..101`, solve `L z_j=e_j` by fixed-order binary128 forward
substitution using the already captured positive diagonal and the known zero
prefix. Execute exactly 176,851 multiply-subtract terms and 5,253 divisions;
report the explicit ledger. No backward solve and no `A` inverse column is
added.

Independently evaluate exact dyadic defects `I-LZ` and `I-ZL`. Both must be
strictly contractive. Also evaluate exact reconstruction `A-L*L^T`; require
containment by the factor's inherited maximum reconstruction bound and report
its exact infinity norm.

## Binary64 candidate

- Project `L,Z` once under the exact R63 profile and bind bit roots.
- Evaluate `I-L64*Z64` and `I-Z64*L64` using R63 `Dot2Err` and independent
  exact dyadics.
- Require exactly `10404/10404` contained entries per side, 20,808 candidate
  dots total, finite outward folds and strict two-sided contraction.
- The candidate decision/root remains independent of exact-oracle booleans.

## Conditioning lower bound

Using exact R63A quantities and `rho_left=||I-XA||inf<1`, compute with directed
binary128 endpoints:

```text
cond_inf(A) >= ||A||inf*||X||inf/(1+rho_left)
```

Report the lower bound and its product with `u64`. Require only that the
directed `u64` lower bound exceed one. Label it `condition_lower_bound`, never
an exact condition number.

## Controls

1. Literal well-conditioned 2x2 lower factor passes exact and binary64 audits.
2. A factor with a projected sub-unit pivot reaches a named noncontraction or
   profile boundary without widening.
3. Deliberately shrink one dot interval; exact oracle rejects it.
4. Corrupt one lower entry after capture; reconstruction rejects it.
5. Hook lifecycle, work ledger and no-RHS-solve controls pass.

## Resolution firewall

- Positive: `BINARY64_CHOLESKY_FACTOR_SPACE_CANDIDATE`.
- Negative: first named capture, reconstruction, triangular, exact inverse,
  profile, containment, right or left factor boundary.
- Does not count: using the full captured `X` as `Z`, recomputing/factoring
  `A`, solving the NNQP RHS, selecting rank, dropping rows, regularization,
  scaling, applying a center, timing or oracle repair.
- Ceiling: one factor representation only; no QR implementation, NNQP/KKT or
  trajectory correspondence.

## Stop and reconsider

- Stop before binary64 if exact `L/Z` does not contract.
- If only one projected side fails, preserve it; do not transpose-repair `Z`.
- Freeze the next Gram/rank or triangular-solve experiment separately.
