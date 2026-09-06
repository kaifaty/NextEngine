# Physical Sound V29 Q0-M — signal-blind Metal source inventory protocol

| Field | Value |
| --- | --- |
| Date | `2026-09-02` |
| Status | `FROZEN_BEFORE_IMPLEMENTATION / METADATA_ONLY / ZERO_SIGNAL / ROLES_UNASSIGNED` |
| Roadmap | [V29 Q0-M](../plans/physical-sound-synthesis-roadmap-v29.md) |
| Predecessors | [V29 rebaseline](physical-sound-v29-validator-first-ml-rebaseline-research-2026-09-02.md), [V15 S0a](physical-sound-v15-s0a-revision-aware-identity-exposure-result-2026-09-01.md), [V15 S0b](physical-sound-v15-s0b-ycb-capability-cost-result-2026-09-01.md), [historical Glass split](physical-sound-kronland-reject-split-freeze-ps2-2026-08-28.md) |
| Architecture | [SPEC-45](../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md), `Proposed` |
| Product effect | None; authored clips remain authoritative and no runtime ML or content contract is authorized. |

## Question and bounded claim

Q0-M asks whether exact publisher metadata contains enough independent object
identities to attempt a fresh Steel/Metal validator release without reading a
waveform, feature, mesh value, force value or protected result.

The owner may publish source candidates and known blockers. It cannot assign a
dataset role, establish freshness, qualify a validator, authorize protected
payload access or call a source acoustically representative. Q1-M owns the
exposure, cluster-power and one-use role freeze.

## Frozen inputs

Only these three immutable files may be read:

| Input | Bytes | SHA-256 | Authority |
| --- | ---: | --- | --- |
| ObjectFolder Real official object/material table | `34,749` | `0111f57a336fdeb805eb03543fb10b2cb174f8d0bcb7ef5f0e30c17324e06db0` | [Official download page](https://objectfolder.stanford.edu/objectfolder-real-download) |
| YCB-impact final metadata capability inventory | `227,019` | `7f1b316e5109122cb9efd1834ab7a6a95cbd9d098d0f8d89971be98e76d158cf` | Exact workbook/OSF/model-index projection frozen by V15 S0b |
| Historical project-disjoint Glass manifest | `9,112` | `9ddec04de271c034f1389f5fcec98ce410028b307ae565b3f890eb7f7a726c33` | Immutable `target_material_label=Glass` assignment surface |

The ObjectFolder page must contain exactly object IDs `1..100`, one name and
one of `Ceramic`, `Glass`, `Wood`, `Plastic`, `Iron`, `Polycarbonate` or
`Steel` per object. Its source identity is:

```text
publisher  = stanford-objectfolder
project    = objectfolder-real
revision   = rendered-table-sha256-0111f57a
license    = NOASSERTION
policy     = external_research_only
```

The YCB inventory must retain its exact publisher/project/component identity,
77 workbook objects, 39 exact vertical recording parents and zero body/signal
access from S0b. Its source identity is:

```text
publisher  = iri-csic-upc-ctu
project    = ycb-impact-sounds
revision   = osf-bj5w8-2022-09-27
license    = NOASSERTION
policy     = external_research_only
```

Unknown redistribution terms never become permission to ship source bytes.
Q0-M stores provenance and keeps every source artifact outside Git.

## Material policy

Every publisher label remains verbatim. Q0-M additionally emits three
non-authoritative inventory relations:

- `exact_steel_candidate` for primary material `Steel`;
- `other_metal_candidate` for primary material `Iron` or `Aluminium`;
- `non_metal_candidate` for every other allowed primary material.

This prevents broad `Metal` pooling from hiding the original Steel target.
Q1-M must freeze the eventual target-submaterial policy before signals open.
Secondary YCB materials remain explicit and never create another independent
object group.

## Physical grouping and historical exclusion

One candidate group is:

```text
(publisher, project, immutable revision, exact object ID,
 exact object-level recording parent when the source requires one)
```

ObjectFolder contributes one group per table row. YCB contributes only rows
with one exact object-bound vertical parent; aggregate horizontal/manual
folders receive no group credit.

The historical manifest is read only for exclusion. Every
`objectfolder-real-demo-N` assignment excludes ObjectFolder object `N`.
Every `ycb-impact-vertical-object-NNN-*` assignment excludes YCB object `NNN`.
All other historical source IDs remain recorded but cannot match a Q0-M
candidate by guess or name similarity.

An excluded row is still emitted with its historical partition and role. It
does not count toward Steel, broad-Metal or reject-parent power. In particular,
historical Metal rows remain `reject_parent`; Q0-M never flips them into Metal
positives. The entire historical split is ineligible for Metal threshold
selection.

## Declared and missing axes

Q0-M distinguishes a publisher declaration from an opened value.

ObjectFolder rows report object and material identity as `known`; geometry,
recorded response, contact position and force profile as
`declared_opaque_payload`; and listener/support conditions as `missing`.
These declarations do not prove payload completeness or Q1 role eligibility.

YCB rows copy the S0b `known`/`partial`/`unknown` capability states and exact
missing-axis lists. Q0-M rejects any attempt to upgrade those states, or any
YCB row whose `training_usable` or `evaluation_complete` flag is true under
the frozen metadata-only input.

## Outputs and decision

The owner writes atomically to a fresh external directory:

```text
metal-source-inventory.json
report.json
```

The inventory contains sorted source descriptors, candidate groups, verbatim
material labels, relation, acquisition axes, provenance, exclusion evidence
and role state `unassigned`. The report binds all input/output hashes, counts
exact Steel and broad Metal separately, reports non-Metal candidates and
source/project diversity, and carries explicit Q1 blockers.

The passing inventory decision is:

```text
Q0M_METAL_SOURCE_INVENTORY_FEASIBLE_Q1M_NEXT
```

It requires, after historical exclusion:

- at least `16` exact-Steel candidates from at least two project revisions;
- at least `16` broad-Metal candidates from at least two project revisions;
- at least `35` non-Metal candidates from at least two project revisions;
- no duplicate physical group, unknown material, aggregate YCB parent or
  source/revision substitution.

Otherwise the successful negative decision is:

```text
Q0M_SOURCE_IDENTITY_INSUFFICIENT_SOURCE_GROWTH_REQUIRED
```

Neither decision assigns roles. Even a feasible inventory leaves Q1-M to
prove exposure/freshness, effective cluster count, per-protected-partition
power and multi-project balance. Q1 may require more sources, never fewer
groups or weaker risk bounds.

## Access ledger

Every build reports exactly:

```text
network_requests = 0
source_payload_bytes_read = 0
archive_member_bodies_read = 0
audio_headers_parsed = 0
mesh_values_decoded = 0
pcm_sample_values_decoded = 0
force_sample_values_decoded = 0
protected_signal_values_decoded = 0
```

Only the three bounded metadata files count as `metadata_bytes_read`.
Repository paths, symlinks, non-regular files, oversized inputs, hash drift,
duplicate JSON keys, non-canonical JSON, unknown fields/states and replacement
of an existing output directory fail closed.

## Required focused verification

Tests must cover at least:

1. a repeat-exact positive fixture with all signal counters zero;
2. exact 100-row ObjectFolder parsing and exact 77-row/39-parent YCB parsing;
3. preservation of Steel, Iron and Aluminium as separate labels;
4. historical Glass target and Metal reject rows excluded without relabelling;
5. duplicate object/group and substituted revision/URL rejection;
6. aggregate or ambiguous YCB parent receiving no object-group credit;
7. lowered minima, one-project concentration and unknown material rejection;
8. signal-bearing input fields, symlink input and output replacement rejection.

The official execution builds twice from separate output paths and compares
both directories recursively. No source payload, generated WAV, dataset,
checkpoint or heavy artifact is added to Git.

## Next boundary

A feasible Q0-M result opens only Q1-M. Q1-M must join the candidate identities
to the complete historical exposure ledger, freeze exact Steel/Metal policy,
estimate source/project cluster power and either publish one complete
one-use role assignment or terminate as `FallbackOutOfDomain` with a named
internet-source gap.
