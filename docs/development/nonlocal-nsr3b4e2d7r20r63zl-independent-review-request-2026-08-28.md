# NSR3-B4E2D7R20R63ZL independent review request

Status: `IMPLEMENTATION_SNAPSHOT_FROZEN / INITIAL_REVIEW_NEXT`.

Review the revision-2 contract and implementation diff before reading author
evidence, task-state, README, roadmap or later synthesis. R63ZJ and R63ZK both
remain `INCONCLUSIVE`. Do not edit or repair the candidate. This is the initial
R63ZL review; return all findings in one batch.

## Frozen identities

| Item | Identity |
|---|---|
| Contract / implementation parent | `de277c8ddff90ddbe6cc969d9d28600d93728a2d` |
| Implementation snapshot | `f6475040616d0f4f55f933ffd4b7c12ec0068c2f` |
| Complete diff SHA-256 | `127248d10141ee4dd8ac14fa9cfb589290271ef42dafc889710c4bf5792b61aa` |
| Contract blob | `3eb6e131aab7c70d8d69c857383bab14b41dc14fc80bfffed0c49dd3fcdc248d` |
| Boundary main | `64501c47ea2e5451a20c1a7ca88e12597ccf98361665a7c646eb6e7c1585f995` |
| Private API | `7f9240bfc9927f580a500f1ba7213d8e17b6085ab963d0a01c0d6bfcf06c0016` |
| Boundary implementation | `da25cd6463b1296a149ddb5bc00d81839a3f3a232c6b59887666e8d6553858bd` |
| CMake | `a950a7b8a196d19821d876ce2ea656949adde5e2d5fec6f0d8ce68a3381bc3bd` |
| Parent cache | `23dbf605ad7b6ae12c4cf6a80404ead9617354ff2848bd010b52c7fa7f83bb84` |
| Expected Dev/Release stdout | `329ee21f1e3ad6f9066c773da7359caf4677f7b65a3cd3397f3dea10e5b1f28c` |
| Author Release binary | `6e3425b0681c9daab9780b45d00f60dfb88ff41427be83057de53122ab4b4d7e` |

## Required audit

1. Verify the exact parent, snapshot, complete diff, contract and source blobs.
   Read the revision-2 contract from the snapshot before source inspection.
2. Audit the type boundary: private constructor, no default/move/assignment,
   no aggregate/raw-fixture construction, private immutable payload ownership,
   no public mutable payload pointer/reference and no invalid moved-from path.
3. Trace the ten admission predicates in order. Recompute the one payload hash,
   two frozen-scalar parses, one `32130`-component copy, context/work/result
   roots, and first-failure receipts for all four admission controls.
4. Confirm the admitted product cannot call the legacy validating wrapper or
   accept a raw fixture. Compare its dot arithmetic operation-for-operation
   with frozen dot2 arithmetic and verify that it deliberately omits only
   unconsumed dot guards/witness roots, not numerical or exactness semantics.
5. Recompute all six admitted and six legacy products component-by-component.
   Verify candidate totals (`6` kernels, `30` final roots, zero repeated
   payload/dot paths) and legacy totals (`6` validations, `60` predicates, `6`
   payload hashes, `12` parses, `36` kernel guards, `5004` dot guards, `7506`
   dot witness roots and `30` final roots).
6. Reconstruct admission, per-product, candidate, reference, checker, control
   and result seals from semantic fields. Confirm copied public work values
   cannot be fed back as authority and semantic comparisons precede roots.
7. Reproduce all seven controls and all seven classifier cases, including
   malformed input before kernel execution, immutable API traits, digest drift
   and unknown fail-safe behavior. Look specifically for an unowned nested
   validation, root/hash, copy, parse or legacy-reference path.
8. Verify authority fields and the stop firewall: no recurrence
   correspondence, representation, timing, runtime/GPU or production claim.
9. Reproduce R63ZI, R63ZG, R63ZH, R63ZJ and R63ZK stdout hashes exactly.

## Optional clean run

Use one detached clean Dev/Release build and at most two focused executions
from snapshot `f6475040`. Expected stdout is `329ee21f...f28c`; record any
path-dependent binary hash separately. Use the frozen cache and
`OMP_NUM_THREADS=1`. The local header-only Boost include is needed only for
the R63ZH regression target, not R63ZL.

ProductChecks remain `NOT_RUN` because this is a localized offline C++
research boundary under Proposed architecture. A clean build or `GO` does not
grant physics correspondence or production authority.

## Required response

Return identity verification, all findings with exact `file:line` evidence,
clean build/run identities if executed, a final `GO` or `NO-GO`, and the exact
fixed-profile claim ceiling. Do not read the author evidence, task-state,
README, roadmap or later author synthesis before completing the frozen audit.
