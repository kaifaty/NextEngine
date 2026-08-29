# NSR3-B4E2D7R20R63ZA finite factor-consumption research

Status: `RESEARCH_COMPLETE / CONTRACT_FROZEN`.

## Strongest bounded conclusion

The next portable producer boundary should be three fixed twofold triangular
preconditioner solves using the already selected exported-binary64 `R` factor:
start generation, the exported R63S initial residual, and its iteration-1
residual. It should not yet execute a PCG scalar or update.

The reason for this narrow boundary is conditioning. A twofold binary64 number
has about 106 significand bits, so `u_dd=2^-106≈1.23e-32`. The current common
block has a condition lower estimate near `2.58e32`; therefore the coarse
product `kappa(H)*u_dd≈3.18` is not below one and no ordinary global mixed-
precision theorem selects success. The heuristic factor scale is milder,
`sqrt(kappa(H))*u_dd≈1.98e-16`, but this is not a proof and the estimates are
not norm-identical.

R63H/R63M provide a more specific engineering route. A factor built privately
in wider arithmetic and exported once to binary64 preserves the weak signal;
the same exported factor, promoted for binary128 consumption, reaches the
selected state-2 PCG frontier. R63ZA must determine whether fixed twofold
binary64 consumption can reproduce this factor action without adopting native
binary128 or hiding the error in a tolerance.

## Boundary decomposition

```text
R63Z immutable K2 common operator
             |
             +-- R63ZA: K2 exported-R consumption, three RHS values
             |
             +-- R63ZB: K2 PCG scalars and state 0..2 updates
             |
             +-- R63ZC: portable dynamic H/R builder correspondence
             |
             +-- corpus / nonlinear transaction / performance qualification
```

This order prevents a failed triangular division, stale factor or interval
dependency from being misdiagnosed as a Krylov-convergence failure.

## Competing hypotheses

| ID | Hypothesis | Evidence | Decision |
|---|---|---|---|
| H0 | Fixed K=2 add/multiply/divide plus row-local a-posteriori enclosure is sufficient for the exported factor | 106-bit arithmetic; exported factor already preserves the weak signal; each division has an exact binary64 diagonal | selected for R63ZA |
| H1 | Strict binary64 triangular consumption is sufficient | R63I completes but has the worst residual and is not the selected portable precision after R63Y | retained only as a negative control |
| H2 | Promote the exported factor to native binary128 | known to work in R63M/R63S, but is not portable and would evade the current question | independent offline reference only |
| H3 | Implement the entire two-update PCG now | would mix division, preconditioning, operator, scalar and update errors in one result | deferred to R63ZB |
| H4 | Use generic iterative refinement/GMRES-IR | literature can extend mixed-precision ranges, but its hypotheses do not establish this block and it changes the frozen producer | later fallback only after a local failure |

## Arithmetic model

A finite scalar is a canonical nonoverlapping pair `(hi,lo)` of binary64
values. `TwoSum`, FMA `TwoProduct`, renormalized twofold add/subtract and
multiply use only round-to-nearest binary64. Division uses long division:

```text
q0 = a.hi / b.hi
r0 = a - b*q0
q1 = r0.hi / b.hi
r1 = r0 - b*q1
q2 = r1.hi / b.hi
q  = renormalize(q0,q1,q2) to two words
```

The quotient does not certify itself. For each triangular row, independently
evaluate the exact dyadic equation residual of the returned `(hi,lo)` and
divide an outward residual bound by the exact nonzero diagonal magnitude.
Input and earlier-row radii propagate monotonically. This produces an a-
posteriori interval around the deterministic finite center without relying on
a fitted relative tolerance.

The exact binary128 source/reference values are dyadics. They are available
only to containment audit and cannot enter a quotient, branch or route. The
finite algorithm output itself is exactly the sum of its two stored words;
its enclosure radius measures distance to the ideal exported-factor solve,
not uncertainty about the deterministic output.

## Frozen subjects and work

Use only the exported R63H factor at root `a7a85789...a4f85`, the immutable
permutation `335235cb...8260f`, and the R63Z operator identity. Consume exactly
three RHS values from the frozen exported R63S lineage:

1. original RHS for start generation;
2. exact exported-lane residual before update 1;
3. exact exported-lane residual after update 1.

Each solve executes 5,151 forward terms, 5,151 backward terms and 204
divisions. Total new work is 30,906 triangular terms and 612 divisions. All
three subjects execute even if an earlier containment fails. No `H*x`, PCG
rho/denominator, alpha/beta, vector update or candidate classification is new
work in R63ZA.

## Primary-source grounding

- Hida, Li and Bailey, *Library for Double-Double and Quad-Double Arithmetic*,
  [PDF](https://www.davidhbailey.com/dhbpapers/qd.pdf), defines normalized
  unevaluated double sums, error-free addition/product building blocks and
  long-division corrections. The source supports the finite arithmetic
  design; our controls and containment decide this case.
- Netlib XBLAS, [reference implementation](https://www.netlib.org/xblas/),
  demonstrates extra-precision add/subtract/multiply/divide and triangular
  solve with 106-bit double-double internal arithmetic and ordinary binary64
  storage. R63ZA remains an independent bounded implementation.
- Ogita, Rump and Oishi, *Accurate Sum and Dot Product*, DOI
  [10.1137/030601818](https://doi.org/10.1137/030601818), supplies the error-
  free transformation basis already selected by R63Y/R63Z.
- Carson and Higham, *A New Analysis of Iterative Refinement*, DOI
  [10.1137/17M1122918](https://doi.org/10.1137/17M1122918), explains why
  residual precision and correction quality, not storage precision alone,
  govern very ill-conditioned refinement. It does not prove this route.
- Simoncini and Szyld, *Theory of Inexact Krylov Subspace Methods*, DOI
  [10.1137/S1064827502406415](https://doi.org/10.1137/S1064827502406415),
  motivates explicit inexact-product/preconditioner bounds before admitting a
  Krylov recurrence. R63ZA establishes only the preconditioner side.

## Claim ceiling and roadmap after success

Success selects portable twofold consumption of one frozen exported factor
for three frozen RHS values. R63ZB may then execute the complete two-lane or
selected exported-lane state `0..2` recurrence with independently enclosed
operator products and final R63Y verification.

It does not make the factor builder portable. A later dynamic-builder gate
must construct the K2 common block and a matching preconditioner from the
runtime binary64 tangent, bind topology/face identity, and independently
replay the same arithmetic obligations. Corpus, adaptive stopping, runtime,
GPU, timing and production remain blocked.

Failure localizes the first row/division or dependency-radius boundary. It
does not authorize a fitted tolerance, post-run third word, exact-oracle
classification, native binary128 runtime, rank drop or immediate GMRES/QR
redesign.
