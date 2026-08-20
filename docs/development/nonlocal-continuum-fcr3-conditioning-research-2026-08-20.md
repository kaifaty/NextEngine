# Nonlocal FCR3 conditioning research — 2026-08-20

Status: `DECISION / FCR3A_BLOCK_PRECONDITIONER / REPORT_ONLY`

## Trigger

FCR2 proves the corrected objective but requires accepted step factors as low
as `4.66e-10` and up to `1735` backtracks on tiny stiff cases. The next
question is therefore conditioning, not product-scale throughput.

## Primary-source audit

### SISPH/SISSM acceleration

The authors' 2025 SISPH predecessor characterizes SISSM as a Jacobi-like
fixed-point method with typically linear convergence. It evaluates Anderson
acceleration but rejects it as GPU-unfriendly because each iteration requires
dot products and reductions. It recommends Chebyshev acceleration as a
low-overhead GPU path and reports comparable iteration convergence with a
chosen spectral-radius parameter. Those results use a different pressure-only
model, `dt=1 ms`, support `1.2dx` and five iterations; they do not establish
the product `1/240 s`, h2/h3 or coupled-term profile.

The Projective Peridynamics SISSM paper also shows that the semi-implicit
linearization can overshoot. It adds a bounded per-vertex step adjustment and
notes a tradeoff between inner Jacobi accuracy and global iterations. The
Nonlocal implementation does not independently close that tradeoff for our
corrected formulas.

### Pairwise descent

The authors list **Semi-Implicit Pairwise Descent for Nonlocal Continuum
Mechanics** as accepted for SIGGRAPH Asia 2026, but both paper and code remain
officially `to appear` as of 2026-08-20. It is highly relevant and stays on the
watch list, but unavailable formulas cannot be made a current dependency.

### Reliable Iterative Dynamics

RID (TOG 2025, DOI `10.1145/3734518`) proposes a dual-descent framework for
stiff simulation and demonstrates SPH applicability. It is a promising FCR3
alternative, but no public implementation artifact was found in the bounded
audit. Re-deriving the full method before measuring a simpler local curvature
preconditioner would expand scope without a discriminator.

## Decision

FCR3 is split:

1. **FCR3-A conditioning:** add a per-particle `3x3` block-Jacobi
   preconditioner while retaining the FCR2 objective and Armijo acceptance as
   truth. The block contains the inertial Hessian, exact viscous pair blocks,
   compression Gauss–Newton blocks and positive surface radial/tangential
   curvature.
2. **FCR3-B fast iteration:** compare corrected SISSM and optional Chebyshev
   relaxation against the selected reference/preconditioner. Pairwise descent
   or RID can enter only as separately specified alternatives when their
   primary formulas/artifacts are available.
3. **FCR3-C profile reclosure:** h2/h3, cadence and coefficient sweep begins
   only after a fast iteration passes reference/objective gates.

The block-Jacobi candidate is local, reduction-free per update and maps to one
small matrix inversion per particle, so it is compatible with a later GPU
implementation. FCR3-A still runs only on CPU binary64.

## Sources

- [SISPH paper, DOI 10.1111/cgf.70043](https://doi.org/10.1111/cgf.70043)
- [Projective Peridynamics SISSM, DOI 10.1109/TVCG.2023.3271511](https://doi.org/10.1109/TVCG.2023.3271511)
- [Reliable Iterative Dynamics, DOI 10.1145/3734518](https://doi.org/10.1145/3734518)
- [Official Peridynamics publication list](https://peridynamics.com/publications.html)

No claim is made about the unpublished pairwise-descent algorithm beyond its
official title/status.
