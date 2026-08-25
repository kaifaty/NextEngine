# NSR3-B4E2D7R19R59 row-local enclosure research

Date: `2026-08-25`

Status: `RESEARCH COMPLETE / ROW-LOCAL OPERATION COUNT SELECTED`.

Parent: R58 `PASS / FIXED_POINT_CONTRACTION_CANDIDATE`, stdout SHA-256
`8078c06230d6436253df966eb617bee202237143a1cf5def0e1dac49028ec257`,
semantic `f7070521395450cc3b54bfe85c9c4c2648abfa87ff37cfaa419da317628d74c3`.

## Question

Why does the authoritative binary64 upper remain positive on 234 rows after
the exact raw expression is strictly negative on all 6000 rows and the inner
model residual has fallen to `9.114810746924865e-36`?

R58 has ended solver/fixed-point attribution. R59 must isolate the first
structural source of enclosure conservatism without changing the witness,
operator arithmetic, gamma definition or runtime state.

## Code audit

The directed JVP evaluates each row independently over
`flat_offsets[row]..flat_offsets[row+1]`. Its actual adjacency degree is

```text
d[row] = flat_offsets[row + 1] - flat_offsets[row].
```

The current certificate instead uses one common bound for every row:

```text
gamma(16 * maximum_degree + 66)
    * (abs(constraint[row]) + absolute_sum[row]).
```

`maximum_degree` is safe, but it charges every row for the densest row's work.
The original operation-count proof is monotone in degree and already assigns
16 operations per directed neighbor plus 66 fixed operations. Therefore the
same proof can be specialized without a new arithmetic assumption:

```text
gamma(16 * d[row] + 66)
    * (abs(constraint[row]) + absolute_sum[row]).
```

This is not a fitted tolerance. For every row `d[row] <= maximum_degree`, so
the local upper is provably no larger than the current upper and is identical
on maximum-degree rows. Slots skipped by the radius guard remain counted,
which preserves conservatism.

The Release target already compiles with `-ffp-contract=off` and
`-fno-fast-math`; R59 does not rely on contraction or reassociation.

## Literature check

The standard roundoff model treats each basic operation as an exact operation
times one bounded relative perturbation and assumes no overflow/underflow for
the ordinary gamma derivation. Hallman and Ipsen give explicit deterministic
summation-error expressions and distinguish general from compensated
summation:

- Eric Hallman and Ilse C. F. Ipsen,
  [Deterministic and Probabilistic Error Bounds for Floating Point Summation Algorithms](https://arxiv.org/abs/2107.01604).

If row-local specialization does not close the certificate, the next smallest
sound mechanism is not a smaller guessed gamma. Ogita, Rump and Oishi provide
error-free `TwoSum`/`TwoProduct` transformations and `Dot2Err`, with a rigorous
working-precision enclosure including underflow:

- Takeshi Ogita, Siegfried M. Rump and Shin'ichi Oishi,
  [Accurate Sum and Dot Product](https://doi.org/10.1137/030601818).
- Siegfried M. Rump, Takeshi Ogita and Shin'ichi Oishi,
  [Accurate Floating-Point Summation Part I: Faithful Rounding](https://doi.org/10.1137/050645671).

Those algorithms are retained for a later staged/EFT contract only if the
local-count experiment leaves positive rows. R59 does not implement them.

## Alternatives

### A. More Dykstra cycles or a third fixed-point outer

Rejected. R58 explicitly closes both, and its binary64 plateau is unchanged
while the model residual continues to contract.

### B. Scale the current gamma until the binary128 result fits

Rejected. This would fit a tolerance to the observed answer and would destroy
the certificate's outcome independence.

### C. Make binary128 the runtime certificate

Rejected. It is an offline attribution oracle and changes the arithmetic and
deployment contract.

### D. Immediately implement compensated/error-free accumulation

Retained as fallback, not selected first. It is mathematically attractive but
changes arithmetic work and needs a separate local-expression enclosure. The
row-local experiment can answer a narrower question with no operator work.

### E. Exact row-local operation count under the existing gamma proof

Selected. It is a read-only 6000-row scalar scan over the exact R58 selected
audit and topology.

## Selected R59 experiment

1. Reproduce exact R58 bytes, semantic route and selected state roots.
2. Rebuild the exact moved workspace once and verify topology/master/geometry.
3. Reuse the captured R58 cycle-64 constraint, directed image, absolute sums,
   witness and binary128 decomposition. Do not execute a new JVP.
4. For every row derive exact adjacency degree and compute both the current
   global-degree and row-local-degree upper.
5. Require current recomputation to be bit-identical to the captured R58 upper
   vector and aggregate.
6. Require local upper `<=` current upper on every row, equality exactly where
   its floating computation dictates, and strict improvement on at least one
   row.
7. Publish degree distribution, current/local positive counts and maxima,
   maximum-degree positive count, per-row comparison roots and exact work.
8. Select one frozen outcome: row-local certificate candidate, staged
   enclosure research required, EFT accumulation research required, or exact
   no-improvement.

R59 is rollback-only research. Even complete row-local closure is one private
certificate candidate, not restoration exit, runtime integration, performance
or production authority.

## Expected consequence

If the local bound closes all rows, freeze R60 as independent validation and
candidate-certificate integration with dense negative controls. If it improves
but does not close, attribute the remaining rows by degree and freeze a staged
local-expression/accumulation bound. If all remaining positives have maximum
degree, move directly to the error-free accumulation branch. In no outcome is
solver depth reopened.
