# NSR3-B4E2D7R20R63D active-row rank contract -- revision 1

| Field | Value |
|---|---|
| Research ID | `NSR3-B4E2D7R20R63D` |
| Architecture snapshot | SPEC-38 Proposed; ADR-076 Proposed; ADR-081 Accepted; R63C semantic `39ca735e...54aa1`; R63 matrix root frozen |
| Engineering consumer | Select the next research branch after ordinary binary64 factor inversion failed |
| Claim class | Exact represented-rank proof plus profile-bound numerical-rank diagnostic |
| Claim status target | One named source/projector/inverse-conditioning explanation or exact first boundary |
| Budget | One capture-only parent replay; two fixed modular eliminations per matrix; two normalized pivot profiles per matrix; no RHS, row drop, center, trajectory or timing |

## Exact claim

The unique R63 passive principal matrix is captured together with the exact
102 original source row IDs that selected it. Reconstructing those rows from
the immutable sparse operator yields a finite `102 x 315` matrix `B`; the
captured projected block is `H` with dimension `102`.

For each of the two predeclared odd primes, deterministic modular elimination
reports the row rank of the exactly represented dyadic coefficients of `B` and
`H`. Rank 102 under either prime proves exact full row rank over the rationals.
The source Gram `G=B B^T` and `H` are diagonally normalized independently and
receive fixed binary64 and binary128 complete-pivoted Cholesky diagnostics.

The frozen result identifies whether binary64 numerical near-dependence is
already visible in the source rows, appears only after the projector metric,
or appears in neither pivot profile despite the inherited inverse failure.

## Exact negation

The first platform, parent, capture, source-row correspondence, finite value,
dimension, modular mapping, modular work, normalization, pivot, root or
control gate fails. A rank-deficient image under both fixed primes is
inconclusive unless a literal exact duplicate source row is present. No
threshold may convert that inconclusive result into an exact-dependency claim.

## Frozen subject

- Case: v5 manifest item 3, `v5-filled-oblique-jet-twist-3x5x7`.
- Particle/scalar dimensions: `105` particles, `315` scalar coordinates.
- Selected passive dimension: `102`.
- Parent semantic: R63C `39ca735ea4c51a0ecf21cf13a1c5a3653a94361724da9ecc57e8a7d4fd854aa1`.
- Matrix root: immutable R63 root already used by R63B/R63C.
- Capture: the existing verified-inverse hook selects that matrix root and
  additionally stores `source_rows[indices[local]]` in local passive order.
- The hook observes the existing parent audit population; exactly one selected
  matrix call and 102 selected columns are required. Nonselected calls are
  report-only and must retain parent behavior.

## Source reconstruction

- Each selected row ID indexes `problem.op.entry_offsets`.
- Each sparse entry maps its three finite binary64 coefficients to scalar
  columns `3*particle + {0,1,2}` and is promoted exactly to binary128.
- Duplicate writes to one scalar coordinate, if any, are accumulated in stable
  sparse-entry order and counted.
- Bind the ordered source row IDs, dense `B`, sparse update count, nonzero
  scalar count and duplicate-write count to roots.
- Require every row ID and particle index in range and the selected row IDs to
  be distinct. Literal duplicate dense rows are counted independently.
- Form symmetric `G=B B^T` in fixed row/column/coordinate order with the
  existing binary128 accumulator. `G` is diagnostic data and cannot replace
  the modular rank of rectangular `B`.

## Exact modular rank

- Primes, in fixed order:
  `p1=2305843009213693951 (2^61-1)` and
  `p2=2147483647 (2^31-1)`.
- Map a finite dyadic `x` using `frexpq(abs(x), &e)`, its exact 113-bit integer
  significand and `2^(e-113) mod p`; negative exponents use the modular inverse
  of two and negative values use canonical additive negation.
- Use `unsigned __int128` products followed by reduction. No floating-point
  value participates after mapping.
- Elimination scans columns left-to-right, selects the first nonzero row at or
  below the current rank, swaps once, normalizes the pivot row, and eliminates
  rows below it in increasing order.
- Run on rectangular `B` and square represented `H` for both primes. Bind
  pivot columns, row swaps, inversions, updates, ranks and matrix roots.
- `rank==102` for either prime is the only modular proof of full represented
  row rank. Two deficient images without a literal duplicate route to
  `*_MODULAR_RANK_INCONCLUSIVE`.

## Numerical pivot diagnostics

- Form `C_ij=M_ij/(sqrt(M_ii)*sqrt(M_jj))` independently for `G` and `H` in
  binary128; reject non-finite/non-positive diagonals.
- Binary64 candidate: cast `C` once to IEEE binary64 and use deterministic
  complete diagonal pivoting, positive square roots and explicit FMA Schur
  updates. Stop at `pivot <= n*2^-53*max_initial_diagonal`; a pivot below the
  negative threshold is a named PSD/profile failure.
- Binary128 comparison: call the existing frozen
  `al_r20_projector_pivoted_rank` on the same binary128 normalized matrix.
- Publish rank, threshold, minimum accepted pivot, rejected pivot,
  permutation, operation counts and roots for all four profiles.
- These ranks are diagnostic labels only. They cannot delete, merge or
  regularize a source row, and binary128 cannot repair a binary64 route.

## Controls

1. Dyadic-to-field mapping preserves zero, sign, one half, one quarter, the
   smallest normal binary128 value and the smallest binary128 subnormal under
   both primes.
2. A literal full-row-rank `2 x 3` dyadic matrix has modular rank two under
   both primes; a literal duplicate-row matrix has rank one.
3. A literal normalized positive-definite matrix has full binary64/binary128
   pivot rank; a literal rank-one matrix stops at rank one without a negative
   pivot.
4. A classifier control distinguishes source numerical near-dependence,
   projector-only numerical rank loss and full-profile rank without treating
   a modularly inconclusive input as exact dependence.
5. Mutating one selected source row ID or one dense coefficient after root
   binding is rejected.
6. Hook lifecycle, capture uniqueness, 102 selected columns and the zero
   RHS/center/replacement/state/timing ledger close. The parent R63B/R63C
   public outputs remain byte-exact in separate regression invocations.

## Resolution firewall

- `SOURCE_EXACT_DUPLICATE` only when two dense represented source rows compare
  entry-for-entry equal.
- `SOURCE_MODULAR_RANK_INCONCLUSIVE` when neither prime proves source full rank
  and no literal duplicate exists.
- `PROJECTED_MODULAR_RANK_INCONCLUSIVE` when source is proven full but neither
  prime proves represented `H` full rank.
- `SOURCE_ROW_NUMERICAL_NEAR_DEPENDENCE` when both represented systems are
  proven exact-full and binary64 source pivot rank is below 102.
- `PROJECTOR_METRIC_NUMERICAL_RANK_LOSS` when exact-full source has binary64
  rank 102 and exact-full `H` has binary64 rank below 102.
- `FULL_RANK_INVERSE_CONDITIONING_BOUNDARY` when both exact-full systems have
  binary64 pivot rank 102; the inherited R63C inverse failure remains part of
  the interpretation.
- First apparatus/control failure has higher precedence than every scientific
  route.

## Does not count

QR/SVD or a fitted rank threshold; dropping/reordering a live constraint;
using binary128 pivot rank as physical truth; applying an NNQP RHS; forming a
new inverse; scaling the actual solver; center/refinement/trajectory work;
timing, runtime/GPU or production inference.

## Stop and reconsider

- Stop at apparatus on any correspondence, mapping, modular, pivot or parent
  regression failure.
- If source near-dependence is selected, freeze physical row-rank semantics
  before implementing RRQR/SVD.
- If projector-only loss is selected, freeze a projector range/nullspace
  formulation before another linear solver.
- If the full-profile boundary is selected, research a separately bounded
  extended-precision/factor certificate; do not resume ordinary binary64
  inverse construction.
- R64 and R65 remain blocked for every R63D route.
