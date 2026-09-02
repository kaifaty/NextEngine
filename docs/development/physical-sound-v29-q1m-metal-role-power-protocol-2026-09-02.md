# Physical Sound V29 Q1-M — Metal exposure, power and one-use role protocol

| Field | Value |
| --- | --- |
| Date | `2026-09-02` |
| Status | `FROZEN_BEFORE_IMPLEMENTATION / METADATA_ONLY / ZERO_SIGNAL / ATOMIC_ROLE_FREEZE_OR_SOURCE_POWER_OOD` |
| Roadmap | [V29 Q1-M](../plans/physical-sound-synthesis-roadmap-v29.md) |
| Predecessor | [Q0-M source inventory](physical-sound-v29-q0m-metal-source-inventory-result-2026-09-02.md) |
| Architecture | [SPEC-45](../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md), `Proposed` |
| Product effect | None; authored clips remain authoritative and no validator, generator, payload access, content contract or runtime ML is authorized. |

## Question and bounded claim

Q1-M asks whether the Q0-M identities can populate a fresh exact-Steel release
without sharing a publisher/project revision, physical object or protected
payload across validator, generator and joint-admission roles.

The owner may freeze the exact target policy, join every available identity to
historical exposure evidence, calculate raw and freshness-qualified cluster
power, and either publish one complete atomic role assignment or a terminal
source-power OOD certificate. It cannot read signal values, train a validator
or generator, choose a threshold, move only the convenient objects from one
project, or partially assign roles after a failed gate.

## Frozen inputs

Only these five immutable metadata files may be read:

| Input | Bytes | SHA-256 | Purpose |
| --- | ---: | --- | --- |
| Q0-M inventory | `172,172` | `9a04c8a2fb9f0c6a3d351dac2796936d933d223e9400afc79a95f06e01665f43` | Exact candidate, source, project and historical-exclusion identities |
| Q0-M report | `2,018` | `a33c0e2eae12663d8c8dc90cca926b1d614bec332a07e4dddea4d75bf403b9d5` | Q0 decision, minima, counts and zero-signal access closure |
| V15 S0a exposure census | `2,318,665` | `ce3c5ab861a91bd85fc9a5a90f24629f15dbdc8d9faa936cd04e791efa667365` | Historical experiment exposure state for revision-aware ObjectFolder identities |
| V15 S0a identity map | `266,858` | `66e9a29e69b72b95836beef9dbd8d7d52cd33b1213c709fc2f57422139d1ea40` | Exact ObjectFolder current-row to physical-group join |
| V15 S0b YCB capability inventory | `227,019` | `7f1b316e5109122cb9efd1834ab7a6a95cbd9d098d0f8d89971be98e76d158cf` | Exact YCB parent identities and prior exposure state |

Q1-M revalidates the Q0 inventory/report relationship and requires every Q0
signal-bearing access counter to remain zero. The historical census predates
Q0-M and was not a universal freshness certificate: a row absent from it is
`unassessed`, never silently fresh. YCB states
`repository_adapter_referenced` and
`not_in_adapter_freshness_unassessed` remain exposed and unassessed
respectively. Neither state is role eligible.

## Exact target policy

This release targets only publisher-labelled `Steel`:

```text
target_policy = exact_primary_material_steel_v1
```

- `exact_steel_candidate` is the only positive class;
- `Iron` and `Aluminium` remain `other_metal_candidate` and receive no Steel
  positive credit;
- `non_metal_candidate` is the only reject-parent class;
- every historical Glass assignment remains excluded with its old role intact;
- no name, shape, hardness or broad `Metal` label may upgrade a row.

This policy is frozen before signal access. A future broad-Metal release is a
new domain with new roles and cannot reuse an opened exact-Steel evaluation.

## Independence unit and role topology

The indivisible source unit is the complete publisher/project/revision group.
All objects and future recordings from that group stay in one role. Object-
level splitting inside one project does not create independent project
evidence and is forbidden even when the object IDs differ.

The frozen roles and their topology-only minimum project counts are:

| Role | Minimum distinct projects | Protected evaluation minimum |
| --- | ---: | --- |
| `validator-development` | `1` | none; threshold/feature development only |
| `validator-calibration` | `1` | none; threshold selection only |
| `validator-holdout` | `2` | `16` exact-Steel positives and `35` reject parents |
| `generator-training` | `1` | generator-only disclosed data |
| `generator-development` | `1` | generator-only disclosed development |
| `generator-method-holdout` | `1` | one-use generator evaluation |
| `joint-admission-shadow` | `2` | `16` exact-Steel positives and `35` reject parents |

Because roles are project-disjoint, the topology floor is nine distinct
publisher/project revisions. The two protected evaluations alone require four
distinct projects. A project containing only one required class cannot satisfy
the missing class through a different project's object split; each protected
evaluation must be multi-project and its exact class counts are reported
separately.

The group minima apply independently to `validator-holdout` and
`joint-admission-shadow`. Therefore the protected-only raw floor is `32`
exact-Steel positives and `70` non-Metal reject parents. These counts do not
reserve any evidence for development, calibration or generator roles and are
therefore necessary, not sufficient.

## Exposure and freshness join

Every Q0 group available after historical exclusion receives exactly one
exposure state:

- ObjectFolder joins by exact current source ID, publisher object ID, verbatim
  publisher name/material and immutable table revision through the S0a
  identity map, then by exact physical-group ID to the census;
- YCB joins by exact workbook object ID and exact recording-parent identity to
  the S0b capability row;
- an absent, ambiguous, duplicated, mismatched or historically exposed join is
  ineligible and recorded explicitly;
- a historical `unexposed` result is still
  `historical_unexposed_not_currently_certified` because the ledger predates
  Q0-M; Q1-M does not manufacture a current freshness claim from it.

The output must account for all available Q0 identities. This join can prove
historical exposure or missing freshness evidence; it cannot open a protected
payload or infer acoustic suitability.

## Ordered gates and atomic decision

The owner evaluates these metadata-only gates in order and reports all
independent failures:

1. exact input/schema/hash and Q0 zero-signal closure;
2. exact-Steel target policy and immutable Glass exclusion;
3. complete exposure-state accounting for every available identity;
4. at least nine project revisions for the frozen role topology;
5. at least four project revisions across the two protected evaluations;
6. at least `32` exact-Steel and `70` non-Metal groups before any unprotected
   role is populated;
7. freshness-qualified project/class power sufficient for every role;
8. a deterministic whole-project assignment with no source, project, object or
   historical-role overlap.

Only a complete assignment may emit:

```text
Q1M_ROLE_POWER_FREEZE_VERIFIED_Q2_AND_P2_ELIGIBLE
```

Any failed gate emits:

```text
Q1M_SOURCE_POWER_INSUFFICIENT_SOURCE_GROWTH_REQUIRED
```

The negative result is a successful Q1-M terminal certificate. It emits no
role assignment, keeps every protected payload sealed, leaves Q2/P2 blocked
and routes the roadmap to a fresh internet-source growth package. Counts,
confidence targets and project independence are never weakened.

## Outputs and access ledger

The owner atomically writes to a fresh external directory:

```text
metal-role-power-audit.json
report.json
```

Both outputs include exact input identities, target policy, role requirements,
raw/freshness-qualified class power, project topology, per-group exposure
accounting, blockers and an access ledger. A negative result assigns every
group `unassigned_source_power_ood` and contains an empty role assignment.

Every run reports exactly zero for:

```text
network_requests
source_payload_bytes_read
archive_member_bodies_read
audio_headers_parsed
mesh_values_decoded
pcm_sample_values_decoded
force_sample_values_decoded
protected_signal_values_decoded
role_signal_values_opened
```

Only the five frozen files count as `metadata_bytes_read`. Repository output,
symlinks, non-regular files, oversized inputs, hash/schema drift, duplicate
JSON keys, ambiguous joins and replacement of an existing output fail closed.

## Required focused verification

Tests must cover at least:

1. repeat-exact structural rejection for two projects with zero signal access;
2. the nine-project topology floor and four-project protected floor;
3. independent `32` Steel / `70` reject protected-only minima;
4. exact Steel remaining separate from Iron and Aluminium;
5. whole-project atomic assignment or no assignment, never an object split;
6. complete exposure accounting, ambiguous ObjectFolder joins and mismatched
   YCB parents;
7. historical `unexposed` never becoming current freshness automatically;
8. Q0 hash/access drift, duplicate group, symlink input and output replacement
   rejection.

Official execution builds twice into separate external directories and
compares them recursively. No dataset, waveform, feature matrix, model,
checkpoint or generated audio enters Git.

## Next boundary

If Q1-M verifies a complete role freeze, Q2 may implement Validator V1 and P2
remains additionally blocked by P1 known truth. If Q1-M reports source-power
OOD, the next Q-lane work is `Q1a-M`: discover and hash-close enough additional
independent internet projects to satisfy the exact same topology, class and
freshness gates before Q1-M is attempted again as a fresh release.
