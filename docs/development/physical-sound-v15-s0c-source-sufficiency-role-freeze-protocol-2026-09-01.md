# Physical Sound V15-S0c — source sufficiency and role-freeze protocol

| Field | Value |
| --- | --- |
| Date | `2026-09-01` |
| Status | `FROZEN_BEFORE_IMPLEMENTATION / METADATA_ONLY / ZERO_SIGNAL` |
| Roadmap | [V15 S0c](../plans/physical-sound-synthesis-roadmap-v15.md) |
| Predecessors | [S0a](physical-sound-v15-s0a-revision-aware-identity-exposure-result-2026-09-01.md), [S0b](physical-sound-v15-s0b-ycb-capability-cost-result-2026-09-01.md), [N1b](physical-sound-v14-n1b-real-source-metadata-inventory-result-2026-09-01.md) |
| Product effect | None; authored clips remain authoritative and SPEC-45 remains `Proposed`. |

## Question and bounded claim

S0c asks whether the currently frozen, internet-published metadata can fill one
immutable first-domain Dataset Contract V1 without opening a waveform, array,
mesh body, force value, video or protected signal:

```text
Metal = 4 generator_train
      + 1 generator_development
      + 1 validator_calibration
      + 1 validator_method_holdout
      + 1 admission_shadow
```

The eight rows must remain distinct physical object and recording-parent
groups. Generator roles require `training_usable`; protected roles require
unexposed `evaluation_complete` T2/T3 rows. The exact N1a role minima, axis
vocabulary and exposure rules are unchanged.

S0c does not infer missing axes, choose roles from audio quality, decode source
signal or authorize S1. It either emits an immutable role descriptor which can
be validated by N1a, or a source-insufficiency certificate with no active role
assignments. Wood and Glass receive explicit status in the same run.

## Frozen exact inputs

| Input | Bytes | SHA-256 |
| --- | ---: | --- |
| S0a `identity-map.json` | `266,858` | `66e9a29e69b72b95836beef9dbd8d7d52cd33b1213c709fc2f57422139d1ea40` |
| S0a `exposure-census.json` | `2,318,665` | `ce3c5ab861a91bd85fc9a5a90f24629f15dbdc8d9faa936cd04e791efa667365` |
| N1b `inventory.json` | `1,118,080` | `2861bb0e42e40fe78e67dab7ed5e0897b40c42caf6ab639235948adfa38fcfc0` |
| S0b `capability-inventory.json` | `227,019` | `7f1b316e5109122cb9efd1834ab7a6a95cbd9d098d0f8d89971be98e76d158cf` |
| N1a protocol | `9,854` | `a8a25ec63fb402867c10a96efd48cfc360a6af2004a9987bee8205351253a754` |

The executable receives all four JSON hashes explicitly and rejects any byte
drift before parsing. The N1a protocol hash is recorded as policy provenance;
the executable additionally freezes the exact role shape and ten-axis policy
as constants and mutation-tests them.

## Source-credit rules

### S0a and N1b

A revision-aware physical group enters the S0c candidate census only when:

1. S0a material is exactly `Glass`, `Metal` or `Wood`;
2. S0a exposure is exactly `unexposed`;
3. S0a reports at least one candidate route;
4. every route resolves to one exact N1b `inventory_object_id` with matching
   source ID, material and object identity;
5. the N1b row is `metadata_candidate` or `member_preflight_required`.

`metadata_candidate` proves only compact member structure. It does not itself
prove the N1a axes. `member_preflight_required` proves even less. Because the
N1b schema contains no per-axis certificates, neither state receives
`training_usable` or `evaluation_complete` credit in S0c.

RealImpact membership is counted at the revision-aware physical-group level.
An exposed RealImpact alias cannot be recycled as protected evidence even when
the publisher setup has otherwise useful acquisition properties.

### YCB impact sounds

Each YCB row is keyed by its exact object ID and exact vertical recording
parent. Horizontal material aggregates, manual aggregate ZIPs and rows without
an object parent receive no object-group credit.

S0b already emits `training_usable`, `evaluation_complete`, missing-axis and
exposure fields. S0c accepts those booleans only when every corresponding axis
is exactly `known`, geometry has an exact payload identity and metric scale,
and freshness is proven `unexposed`. `not_in_adapter_freshness_unassessed` is
not fresh. Thus S0c cannot promote an S0b partial axis or merely available URL.

YCB identities are independent source objects. Name similarity to an
ObjectFolder object never creates an alias or deduplicates two physical groups.

## Eligibility and deterministic selection

The frozen axes are:

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

`training_usable` requires the first nine axes; `evaluation_complete` requires
all ten. Unknown, partial, missing or unrepresented axes fail closed.

If a material has enough eligible rows, role selection is deterministic:

1. sort eligible physical groups by their canonical SHA-256 identity;
2. reserve three unexposed evaluation-complete T2/T3 groups for validator
   calibration, method holdout and admission shadow;
3. allocate four generator-train groups, then one generator-development group
   from the remaining training-usable groups;
4. reject any physical-group or recording-parent overlap;
5. validate the resulting descriptor through the existing N1a contract
   validator before publishing it.

S0c publishes no partial role freeze. If either five generator groups or three
protected groups are unavailable, all assignments for that material remain
empty and the material decision is source-insufficient.

## Outputs

The builder writes atomically to a fresh external directory:

```text
source-sufficiency.json
role-freeze-decision.json
report.json
```

`source-sufficiency.json` contains sorted per-material and per-source candidate
counts, exact physical-group observations and explicit blockers.
`role-freeze-decision.json` contains the unchanged role policy, assignments or
an empty assignment set, and the next permitted action. `report.json` binds all
input/output hashes, access counters and the overall decision.

No dataset, payload, checkpoint, generated WAV or report is added to Git.

## Access ledger

Every successful or failed build reports:

```text
network_requests = 0
source_payload_bytes_read = 0
archive_member_bodies_read = 0
wav_headers_parsed = 0
npy_headers_parsed = 0
mesh_values_decoded = 0
video_frames_decoded = 0
pcm_sample_values_decoded = 0
force_sample_values_decoded = 0
protected_signal_values_decoded = 0
```

Only the four frozen canonical JSON inputs may be read. Symlinks, non-regular
files, oversized inputs, duplicate JSON keys, non-canonical JSON, unknown
schemas/fields/states, nonfinite values, hash drift, dangling routes,
cross-document identity disagreement and an existing output directory reject.

## Required mutations

Focused tests must cover at least:

- repeat-exact real-shape fixture with every signal counter zero;
- one-byte input drift and non-canonical JSON;
- unknown schema, field, material, exposure or candidate state;
- missing/dangling N1b route and material mismatch;
- false-fresh YCB promotion from `not_in_adapter_freshness_unassessed`;
- YCB `training_usable` with a missing/partial required axis;
- YCB `evaluation_complete` without support, exact geometry identity or scale;
- RealImpact exposed alias recycled into a protected role;
- role-minimum reduction, partial role assignment and cross-role parent reuse;
- output replacement and symlink input.

## Exit decisions

```text
S0C_ROLE_FREEZE_PASS_S1_NEXT
```

is permitted only when one material produces a complete N1a-valid
`4/1/1/1/1` descriptor and two real builds are byte-identical.

```text
S0C_SOURCE_INSUFFICIENT_SOURCE_GROWTH_REQUIRED
```

is the successful negative result when all guards pass but no material can
fill the unchanged shape. It authorizes only a new source-growth package:
publisher metadata/member preflight or an additional internet-published T2/T3
source. It does not authorize signal decode, role reduction or S1.

Any malformed or drifting input produces `INVALID_S0C_SOURCE_SUFFICIENCY` by
failing before output publication.
