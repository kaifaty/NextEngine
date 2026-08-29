# NSR3-B4E2D7R20R63 research contract -- revision 1

| Field | Value |
|---|---|
| Research ID | `NSR3-B4E2D7R20R63` |
| Parent | R62 strict-binary64 architecture; R60 semantic `c8f11806...1ae2a` and immutable dimension-102 tuple |
| Engineering consumer | Decide whether working-precision rigorous dot products are arithmetically sufficient for the first non-binary128 inverse-defect gate |
| Claim class | Finite arithmetic correspondence for one projected tuple |
| Claim status target | `SUPPORTED_BOUNDED` or first exact arithmetic/profile boundary |
| Budget | One capture-only R60 replay, 20,808 candidate dots, exact dyadic oracle for the same dots, literal controls; no center, solve, retry or timing |

## Exact claim

Project every represented entry of R60's immutable dimension-102 matrix and
inverse independently from binary128 to IEEE-754 binary64 using
round-to-nearest/ties-to-even. For all `102*102` entries of `I-A*X` and all
`102*102` entries of `I-X*A`, a strict-binary64 implementation of
Ogita--Rump--Oishi `Dot2Err`, using explicit FMA product residuals, returns a
finite nonnegative error interval that contains the exact dyadic dot product
of the projected binary64 operands. Outward row folds of those intervals
prove both projected defects strictly contractive.

## Exact negation

The first profile check, error-free transformation control, dot containment,
finite-bound check or two-sided contraction test fails. That exact failure is
the R63 result; no wider type, reordered reduction, center iteration or solver
retry is permitted.

## Frozen inputs and projection

- Capture only v5 case index 3 through the exact R60 capture route.
- Require dimension 102 and the exact R60 parent case/material/tuple roots.
- Convert each matrix and inverse scalar once with the C++ binary128-to-double
  round-to-nearest cast under the audited rounding mode.
- Hash the binary64 bit patterns in row-major order. These projected roots are
  outputs, not fitted inputs.
- The exact oracle reconstructs each binary64 operand as an exact dyadic and
  evaluates products/sums with arbitrary integers. It observes but cannot
  change candidate bounds, row folds or the route.

## Strict candidate arithmetic

Let `u=2^-53`, `eta=2^-1074`, and `n=102`. Require `2*n*u < 1`.

For each dot, execute Algorithm 5.8 in the written scalar order:

```text
[p,s] = TwoProductFMA(x0,y0)
e = abs(s)
for i = 1..n-1:
  [h,r] = TwoProductFMA(xi,yi)
  [p,q] = TwoSum(p,h)
  t = q+r
  s = s+t
  e = e+abs(t)
res   = p+s
delta = (n*u)/(1-2*n*u)
alpha = u*abs(res) + (delta*e + 3*eta/u)
err   = alpha/(1-2*u)
```

`TwoSum` is the branch-free Knuth transform. `TwoProductFMA(a,b)` is
`high=a*b; low=std::fma(a,b,-high)`. Every operation above is a distinct
binary64 operation; compiler contraction/reassociation is forbidden. The
theorem supplies the inclusive interval `[res-err,res+err]`; exact dyadic
containment is the independent correspondence check.

For a defect entry, add the exact binary64 identity scalar (`1.0` or `0.0`) to
the negated dot interval with outward `nextafter`. Fold absolute entry bounds
into each infinity-norm row with upward `nextafter`; reject nonfinite results.

## Execution profile gate

Require and report:

- Linux x86_64 GCC research profile, `sizeof(double)==8`, radix 2,
  `DBL_MANT_DIG==53`, IEC-559/IEC-60559 support;
- `fegetround()==FE_TONEAREST`;
- literal subnormal preservation controls for scalar multiply/add and FMA;
- literal FMA-product reconstruction and cancellation controls;
- compile target retains `-ffp-contract=off` and `-fno-fast-math`.

No profile claim is made for Windows, GPU shaders, FTZ-enabled hardware or a
different compiler.

## Positive and negative controls

1. `TwoSum(2^53,1)` reconstructs the exact dyadic sum.
2. `TwoProductFMA(1+2^-27,1-2^-27)` reconstructs the exact dyadic product.
3. Dot `[1, 2^53, -2^53] dot [1,1,1]` contains exact `1` despite cancellation.
4. A minimum-subnormal product/sum/FMA control exercises the underflow term and
   must contain its exact dyadic result without FTZ.
5. Literal `A=X=[1]` contracts and literal `A=[1], X=[-1]` does not.
6. Deliberately shrink one successful interval below its exact error; the
   independent oracle must reject it.

## Required evidence

- parent/capture/tuple and projected-bit roots;
- exact platform/profile flags and control root;
- right and left counts: exactly `10404/10404` contained;
- right/left exact and candidate outward infinity norms, contraction flags,
  maximum dot bound, maximum exact error and maximum error-to-bound ratio;
- exact candidate dot count `20808` and exact-oracle count `20808`;
- candidate root, oracle root, semantic hash and byte-identical rerun hash;
- R60, R59 and R50 byte/semantic regression.

## Resolution firewall

- Positive: `BINARY64_DOT2ERR_PROJECTED_INVERSE_CANDIDATE`.
- Negative: first named profile, EFT, containment or contraction boundary.
- Does not count: using binary128 in the candidate, changing the captured
  tuple, evaluating a center, applying a correction, solving another inverse,
  comparing wall time or allowing the oracle to repair a decision.
- Ceiling: arithmetic correspondence for one 102-dimensional projected tuple;
  no full certificate, trajectory, runtime, GPU, performance or production
  conclusion.

## Stop and reconsider

- If an EFT/profile control fails, stop before the matrix audit.
- If containment fails, preserve the first dot and its exact operands/result;
  do not inflate the bound empirically.
- If containment passes but contraction fails, preserve the projected norm
  boundary; do not return to binary128 or refine the inverse inside R63.
- R64 is authorized only after every R63 gate passes.

