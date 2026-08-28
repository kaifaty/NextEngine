# NSR3-B4E2D7R20R63ZK independent review request

Status: `AUTHOR_SNAPSHOT_FROZEN / INDEPENDENT_REVIEW_NEXT`.

Review the frozen contract and implementation diff before reading later author
synthesis. Treat R63ZJ as an untrusted producer and preserve its
`INCONCLUSIVE` verdict. Do not edit or repair the candidate. Return findings by
severity and a final `GO` or `NO-GO`.

## Frozen identities

| Item | Identity |
|---|---|
| Contract snapshot / implementation parent | `388f7462d53808a644b798f59dca52ae0d938eb6` |
| Implementation snapshot | `33e24152cd0634f01df25b27f7e3fd18ebb67c56` |
| Complete diff SHA-256 | `e6872a8e519f88535f4db536f4c5c976c7ff507643b15e3cec1f7a5d9f710688` |
| Contract blob | `fa9d229e689c6742ffef16da01f5738732b2aace7992e64bff58b5309242daa7` |
| Checker main | `20746b127dd6f5c70f58803d366ac62b99d4a808cdec2e4c00d26421c89a7ebd` |
| Private API | `53ba5d87e0a62ca2e13d90363ed26d955dfe446080d1e6ca276e8083a44815c3` |
| Boundary primitives | `f43c68c27ed846e85a8d088283653444929f1b4780a9785d3180398c2a6acd10` |
| CMake | `397deaf39a47d3e1a2cdc1a265f741caa0bb092fd682953730ef5138fa3c6dd3` |
| Parent cache | `23dbf605ad7b6ae12c4cf6a80404ead9617354ff2848bd010b52c7fa7f83bb84` |
| Expected stdout | `371290103aee0ec18dbb40251785b44866fe52c4e167405a4418886713f29041` |
| Author Release binary | `49486450b598516263b37003059e892f583027e99271c901fb2c4aa243588175` |

## Required adversarial audit

1. Confirm the expected replay calls the R63ZJ producer only once to capture an
   untrusted DTO and never calls its validator or classifier while building
   expected values.
2. Trace projected RHS/scale, all three factor solves, the exact
   `Kx0/Kp0/Kp1` inputs, six high/low tangent calls, four dots, three divides
   and all updates. Confirm the scheduler is independent of the author
   transaction scheduler and preserves the exact K2 order.
3. Verify the generic K2 wrappers expose arithmetic primitives only. Look for
   a hidden call back into the author transaction, producer certificate fields
   or route.
4. Recompute all comparison counts and ensure every state vector component,
   product input/output component, scalar identity, certificate semantic field
   and candidate-work field is compared bit-exactly. Opaque producer roots
   must not substitute for semantics.
5. Verify all three certificates are derived from independently replayed
   solution components and the immutable verifier profile. Re-derive the
   reject/reject/pass ladder and state-2 sign root.
6. Audit producer versus checker work separation, including certificate work,
   semantic comparisons, controls and classifier cases. Flag unowned replay or
   control work that affects the route.
7. Reconstruct all 18 controls. In particular confirm state, certificate and
   recurrence-work mutations remain accepted by the old author validator after
   resealing but are rejected by the independent comparisons for the intended
   reason.
8. Check fail-safe malformed input, callback identity/aggregate, classifier
   precedence, result sealing and authority fields.
9. Confirm the new generic API does not change legacy outputs; reproduce
   R63ZI, R63ZG, R63ZH and R63ZJ hashes.
10. State the exact fixed-profile claim ceiling. A `GO` permits only the next
    portable product-representation discriminator; it grants no timing,
    corpus, runtime, GPU or production authority.

## Optional clean run

Use one detached Release build and at most two focused executions from exact
snapshot `33e24152`. Expected stdout is `37129010...9041`; record any
path-dependent binary hash separately. Use the frozen cache above and
`OMP_NUM_THREADS=1`.

ProductChecks remain `NOT_RUN` because this package is a localized offline C++
checker under `Proposed` architecture. Any load-bearing finding returns
`NO-GO`; at most one batched repair re-review is allowed.

## Required response

Return snapshot/diff/blob verification, findings with exact `file:line`
evidence, clean build/run identities if executed, final `GO` or `NO-GO`, and
the exact permitted claim and production firewall. Do not read later evidence,
task-state, plan or roadmap before completing the frozen audit.
