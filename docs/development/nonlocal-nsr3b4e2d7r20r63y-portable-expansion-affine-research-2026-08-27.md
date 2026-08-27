# NSR3-B4E2D7R20R63Y portable expansion-affine research

Status: `RESEARCH_COMPLETE / CONTRACT_FREEZE_NEXT`.

## Question

Can the R63X correlation-preserving certificate be evaluated with portable
binary64 error-free transformations, without native binary128 arithmetic in
the finite classifier, before attempting a sparse runtime producer?

## Why generic sparse verification is not selected first

The frozen principal block has a condition lower estimate near `2.58e32`.
Its common inverse has infinity norm `1.04e33`, while the operator infinity
norm is only `0.254`. Ordinary binary64 projection already produces inverse
defects of order `1e15`.

Rump's 2026 sparse-system methods are a major new result, but their stated
binary64 target is condition number up to roughly `1e16`; the paper calls
remaining examples above `3e20` outside the scope of those verification
methods. The algorithms use accurate/extended residuals and multi-part
approximations even in their successful range. Terao--Ozaki likewise avoid a
dense approximate inverse through an `LDL^T`-based singular-value bound, but
that changes the verifier theorem and can overestimate the forward error.

Our `102 x 315` tangent has 17,748 nonzeros, but the naive composite `ZT` has
32,130 possible entries, over three times the dense `102 x 102` `ZH*`
profile. A direct finite triangular solve is also not algebraically identical
to the frozen dense `Z`: rounding makes a solve on an arbitrary right-hand
side differ from the linear combination of the 102 frozen basis solves used
to define `Z` and its exact defect.

Therefore sparse factorization is not a semantics-preserving next edit. First
determine the smallest portable arithmetic width that preserves R63X.

## Selected discriminator

For each exact R63X scalar `q`, build fixed-width expansions

```text
onefold: q = q0      + delta, |delta| <= radius
twofold: q = q0 + q1 + delta, |delta| <= radius
```

where every `qa` and `radius` is binary64 and the exact rational/dyadic source
is used only by the profile builder and containment oracle. The finite
certificate receives binary64 arrays only.

For width `K`, each image row is one flattened affine dot:

```text
sum_a g[a]
  - sum_j sum_a sum_b M[j,a] * x[j,b]
```

evaluated by the existing strict-binary64 `Dot2Err` error-free
transformation. Outward representation error includes

```text
|M_center| rx + rM |x_center| + rM rx
```

for every matrix/solution pair, plus the vector remainder and the Dot2 error.
No independently widened residual vector exists.

Both widths and both immutable lanes execute states `0..2` completely. The
independent exact R63V images audit all intervals; they cannot classify or
repair a finite result. The route selects the smallest width that reproduces
reject/reject/pass and the frozen `24+/78-` state-2 sign root.

## Primary-source basis

- Ogita, Rump and Oishi, *Accurate Sum and Dot Product*, DOI
  [`10.1137/030601818`](https://doi.org/10.1137/030601818): error-free
  `TwoSum`/`TwoProduct` and Dot2 error bounds.
- Yamanaka et al., *A Parallel Algorithm for Accurate Dot Product*, DOI
  [`10.1016/j.parco.2008.02.002`](https://doi.org/10.1016/j.parco.2008.02.002):
  K-fold accurate dot products admit parallel forms.
- Rump and Ogita, *Verified error bounds for matrix decompositions*, DOI
  [`10.1137/24M165096X`](https://doi.org/10.1137/24M165096X): matrix products
  may be stored as high/low terms and evaluated with accurate dot products.
- Rump, *Verified error bounds for sparse systems, Parts I and II* (to appear
  in SIMAX, 2026):
  [Part I](https://www.tuhh.de/ti3/paper/rump/sparselss_I_final.pdf),
  [Part II](https://www.tuhh.de/ti3/paper/rump/sparselss_II_final.pdf).
  The papers establish the sparse-verification frontier and explicitly use
  extended/multipart residuals.

The application of these methods to the R63X affine certificate is an
engineering hypothesis, not a claim made by those authors.

## Rejected alternatives for this gate

- Native binary128 runtime: correct on the current host, but not a portable
  CPU/GPU production primitive.
- Generic sparse `LDL^T` verification: valuable later, but it changes the
  theorem and its published conditioning range is far below this block.
- `ZT(T^Tx)` with independent interval intermediates: likely recreates the
  dependency loss of R63W and expands the rectangular profile.
- Arbitrary precision/MPFR: valid fallback for tools, not the smallest
  portable candidate.
- Rank truncation or regularization: changes the mathematical problem and
  requires a new physical identity.

## Ceiling

R63Y is an arithmetic feasibility gate over the same dense offline R63X
profile. Even a twofold pass does not make candidate generation, stopping,
sparse execution, GPU correspondence, performance or production ready. A
success authorizes research of a factorized sparse producer using the selected
width and an explicit offline verifier boundary. A failure authorizes a
separate threefold or formulation discriminator, not a tolerance fit.
