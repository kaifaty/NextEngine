# NSR3-B4E2D7R8 extended-precision energy research

Date: `2026-08-22`

Status: `RESEARCH_COMPLETE / CONTRACT_FROZEN / NOT_RUN`

## Question

At the tight D7R7 state, topology is stable but the analytic quadratic model
predicts a sub-ULP descent while both binary64 energy differences report a
much larger ascent. Is that ascent present in a higher-precision evaluation
of the same formula and exact binary64 coordinates?

## Why extended precision is the next discriminator

Changing the trust policy cannot answer whether the energy oracle or its
derivatives are locally authoritative. A separate high-precision energy
implementation can:

1. promote every binary64 coordinate, multiplier and frozen constant exactly;
2. recompute radius, kernel, density, PHR energy and inertia independently;
3. compare extended and binary64 pair membership at the support horizon;
4. resolve current-minus-trial energy in a 64-bit significand rather than the
   binary64 53-bit significand;
5. leave every candidate trial rejected exactly as D7R7 did.

This is sufficient for a sign discriminator. It is not a proposal to run the
production solver in `long double`, and it intentionally makes no Windows or
GPU claim.

## Selected profile and controls

The oracle is admitted only when:

```text
platform              Linux x86-64
FLT_RADIX              2
sizeof(long double)    16
LDBL_MANT_DIG          64
```

All formula constants enter as the exact promoted binary64 values used by the
current solver, including the already frozen kernel normalization. Arithmetic
inside radius, kernel, density, PHR energy and inertia uses `long double`.

Both naive fixed-order and compensated extended sums are emitted. A sign is
called resolved only when the two sums agree and the reduction magnitude is
at least `1024` extended-total ULPs. This threshold is frozen before running.

## Scope

Reevaluate every failed-inner trial from all three unique D7R7 states. Record:

- extended current/trial total, PHR and inertia components;
- naive and compensated reductions and extended ULP ratios;
- extended pair counts and any binary64/extended membership disagreements;
- margins to both cubic-spline branch surfaces `r=h/2` and `r=h`;
- binary64 predicted/raw/direct values from the unchanged parent trace.

The oracle must be independently coded and must not call the binary64
`evaluate_al_vector_inner` for its extended result.

## Routing intent

1. A pair-membership disagreement routes to representation/topology precision
   research.
2. Matching topology plus resolved positive extended reduction against a
   negative binary64 result routes to binary64 energy-evaluation research.
3. Matching topology plus resolved negative extended reduction against a
   positive analytic model routes to local gradient/HVP derivative reclosure.
4. Unresolved signs route to a stronger precision oracle, not parameter
   tuning.

