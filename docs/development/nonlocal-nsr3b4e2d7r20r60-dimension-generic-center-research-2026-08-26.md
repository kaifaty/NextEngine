# NSR3-B4E2D7R20R60 dimension-generic centered certificate research

Status: `RESEARCH COMPLETE / REPORT-ONLY TUPLE AUDIT SELECTED`.

## Question

Is R59's sole blind failure caused only by R50's frozen `dimension == 65`
policy, or does the captured inverse/solution cease to admit the same rigorous
centered fixed-point certificate at its actual dimension?

## Mathematical route

For the captured matrix `A`, represented inverse `X`, right-hand side `b` and
legacy solution `x0`, define

```text
C = I - X A
r = b - A x0
z = X r
c0 = 0
c(k+1) = z + C c(k)
x(k) = x0 + c(k)
g(k) = z + C c(k) - c(k)
```

When an outward enclosure gives `||C||inf <= rho < 1`, the fixed-point
correction is unique and differs from `c(k)` by at most
`||g(k)||inf / (1-rho)`. Component intervals around `x(k)` can therefore
certify the sign of every passive solution component. The existing practical
certificate additionally encloses Dot2 arithmetic, residual-input uncertainty
and center summation.

R60 also audits `I-A X`. Right contraction is not required by the left fixed-
point argument alone, but two-sided contraction is frozen as a stronger
admission discriminator for a root-agnostic future callback.

## Selected experiment

Replay only immutable v5 case index 3 with a capture-only refiner that returns
the exact R59 dimension-rejection root and never supplies a replacement. Then:

1. bind the actual dimension and all four tuple roots;
2. compute exact dyadic and independent Dot2 enclosures of `I-A X` and
   `I-X A`;
3. evaluate the existing centered certificate independently at depths
   `4`, `8` and `16`;
4. report every work count and sign classification without changing the
   trajectory.

Literal one-dimensional good/bad inverse controls must respectively show
strict contraction plus a passing centered certificate, and non-contraction
plus a rejected certificate. No web search is needed: this is an
artifact-specific finite-precision certificate using already frozen exact and
Dot2 machinery, not a novelty or external-theorem claim.

## Interpretation

- all two-sided enclosures contract and depth 16 resolves every sign:
  dimension-generic centered certificate candidate;
- contraction fails: inverse-quality boundary;
- contraction holds but signs remain unresolved: centered-depth/arithmetic
  boundary;
- capture, exact containment or controls fail: apparatus rejection with no
  solver claim.

Even a positive result is `SUPPORTED_BOUNDED + EXACT_CERTIFICATE` for one
immutable tuple. It does not prove all dimensions, remove R50's guard, apply a
correction, certify 6/6 trajectories, authorize runtime/GPU code or establish
production readiness.
