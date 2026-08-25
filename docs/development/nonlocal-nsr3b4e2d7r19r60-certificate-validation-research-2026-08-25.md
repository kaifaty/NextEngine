# NSR3-B4E2D7R19R60 certificate-validation research

Date: `2026-08-25`

Status: `RESEARCH COMPLETE / INDEPENDENT VALIDATION SELECTED`.

Parent: R59 `PASS / ROW_LOCAL_ENCLOSURE_CERTIFICATE_CANDIDATE`, stdout
SHA-256 `495bbd213784c4e4f7500c33e2be32c05d41b859145282770364a8fc3a143a4b`,
semantic `e97d68233ce067e7fb948d6cd38bf79ecafb403e16961fadb9a3e9bcb0ce071d`.

## Question

Is the R59 row-local certificate sufficiently independently validated to
justify a later integration contract, or did the same implementation merely
confirm its own accounting?

R59 proves that the unchanged gamma formula closes the retained witness when
fed exact per-row adjacency degree. R60 must not change the witness or seek a
smaller bound. It must attack the candidate's assumptions.

## Required validation layers

### 1. Structural degree ownership

The certificate may never accept a caller-supplied or cached lower degree.
For each row it must derive

```text
degree[row] = flat_offsets[row + 1] - flat_offsets[row]
```

from the workspace owning the directed traversal. A separate validator will
compare a claimed vector with those offsets exactly. Correct nominal and dense
fixtures must pass; forced undercount, forced overcount, decreasing offsets and
maximum-degree mismatch must fail before any certificate classification.

### 2. Arithmetic-domain audit

The standard relative-error model behind `gamma(n)` assumes ordinary finite
operations without overflow and, unless an additive underflow term is carried,
without subnormal intermediate results. R60 will independently replay the
local scalar expression and audit all nonzero displacement, relative,
Jacobian, component-product, dot-partial, term, row-accumulator, absolute-sum,
raw, bound and upper intermediates. Any subnormal or nonfinite value selects a
stronger-bound requirement rather than validation.

### 3. Independent high-precision dominance

R60 will perform one fresh directed binary128 row traversal from the exact R58
selected witness and binary64-owned coefficients. It will use compensated
binary128 folds and a row-local binary128 forward bound. For every row require

```text
binary128(local_binary64_upper[row])
    >= full128_raw[row] + full128_forward_bound[row].
```

This containment is stronger than overlap of two enclosures. It is sufficient,
though not necessary; a single failure rejects validation and does not license
fitting the binary64 bound to the oracle.

Ogita, Rump and Oishi's error-free transformations remain the fallback if the
existing arithmetic path cannot be validated, not a mechanism to reinterpret
a failed row:

- [Accurate Sum and Dot Product](https://doi.org/10.1137/030601818).
- [Accurate Floating-Point Summation Part I: Faithful Rounding](https://doi.org/10.1137/050645671).

Hallman and Ipsen's explicit deterministic summation analysis is retained for
the standard-model assumptions and separation of ordinary and compensated
accumulation:

- [Deterministic and Probabilistic Error Bounds for Floating Point Summation Algorithms](https://arxiv.org/abs/2107.01604).

## Alternatives

### Immediate integration after R59

Rejected. R59's global/local comparison shares the same directed image,
absolute sum and gamma implementation as the candidate.

### Full binary128 sign check only

Rejected. It shows the witness is feasible but does not validate that the
binary64 upper is conservative or that degree cannot be undercounted.

### Randomized property tests

Retained only as future breadth. Random success cannot prove containment; the
first bounded validation should target exact nominal state and deterministic
negative controls.

### Generic EFT/compensated runtime path now

Rejected as premature. R59 already has a substantial negative margin. Add
arithmetic complexity only if independent validation exposes a need.

### Three-layer structural/domain/dominance validation

Selected. It adds no solver work and has predeclared failure routes.

## Selected R60 experiment

1. Reproduce exact R59 stdout, semantic, route, source roots and all aggregate
   observations through a passive capture.
2. Rebuild one exact moved workspace and verify topology, master and witness.
3. Recompute the row-local binary64 scan independently and require exact R59
   roots/counts/maxima.
4. Execute deterministic dense degree controls, then mutate undercount,
   overcount, offset order and declared maximum one at a time; require rejection.
5. Execute one full binary128 directed traversal and one arithmetic-domain
   audit, in stable row/slot order.
6. Compare all 6000 local binary64 uppers with full128 upper endpoints; publish
   dominance count, minimum containment margin and worst row/degree.
7. Require zero local positives, zero dominance failures, zero subnormal and
   nonfinite intermediates, exact negative controls, exact work and rollback.
8. Select validation candidate, stronger arithmetic/domain bound required,
   dominance rejection or structural-proof rejection by frozen precedence.

R60 remains private rollback-only research. PASS authorizes only research and
freezing of a later integration contract. It does not itself change the
authoritative audit, apply the witness, exit restoration, run timing or grant
runtime/production authority.

## Expected consequence

On complete validation, freeze R61 as the smallest integration boundary: a
row-local directed-audit owner with exact topology ownership, legacy global
shadow comparison, negative controls and no change to operator arithmetic. On
failure, retain R59 as diagnostic only and select the specific structural,
underflow or EFT branch identified by R60.
