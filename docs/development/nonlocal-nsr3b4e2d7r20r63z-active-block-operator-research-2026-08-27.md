# NSR3-B4E2D7R20R63Z active-block operator research

Status: `RESEARCH_COMPLETE / CONTRACT_FROZEN`.

## Strongest bounded conclusion

For the frozen dimension-102 common block, a twofold block-dense `H*` operator
artifact is the smallest structurally plausible next producer kernel. This is
an architecture selection for one finite research block, not a performance or
production claim.

The physical tangent has `17,748` nonzero values out of `102*315=32,130`
(`55.238%` density). A sparse two-stage product visits at least
`2*17,748=35,496` stored coefficients per vector. A complete `102 x 102`
common block has at most `10,404` coefficients, so the two-stage path performs
`3.4118x` as many coefficient visits before expansion arithmetic, indirect
indexing and intermediate storage. Its twofold component/radius payload is
also larger before indices: `425,952` bytes for tangent nonzeros versus
`249,696` bytes for the full common block.

This does not say dense storage wins for a future larger or genuinely sparse
block. It says the current active block is not sparse enough to justify a
more dependency-sensitive representation before a correctness discriminator.

## Engineering consumer and boundary

The one decision is the first portable operator representation used by a
future candidate producer. R63Y already selected a separate twofold affine-
image verifier. R63Z therefore defines three identities, with no shared
authority by implication:

```text
CommonOperatorIdentity
  = face/projector + tangent/common roots + dimension + width + order

ProducerOperatorArtifact
  = identity + twofold H* coefficients/radii

CandidateArtifact
  = identity + source/state + twofold x coefficients/radii

VerifierArtifact (R63Y)
  = same identity + independent twofold g/M/rho
```

The producer artifact may generate products; it cannot certify itself. The
verifier rejects any stale or mismatched identity before a finite dot. The
offline exact oracle only audits containment and cannot repair or classify the
finite route.

## Competing hypotheses

| ID | Hypothesis | Evidence | Decision |
|---|---|---|---|
| H0 | Materialize the small common block once and apply it as one twofold dot per row | at most 10,404 coefficients; final reduction preserves correlation; R63Y already proves the arithmetic building blocks | selected for R63Z discriminator |
| H1 | Keep `T` in CSR/CSC and evaluate `T(T^T p)` | finite-horizon/global sparsity is attractive, but this block is 55% full, needs 35,496 coefficient visits and introduces an interval/expansion intermediate | retain as later counterfactual for larger/sparser blocks |
| H2 | Flatten the sparse bipartite join into one dot | preserves correlation but expands each row through column adjacency; work is proportional to `sum_k degree(k)^2` and approaches the existing Gram build | not the smallest current kernel |
| H3 | Use generic verified sparse factor solving | 2026 methods require accurate multipart residuals and target a much lower ordinary binary64 conditioning range than this `~2.58e32` block | not selected as verifier replacement |

## Primary-source grounding

- Lange and Rump, *Accurate floating-point matrix residuals* (2026 manuscript),
  [PDF](https://www.tuhh.de/ti3/paper/rump/MatrixResidualsFinal.pdf), derive
  matrix splittings whose products can be accumulated accurately and note the
  practical advantage of level-3 BLAS over scalar cascaded multiword kernels.
  This supports a future portable common-block builder, not correctness of our
  block by citation.
- Rump, *Verified error bounds for sparse systems, Parts I and II* (to appear
  SIMAX 2026),
  [Part I](https://www.tuhh.de/ti3/paper/rump/sparselss_I_final.pdf) and
  [Part II](https://www.tuhh.de/ti3/paper/rump/sparselss_II_final.pdf), shows
  why factor splitting and accurate multipart residuals matter. The method's
  hypotheses do not automatically cover this block.
- Hida, Li and Bailey, *Algorithms for Quad-Double Precision Floating Point
  Arithmetic*, DOI
  [10.1109/ARITH.2001.930115](https://doi.org/10.1109/ARITH.2001.930115),
  establishes unevaluated multi-component arithmetic as an implementable
  finite representation. R63Y, not the citation, selects exactly two words.
- Iakymchuk et al., *Reproducibility strategies for parallel Preconditioned
  Conjugate Gradient*, DOI
  [10.1016/j.cam.2019.112697](https://doi.org/10.1016/j.cam.2019.112697),
  demonstrates FPE/ExBLAS-based reproducible Krylov reductions on parallel
  systems. It supports later CPU/GPU correspondence research, not a current
  speed or convergence claim.

## Frozen discriminator

R63Z projects every exact common `H*` entry into the selected twofold binary64
format and binds a complete operator identity. It then projects both immutable
R63Y lanes at states `0..2` and evaluates all six `H*x` products as one
flattened `Dot2Err` per row. Representation radii use the same outward product
formula as R63Y.

The independent exact common matrix audits all `6*102=612` outputs. Both lane
ladders execute completely. Success requires finite apparatus, exact artifact
identity, all 612 containments and mutation/stale/orientation controls. There
is no PCG scalar, direction, preconditioner solve, candidate update, state-2
stopping decision, sparse construction or timing.

## Claim ceiling and roadmap after success

Success selects only the portable common-operator matvec artifact for the
frozen block. The next producer stages remain:

1. twofold factor/preconditioner consumption and start generation;
2. twofold PCG scalar/update recurrence through the frozen state-2 frontier;
3. complete candidate/verifier identity and failure-atomic transaction;
4. portable dynamic common-block construction from `T` with independent exact
   correspondence;
5. broader active-set corpus, CPU/GPU correspondence and only then measured
   performance.

Failure localizes coefficient projection, product reduction, identity or
containment. It does not authorize a tolerance, onefold fallback, exact-oracle
classification, a sparse rewrite, wider adaptive arithmetic or production.

