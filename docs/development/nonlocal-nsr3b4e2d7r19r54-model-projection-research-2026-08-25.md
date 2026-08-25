# NSR3-B4E2D7R19R54 model-to-projection research

Date: `2026-08-25`

Status: `RESEARCH COMPLETE / MODEL-TO-PROJECTION DECOMPOSITION SELECTED`.

## Question

R53 proves that fresh pair-once, captured directed and full binary128 row
evaluation all retain the same 219 positive rows at the R52 64-sweep witness.
Why is the recursively updated Hildreth master maximum only
`1.8707447849842052e-20`, while a fresh row evaluation reaches
`7.7539982074e-14`?

## Research basis

Hildreth's method is a row-action dual coordinate method for a convex QP; its
original formulation and the later primal-dual interpretation establish the
exact-arithmetic equivalence used by our Gram solve
([Hildreth 1957](https://doi.org/10.1002/nav.3800040113),
[Lent--Censor 1980](https://doi.org/10.1137/0318033)). Exact-arithmetic
equivalence is not an attainable-accuracy proof.

Finite-precision iterative methods can drive a recursively updated residual
orders of magnitude below the explicitly recomputed or true residual.
Greenbaum derives this residual-gap boundary and shows that the recursive norm
can approach zero while the actual norm stalls at a roundoff-dependent floor
([Greenbaum 1997](https://doi.org/10.1137/S0895479895284944)). Recent residual
smoothing work uses the same explicit distinction
`true residual - recursively updated residual`
([Aihara et al.](https://doi.org/10.1137/24M1720007)).

Explicit residual replacement or iterative refinement are plausible remedies,
and high-precision QP refinement has a formal basis
([Gleixner et al.](https://doi.org/10.1007/s12532-019-00154-6)). They are not
selected yet because our witness also passes through binary64 correction
assembly, anchor addition and a ball/box projection. The first transition that
creates the observed floor must be measured before choosing a remedy.

## Selected discriminator

At the unchanged R52 64-sweep record:

1. retain the captured recursively updated 494-row master predictor;
2. directly recompute `u - G lambda` in stable binary64 order and with a
   compensated binary128 fold of the same frozen binary64 data;
3. evaluate one fresh pair-once JVP of the already assembled binary64
   correction and compare `u + A correction` with the direct Gram result;
4. reconstruct the unprojected `anchor + correction`, evaluate one fresh
   pair-once JVP and compare its raw rows with
   `raw(anchor) + A correction`;
5. compare the unprojected target rows with the captured projected-witness
   rows from R53/R52;
6. report target/projected/displacement roots and norms, changed component
   count, maximum displacement, exact box violations, ball status and an
   analytic clamp check without executing a new projection;
7. report positive counts, maxima, worst rows, the values of every stage at the
   final worst projected row, and maximum rowwise gap for each transition.

New operator work is exactly two pair-once JVPs: correction and unprojected
target. Gram data, lambda, projected witness and directed certificate are
captured. A single compensated binary128 Gram fold is offline evidence only.

## Frozen classification

Compare the four maximum absolute stage gaps without a fitted tolerance:

1. recursive predictor versus compensated direct `u-G lambda`;
2. direct Gram versus fresh `u+A correction`;
3. `raw(anchor)+A correction` versus fresh target raw;
4. target raw versus projected-witness raw.

The largest exact gap selects, in stable tie order:

- explicit Hildreth residual replacement;
- Gram/correction assembly repair;
- anchor-addition reevaluation;
- ball/box projection model repair;
- or a model/projection-consistent candidate if every gap is exactly zero.

This route identifies the dominant transition, not a production fix. Every
stage and the final worst-row trace remain available even if several gaps are
nonzero.

## Alternatives not selected

- **More sweeps or active-face CG:** rejected because they can make only the
  recursive/model state smaller before the true-residual gap is explained.
- **Immediate residual replacement:** plausible, but premature until direct
  Gram recomputation is compared with correction assembly and projection.
- **Runtime binary128:** rejected; R53 shows that extra precision confirms the
  residual rather than removing it.
- **Weaker gamma or fitted zero:** rejected; 219 rows are raw-positive outside
  the binary128 forward envelope.
- **Another nonlinear outer:** rejected because it changes the witness and
  destroys attribution.

## Recommendation

Freeze and implement one rollback-only R54 decomposition. Preserve all R53 and
certificate bytes. Use its largest exact transition plus the final-worst-row
trace to select one narrowly scoped correction experiment.
