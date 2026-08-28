# NSR3-B4E2D7R20R63ZK independent re-review request

Status: `REPAIR_SNAPSHOT_FROZEN / SINGLE_RE_REVIEW_NEXT`.

Review the revision-2 contract and repaired implementation diff before reading
later author evidence or synthesis. Preserve R63ZJ as `INCONCLUSIVE`. Do not
edit or repair the candidate. This is the experiment's only re-review: any
remaining load-bearing finding closes R63ZK as `INCONCLUSIVE`.

## Frozen identities

| Item | Identity |
|---|---|
| Revision-2 contract / implementation parent | `3097bc08a31a2a4c017cb696a6990a3138ff056a` |
| Repaired implementation snapshot | `7635d664fe35bbfde725956307e62f6a598009ab` |
| Complete diff SHA-256 | `2e652018528d5d03f8921f970070254587c181fbc3a9f85942ef504c3f3fac63` |
| Contract blob | `0a231bb60646b4c4d655bcba2747251b9cd8a68edd38f3cb85af5ec288cc40ea` |
| Checker main | `02bec9996c754fc7c12ac3b86617ff3350049ccbf57c452018023d6a1e018063` |
| Private API | `13b0c894805d4a57968afbfd0f146c686c475d72b7ff0d78d7838c18f1ce4f77` |
| Boundary primitives | `621300de50157b1e3da1adfc114618990460f16217678aa99787d1c2caa0a899` |
| CMake | `397deaf39a47d3e1a2cdc1a265f741caa0bb092fd682953730ef5138fa3c6dd3` |
| Parent cache | `23dbf605ad7b6ae12c4cf6a80404ead9617354ff2848bd010b52c7fa7f83bb84` |
| Expected Dev/Release stdout | `73df9f1b7f7beeb97e16e90b9dd498c2c0ee33fe64eeb38425e0c5a11cb92755` |
| Author Release binary | `4ccee9be8f1cb34d98dfd14010b9d2dc31982af22122008306d057af42d1d35c` |

## Required repair audit

1. Verify the exact parent, snapshot, complete diff, contract and source blobs.
   Read the revision-2 contract from the parent before the implementation.
2. Confirm exactly one untrusted producer capture and no call to the R63ZJ
   scheduler, validator or classifier while expected products, states,
   certificates, work or routes are derived.
3. Trace projected RHS/scale, three solves, `Kx0/Kp0/Kp1`, six high/low tangent
   calls, four dots, three divisions and every update. Check the four strict
   positivity gates in the original transaction order and impossible-guard
   fail-safe behavior.
4. Verify the DTO exports every applicable K2 scalar as explicit `high/low`,
   and that the checker compares both binary64 components plus guards and
   roots. Opaque root equality alone must never establish scalar semantics.
5. Recompute every semantic comparison: product guards/metadata/components/
   identities, callback aggregate, recurrence guards, all four state vector
   families, scalars, all nine certificate fields, state/recurrence/hybrid
   roots and both work ledgers. Confirm certificate-set identity is classified
   with certificate semantics rather than hiding a first-specific mismatch.
6. Reconstruct the expected and actual producer receipts. They must publish
   equal complete payload semantics but distinct role seals. Confirm the
   producer cannot receive credit for checker or control work.
7. Recompute the checker receipt, including targeted fixture/factor/profile
   validation, positivity, numerical work, all comparison categories, root
   derivation paths and result seal. Check all declared constants against loop
   counts and the executed dataflow.
8. Reconstruct all `64` controls and the separate control receipt. Confirm all
   nine certificate-field controls are fully resealed, author-valid and first
   rejected as certificate mismatches; scalar high/low/guard and hybrid
   certificate-work drift remain author-valid; state and recurrence-work
   counterexamples remain author-valid. Recompute the `19` old-validator
   replays and their `114` tangent kernels plus lower work.
9. Verify all `61` control trace comparisons are accounted by category, every
   outcome is sealed, result-seal mutations reject, and classifier precedence
   exhausts `12 * 8 = 96` cases including an unknown mismatch.
10. Check malformed-cache failure is structured and fail closed, and verify
    callback/identity/result authority fields cannot grant timing, runtime,
    GPU or production authority.
11. Reproduce R63ZI, R63ZG, reviewed R63ZH and closed R63ZJ hashes exactly.

## Optional clean run

Use one detached clean Release build and at most two focused executions from
snapshot `7635d664`. Expected stdout is `73df9f1b...2755`; record a
path-dependent binary hash separately. Use the frozen cache above and
`OMP_NUM_THREADS=1`.

ProductChecks remain `NOT_RUN` because this is a localized offline C++ checker
under Proposed architecture. Do not infer production authority from a clean
build or a `GO`.

## Required response

Return identity verification, findings with exact `file:line` evidence, clean
build/run identities if executed, a final `GO` or `NO-GO`, and the exact
fixed-profile claim ceiling. Do not read the repair evidence, task-state,
README, roadmap or later author synthesis before completing the frozen audit.
