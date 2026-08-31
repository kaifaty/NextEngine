# Physical Sound V14-N1a — Dataset Contract V1 protocol

| Field | Value |
| --- | --- |
| Date | `2026-09-01` |
| Status | `FROZEN_BEFORE_IMPLEMENTATION / SYNTHETIC_ONLY / ZERO_SIGNAL` |
| Roadmap | [V14 N1](../plans/physical-sound-synthesis-roadmap-v14.md) |
| Parent | [V14 rebaseline](physical-sound-v14-data-first-neural-rebaseline-2026-08-31.md) |
| Architecture | [SPEC-45](../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md), `Proposed` |
| Product effect | None; authored clips remain authoritative. |

## Question

Can one current-only experimental contract freeze a multi-object neural dataset
before signal access, distinguish training usability from evaluation
completeness, bind prior exposure, and reject object/recording leakage while
producing byte-identical evidence with zero source, PCM or force reads?

N1a defines and tests the contract only. N1b will inventory real publisher
metadata into candidate rows. N1c will freeze the real Glass/Wood/Metal roles.
N1a does not fetch a source, decode a waveform, select a real object, train a
model, release a validator, cook a clip or add a public/runtime schema.

## Contract boundary

Schema identity:

```text
nextengine.experimental-physical-sound-dataset-contract.v1
```

The contract is external research evidence. Repository code validates and
canonicalizes its current shape, but generated contracts and reports remain
outside Git. `public_contract` and `runtime_consumer_allowed` are always false.

The exact top-level fields are:

```text
schema
contract_id
revision
claim_kind
parent_exposure_ledger
role_policy
axis_policy
sources
objects
access_policy
authority
```

`claim_kind` is exactly `learned_canonical_impact_prior`. It cannot be widened
to measured transfer, true material recovery or arbitrary-force response.

## Parent exposure ledger

The contract binds one complete Exposure Ledger V0 document by:

```text
schema
sha256
role_root_sha256
```

The N1a builder receives the ledger as a separate canonical JSON input, checks
its exact byte hash and role root, and derives observed roles per object parent
from ledger entries. Absence from the ledger is useful only because M1c already
froze the historical store census; the contract never infers freshness from a
material label or filename.

For every dataset object, `prior_exposure` contains the exact M1b-compatible
object-parent hash, the sorted observed ledger roles and one derived state:

- `unexposed` — no matching ledger entry;
- `generator_exposed` — only fit/development-side roles were observed;
- `protected_or_unknown_exposed` — any holdout, validator, shadow,
  `historical_unknown` or fallback role was observed.

Only `unexposed` objects may enter protected roles. `generator_exposed` objects
may remain on the generator side. `protected_or_unknown_exposed` objects are
excluded from the new contract rather than recycled.

## Source and object identity

Each source declares:

```text
source_id
tier
source_namespace
publisher_id
project_id
revision_id
source_revision_group_sha256
provenance_sha256
```

The allowed tiers are the V14 `T0`–`T4` vocabulary. The source-revision group
hash is derived from publisher, project and revision. It is retained for
clustered reporting and later leave-source analysis.

Each object declares:

```text
object_group_sha256
known_alias_object_group_sha256s
physical_object_group_id
source_id
publisher_object_id
material_family
role
quality_mask
axes
recording_parent_sha256s
source_member_root_sha256
prior_exposure
```

`object_group_sha256` is the M1b-compatible hash of source namespace, project
and publisher object ID. `physical_object_group_id` groups the same real object
across publisher mirrors or datasets. Every recording parent also has one
stable SHA-256. Object and recording-parent groups are globally role-disjoint.
`known_alias_object_group_sha256s` contains the current object-parent hash and
every metadata-proven cross-source alias. `prior_exposure` queries and unions
ledger roles across that exact sorted alias set, so an object cannot become
fresh merely by entering through another publisher adapter.

A source archive or publisher revision may contain several independent
objects assigned to different roles. That is not sample leakage by itself;
the revision remains an explicit statistical cluster. Treating a monolithic
archive as one indivisible sample would make ObjectFolder/RealImpact evaluation
impossible without improving independence.

## Roles and minimum shape

Active roles are exactly:

```text
generator_train
generator_development
validator_calibration
validator_method_holdout
admission_shadow
```

`excluded` is a non-counting role for source OOD or prior-exposure conflicts.
The three protected roles are validator calibration, validator method holdout
and admission shadow.

The minimum eligible shape for each of `Glass`, `Wood` and `Metal` is:

```text
4 generator_train
1 generator_development
1 validator_calibration
1 validator_method_holdout
1 admission_shadow
```

The synthetic N1a fixture must meet this exact 24-object minimum. N1b may
inventory more candidates; N1c may freeze only a preregistered deterministic
subset that still meets the minimum.

## Signal-blind quality mask

Quality classes are:

- `training_usable` — structurally sufficient for generator train/development;
- `evaluation_complete` — every required axis is known and may receive
  evaluation credit;
- `source_ood` — excluded, with at least one exact structural failure.

Every quality mask contains a reason code and a sorted non-empty evidence list.
Evidence kinds are limited to publisher metadata, artifact presence, byte
identity, format header, axis binding and exposure ledger. Each row carries an
exact SHA-256, `Pass` or `Fail`, and `signal_values_decoded: 0`. Unknown kinds,
signal-derived evidence or a nonzero counter reject.

`training_usable` and `evaluation_complete` require only passing evidence.
`source_ood` requires a structural failure and role `excluded`.
`training_usable` cannot enter a protected role or receive evaluation credit.

## Axes

Every object carries the complete sorted V1 vocabulary:

```text
canonical_excitation
contact_geometry_binding
contact_position
geometry
geometry_scale
listener_condition
material_identity
object_identity
recorded_response
support_condition
```

Each axis is `known` with one exact evidence SHA-256 or `absent` with null
evidence. Unknown/missing axes fail closed.

`training_usable` requires every axis except `support_condition` to be known.
`evaluation_complete` requires all ten. A protected role additionally requires
`evaluation_complete`, a real-contact tier (`T2` or `T3`) and at least one
recording parent. Every active generator role also requires at least one
recording parent. T0/T1 may only train/develop; T4 is validator-corpus evidence
outside this generator contract and is therefore excluded here.

## Leakage and grouping policy

The validator enforces:

1. one `physical_object_group_id` belongs to one role;
2. one recording-parent SHA-256 belongs to one role;
3. an exact object group appears once;
4. protected roles contain only `unexposed`, `evaluation_complete` T2/T3
   objects;
5. generator-exposed objects never cross to protected roles;
6. source revision, material and role counts are reported separately so a
   large archive cannot masquerade as independent publisher evidence.

Sources are sorted by `source_id`; objects by `object_group_sha256`; axes,
evidence, recording parents and observed roles use canonical sorted order.

## Access and outputs

`access_policy` requires:

```text
network_allowed: false
source_artifact_access_allowed: false
signal_decode_allowed: false
outputs_external: true
```

The builder reads only canonical contract and ledger JSON. It writes atomically:

```text
contract.json
descriptor.json
report.json
```

The report separately counts contract/ledger metadata bytes and scalars. These
build counters must remain zero:

```text
network_requests
source_artifact_bytes_read
pcm_sample_values_decoded
force_sample_values_decoded
protected_signal_values_decoded
```

Two fixture builds must be byte-identical. Outputs inside the repository,
replacement of an existing output, symlinks, noncanonical/duplicate JSON,
unknown fields, malformed hashes and oversized inputs reject.

## Focused gates

Positive coverage must prove the exact 24-object Glass/Wood/Metal role shape,
all protected objects complete/unexposed, source revisions reported as clusters
and zero source/signal access.

Negative fixtures must reject:

- changed ledger hash/role root or malformed ledger identity;
- missing/unknown field, role, tier, material, quality class or axis;
- changed derived source/object group hash;
- omitted/mismatched cross-source alias exposure query;
- duplicate object, physical-object cross-role or recording-parent cross-role;
- protected `training_usable`, missing-axis or non-real-tier object;
- prior generator/unknown exposure promoted into a protected role;
- source OOD used as active data or active data with failed quality evidence;
- signal-derived/nonzero quality evidence;
- unmet per-material role minimum;
- noncanonical, duplicate-key or oversized input;
- output inside the repository.

## Decision and stop rule

- `N1A_DATASET_CONTRACT_V1_FIXTURE_PASS`: every positive/negative guard and
  repeat-exact build passes with all source/signal counters zero. This opens
  only N1b metadata inventory.
- Any schema, grouping, exposure, quality, determinism, accounting or
  confinement failure is `INVALID_N1A_DATASET_CONTRACT`. Repair contract and
  implementation agreement before inspecting real source members.

N1a never authorizes real PCM/force decode, role freeze, training, validator
calibration, clip cooking or runtime/public promotion.
