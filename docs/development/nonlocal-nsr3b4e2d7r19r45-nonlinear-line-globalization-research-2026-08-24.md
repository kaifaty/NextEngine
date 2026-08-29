# NSR3-B4E2D7R19R45 bounded nonlinear line-globalization research

Date: `2026-08-24`

Status: `RESEARCH COMPLETE / CONTRACT FROZEN / IMPLEMENTATION NEXT`.

## Question left by R44

R44 proves a strict contact-feasible first-order descent direction for both
the complete normalized merit and the density hinge at the R43 projected
trial. It does not choose a meaningful finite scale. In raw-gradient units an
arbitrary `alpha=1` has no physical or trust-region meaning, while an
outcome-fitted scale would invalidate the experiment.

The next discriminator must answer a narrower question:

> Does the R44 direction contain a finite step which remains inside the
> inherited trust/contact/topology domain and nonlinearly improves density and
> complete merit enough to make the whole R43 normal+tangential step
> admissible relative to its original source?

The normal/tangential decomposition follows feasible trust-region SQP: a
normal step first addresses constraint violation and a tangential step then
recovers objective progress without abandoning feasibility. The stable
source-anchored neighbor ownership follows the finite-displacement list idea
used by Verlet. Primary references are Wright and Tenny,
[*A Feasible Trust-Region Sequential Quadratic Programming Algorithm*](https://doi.org/10.1137/S1052623402413227),
and Verlet,
[*Computer Experiments on Classical Fluids. I. Thermodynamical Properties of
Lennard-Jones Molecules*](https://doi.org/10.1103/PhysRev.159.98).

## Scale-independent parameterization

Let `n` be the exact R43 projected dimensionless normal endpoint and `d` the
exact R44 contact-projected direction. Define

```text
u = d / ||d||_2
v(alpha) = n + alpha*u
y(alpha) = x_source + SPACING*v(alpha), alpha >= 0.
```

`alpha` is now the global-L2 dimensionless length of the tangential addition,
not a multiplier in raw gradient units.

The closed line domain is the minimum of three independently inherited bounds:

1. `alpha_trust = 0.25 - ||n||_2`. The triangle inequality certifies
   `||v(alpha)||_2 <= 0.25`.
2. `alpha_skin = (r_skin - max_i||n_i||_2)/max_i||u_i||_2`, where
   `r_skin = 0.5*skin*sqrt(1-2e-12)/SPACING`. This conservative triangle
   bound implies the unchanged R42 certificate
   `4*max_i||SPACING*v_i||^2 <= skin^2*(1-2e-12)`.
3. `alpha_contact`, the exact minimum positive coordinate crossing from the
   R43 projected trial to every source-inactive lower/upper box face along
   `u`. Source-active faces need no finite cap because both `n` and `u` obey
   their tangent half-spaces.

All numerators and denominators must be finite and nonnegative. The executable
upper point is one binary64 `nextafter` below the minimum positive bound and
is audited directly. A zero or nonfinite domain is a scientific rejection,
not permission to add a contact epsilon.

## Frozen finite ladder

Candidates are generated largest-first:

```text
alpha_0 = nextafter(alpha_domain, 0)
alpha_k = ldexp(alpha_0, -k), k = 0..24.
```

The cap `24` is fixed from binary64 scale rather than the nominal outcome:
`2^-24` is the binary32 unit-roundoff scale and approximately the square root
of binary64 unit roundoff. Continuing a simple first-order backtrack below
that relative interval would test finite-difference cancellation rather than
a new globalization hypothesis. It also gives a fixed maximum of 25 nonlinear
trials.

## Nonlinear admission

For each candidate, rebuild the exact `r<=H` mask and normalized workspace
from the unchanged R42 superset. The first (largest) candidate may be selected
only if all of these hold:

- direct global trust, R42 displacement certificate and all 36,000 box-face
  tests pass with no new or worsened penetration;
- the exact active mask is covered by the superset;
- density hinge reduction from the R43 projected trial is positive and its
  actual-to-first-order ratio is at least the inherited `0.1`;
- complete-merit reduction from the R43 projected trial is positive and its
  actual-to-first-order ratio is at least the same inherited `0.1`;
- density hinge and complete merit are both strictly better than the original
  R40/R43 source, so the complete composite step, not merely the tangential
  suffix, progresses;
- both local and source complete-merit signs are confirmed in normalized long
  double with binary64-owned membership. Binary128 remains conditional on an
  unresolved or disagreeing long-double sign and remains diagnostic only;
- a freshly built independent canonical workspace matches the selected
  superset-masked workspace bit-for-bit.

The two predicted reductions use the exact R44 unit-direction slopes. No
coefficient, penalty, trust bound, contact tolerance or line length is changed
after observing a trial.

## Falsifiable routes

- `NONLINEAR_LINE_GLOBALIZATION_CANDIDATE`: one finite composite candidate
  closes every gate;
- `COMPOSITE_MERIT_RECOVERY_REQUIRED`: a finite local common-descent step is
  found but no candidate recovers positive complete merit from the source;
- `FINITE_COMMON_DESCENT_NOT_OBSERVED`: no candidate closes both nonlinear
  model ratios;
- `LINE_GLOBALIZATION_PRECISION_REQUIRED`: an otherwise admissible candidate
  has an unresolved/disagreeing accepted sign;
- topology, contact, domain, correspondence and work failures retain separate
  fail-closed routes.

The selected state remains rollback-only. A PASS authorizes research of a
repeated composite outer iteration and its termination policy; it does not
commit state, run another outer, time the solver or claim runtime/production
authority.

Frozen contract:
[R45 nonlinear line globalization](../plans/nonlocal-nonlinear-solver-research/03b4e2d7r19r45-nonlinear-line-globalization-contract.md).
