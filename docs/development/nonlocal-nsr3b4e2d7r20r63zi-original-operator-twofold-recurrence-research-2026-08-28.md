# NSR3-B4E2D7R20R63ZI original-operator twofold recurrence research

Status: `RESEARCH_COMPLETE / CONTRACT_FROZEN / IMPLEMENTATION_NEXT`.

## Decision

Reviewed R63ZH stops certificate repair: the common K2 solution is genuinely
outside the unchanged verifier affine model. R63M already demonstrated the
useful algorithmic shape—exported-factor PCG reaches a complete original-
system certificate at iteration 2 in wide arithmetic. The next portable step
must therefore keep the factor in a preconditioner role and restore the
operator representation that preserves the equation.

R63ZI freezes exactly one exported-lane K2 recurrence whose producer operator
is the tangent Gram

```text
K_T = sigma T T^T,
```

materialized as a width-two binary64 `102 x 102` artifact. Projected portable
RHS/scale, exported factor, permutation, two-update recurrence and R63Y
verifier remain unchanged from R63ZB. The common K2 artifact is retained only
as a full negative control.

This is not a return to certificate decomposition. The candidate must produce
three states without verifier feedback; the verifier and an exact dyadic
operator/product oracle run only after the candidate is sealed.

## Why this is the production-aligned discriminator

- R63ZC and R63ZD prove that tangent products are sufficient and every schedule
  containing a common product rejects; only `TTT` passes.
- R63ZH proves that the rejected common solution is not rescued by more
  certificate arithmetic.
- R63ZA already closes portable factor consumption, so changing the factor or
  recurrence would add an unneeded variable.
- A passing K2 tangent-Gram producer would remove native binary128 from the
  recurrence while retaining the original equation. The next remaining
  producer gate would be dynamic construction from runtime binary64 tangent
  storage rather than another solver redesign.

The complete revision-1 requirements, controls, exact work and stop rules are
frozen in the
[R63ZI contract](../plans/nonlocal-nonlinear-solver-research/03b4e2d7r20r63zi-original-operator-twofold-recurrence-contract.md).

## Boundaries

R63ZI is one offline fixed-profile arithmetic experiment. It changes no
physical model, objective, operator semantics, factor, RHS, scale, iteration
count, verifier, nonlinear state or runtime owner. It authorizes no adaptive
stop, dynamic builder, corpus, timing, GPU, Rust/runtime integration or
production claim. SPEC-38 and ADR-076 remain `Proposed`; ADR-081 promotion
guardrails and all continuum ProductChecks remain in force.
