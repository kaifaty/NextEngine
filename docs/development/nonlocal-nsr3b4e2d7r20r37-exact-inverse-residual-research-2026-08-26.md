# NSR3-B4E2D7R20R37 exact inverse-residual research

Status: `RESEARCH COMPLETE / EXACT DYADIC DISCRIMINATOR SELECTED`.

## Question

Does the already computed 65-column torsion inverse candidate have a
mathematically contractive residual, with R36 failing only because of its
binary128 arithmetic enclosure, or is the candidate itself insufficient?

## Why this is the next smallest test

R36 rules out factor/column-solve failure: all 65 columns complete and their
largest observed residual is below `4e-4`. It does not rule out either of two
very different explanations for `rho=37.54`:

1. `||I-AX||_inf >= 1` for the represented matrix `A` and inverse candidate
   `X`, so no more accurate dot product can rescue this candidate;
2. `||I-AX||_inf < 1`, but cancellation makes the current gamma enclosure too
   wide.

The Krawczyk--Rump verification framework uses a computed approximate inverse
and a rigorous enclosure of its residual; failure of one sufficient enclosure
does not establish singularity. See Rump's review, Theorem 10.6 and the
discussion of verified linear systems:
https://www.tuhh.de/ti3/rump/intlab/ActaNumerica2010.pdf

Ogita, Rump and Oishi provide error-free transformations and `Dot2/DotK`
algorithms whose error bounds behave like doubled or K-fold working precision:
https://doi.org/10.1137/030601818 and
https://www.tuhh.de/ti3/paper/rump/OgRuOi05.pdf. Their round-to-nearest verified
linear-system method also explicitly separates approximate solution work from
the verification enclosure:
https://www.tuhh.de/ti3/paper/rump/OgRuOi05z.pdf.

Those methods are possible remedies, not evidence yet. Before implementing
one, an exact dyadic audit can answer the binary question without another
solver decision: every binary128 matrix/inverse entry is a finite dyadic
rational, so every entry of `I-AX` and every absolute row sum can be accumulated
exactly with integers.

## Hypotheses

| ID | hypothesis | discriminator |
|---|---|---|
| E1 | represented inverse candidate is contractive | exact dyadic `max_i sum_j abs((I-AX)_ij) < 1` |
| E2 | represented inverse candidate is noncontractive | exact dyadic norm is at least one |
| E3 | capture does not correspond to R36 | matrix/factor/audit roots, dimensions or all 65 column outcomes differ |
| E4 | exact apparatus is wrong | exact dyadic controls, permutation repeat or binary128 containment fails |

## Selected experiment

Replay only the frozen torsion R35 failure once. At its sole existing verified
inverse audit, capture the already computed matrix and 65 inverse columns; do
not solve any extra right-hand side. Convert each binary128 operand exactly to
`integer * 2^exponent`, compute `I-AX` and its infinity norm with arbitrary-size
integers, and publish canonical dyadic roots plus an outward binary128 view.

Controls include identity, diagonal power-of-two, a cancellation matrix,
noncontractive negative and operand/order permutation. The current R36 norm is
reproduced separately. No compensated verifier is selected unless the exact
candidate is contractive.

## Decision tree

- exact norm `<1`: classify `ARITHMETIC_ENCLOSURE_DOMINATES`; research a
  bounded `Dot2`/Krawczyk verifier next;
- exact norm `>=1`: classify `INVERSE_CANDIDATE_NONCONTRACTIVE`; research
  scaling/factorization or support degeneracy instead;
- any root/control mismatch: reject R37 apparatus and preserve R36 only.

Counterflow is deliberately not modified in R37. Its direct final-support
refinement is a separate later discriminator after the inverse verification
mechanism is understood.
