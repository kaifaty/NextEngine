# NSR3-B4E2D7R19R46 filter-globalization research

Date: `2026-08-25`

Status: `RESEARCH COMPLETE / CONTRACT FROZEN / IMPLEMENTATION NEXT`.

## Question left by R45

R45 closes the bounded scalar-line hypothesis: the exact R44 tangential
direction is locally useful, but none of its 25 predeclared finite samples
repays the small complete augmented-Lagrangian merit increase introduced by
the R43 normal step. Trust, contact, topology and precision are not the
remaining blocker.

This does not prove that the R43 normal step is physically wrong. It proves
that accepting every normal step only through one monotone penalty/merit
number is too restrictive for this state. R43 reduces the positive density
violation norm from about `8.114e-8` to `3.702e-8`, while its complete AL
merit rises by about `1.761e-15`.

The next discriminator therefore asks:

> Is the exact R43 normal step a robust feasibility-improving filter
> candidate when the physical objective and constraint violation are kept as
> separate coordinates, without committing the state or choosing a
> production filter policy?

## Why a filter, and what it is not

Fletcher and Leyffer introduced filters to avoid forcing objective progress
and feasibility progress into one penalty function. The trust-region
SQP-filter method accepts a trial relative to every filter entry when it lies
sufficiently below or to the left of the entry's exclusion envelope. The
trust-region variant explicitly separates normal and tangential components
and provides a global-convergence framework:

- Fletcher and Leyffer,
  [*Nonlinear programming without a penalty function*](https://doi.org/10.1007/s101070100244);
- Fletcher, Gould, Leyffer, Toint and Wächter,
  [*Global Convergence of a Trust-Region SQP-Filter Algorithm for General
  Nonlinear Programming*](https://doi.org/10.1137/S1052623499357258).

R46 is only a one-step admission discriminator. It does not claim that the
current solver already implements the complete convergence algorithm,
switching condition, trust-radius update, restoration phase or finite filter
lifecycle required for production.

## Correct coordinates for the current formulation

The current normalized inner problem has an inertial objective and positive
density inequalities. Its filter coordinates are therefore

```text
f(y) = 0.5 * ||y - y_hat||_2^2
h(y) = ||max(c(y), 0)||_2
psi(y) = 0.5 * h(y)^2.
```

The PHR/dual density term is deliberately excluded from `f`: putting it in
both `f` and `h` would double-count the constraint and reduce the filter to a
renamed merit test. The inertial objective is finite and globally bounded
below by zero, satisfying the boundedness fact needed by the filter study.

R46 evaluates objective change through the existing pre-cancelled inertia
difference, not by subtracting two nearly equal totals. It evaluates `h`
from freshly rebuilt exact-support workspaces at the immutable source and R43
trial.

## Margin without outcome fitting

For a filter entry `(h_j, f_j)`, the sloping-envelope test used here is

```text
h_trial <= (1 - gamma) * h_j
    OR
f_trial <= f_j - gamma * h_j,
```

with `0 < gamma < 1`. Rather than choose a decimal from the already known R43
result, R46 audits the fixed exact dyadic ladder

```text
gamma_k = 2^-k, k = 1..24,
```

strongest first. A robust feasibility candidate must pass the feasibility
branch even at `gamma=1/2`, and consequently at every weaker audited margin.
The strongest margin must clear a fixed floating-point resolution guard of
`1024*epsilon*max(h_source,h_trial,min_normal)`; this is a numerical sign
certificate, not a physical tolerance.

The ladder is a discriminator, not a selected production parameter. A later
stage must freeze the actual filter constants together with switching,
trust-region and restoration rules before any repeated transaction.

## One-step filter ownership

The current filter is initially empty. Admission is nevertheless tested
against `F union {(h_source,f_source)}`, as required when considering a move
from the current iterate. If the normal step passes by feasibility, R46 roots
the hypothetical next filter containing the source entry. The trial itself
is not committed and no next iterate or outer solve executes.

The existing R43 model gate remains mandatory: predicted and actual hinge
reduction must be positive and their ratio must be at least the inherited
`0.1`. Thus the filter cannot hide a failed local model. Exact R43 contact,
stable-superset coverage and correspondence are rechecked, and every source,
trial and parent root must roll back unchanged.

## Falsifiable result

- `FILTER_FEASIBILITY_STEP_CANDIDATE`: the exact R43 trial passes the
  strongest and all weaker frozen envelopes by its `h` coordinate, retains
  model/topology/contact ownership and produces an exact hypothetical filter
  update;
- `FILTER_RESTORATION_REQUIRED`: the model is trustworthy but the trial does
  not clear the envelope;
- separate parent, coordinate, model, topology/contact, filter-update, work
  and rollback failures remain fail-closed.

A PASS only authorizes research of a complete normal/tangential switching and
restoration transaction. It does not authorize accepting R43 into state,
running another outer, selecting a tolerance, timing, runtime integration or
production use.

Frozen contract:
[R46 filter globalization](../plans/nonlocal-nonlinear-solver-research/03b4e2d7r19r46-filter-globalization-contract.md).

## Pre-evidence identity reclosure

The first executable attempt exposed an identity-serialization defect before
scientific admission: the documented SHA included a trailing line feed while
the raw executable projection did not. The contract now records the correct
raw 2,406-byte SHA `afa98b31...ee4`. No identity text, coordinate, filter
margin, model gate, route or work rule changed; the invalid attempt receives
no outcome credit.
