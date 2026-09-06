# Physical Sound V13-M1b — Exposure Ledger V0 protocol

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Status | `FROZEN_BEFORE_IMPLEMENTATION / SYNTHETIC_ONLY` |
| Roadmap | [V13 M1](../plans/physical-sound-synthesis-roadmap-v13.md) |
| Parent | [M1a result](physical-sound-v13-m1a-research-record-v0-result-2026-08-31.md) |
| Product effect | None; no source, model, content or runtime promotion |

## Question

Can one schema-agnostic, hash-closed catalog turn heterogeneous prior
manifest/report JSON into a deterministic exposure ledger, while proving that
the ledger builder itself reads zero source signal and rejecting a deliberate
fit/validator contact-parent leak?

M1b implements the catalog, resolver, aggregation and synthetic guards. M1c
will freeze and build the real historical catalog over the external
physical-sound store. M1b does not claim that the historical inventory is
already complete.

## Why the catalog is declarative

The current external store contains hundreds of experiment directories and
more than a thousand JSON files from many retired schemas. A generic recursive
search for keys such as `object_id` would silently invent semantics and could
confuse metric arrays, copied provenance and actual sample identity.

Therefore each semantic exposure is declared once in a reviewable catalog and
bound to exact source evidence by JSON Pointer plus SHA-256 of the canonical
pointed value. The builder understands only the catalog schema, not every old
experiment schema. It verifies rather than guesses the mapping.

## Input catalog

Schema identity:

```text
nextengine.experimental-physical-sound-exposure-catalog.v0
```

Exact top-level fields:

```text
schema
revision
store_root_id
partition_policy
artifacts
exposures
network_allowed
source_signal_access_allowed
outputs_external
```

`network_allowed` and `source_signal_access_allowed` are `false`;
`outputs_external` is `true`. Unknown fields, duplicate JSON keys,
noncanonical JSON, booleans used as counters, malformed hashes and inputs over
the frozen limits fail closed.

Each artifact has:

```text
artifact_id
relative_path
sha256
bytes
kind: manifest | report | record | other_json
parse_json
schema_hint
```

Paths are relative, normalized, free of symlinks and confined below the
explicit `--store` root. Every artifact is byte/hash checked. Only artifacts
with `parse_json=true` may supply semantic evidence; other reports may be
hash-accounted without decoding their JSON payload.

## Exposure identity

Each exposure has one exact identity:

```text
source_namespace
project_id
object_id
contact_id | null
listener_id | null
impact_id | null
mutation_parent_id | null
payload_kind
```

The identity is not a material label. IDs are strings even when a publisher
uses integers. `payload_kind` distinguishes `microphone`, `force`,
`derived_response`, `geometry`, `metadata` and `other`, while leakage checks
also use parent groups that deliberately ignore payload kind.

Each exposure also declares:

```text
role
access_kind
values_decoded
protected
evidence[]
```

Roles are the frozen vocabulary:

```text
source_inventory
estimator_fit
generator_development
representation_holdout
validator_calibration
validator_method_holdout
method_holdout
admission_shadow
historical_unknown
fallback_only
```

Access kinds are `metadata_only`, `payload_hashed`, `derived_signal_decoded`
and `signal_decoded`. Only the final two may have `values_decoded > 0`.
`protected=true` is legal only outside fit/development roles and contributes
to the protected historical counter.

Every evidence reference has exact fields:

```text
binds
artifact_id
json_pointer
value_sha256
```

`binds` is one identity field or `role`, `access_kind`, `values_decoded`.
At minimum `project_id`, `object_id` and every non-null optional identity field
must be evidenced. The builder implements RFC 6901 array/object traversal,
canonicalizes the resolved JSON value and compares its exact SHA-256. An
unresolved pointer, changed value, duplicate binding or reference to an
unparsed artifact fails.

The catalog mapping from publisher-specific role paths to the normalized role
is itself reviewed and hash-closed. A pointer such as
`/roles/estimator_fit/0` may bind the `contact_id`; the catalog supplies the
normalized role.

## Partition and leakage policy

`partition_policy` is exactly one of:

- `historical_union`: preserve all observed roles and conservatively mark the
  identity exposed; historical overlaps are reported but do not make the
  inventory unrepresentable;
- `disjoint_evaluation`: fail when one contact parent or mutation parent
  appears in more than one role partition.

The partition sides are:

```text
fit_side       = source_inventory, estimator_fit, generator_development
holdout_side   = representation_holdout
validator_side = validator_calibration, validator_method_holdout,
                 method_holdout, admission_shadow
fallback_side  = fallback_only
unknown_side   = historical_unknown
```

For `disjoint_evaluation`, a contact parent cannot cross any two sides. A
non-null mutation parent cannot cross sides even when contact, listener or
payload IDs differ. `unknown_side` is never admissible in a disjoint catalog.
This is stricter than exact-row deduplication and prevents force/microphone or
augmented variants of the same event from leaking across fit and validator.

## Outputs

The builder writes an external atomic directory containing:

```text
catalog.json     exact canonical input copy
ledger.json      aggregated identities, roles, access and group hashes
summary.json     M1a-compatible hash summary
report.json      build access/accounting and decision
```

Ledger schema:

```text
nextengine.experimental-physical-sound-exposure-ledger.v0
```

`summary.json` has exactly the M1a fields:

```text
schema
sha256
role_root_sha256
sample_identity_count
signal_values_decoded
protected_signal_values_decoded
```

`sha256` is the canonical `ledger.json` hash. `role_root_sha256` hashes the
sorted list of identity hashes and observed normalized roles. Object, contact,
listener and mutation-parent group hashes are stored in the full ledger for
selection and leakage checks.

The report separates:

- build-time `artifact_bytes_hashed`, `metadata_files_parsed` and
  `metadata_scalar_values_parsed`;
- build-time `network_requests`, `source_bytes_read`,
  `signal_values_decoded` and `protected_signal_values_decoded`, all zero;
- historical exposure totals declared and evidenced by the catalog.

Hashing/parsing manifest or report JSON is metadata access, not source-signal
decode. Historical counters never masquerade as bytes read by the M1b build.

## Synthetic fixture and gates

The positive fixture contains multiple payloads for distinct contacts across
fit, holdout and validator roles. Two builds must be byte-identical and must
emit zero build-time source/signal counters.

Focused negative fixtures must reject:

- changed artifact hash/size or path escape/symlink;
- missing, changed, duplicate or unresolved evidence pointer;
- unevidenced identity component;
- unknown role/access/partition/schema/field;
- duplicate exact exposure;
- nonzero values under metadata/hash-only access;
- protected fit/development exposure;
- one contact parent crossing fit and validator sides;
- one mutation parent crossing roles through different contact/payload IDs;
- output inside the repository;
- noncanonical or oversized catalog JSON.

## Decision and stop rule

- `M1B_EXPOSURE_LEDGER_V0_FIXTURE_PASS`: every positive/negative guard and
  repeat-exact synthetic build passes. This opens only M1c real catalog freeze.
- Any resolver, lineage, leakage, accounting, determinism or confinement
  failure is `INVALID_M1B_EXPOSURE_LEDGER`; repair implementation/protocol
  agreement before touching the historical catalog.

No M1b result selects a fresh source, authorizes waveform decode, declares an
object clean, validates a formula or changes authored-clip/runtime authority.
