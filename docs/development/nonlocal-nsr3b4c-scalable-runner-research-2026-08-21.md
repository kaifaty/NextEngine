# NSR3-B4C scalable canonical pressure-runner research -- 2026-08-21

Status: `COMPLETE / DECOMPOSED / B4C0_SELECTED`

## Question

What is the smallest sequence that can replace B4B2's quadratic pressure
oracle with a bounded canonical runner without changing its objective,
floating reduction order, contact KKT, failure semantics or continuation
state?

## Executable-path audit

The selected B4B2 path performs spatial work in four places:

1. `evaluate` builds fluid density, pressure energy, the fluid gradient and
   equal/opposite static-support reaction;
2. `apply_hessian` repeats the same membership for pressure HVPs used by
   trust-region CG and the spectral estimator;
3. every trial state rebuilds pressure topology before acceptance;
4. the feasible contact-onset predictor evaluates and, when active, estimates
   the spectrum at an uncommitted clamped macro prediction.

The box KKT itself is analytic per coordinate and requires no neighbor query.
Replacing only the objective loop would therefore leave the HVP, trial or
forecast path quadratic and would not establish one solver identity.

The existing NSR2-B cell operator already proves exact fluid-only unique pairs
and deterministic adjacency. It cannot be reused literally: static support is
not a pressure centre, contributes only to fluid density, owns an
equal-and-opposite reaction row, and has a separate stable identity space.

## Required reduction order

Exact binary64 correspondence requires more than equal set membership.
B4B2's all-pairs oracle accumulates:

```text
density: (fluid i, fluid j>i), then (fluid i, support b)
gradient/HVP centre i: fluid neighbors by ascending canonical index,
                       then support by ascending canonical index
```

The joint operator must publish one lexicographically sorted unique pair list:

```text
(fluid_i, participant_j)

fluid-fluid:   i < j
fluid-support: participant_j = fluid_count + support_index
support-support: forbidden
```

Each fluid-centred adjacency row is sorted by the combined participant index.
This reproduces the all-pairs arithmetic order; merely iterating cell order or
using an unordered hash is insufficient.

Input storage order is not solver order. Fluid `SampleId` and static
`SupportId` are canonicalized independently before cell construction. The two
namespaces may share a numeric value, but duplicates inside either namespace
fail before pair allocation.

## Capacity and failure model

The research candidate inherits the Proposed SPEC-38 ceilings of `50,000`
fluid samples and `32,768` static support samples. It additionally freezes
`160` admitted participants per fluid centre and `160*N_fluid` joint pairs,
matching the selected tiny solver's bound.

Sample counts, ID uniqueness, finite positions and checked cell coordinates
are validated before cell storage allocation. Pair membership is counted in a
first pass; degree and pair excess fail without a partial pair list. A second
pass allocates exactly the admitted pair count. There is no truncation,
spill-to-quadratic fallback or retained partial result.

## Why canonical publication is later

Canonical publication changes continuation: after each accepted physical
substep the next substep must decode only the published micrometre integers.
That is not a serialization-only check and cannot inherit the binary64 B4B2
trajectory root. It must follow proof that every private trial uses the exact
joint neighborhood and tape.

## Decomposed B4C sequence

1. **B4C0 joint membership.** Exact all-pairs versus sorted-cell pair list,
   density/gradient/HVP order, repeat/permutation digest and typed capacity
   negatives on bounded B4A/B4B states.
2. **B4C1 pressure coefficient tape.** Build pressure density/Jacobian/radial
   coefficients once per outer state and require bit-exact HVP/reaction output
   against B4C0, including active-set and support-only rows.
3. **B4C2Q query substitution.** Replace every current/trial/forecast query in
   one KKT substep and prove atomic workspace promotion/destruction.
4. **B4C2T controller substitution.** Replace the complete B4B2 adaptive and
   fixed-reference controller; require the old binary64 report semantics and
   aggregate state to remain exact before publication.
5. **B4C3 canonical transaction.** Publish every accepted physical substep
   once, decode only the published integers for continuation, publish no frame
   on failure, and require exact repeat/reverse/affine trajectory roots plus
   separately frozen aggregate bounds.

A later B4C4 may package the complete bounded canonical runner result before
B4D reference rehydration. No nominal trajectory precedes it.

## Decision

Freeze B4C0 first. Reuse the existing sorted-cell concept and the independent
NPR1-A canonical publisher later, but do not import the stopped NPR1 solver or
its invalid formula identity. Windows, CUDA, performance, public schemas and
runtime integration remain out of scope.
