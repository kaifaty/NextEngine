# NSR3-B4E2D7R20R63ZK initial independent review evidence

Status: `INITIAL_REVIEW_NO_GO / ONE_BATCHED_REPAIR_AUTHORIZED`.

## Verdict

Independent review of frozen snapshot
`33e24152cd0634f01df25b27f7e3fd18ebb67c56` returned `NO-GO`.
The snapshot remains author evidence only and cannot authorize the portable
representation discriminator. R63ZJ remains `INCONCLUSIVE`.

The reviewer verified the contract parent `388f7462...8eb6`, complete diff
`e6872a8e...0688`, all frozen source blobs and cache identity. A detached
Release build reproduced binary `49486450...8175`; two executions reproduced
stdout `37129010...9041`. R63ZI/R63ZG/R63ZH/R63ZJ regressions remained exact at
`98736993...0a1e`, `ca2a0f80...29c9`, `181ac246...6722` and
`b9ded7d7...7f8a`. `git diff --check` passed.

## Load-bearing findings

1. The trace exposed rho/denominator/alpha/beta only as opaque roots, so the
   checker could not compare their K2 high/low semantics directly.
2. The independent scheduler omitted the frozen strict-positive guards for
   `rho0`, `denominator0`, `rho1` and `denominator1`.
3. Producer and checker work were not separately and completely sealed.
   Metadata/hash/validation paths and all control comparisons were absent;
   three author-valid controls alone executed 18 unowned tangent kernels.
4. The classifier collapsed every `Mismatch` into one generic correspondence
   rejection instead of returning the first specific `HYBRID_TRACE_*` route.
5. Product/state/recurrence semantic guards and several identity fields were
   not compared. A read-only harness changed `product.exact`, `state.exact`,
   `recurrence.complete` or a product artifact root without producing any
   mismatch. Harness output and binary are `23ccf6f1...fbff` and
   `8948da3d...8e90`.

## Positive boundary retained

The review confirmed that the expected scheduler is separately implemented,
does not call the R63ZJ scheduler/validator/classifier, recomputes all six
tangent callbacks, derives all three certificates from replayed solutions and
derives the ladder only from those certificates. Shared small K2 arithmetic
primitives are within the frozen contract. The failure is in completeness and
sealing of the checker, not a numerical refutation.

## Repair decision

Use the contract's only batched repair for all five findings together:
explicit scalar payloads and positivity guards; complete semantic fields and
controls; distinct expected/actual producer, checker and control work seals;
and mismatch-specific fail-safe classification. A remaining load-bearing
finding in the single re-review closes R63ZK as `INCONCLUSIVE`.

No timing, corpus, dynamic building, width-three, runtime/Rust/GPU,
ProductCheck or production authority exists.
