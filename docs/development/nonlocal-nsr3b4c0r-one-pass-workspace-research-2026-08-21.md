# NSR3-B4C0R one-pass neighborhood workspace research -- 2026-08-21

Status: `COMPLETE / ONE_PASS_PRE_ADMISSION_SELECTED`

## Diagnosis

B4C0's cell broad phase is correct and reduces P1 candidate distance tests
from `27,240` to `18,120` per scan. The failure is caused solely by replaying
that exact scan to allocate a perfectly sized pair vector.

ADR-081 and SPEC-38 require worst-case capacity admission before execution;
they do not require exact-size allocation after membership is known. A private
workspace may therefore reserve its frozen maximum before the query, emit in
one pass and clear the whole candidate on overflow.

## Alternatives

1. **One-pass pre-admitted pair workspace.** Retains cell edge, membership and
   ordering. It removes the duplicate distance scan at the cost of reserving
   the declared maximum. Selected as the smallest causal repair.
2. **Finer cells.** Could reduce distance tests but increases cell probes and
   would select a new geometry parameter from this one failed fixture without
   a time/memory model. Rejected for this retry.
3. **Squared-distance count then exact fill.** Removes square roots from the
   first pass but still executes two membership checks and complicates exact
   cutoff equivalence. Rejected.
4. **Verlet skin/reuse across nonlinear trials.** Potentially larger later
   gain, but changes cache lifetime and admissible displacement; defer until a
   complete cell/tape runner exposes rebuild cost.

## Bounded representation

Fluid and combined-participant indices fit `u32` under the frozen
`50,000 + 32,768` sample capacities. At `160*N_fluid`, pair payload reservation
is at most `64,000,000` bytes. Degree-bounded adjacency payload is at most
`32,000,000` bytes. The current nested-vector row overhead remains diagnostic
and B4C1 must design compact CSR before any nominal runner claim.

## Decision

Freeze B4C0R as the sole construction-policy repair. It inherits every B4C0
math/order/permutation/failure gate and changes no cell edge, cutoff, solver or
physical coefficient.
