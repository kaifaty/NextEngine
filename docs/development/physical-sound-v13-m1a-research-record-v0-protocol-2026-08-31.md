# Physical Sound V13-M1a — Research Record V0 protocol

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Status | `FROZEN_BEFORE_IMPLEMENTATION / SYNTHETIC_ONLY` |
| Roadmap | [V13 M1](../plans/physical-sound-synthesis-roadmap-v13.md) |
| Architecture | [SPEC-45](../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md), `Proposed` |
| Product effect | None; authored clips remain authoritative |

## Question

Can one current-only experimental record preserve exact source/claim/evidence
state, reject claim widening and lifecycle skips, and reproduce byte-for-byte
without reading any new signal?

M1a does not build the prior-exposure ledger, select a fresh object, fit a
formula, release a validator, cook audio or add a public/runtime schema. M1b
will build the ledger referenced by this record. M1c will supply repeated
zero-signal evidence over existing manifests.

## Record boundary

The record is external research evidence with schema identity:

```text
nextengine.experimental-physical-sound-research-record.v0
```

Repository code defines and validates the current shape. Generated records,
reports, arrays and audio remain outside Git. The record is never:

- a `crates/contracts` type;
- a `ContentManifestV1` or neutral-content entry;
- a save/replay owner;
- a runtime model or admission authority;
- evidence that SPEC-45 is shipped.

Every record has the exact top-level fields below and rejects unknown fields:

```text
schema
record_id
revision
previous_record_sha256
claim_kind
lifecycle
source
axes
exposure_ledger
evidence
validator
fallback
public_contract
runtime_consumer_allowed
```

`public_contract` and `runtime_consumer_allowed` are always `false`.
`fallback.required` is always `true` and names one exact authored-clip
identity. A successful formula or atlas does not remove that fallback.

## Claim kinds and axes

Exactly two claim kinds exist:

| Claim | Meaning | Required known axes |
| --- | --- | --- |
| `canonical_impact_field` | Canonical normalized impact plus surface contact predicts sound at one declared listener condition. | object identity, recorded response, geometry, contact position, listener condition, canonical excitation |
| `measured_transfer_field` | An arbitrary measured input waveform may be applied through the declared transfer model. | common axes plus raw force, common timebase and force-frequency coverage |

Every record carries every axis from the frozen axis vocabulary, sorted by
axis ID. Each is `known`, `absent` or `not_applicable`. `known` requires one
exact evidence SHA-256; the other states require `null`. Missing axes and
unknown axes fail closed. Material labels, filenames or model outputs never
fill an absent axis.

The complete vocabulary is:

```text
canonical_excitation
common_timebase
contact_position
force_calibration
force_frequency_coverage
geometry
listener_condition
microphone_calibration
object_identity
raw_force
recorded_response
support_condition
```

The stricter measured-transfer claim cannot become `SourceQualified` without
raw force, common timebase and force-frequency coverage. Thus V12 object `41`
cannot be relabelled as measured-transfer capable after its failed certificate.

## Exposure-ledger reference

M1a stores only a hash-closed ledger summary:

```text
schema
sha256
role_root_sha256
sample_identity_count
signal_values_decoded
protected_signal_values_decoded
```

The exact ledger rows and leakage rules belong to M1b. M1a requires finite
non-negative integer counters and a known experimental ledger schema. An
initial `SourceQualified` record requires both signal counters to be zero.

## Lifecycle

Progressive states are `SourceQualified`, `FormulaValidated` and
`AtlasAdmitted`. `FallbackOnly` and `FallbackOutOfDomain` are terminal
outcomes. A terminal record is immutable; a materially new method uses a new
record family/revision rather than reopening it in place.

Allowed transitions are exactly:

| Previous | Current | Transition |
| --- | --- | --- |
| none | `SourceQualified` | `qualify_source` |
| `SourceQualified` | `FormulaValidated` | `formula_validated` |
| `SourceQualified` | `FallbackOnly` | `formula_tournament_rejected` |
| `SourceQualified` | `FallbackOutOfDomain` | `source_out_of_domain` |
| `FormulaValidated` | `AtlasAdmitted` | `atlas_admitted` |
| `FormulaValidated` | `FallbackOnly` | `validator_rejected` |
| `FormulaValidated` | `FallbackOutOfDomain` | `validator_out_of_domain` |

The initial record is revision `0`. Every successor increments by one, binds
the exact canonical SHA-256 of the previous record and preserves record ID,
claim kind, source identity, axes and fallback identity. Direct
`SourceQualified -> AtlasAdmitted`, same-state rewrite, claim widening,
terminal reopening and revision/hash/source drift fail closed.

State-dependent evidence:

- `SourceQualified`: no formula, held metric, validator or cooker artifact;
- `FormulaValidated`: formula and held-metric hashes required; validator is
  `NotRun` and cooker is absent;
- `AtlasAdmitted`: formula, held metrics, independent validator release/votes
  and cooker manifest required; validator decision is `Pass`;
- `FallbackOnly`: no cooker; validator may be `NotRun` for tournament reject
  or independent `Reject` after a formula;
- `FallbackOutOfDomain`: no cooker; validator may be `NotRun` for source OOD
  or independent `FallbackOutOfDomain` after a formula.

Learned votes can never independently create `AtlasAdmitted`; the record only
binds the complete future independent validator release.

## Canonicalization and current-only compatibility

Canonical bytes are UTF-8 JSON with sorted keys, two-space indentation, one
final newline and no NaN/infinity. Duplicate JSON keys, noncanonical encoding,
oversized input, booleans in integer fields, malformed hashes and unknown
fields reject.

ADR-046 forbids inventing migration obligations for an unconsumed alpha
format. Therefore V0 supports only:

```text
decode current V0 -> validate -> emit identical canonical current V0
```

An unknown/older/newer schema is rejected without rewriting the source. A
future actual successor may define a copy-on-write migration after a concrete
consumer exists; M1a does not fabricate one.

## Synthetic fixture and gates

The implementation must generate, in an external empty output directory, one
synthetic branch set:

1. `SourceQualified`;
2. `FormulaValidated` from it;
3. `AtlasAdmitted` from the formula record;
4. `FallbackOnly` from the source record;
5. `FallbackOutOfDomain` from the source record.

Two independent builds must be byte-identical for the schema descriptor,
every record and the report. The report must state zero network requests,
zero source bytes, zero signal values and zero runtime/public authority.

Focused tests must reject:

- missing/unknown axis and required-axis absence;
- changed or malformed hash/source/claim;
- duplicate/noncanonical JSON;
- missing authored fallback;
- direct admission or other lifecycle skip;
- wrong previous hash/revision/state;
- terminal-state reopening;
- `AtlasAdmitted` without independent validator/cooker evidence;
- output inside the repository.

## Decision and stop rule

- `M1A_RESEARCH_RECORD_V0_FIXTURE_PASS`: every positive and negative fixture,
  current-version roundtrip and repeat-exact gate passes. This opens only M1b.
- Any validation, determinism, accounting or confinement failure is
  `INVALID_M1A_RESEARCH_RECORD`; repair implementation/schema agreement before
  M1b. Do not weaken state, axis or fallback requirements.

No M1a result authorizes source selection, fresh signal access, formula
training, validator release, content cooking or runtime use.
