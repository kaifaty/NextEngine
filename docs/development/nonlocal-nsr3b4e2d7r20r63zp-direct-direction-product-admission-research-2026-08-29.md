# NSR3-B4E2D7R20R63ZP direct direction-product admission research

Status: `APPARATUS_PASS / SERIALIZED_PACKAGE_PENDING / NO_ENDPOINT_AUTHORITY`.

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

## Apparatus implementation

The first probe is a strict-C++20 target named
`nonlocal-formula-r63zp-direct-product-dyadic-preflight-release`. It uses the
reviewed R63ZM whole-file hashes, derives `x0/p0`, consumes only role-2
`H*x0`, evaluates one two-stage tangent product and reconstructs the exact
real result with a dependency-free signed big-integer dyadic implementation.

The exact schedule executed:

```text
candidate file reads / bytes / hashes     3 / 1,046,961 / 3
candidate factor solves / terms           2 / 20,604
candidate direct-product terms            64,260
exact binary128 decodes                    32,543
exact integer multiplies / additions      64,566 / 64,464
exact alignment shifts / shifted bits     35,138 / 1,263,815
exact interval comparisons                208
```

No Boost/GMP development headers were available on the host, so the probe uses
a small local unsigned-limb magnitude plus explicit sign. This did not become
a single-oracle assumption: a separate Python arbitrary-integer cross-check
over a temporary raw sidecar reproduced the complete exact result.

## Result

Final identities are:

```text
source                     5f91be6d9097ff508a23464ba94cbc3c98abdc110d12cff3d7941a11f3642f16
Release executable         790352c8eac02e4c5974e424f40ce06e332000c22ae2cb0c11dd478b2d3df423
Release stdout run 1       2a419ecfbf4a1198a26102d5ea3608a57193fb81a536c3e81a08b8e6920082fa
Release stdout run 2       2a419ecfbf4a1198a26102d5ea3608a57193fb81a536c3e81a08b8e6920082fa
ASan/UBSan stdout          2a419ecfbf4a1198a26102d5ea3608a57193fb81a536c3e81a08b8e6920082fa
exact product root         271facfdbbb97c777d1a661cf73f5a1eaa25853d42f9e520cab9c785af272f2f
```

The route is `DIRECT_DIRECTION_PRODUCT_ADMISSION_CANDIDATE` with:

```text
component containments          102/102
minimum slack                   452 bits * 2^-567; log2 floor -116
exact rho contained             yes
exact denominator contained     yes
strict positive curvature       yes
step interval contains alpha    yes
fixed compensated update        102/102
```

The independent Python calculation reproduced `102/102`, exact product root
`271facfd...272f2f` and minimum-slack tuple `452/-567/-116`. It compares
against the exact real `primary +/- bound` interval, which is slightly
stronger than the C++ probe's outward-rounded endpoint comparison.

Negative apparatus controls behave first-specifically:

- same-size cache, artifact and audit mutations return identity failure `65`
  before stdout;
- the `bound-zero` control returns `75`, route
  `DIRECT_PRODUCT_BOUND_REJECTED`, `0/102` containments and first escape `0`;
  and
- the exact product root is unchanged under that bound-only control.

LeakSanitizer is unavailable under the desktop ptrace environment; ASan/UBSan
with leak detection disabled completed cleanly and matched Release output.

## Conclusion and next boundary

H1 passes the apparatus discriminator; H2 and H3 are rejected on the fixed
profile. H4 remains open because no serialized producer/checker work and seal
boundary exists. The result authorizes only implementation of that frozen
package. It does not itself admit `H*p0`.

The smallest next action is to freeze exact artifact/audit byte layouts and
implement separate producer and checker translation units. The checker must
retain an independent exact-dyadic implementation, actual executed-work
counts and the future-state firewall. Stop before `p1`, a second product,
recurrence, representation choice or production integration.
