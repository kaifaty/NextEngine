# NSR3-B4E2D7R20R63ZH center-producer decomposition research

Status: `AUTHOR_EXECUTED / REVIEW_NOT_TESTED / CLAIM_INCONCLUSIVE`.

## Question

Reviewed R63ZG shows that the common endpoint's affine-image center alone is
sufficient to reproduce certificate rejection for both immutable solution
representations. Its representation radius alone passes, and replacing the
unamplified image budget by the `1.00914x` contraction-amplified budget changes
no classification.

The fixed R63Y center producer is:

```text
r_i(x) = v_i - sum_j M_ij x_j
```

where `v`, `M` and `x` are width-two binary64 expansions. The remaining
decision is not merely which term is large, but whether the common center is a
faithful residual of the frozen verifier affine model or a finite-arithmetic
artifact. This is the stop discriminator for the certificate-investigation
lineage.

## Why another ordinary decomposition would be a dead end

The common center infinity is about `3.066e13`, while its complete image radius
is about `3.404e-3`. The existing enclosure therefore already makes ordinary
roundoff or representation radius an implausible explanation by roughly 16
orders of magnitude. Repeatedly splitting the same outward radius cannot
decide whether the solution itself belongs to the affine model being
certified.

R63ZH instead checks one exact algebraic identity. With tangent/common
component-center solutions `x_t`, `x_c` and the same frozen `v,M`:

```text
r_c - r_t = -M (x_c - x_t)
```

If a separate exact-dyadic oracle closes this identity in every row and each
exact center remains inside the R63Y enclosure, the certificate arithmetic is
working: the common solution has moved away from the verifier affine system.
Together with reviewed R63ZC, which already isolates common-operator use, this
ends the certificate-fix search and returns the engineering decision to an
original-operator-preserving accelerator/preconditioner design.

If exact residuals escape their finite enclosures, the finite center producer
is defective and a numerical repair may be researched. If the transport
identity fails, the apparatus or indexing is defective and no solver
conclusion is admitted.

## Minimal discriminator

Add one private solution-expansion DTO without modifying the reviewed R63ZG
detail DTO or certificate. For the frozen tangent/common final solutions:

1. copy the unchanged R63Y width-two solution components and radii;
2. reconstruct every `v_i-M_i x` with a separate local exact-dyadic oracle
   based on canonical integer/exponent arithmetic, not Dot2Err;
3. require each exact center to lie in the immutable
   `image_center +/- image_radius` enclosure;
4. find the exact and finite maximum common row and require the same index;
5. compute every exact `r_c-r_t` and `-M(x_c-x_t)` and require literal dyadic
   equality;
6. seal the maximum-row vector term, tangent/common matrix products,
   displacement transport and all 102 row comparisons.

No fitted epsilon is needed. Equality is exact; containment uses the already
published outward interval. The common maximum-row interval must exclude zero.

## Competing hypotheses

| ID | Hypothesis | Decisive observation |
|---|---|---|
| H0 | finite Dot2 center arithmetic is defective | an exact component-center residual escapes the immutable R63Y image enclosure |
| H1 | representation radii create the rejection | already disfavored by reviewed R63ZG; radius-only cells would need to reject after exact correspondence |
| H2 | solution displacement creates a true fixed-profile residual | all centers are contained, all 102 exact transport identities close and the common maximum-row interval excludes zero |
| H3 | fixture, indexing or lane correspondence is defective | exact transport, roots, maximum-row identity or parent regressions fail |

## Independence and prior art

This discriminator is internal correspondence, not a novelty or method-choice
claim, so external literature cannot decide it. No web search is required.
The candidate center remains the existing Dot2Err producer. The oracle uses a
separate exact `cpp_int` dyadic representation and direct fused-sum formula;
it must not call Dot2Err or reuse the candidate's reduction/root helpers.
Independent review must verify operand order, signs, row-major indexing and
that the exact oracle does not consume candidate centers.

## Stop decision

The [author evidence](nonlocal-nsr3b4e2d7r20r63zh-center-producer-decomposition-evidence-2026-08-28.md)
returns `COMMON_SOLUTION_OUTSIDE_VERIFIER_AFFINE_MODEL`: all `204/204`
centers are contained, exact and finite common maxima both select row `65`,
the common maximum interval excludes zero, and all `102/102` transport
identities close. Dev and two Release outputs are byte-identical. Independent
review is still `NOT_TESTED`, so this is an author result rather than the
reviewed stop decision required below.

- On `COMMON_SOLUTION_OUTSIDE_VERIFIER_AFFINE_MODEL`, stop decomposing or
  modifying the certificate. Preserve the original affine operator as the
  equation being solved; any common operator may be researched only in an
  equation-preserving accelerator/preconditioner role.
- On exact enclosure failure, freeze a new finite-center repair contract. Do
  not silently widen the radius or tolerance.
- On apparatus, identity, work, transport or review failure, close R63ZH as
  `INCONCLUSIVE`; do not open a deeper row/term factorial under this revision.

## Boundaries

R63ZH changes no operator, recurrence, solution, profile, certificate, radius,
precision, iteration, tolerance or nonlinear state. It measures no timing and
has no corpus, CPU/GPU runtime or production authority. SPEC-38 and ADR-076
remain `Proposed`; ADR-081 promotion guardrails remain in force.
