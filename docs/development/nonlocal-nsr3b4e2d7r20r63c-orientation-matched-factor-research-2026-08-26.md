# NSR3-B4E2D7R20R63C orientation-matched factor research

Status: `COMPLETE / ORIENTATION_MATCHED_BINARY64_FACTOR_GATE_SELECTED`.

## Question

R63B found a strong orientation asymmetry after projecting one binary128
triangular inverse `Z` to binary64:

```text
||I - L64 Z64||inf = 1.29e-14
||I - Z64 L64||inf = 1.579
```

Does that refute binary64 factor verification, or only the decision to project
one inverse representation and demand that it work in both multiplication
orders?

## Analytic basis

For square `A` and approximate left inverse `R`, a rigorous
`||I-R A||<1` proves nonsingularity and supports an a-posteriori solution
error bound. Rump states the nonsingularity result as Theorem 1.1 and the
residual error bound as Theorem 10.2. The same review explicitly warns that
left and right inverse residuals can differ drastically for ill-conditioned
matrices.

Source: [Rump, Verification methods: rigorous results using floating-point arithmetic, Acta Numerica 2010](https://www.tuhh.de/ti3/rump/intlab/ActaNumerica2010.pdf).

Oishi and Rump give the construction relevant to the R63B asymmetry. To obtain
a left-oriented approximate inverse `X` of triangular `T`, compute each row
`x_i^T` by solving `T^T x_i=e_i`; their Theorem 4.2 bounds `|X T-I|`. The
paper also develops factor-based verified error bounds without requiring one
explicit full inverse of the original matrix.

Source: [Oishi and Rump, Fast verification of solutions of matrix equations, Numerische Mathematik 2002](https://www.tuhh.de/ti3/paper/rump/OiRu02.pdf).

Higham's triangular-system analysis establishes the componentwise backward
stability of fixed-order forward/back substitution under its arithmetic
hypotheses, while also documenting counterexamples to assuming normwise
forward accuracy solely from the triangular form. This supports executing the
solve but not skipping the independent residual certificate.

Source: [Higham, The Accuracy of Solutions to Triangular Systems, SIAM Journal on Numerical Analysis 1989](https://nhigham.com/wp-content/uploads/2023/08/high89t.pdf).

The 2026 factor-splitting work reinforces both the opportunity and the limit:
normal equations square conditioning; balanced factors can move a problem
towards the binary64 `1/u` boundary, but verified success is still conditional
and not guaranteed. Our matrix condition lower bound is order `1e32`, so its
factor is plausibly at exactly that boundary rather than safely inside it.

Source: [Rump, Verified Error Bounds for Sparse Systems Part I: The Splitting of a Matrix into Two Factors, to appear in SIMAX 2026](https://www.tuhh.de/ti3/paper/rump/sparselss_I_final.pdf).

LAPACK's robust triangular routine separately handles no-transpose and
transpose systems and introduces scaling to prevent overflow. R63C does not
adopt LAPACK or scaling, but must fail closed on any non-finite intermediate so
that a later production design cannot infer overflow safety from this probe.

Source: [LAPACK `xLATRS`, triangular solve with robust scaling](https://www.netlib.org/lapack/explore-html/de/d23/group__latrs.html).

## Competing routes

| Route | Evidence for | Evidence against / gap | Decision |
|---|---|---|---|
| Reuse projected `Z` with a one-sided residual theorem | `LZ` already contracts | forward and transpose solution bounds require different orientations; no RHS gate is frozen | retain as later fallback, not first |
| Construct orientation-matched binary64 factor inverses | primary method computes inverse rows from transposed solves; directly targets the failed order | factor lies near binary64 conditioning limit | selected smallest discriminator |
| Rank-revealing QR/pivoted Cholesky | Gram structure and condition lower bound indicate near dependence | requires rank semantics, threshold and active-row source capture | defer until arithmetic factor gate fails |
| Software extended precision | known way to move the precision boundary | runtime cost and portability are unqualified | later rare fallback only |

## Selected representation

Project the existing factor to binary64 once. Construct two candidates directly
in binary64 rather than projecting the binary128 inverse:

```text
Z_R: columns from L z_j = e_j
     desired certificate for U=L^T:
     X_U = Z_R^T,  X_U U = (L Z_R)^T

X_L: rows x_i^T from L^T x_i = e_i
     desired certificate for L:
     X_L L
```

The required induced norms are therefore
`||I-X_L L||inf` and `||I-L Z_R||1`. R63C also reports the two opposite-order
defects but cannot use them to repair a failed required orientation.

Each substitution uses explicit round-to-nearest binary64 FMA for the fixed
multiply-subtract order and ordinary binary64 division. R63 `Dot2Err` supplies
the candidate defect intervals; exact dyadics independently validate every
entry. A positive result means only that factor-oriented binary64 verification
remains arithmetically plausible.

## Branch after the result

- both required orientations contract: freeze an R63D one-RHS sequential
  triangular solution/sign certificate, including uncertain intermediate
  propagation and exact R60 sign correspondence;
- either required orientation fails with complete containment: stop ordinary
  binary64 factor inversion and research source-row rank revelation versus a
  rare extended-precision certificate;
- apparatus, reconstruction or containment fails: repair only the apparatus;
  make no arithmetic/rank claim.

## Ceiling

R63C solves identity columns only. It does not apply an NNQP right-hand side,
form the inverse of `A`, choose a rank, remove a constraint, compute a center,
execute a trajectory or authorize runtime, GPU, timing or production work.
R64 and R65 remain blocked.
