# Physical sound PS-2 — explicit reject-parent import

Date: 2026-08-28

Status: `EXPLICIT_ROLES_VALIDATED / DEV_REJECT_PARENTS_8_OF_35 / GLASS_7_OF_16 / PASS_DISABLED / NO_CORPUS_ADMISSION_AUTHORITY`

## Outcome

`physical-sound-registry identified-corpus` now supports an explicit
`corpus_role` on every source assignment:

- `target` declares an in-domain object source;
- `reject_parent` declares an out-of-domain parent that may later supply
  expected-`Reject` validation cases;
- omitted roles retain the historical implicit report behavior only when every
  assignment omits the role.

Once any role is present, every source must be assigned and the manifest must
contain both roles. The normalizer cross-checks each role against the exact
adapter-provided material identity. A non-Glass source cannot be counted as a
Glass target, and a Glass source cannot be counted as a reject parent.

The complete cached E3 corpus now measures:

| Measurement | Result | Required |
| --- | ---: | ---: |
| Glass target object groups | 7 | 16 |
| Glass target recordings | 28 | — |
| Explicit reject-parent object groups | 8 | 35 |
| Reject-parent recordings | 16 | — |
| Missing target groups | 9 | — |
| Missing reject-parent groups | 27 | — |

All 15 objects and 44 recordings remain in `dev`. Calibration, holdout and
shadow remain empty, so this result supplies parent identities only. It does
not claim that controlled mutations have been generated, that the validator
rejected them, or that a false-pass bound has been measured.

## Imported roles

The seven existing Glass groups are explicitly `target`. Eight independently
identified non-target AV-MSF objects are explicitly `reject_parent`:

| Material | Object groups | Recordings |
| --- | ---: | ---: |
| Ceramic | 3 | 6 |
| Wood | 2 | 4 |
| Iron | 1 | 2 |
| Plastic | 1 | 2 |
| Polycarbonate | 1 | 2 |

Counting is by canonical publisher/project/revision/object identity, never by
recording count or material label alone. Sources sharing an AV-MSF project
revision remain one source group and one partition; the explicit roles do not
weaken the existing leakage check.

## Fail-closed behavior

The executable boundary rejects before report publication when:

- only part of a manifest has explicit roles;
- either explicit role is absent;
- a `target` source's adapter material differs from the manifest target;
- a `reject_parent` source has the target material;
- an assignment is missing, duplicated, unsorted or references an unaudited
  source;
- the underlying source is not exact adapter-backed E3 evidence.

Historical manifests with no roles still serialize the same report bytes. The
new role and partition fields are emitted only for explicit-mode reports.

## Exact external evidence

All manifests, caches and generated reports remain outside Git under:

`/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-explicit-reject-parents-v1/`

| Evidence | SHA-256 |
| --- | --- |
| Explicit-role identified manifest | `a6a7f3cc403744a049a6ee7f380054e8a46e65e449fa85aff94077a75cba7159` |
| Explicit-role reports A/B/offline | `2ac5c3128a1950d4dfb36324baf7e9fe0336e1f9b2884ed1255c233c8f3b9e48` |
| Historical implicit report after the change | `fca10741118661e8661eba5e0233824b68d2095fcf1ddb57a3e7942d49561cf2` |
| Partial-role negative manifest | `d64218c6bd60b6ad7f8415d30b709fe0aed3becabb48116d7a6a60eecc71b7b3` |
| Partial-role rejection result | `4d63c7a3d958e0790ff0d266532384a1ab7a73f083b799252ee353d9284e23bc` |
| Material-role negative manifest | `40a3eb1dfb2bcf39df3da5be1daf3d3c036d3cbb080d102c3f2bc9266eacdf7c` |
| Material-role rejection result | `a91a1e2c4d0864f7483eaeb4b315bce438d1a04c94c84a65c030d85334f5ae82` |

The three explicit-role reports are byte-identical across two imported cache
roots and one offline repeat. The prior implicit report retains its frozen
hash, proving that optional role serialization did not rewrite earlier
evidence.

## Decision and next action

Keep automatic `Pass`, PS-3 and AV-P0D disabled. The next PS-2 package should
prefer a different published project that contributes stable object-level
Glass targets and explicitly identified hard non-Glass parents. It must not
open calibration, holdout or shadow by moving the current small corpus between
partitions.

After at least 35 diverse reject parents and 16 in-domain groups exist, freeze
the group-level partition plan, generate only predeclared expected-`Reject`
controls, and run one untouched validator release. Insufficient coverage keeps
the domain on clip fallback and never creates a human approval queue.
