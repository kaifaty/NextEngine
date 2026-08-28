# NSR3-B4E2D7R20R63ZJ product-precision localization research

Status: `RESEARCH_COMPLETE / CONTRACT_FROZEN / IMPLEMENTATION_NEXT`.

## Decision

R63ZI proves that exact containment of a width-two dense tangent-Gram artifact
and its observed products is not sufficient to preserve the state-2 weak
direction. R63ZC separately proves that the frozen binary128 tangent-product
recurrence does preserve it. The unresolved arithmetic interval spans both
operator application and the rest of the recurrence.

R63ZJ changes exactly one boundary: each of the three logical operator calls
applies the frozen binary128 tangent kernel separately to the exact high and
low binary64 input vectors, projects both outputs to K2 and combines them with
canonical K2 addition. Factor solves, rho/denominator dots, divisions, state
updates, projected inputs, iteration count and verifier remain the R63ZI K2
implementation.

This is cheaper and more informative than jumping to width three. A pass
implicates dense coefficient/product precision and preserves the rest of the
K2 recurrence. A rejection says that even a wide operator call followed by K2
product projection is insufficient, requiring a smaller product-width/update
factorial before representation design.

The complete frozen claim, work ledger and stop rules are in the
[R63ZJ contract](../plans/nonlocal-nonlinear-solver-research/03b4e2d7r20r63zj-product-precision-localization-contract.md).

## Boundaries

R63ZJ is an offline localization experiment and deliberately consumes native
binary128 inside the product callback. It is not a portable candidate and
cannot authorize runtime, GPU, timing or production work. SPEC-38 and ADR-076
remain `Proposed`; ADR-081 and all later continuum ProductChecks remain in
force.

## Revision-1 apparatus rejection

The first dev preflight found one coordinate of the initial K2 solution whose
high/low exponent gap exceeds binary128's 113-bit significand. Collapsing that
pair to one binary128 value is therefore not exact. Revision 1 stopped at
`initial_arithmetic` before any tangent product and carries no endpoint claim.
Revision 2 uses linear component decomposition, the smallest correction that
preserves the frozen K2 input and operator equation.
