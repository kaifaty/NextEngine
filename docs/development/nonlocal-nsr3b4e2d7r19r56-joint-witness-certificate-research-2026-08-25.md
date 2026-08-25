# NSR3-B4E2D7R19R56 joint-witness certificate research

Date: `2026-08-25`

Status: `RESEARCH COMPLETE / EXACT CYCLE-64 CERTIFICATE DECOMPOSITION SELECTED`.

## Question

R55 solves density halfspaces and the contact box in one grouped-box Dykstra
state. Its cycle-64 witness is box/ball feasible and has fresh binary64
`h=7.3368315976e-20`, but 308 raw and 476 directed-positive rows remain.
Are those rows a true residual of the frozen binary64 witness, cancellation in
the binary64 evaluation, unresolved high-precision signs, or only the
conservative directed enclosure?

This is a certificate question. Extending Dykstra, changing its projection or
optimizing its 68 pair passes before answering it would move or obscure the
first exact boundary.

## Evidence carried from R53--R55

- R53 proved that pair-once and directed binary64 operators can be compared
  row-for-row and that compensated binary128 recomputation can separate term
  folding from local product error.
- R54 proved that the old `solve -> clamp` pipeline created a real projected
  residual; more precision in its Gram solve was not the remedy.
- R55 removed that model error. The exact cycle-64 roots are
  `19da2a33...270406` for the witness and `dd2db6e5...45bc6d` for the record;
  every positive row remains inside the 494-row master.
- Accurate summation can provide substantially sharper residual evaluation and
  error bounds than an ordinary working-precision fold; see the primary
  analyses by
  [Ogita--Rump--Oishi](https://doi.org/10.1137/030601818) and
  [Ahrens--Demmel--Nguyen](https://doi.org/10.1145/3389360). This supports an
  offline discriminator, not a runtime binary128 decision.

## Selected discriminator

Replay R55 exactly and retain its actual cycle-64 witness, correction, box
dual and captured directed audit without changing the R55 report bytes. On one
fresh moved workspace:

1. verify the cycle-64 witness, record, state, correction and box-dual roots;
2. execute one fresh pair-once binary64 JVP;
3. reuse the captured directed binary64 image and current gamma upper;
4. traverse the stable directed rows once, accumulating ordinary binary64
   terms in compensated binary128;
5. in the same traversal recompute frozen binary64 coefficients, positions and
   witness components in binary128 and attach a conservative binary128 forward
   envelope;
6. report exact row counts, maxima, worst rows, roots, pair/direct differences,
   binary64-versus-resolved-sign disagreements and bound-only positives.

The binary128 path is an offline oracle. It does not replace the current
directed certificate, mutate the witness or select production arithmetic.

## Frozen classification

Precedence after all identity/work/rollback gates:

1. nonzero pair-once versus directed binary64 difference selects operator
   alignment required;
2. any binary128-resolved positive raw row selects true high-precision raw
   residual confirmed;
3. otherwise any binary128-unresolved row selects sign unresolved;
4. otherwise any binary64 raw-positive row selects binary64 cancellation
   confirmed, because every high-precision row is then resolved negative;
5. otherwise any directed-positive upper selects enclosure tightening
   required;
6. only zero directed-positive rows selects the existing next-TRQP
   compatibility route.

No numerical tolerance is fitted. A resolved sign must lie outside the same
conservative binary128 forward envelope used by R53.

## Consequences by route

- **True positive residual:** retain grouped-box Dykstra and research a
  residual-replacement/polish method that works on the exact joint sets; do not
  call the current depth compatible.
- **Unresolved signs:** strengthen only the offline oracle/enclosure proof;
  do not add solver cycles first.
- **Binary64 cancellation:** research an accurate/reproducible binary64
  residual refresh or certified comparison while leaving state binary64.
- **Bound-only positives:** derive a tighter row-local directed enclosure
  before changing the solver.
- **Compatibility:** return to restoration-exit/filter transaction research;
  the R56 diagnostic itself still commits no state.

## Rejected alternatives

- More Dykstra cycles or a post-hoc zero tolerance: outcome-fitted and unable
  to distinguish arithmetic from mathematical residual.
- Runtime binary128: unnecessary for a discriminator and incompatible with
  the current CPU/GPU architecture boundary.
- Lowering gamma: the current enclosure remains authority until a tighter one
  is proved.
- Performance work: shared-host performance selection remains stopped, and
  certificate correctness precedes optimization.

## Recommendation

Freeze and implement one rollback-only R56 command with exactly one fresh
pair pass and one compensated binary128 directed-row traversal. Preserve R55
stdout exactly and select only the first frozen scientific route.
