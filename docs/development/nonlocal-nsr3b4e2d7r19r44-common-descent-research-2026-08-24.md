# NSR3-B4E2D7R19R44 contact-feasible common-descent research

Date: `2026-08-24`

Status: `RESEARCH COMPLETE / CONTRACT FROZEN / IMPLEMENTATION NEXT`.

## Question left by R43

R43's projected normal step is contact-feasible and strongly reduces density
violation, but increases complete merit. Classical feasible SQP separates a
normal feasibility step from a tangential objective step; see Wright and
Tenny, *A Feasible Trust-Region Sequential Quadratic Programming Algorithm*,
[`10.1137/S1052623402413227`](https://doi.org/10.1137/S1052623402413227).

Before solving a null-space QP, test the cheaper first-order possibility: the
negative complete-merit gradient, projected into the same active-contact
tangent cone, may already be a descent direction for both complete merit and
the density hinge objective.

## Selected discriminator

At the exact R43 projected trial, rebuild the stable-superset trial workspace
and complete normalized inner gradient. Map it to dimensionless coordinates:

```text
g = SPACING * gradient_x complete_merit
d_raw = -g
d_contact = projection_of(d_raw, source_active_contact_cone)
```

No line step is taken. Fresh trial operator `A` computes `A d_contact`. Audit
two directional derivatives in long-double accumulation:

```text
merit slope       = g^T d_contact
feasibility slope = max(c_trial, 0)^T A d_contact
```

Strictly negative signs for both select a contact-feasible common-descent
candidate. Negative merit but positive feasibility slope selects an actual
density-null-space projection study. Nonnegative merit slope rejects the
simple decomposition itself.

The direction is reported both unscaled and unit-global-L2 normalized, with
roots, contact ownership, density-image norm and cosine-like normalized
slopes. No line search, tolerance fitting or moved state occurs.

## Scientific routes

- `CONTACT_FEASIBLE_COMMON_DESCENT_CANDIDATE`;
- `DENSITY_NULLSPACE_PROJECTION_REQUIRED`;
- `COMPOSITE_DECOMPOSITION_REQUIRED`;

Parent/source/work failures precede these routes. A candidate authorizes only
a separately frozen bounded line-globalization experiment.

Frozen contract:
[R44 common descent](../plans/nonlocal-nonlinear-solver-research/03b4e2d7r19r44-common-descent-contract.md).
