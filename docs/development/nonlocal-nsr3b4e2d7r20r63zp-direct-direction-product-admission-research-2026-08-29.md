# NSR3-B4E2D7R20R63ZP direct direction-product admission research

Status: `CONTRACT_FROZEN / APPARATUS_PENDING / NO_ENDPOINT_AUTHORITY`.

## Decision

R63ZO rejected both adjacent-state absolute-envelope constructions. The loss
is not the already reviewed `H*x0/H*x1` product bounds: their weighted effect
on the denominator is about `2^-54`. The unavoidable update-rounding image
under `|H|` contributes about `2^-3`, making the step interval roughly
`2^106` wider than the scalar consistency cell.

The next question therefore retains the explicit matrix-vector product in the
PCG schedule. It does not try another recurrence checker or infer work from
future states. The frozen boundary is the
[R63ZP contract](../plans/nonlocal-nonlinear-solver-research/03b4e2d7r20r63zp-direct-direction-product-admission-contract.md).

## Competing hypotheses

| ID | Hypothesis | Discriminator |
|---|---|---|
| H1 | the existing binary128 two-stage Dot2 primary-plus-bound is a valid fixed direct-product artifact | an independent signed-dyadic checker contains all `102` exact real components |
| H2 | nested Dot2 propagation or scale rounding leaves at least one component outside the published bound | report the first exact escaping component before scalar work |
| H3 | the product is contained but its accumulated uncertainty cannot certify positive curvature or the step | exact denominator and division enclosures reject before the update consequence |
| H4 | the numerical product is sound but a minimal causal work/seal boundary cannot be made complete | adversarial early-stop, reseal and future-state controls expose an unowned path |

H1 is plausible because the R63ZO late oracle was exact/no-underflow, remained
contained and had positive curvature. That observation is not admission: the
same preflight produced both candidate and oracle bounds. R63ZP changes the
trust structure by checking the exact real tangent product through independent
integer dyadics rather than replaying the candidate Dot2 implementation.

## Why this is not R63ZN or R63ZK

R63ZN tried to admit a causal prefix and failed formal review on exact checker
work. R63ZK checked a complete recurrence derived from an untrusted hybrid DTO
and failed on invisible repeated validation/hash work. R63ZP has neither
surface:

- exactly one new product is produced;
- no cached future state is available before the artifact seal;
- no `p1`, second product, recurrence, certificate or ladder exists;
- the checker owns its raw-file reads and exact-dyadic arithmetic directly;
- comparison loops are unconditional or count only executed predicates; and
- the numerical oracle is structurally different from the candidate.

This distinction does not rehabilitate or consume either earlier package.

## Numerical and verification basis

The already recorded primary sources remain applicable:

- Rump and Ogita's verified matrix-decomposition work motivates exact or
  outward componentwise inclusion rather than primary-only equality;
- Bagnara et al. motivate explicit IEEE-format decoding and fail-closed
  treatment of special values; and
- the Netlib PCG template retains the matrix-vector product as an explicit
  algorithmic operation rather than reconstructing it from rounded iterates.

The fixed size makes an exact signed-integer dyadic oracle practical: each
binary128 value decodes to one integer significand and one base-two exponent;
products are exact integer multiplies and sums align only powers of two. No
decimal tolerance, MPFR dependency, observed-error radius or binary128 oracle
round-trip is required.

## Smallest next action

Before implementing a serialized artifact, build one standalone exact-dyadic
apparatus probe that consumes only the reviewed R63ZM files, derives `p0`,
computes the candidate Dot2 product and proves or refutes all `102` component
containments. It must publish the first escaping component or the minimum
bound slack, exact dyadic work counts and two stable output hashes. Stop before
receipt/checker code if containment fails.

No R63ZN repair, recurrence, future-state input, portable representation,
width selection, corpus, timing, runtime, Rust, GPU or production work is
authorized.
