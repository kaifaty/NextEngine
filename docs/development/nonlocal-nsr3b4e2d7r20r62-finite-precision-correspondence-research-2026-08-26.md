# NSR3-B4E2D7R20R62 finite-precision correspondence research

Status: `SUPPORTED_BOUNDED / STRICT_BINARY64_DOT2ERR_ARCHITECTURE_SELECTED`.

## Question

What is the smallest finite-precision architecture that could eventually move
the rare R61 inverse/center certificate away from the Linux/GCC `binary128`
research profile without turning an unverified approximation into a runtime
decision?

This is an arithmetic-architecture question. It does not authorize a runtime
callback, change the solver trajectory or claim that Nonlocal is production
ready.

## Claim and negation

The selected candidate must provide a rigorous interval for every `binary64`
dot product using only a strict IEEE-754 round-to-nearest profile, compose those
intervals into a fail-closed contraction test, and admit an independent exact
oracle that cannot influence the candidate decision.

The claim is refuted if the candidate bound is heuristic, depends on
uncontrolled contraction/reassociation, silently flushes subnormals, requires
runtime `binary128`, or can pass when the independent exact result lies outside
the reported interval.

## Primary-source findings

### Compensated `binary64`

Ogita, Rump and Oishi give error-free transformations `TwoSum` and
`TwoProduct`, then Algorithm 5.8 `Dot2Err`. Its returned interval is rigorous
in round-to-nearest and remains valid in the presence of underflow when the
algorithm completes. The paper also states the admission condition
`2*n*u < 1`; for the currently frozen maximum `n=120` in binary64 this has an
enormous safety margin. With an explicit FMA, the product split is available
without a wider floating-point type.

Source: [Accurate Sum and Dot Product, SIAM J. Sci. Comput. 26(6), 2005](https://www.tuhh.de/ti3/paper/rump/OgRuOi05.pdf).

Ozaki et al. independently use the same round-to-nearest accurate-dot
construction to obtain verified linear-system bounds without changing the
rounding mode. This supports a portable certificate design, but only when the
execution profile and operation order are explicit.

Source: [A method of obtaining verified solutions for linear systems suited for Java, JCAM 199, 2007](https://ogilab.w.waseda.jp/ogita/math/doc/2007_OzOgMiOiRu.pdf).

### What a verified inverse proves

Rump's verification survey distinguishes a small computed residual from a
verified theorem premise. For the current centered correction, the relevant
premise is an outward upper bound on `||I-X*A||inf < 1`; the right defect
`||I-A*X||inf < 1` remains an additional admission check. Left and right
defects must not be assumed interchangeable.

Source: [Verification methods: Rigorous results using floating-point arithmetic, Acta Numerica 2010](https://www.tuhh.de/ti3/rump/intlab/ActaNumerica2010.pdf).

### Exact accumulation

A long/superaccumulator can represent an exact binary64 sum or dot product and
round it once. It is therefore a strong independent oracle and a possible later
rare-path alternative, but it has substantially more representation,
normalization and portability surface than the compensated candidate. The
existing private accumulator in `balanced_canonical.cpp` is a scaled-sum
mechanism, not a generic exact product accumulator and cannot be relabelled as
one.

Primary background is summarized in the Ogita--Rump--Oishi paper above and in
[Exact Dot Product as Basic Tool for Long Interval Arithmetic](https://www.math.kit.edu/iwrmm/seite/preprints/media/preprint%20nr.%2010-01.pdf).

### Mixed-precision iterative refinement

Mixed-precision iterative refinement can improve a candidate linear solution
under analyzable conditions. It does not by itself certify the inverse defect,
the sign of all active-set components or equality of solver trajectories.
Therefore it may later generate a better center, but it is not selected as the
verifier.

Source: [Carson and Higham, Accelerating the Solution of Linear Systems by Iterative Refinement in Three Precisions](https://epubs.siam.org/doi/10.1137/17M1140819).

## Selected architecture

```text
strict binary64 candidate
  fixed operation/reduction order
  explicit std::fma product residual
  TwoSum + Dot2Err interval
  outward positive folds
  two-sided inverse-defect gate
  fail closed
              |
              | correspondence only
              v
offline exact dyadic oracle
  exact value of the binary64 inputs
  exact containment audit
  no influence on candidate decision
```

The execution profile must bind:

- IEC 60559/IEEE-754 radix-2 binary64 with 53-bit significand;
- round-to-nearest/ties-to-even;
- gradual underflow and no FTZ/DAZ;
- explicit `std::fma` only where `TwoProductFMA` requires it;
- compiler contraction disabled elsewhere, no fast math and no reassociation;
- fixed scalar reduction order;
- rejection on nonfinite input/output, overflow or failed admission condition.

The current feasibility target already uses `-ffp-contract=off` and
`-fno-fast-math`. R63 must still audit the relevant C++/platform properties and
must not infer FTZ/DAZ state from compiler flags alone.

## Why the alternatives are not selected first

| Candidate | Disposition | Reason |
|---|---|---|
| strict binary64 `Dot2Err` | selected for the next discriminator | rigorous working-precision interval, small mechanism, no rounding-mode switch |
| exact long accumulator | offline oracle / later fallback research | strongest arithmetic result but materially larger implementation and portability surface |
| binary128 shadow | research parent only | does not solve runtime portability and must not decide binary64 execution |
| mixed-precision refinement | possible future candidate generator | convergence/accuracy improvement is not a certificate |
| ordinary compensated sum without bound | rejected | accuracy evidence is not an enclosure |

## Evidence ladder

1. R63: arithmetic only. Project the immutable R60 102-dimensional tuple to
   binary64 and prove `Dot2Err` contains the exact dyadic dot for every entry of
   both inverse defects. No center and no trajectory.
2. If R63 passes, R64: build the complete binary64 centered/sign certificate
   and the independent counterflow slope certificate on immutable captured
   tuples.
3. If R64 passes, R65: compare strict-binary64 solver decisions and trajectory
   roots against a separately frozen corpus expectation. Arithmetic
   correspondence alone cannot skip this stage.
4. Only after those gates may a separate proposal discuss a runtime rare path,
   float64 production states, physical corpus, GPU correspondence or cost.

## Bounded conclusion

R62 selects a falsifiable arithmetic architecture, not an implementation
result. It neither proves that the R60 contraction survives binary64
projection nor that a binary64 center preserves an active-set trajectory. The
next executable claim is deliberately limited to interval containment and
two-sided contraction of one immutable projected tuple.

## Result pointer

The arithmetic intervals passed exact correspondence, but the naively
projected inverse itself is noncontractive. See the
[R63 negative evidence](nonlocal-nsr3b4e2d7r20r63-binary64-dot2err-evidence-2026-08-26.md).
