# NSR3-B4E2D7R20R63ZH independent review evidence

Status: `SUPPORTED_BOUNDED / REVIEWED / GO`.

## Verdict

A fresh reviewer with no forked author context inspected the frozen
[review request](nonlocal-nsr3b4e2d7r20r63zh-independent-review-request-2026-08-28.md),
pre-execution contract, executable diff, source blobs and raw outputs before
reading any later author narrative.

```text
snapshot_verified     YES
correspondence_review GO
findings               none
candidate_repaired     no
```

The reviewer made no repository edits and reported neither load-bearing nor
non-load-bearing findings.

## Frozen identity verification

| Item | Verified SHA-256 or identity |
|---|---|
| Parent of author snapshot | `7a09048d0050202c4b329e25cd63eba446d786b4` |
| Author snapshot | `12bd39864931278c5598c3342cbc9cdb1f2bd041` |
| Review-packet commit | `c67b46126ee12c8f954fbf7c425cec157f33d1e1` |
| Executable-only diff | `74fbb15c104369356c0f9c1bb17abbbd204d15a95b8862b17494ec020f62f986` |
| Complete author-commit diff | `afa28aec5d006256770fdc1275cd002e2965ddd611a6ccabf06bb164c7c34068` |
| Parent contract blob | `4e8d08a13f3011cc2a11cb52beb0394c1a7f92f53576381facd9957f59e3d54e` |
| Dev and both author Release outputs | `181ac246c7a0d6e1c24543fb70e684a9357a0c183858c371867dcd068fe96722` |
| Author Release binary | `05f2ff12d44fa8668007808886e09dac8997c2aaafa5ffd584b4f25032fa52f0` |
| Parent cache | `23dbf605ad7b6ae12c4cf6a80404ead9617354ff2848bd010b52c7fa7f83bb84` |
| Boost 1.90 package | `7b89698c907fd5d33ccd439674fa53803139923822181c87b910579827aca379` |
| R63ZG regression | `ca2a0f80b692a466aa97b4725fcc0ac3f553db95bb7ddf72494a754a7e2029c9` |
| R63ZF regression | `be74e412ec6d708128506eeef5cd7284a56a2f150de8e1decb5be12ca394de0e` |
| R63ZE regression | `4fcb94cd94c176e515aedcdb7d0d724e17bac77ec0a94c215ced2dbf4d8ef4a4` |
| Kernel regression | `e4ff7dc634c04338d980d61214de0ca95dd9480ee34f4a139e62feb26cd381d3` |

All five prescribed source-blob hashes also matched. The reviewer verified
that the path-scoped executable diff and the complete commit diff are distinct
by construction rather than mismatched evidence.

## Independent audit

The reviewer independently checked:

- exact binary64 decoding for zero, subnormal and normal finite values,
  including sign, exponent bounds, canonical removal of powers of two and
  rejection of NaN/infinity;
- both 204-component/102-radius solution DTOs, their frozen source roots,
  copied ownership and reconstructed certificate-solution roots;
- `v_i-sum_j M_ij x_j` operand order, row-major indexing and all four
  matrix/solution expansion-lane products;
- absence of candidate centers, Dot2Err and candidate reduction helpers from
  the exact oracle;
- inclusive containment in the immutable R63Y intervals, equality boundaries
  and the rejecting synthetic interval control;
- exact/finite maximum selection at zero-based row `65`, strict-first tie
  behavior, positive interval exclusion of zero and complete maximum-row
  sealing;
- independent residual-difference and displacement-transport paths for all
  102 literal canonical-dyadic equalities, including reverse-sign and
  transpose controls;
- all structural work formulas for dimension 102 and width two, including
  `83,232` center products, `41,616` transport products, `124,848` exact
  multiplications, `157,184` exact additions and `303,862` normalizations;
- exhaustive 256-case classifier precedence, failure publication order,
  stale/resealed mutations, parent-fixture corruption, result sealing and
  zero candidate/operator/solver/certificate updates.

The reviewer independently reconstructed the work root
`40794c9c2c5ee8088bc99304affb354d984e5af0d803c89aa0c2c798c760ab69`
and result semantic
`f1577fd413feb492e7d1ebfd2965e8f895cc170b57187d90fb907fe598a6a5ea`.

## Clean rebuild and focused rerun

Exactly one detached clean build and one focused execution were performed from
the author snapshot with CMake 4.2.3, GNU C++ 15.2.0, Boost 1.90, OpenMP one
thread and strict `-ffp-contract=off -fno-fast-math` flags.

| Fresh artifact | SHA-256 |
|---|---|
| Reviewer Release stdout | `181ac246c7a0d6e1c24543fb70e684a9357a0c183858c371867dcd068fe96722` |
| Reviewer Release binary | `05f2ff12d44fa8668007808886e09dac8997c2aaafa5ffd584b4f25032fa52f0` |

The fresh stdout is byte-identical to the frozen author Release output
(`cmp_exit=0`). Raw reviewer artifacts remain outside Git at
`/tmp/nextengine-r63zh-review.json` and under
`/tmp/nextengine-r63zh-review.0ehYtH/`.

## Bounded result and stop decision

For the frozen 102-row R63Y width-two verifier profile and the frozen
tangent/common final solutions, all 204 exact component-center residuals lie
inside their immutable R63Y enclosures; exact and finite common maximum row
agree at row 65; that interval excludes zero; and all 102 exact identities
`r_c-r_t=-M(x_c-x_t)` close. Together with frozen reviewed R63ZG, this is
`SUPPORTED_BOUNDED / REVIEWED` evidence for
`COMMON_SOLUTION_OUTSIDE_VERIFIER_AFFINE_MODEL`.

The certificate-repair decomposition lineage stops here. The original affine
operator remains the equation being solved. A common operator may be
researched next only as an original-equation-preserving accelerator or
preconditioner whose residual and acceptance remain defined by the original
operator.

The ceiling remains fixed-profile internal correspondence only. This review
grants no timing, runtime-performance, GPU, production, corpus/generalization,
alternate-profile or broader solver/model authority. SPEC-38 and ADR-076
remain `Proposed`; ADR-081 guardrails remain in force. ProductChecks are
`NotRun(report-only mathematical correspondence; no product promotion)`.
