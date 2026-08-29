# NSR3-B4E2D7R20R63ZN fixed-artifact initial recurrence research

Status: `CONTRACT_FROZEN / IMPLEMENTATION_PENDING`.

## Decision

Use R63ZM as an immutable parent product boundary for exactly the initial
original-input binary128 recurrence prefix. The next package will independently
solve `x0`, prove correspondence with R63ZM role 2, consume that role's value
as `H*x0`, and independently derive `r0`, `z0` and positive `rho0`.

Do not attempt a complete recurrence from the six parent products. The parent
contains products of projected RHS, original RHS, baseline states 0/1/2 and
common-projected state 2. It does not contain products of the dynamic search
directions `p0` and `p1`.

The complete frozen boundary is the
[R63ZN revision-1 contract](../plans/nonlocal-nonlinear-solver-research/03b4e2d7r20r63zn-fixed-artifact-initial-recurrence-contract.md).

## Competing hypotheses and belief update

| Hypothesis | Discriminator | Result |
|---|---|---|
| H1: all three recurrence operator calls can be read directly from R63ZM | compare R63ZM role mapping with the frozen `H*x0/H*p0/H*p1` schedule | falsified: only role 2 can correspond to `H*x0`; no role is `p0` or `p1` |
| H2: later direction products can be recovered bit-exactly from state-product differences | test whether rounded binary128 state updates preserve exact `x1-x0=alpha*p0` | falsified by a one-scalar exact counterexample |
| H3: role 2 can drive a non-circular initial prefix | derive `x0` from factor/RHS before reading role 2, compare all components, then consume only its product value | selected; smallest causal consumer |
| H4: immediately rebuild the complete dynamic recurrence | compare required trust/work surface with H3 | deferred until H3 passes; it would add two dynamic products, two updates and certificates at once |

Before inspection, H1/H2 were plausible because the mathematical operator is
linear and R63ZM transports products of all three baseline states. After the
role trace and rounding counterexample, their probability is effectively zero
for a bit-exact checker. H3 is now the only bounded path that consumes rather
than recomputes a parent operator result without trusting a cached state as the
producer of `x0`.

## Exact structural evidence

R63ZM's raw probe maps cache vectors as:

```text
role 0 = projected RHS
role 1 = original RHS
role 2 = baseline solution 0
role 3 = baseline solution 1
role 4 = baseline solution 2
role 5 = common-projected solution 2
```

The frozen recurrence schedule independently computes the factor solve
`x0`, applies the first operator to that result, forms `r0`, solves `z0`, and
then applies later products to `p0=z0` and derived `p1`. Thus role 2 has a
well-defined admission use only after the consumer's `x0` equals its input.
Roles 3/4 authenticate `H*x1/H*x2`, not `H*p0/H*p1`.

## Binary128 counterexample to state-difference reconstruction

The bounded control used GCC binary128 with strict flags and:

```text
x0 = 1
p = 2^-113
alpha = 1
x1 = fl(x0 + alpha*p)
```

Observed output:

```text
x1_equals_x0=1
state_difference=+0x0.000000000000000000000000000000000000p+0
direct_direction=+0x1.000000000000000000000000000000000000p-113
equal=0
```

Exact identities:

| Item | SHA-256 |
|---|---|
| source | `8bea012b86e32507cfff3817895edfc1d5b503ee15f0a3c02022d2a02609d3b5` |
| Release-style binary | `3ae0a7560870079dace67167420367b215e6c0d240362e103c2a2f85a91a8dbc` |
| stdout | `875f5ea6cfb3e4294b14c99c3645ccda4a4a3c1f97a6777fc9ca7fd1abbba0ea` |

This does not claim the frozen 102-dimensional update hits this exact tie. It
is a constructive refutation of the general identity required to make
state-product subtraction a bit-exact replacement for a direction product.
Using it in the checker would therefore require a new error-bound contract,
which R63ZN explicitly forbids.

## Architecture and claim boundary

The work remains offline serial numerical research. SPEC-38 and ADR-076 are
still `Proposed`; ADR-081 guardrails remain binding. The package is not a
public contract or production consumer, uses no runtime state, and cannot
authorize a portable representation or wider roadmap stage.

The smallest next action is to implement the frozen candidate/checker pair
with complete fixed-capacity work receipts. If the independently solved `x0`
does not match the role-2 input or the consumed product cannot reproduce the
positive initial prefix, stop R63ZN rather than importing later cached states
or widening the claim.

