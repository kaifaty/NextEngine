# NSR3-B4E2D7R20R63C research contract -- revision 1

| Field | Value |
|---|---|
| Research ID | `NSR3-B4E2D7R20R63C` |
| Architecture snapshot | SPEC-38 Proposed; ADR-076 Proposed; ADR-081 Accepted; R63B semantic `28bfb856...160fc`; R60 factor lineage |
| Engineering consumer | Decide whether orientation-matched binary64 factor verification is arithmetically plausible before rank semantics |
| Claim class | Profile-bound finite arithmetic/correspondence certificate |
| Claim status target | `SUPPORTED_BOUNDED` factor orientation candidate or exact first boundary |
| Budget | One capture-only replay; existing factor only; two direct binary64 triangular inverse constructions; four exact and four `Dot2Err` defect audits; no RHS, center, rank or timing |

## Exact claim

The unique R63 factor is projected once to binary64. A direct column-oriented
inverse `Z_R` is constructed by fixed-order binary64 forward substitution in
`L z_j=e_j`; a distinct row-oriented inverse `X_L` is constructed by solving
`L^T x_i=e_i` with fixed-order binary64 back substitution and storing `x_i^T`
as row `i`. All operations are finite and their exact work ledgers close.

Strict R63 `Dot2Err` intervals contain the exact dyadic values of all four
defects. The two orientation-matched gates satisfy

```text
||I - X_L L||inf < 1
||I - L Z_R||1   < 1
```

where the second identity is exactly the infinity-norm left-inverse gate for
`X_U=Z_R^T` and `U=L^T`. Opposite-order defects are report-only.

## Exact negation

The first capture, projection, finite substitution, work, EFT/profile,
containment or required orientation gate fails. A report-only opposite side
cannot replace a failed required side. No scaling, refinement, transposition of
the wrong candidate, wider candidate arithmetic or rank action is permitted.

## Fixed definitions and arithmetic

- Dimension and factor: 102 and immutable R63B factor root
  `782588f764834bf498b2a25ad4ab20e84083b51a3d9016786e0c411227c70358`.
- Factor projection: one round-to-nearest cast per binary128 entry; bind its
  binary64 bit root.
- Platform: frozen R63 Linux x86_64 GCC IEEE binary64 profile, explicit FMA,
  `FLT_EVAL_METHOD=0`, round-to-nearest, gradual underflow, no fast math.
- Column solve: for `j=0..101`, rows `j..101`, accumulate
  `rhs-fma(L[row,k],z[k],...)` in increasing `k`, then divide by the positive
  diagonal.
- Row solve: for `i=0..101`, solve `L^T x_i=e_i` from row `i` down to zero,
  accumulating in decreasing triangular dependency order, then store the
  solution transpose as row `i` of `X_L`.
- Each construction executes 176,851 FMA terms and 5,253 divisions; no
  binary128 candidate operation is allowed.
- Exact dyadics are offline correspondence oracles only and cannot affect the
  candidate decision.

## Candidate audits

Evaluate exactly four 102x102 defect matrices:

1. required lower left `I-X_L L`, infinity norm;
2. report-only lower right `I-L X_L`;
3. required upper-via-transpose `I-L Z_R`, one norm;
4. report-only column-candidate left `I-Z_R L`.

Every matrix executes exactly 10,404 candidate dots and 10,404 independent
exact-dyadic oracle dots. Candidate row/column norm folds are outward and
finite. Require `10404/10404` contained entries per matrix. The candidate root
and decision contain no exact-oracle boolean or exact norm.

## Controls

1. A literal well-conditioned 2x2 lower factor constructs both candidates and
   passes both required orientation gates.
2. A literal factor whose diagonal projects to zero reaches a named finite/
   diagonal boundary without widening or scaling.
3. Deliberately shrink one `Dot2Err` interval; the exact oracle rejects it.
4. Corrupt one projected factor entry after root binding; the root gate rejects
   it before candidate classification.
5. Capture uniqueness, hook lifecycle and exact work counts pass; NNQP RHS,
   center, replacement, state and timing counts remain zero.

## Resolution firewall

- Positive: `BINARY64_ORIENTATION_MATCHED_FACTORS_CANDIDATE`.
- Negative: first named capture, projection, construction, work, containment,
  lower-left or upper-transpose boundary.
- Does not count: reusing the projected binary128 inverse as either new
  candidate, requiring only `||I-LZ||inf`, using an opposite defect to repair a
  required one, solving the NNQP RHS, forming `A^-1`, rank/scaling/regularizing,
  applying a center, trajectory, timing or production inference.
- Ceiling: one factor at one immutable 102-dimensional tuple; no universal
  binary64 theorem, solver admission, runtime/GPU or production claim.

## Stop and reconsider

- Stop ordinary binary64 factor-inverse work if either required exact projected
  defect is noncontractive with complete candidate containment.
- Stop at apparatus if candidate containment or fixed work/profile fails.
- If positive, freeze R63D separately before any RHS solve or sign decision.
- R64/R65 remain blocked regardless of result.
