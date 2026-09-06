# Physical Sound V29 Q0-M — signal-blind Metal source inventory result

| Field | Value |
| --- | --- |
| Date | `2026-09-02` |
| Status | `COMPLETE / REPEAT_EXACT / ZERO_SIGNAL / SOURCE_IDENTITIES_FEASIBLE / ROLES_UNASSIGNED` |
| Decision | `Q0M_METAL_SOURCE_INVENTORY_FEASIBLE_Q1M_NEXT` |
| Protocol | [Q0-M protocol](physical-sound-v29-q0m-metal-source-inventory-protocol-2026-09-02.md) |
| Roadmap | [V29 Q0-M](../plans/physical-sound-synthesis-roadmap-v29.md) |
| Product effect | None; no payload opened, role assigned, validator qualified or runtime/content path promoted. |

## Outcome

Q0-M builds one deterministic inventory from the official ObjectFolder Real
object/material table, the final YCB metadata capability projection and the
immutable historical Glass assignment manifest. It preserves exact material
labels and removes every overlapping historical identity before measuring
source power.

The raw identity surface is sufficient to attempt Q1-M:

| Available after exclusion | ObjectFolder Real | YCB impact | Total | Required |
| --- | ---: | ---: | ---: | ---: |
| Exact `Steel` candidates | `17` | `6` | `23` | `16` |
| Other Metal (`Iron`/`Aluminium`) | `15` | `1` | `16` | reported separately |
| Broad Metal total | `32` | `7` | `39` | `16` |
| Non-Metal candidates | `63` | `7` | `70` | `35` |
| Project revisions per required class | — | — | `2` | `2` |

There are `139` exact object/parent groups before exclusion and `109` after
exclusion. This is identity power only. It does not prove acoustic quality,
effective statistical independence, freshness or per-protected-partition
power.

## Steel remains distinct

The inventory does not collapse every hard metal into the requested Steel
sound:

- `Steel` is `exact_steel_candidate`;
- `Iron` and `Aluminium` are `other_metal_candidate`;
- every publisher label remains verbatim, including YCB secondary materials.

This gives Q1-M enough exact Steel groups to choose the narrow domain without
using Iron or Aluminium as silent positives. Q1-M must freeze that target
policy before any signal opens.

## Historical Glass split remains immutable

The owner matched and excluded `30` current candidates by exact identity:

| Overlap | Count | Consequence |
| --- | ---: | --- |
| ObjectFolder Real demo identities | `5` | Current rows remain visible but cannot enter Q1 roles. |
| YCB vertical identities | `25` | Current parent rows remain visible but cannot enter Q1 roles. |
| Historical exact Steel rows | `3` | Preserve old role `reject_parent`; never relabel as Metal positive. |
| Historical Iron/Aluminium rows | `3` | Preserve old role `reject_parent`; never pool into Metal. |
| Historical non-Metal rows | `24` | Excluded from Metal threshold selection despite compatible negative semantics. |

The other `34` historical assignments do not match either Q0 source project
and receive no name-based or inferred match. The complete old split remains
ineligible for Metal threshold selection; Q0 only records exact overlaps.

## Declared axes and remaining gaps

ObjectFolder contributes exact object/material rows and publisher declarations
for opaque geometry, recorded response, contact position and force profile.
Listener and support conditions remain missing.

YCB contributes its frozen `known`/`partial`/`unknown` axes. Exact vertical
parents exist, but every row remains `training_usable=false` and
`evaluation_complete=false`; contact-to-geometry binding, exact scale and
other acquisition facts remain partial or unknown.

Therefore Q1-M still has five blockers:

```text
effective_cluster_power_not_frozen
exposure_freshness_not_reaudited
one_use_roles_unassigned
protected_payloads_sealed
target_submaterial_policy_unfrozen
```

Q0 feasibility means only that another source search is not required before
attempting this audit. Q1 may still end in `FallbackOutOfDomain` if fresh,
project-balanced protected roles cannot meet the unchanged risk contract.

## Exact evidence

External root:

```text
/home/kaifaty/.codex/experiments/nextengine/physical-sound/physical-sound-v29-q0m.kPdf2S
```

Run A and run B are recursively byte-identical:

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `metal-source-inventory.json` | `172,172` | `9a04c8a2fb9f0c6a3d351dac2796936d933d223e9400afc79a95f06e01665f43` |
| `report.json` | `2,018` | `a33c0e2eae12663d8c8dc90cca926b1d614bec332a07e4dddea4d75bf403b9d5` |

Frozen inputs:

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| ObjectFolder official table | `34,749` | `0111f57a336fdeb805eb03543fb10b2cb174f8d0bcb7ef5f0e30c17324e06db0` |
| YCB capability inventory | `227,019` | `7f1b316e5109122cb9efd1834ab7a6a95cbd9d098d0f8d89971be98e76d158cf` |
| Historical Glass manifest | `9,112` | `9ddec04de271c034f1389f5fcec98ce410028b307ae565b3f890eb7f7a726c33` |

Implementation identities:

| Artifact | SHA-256 |
| --- | --- |
| Runner | `74f0f3e04452fafc39c6059be5e77ddeb1726c49588f3747fa17d73214151209` |
| Focused tests | `f74fd9db3072033658555a0c5724bb15fcb3a40e2a3bcae8ab5fe1fcac618728` |
| Protocol | `a9d2201c11dde1076a8b883092c6d4e559d3a620cb7663544ec70d2e85ce5449` |

## Access and verification

Both official runs read `270,880` metadata bytes and report zero network
requests, source payload bytes, archive members, audio headers, mesh values,
PCM samples, force samples and protected signal values.

The focused suite passes `11/11` tests. It covers repeat exactness, exact
ObjectFolder/YCB shape, separate Steel/Iron/Aluminium labels, historical-role
preservation, project concentration, duplicate/unknown identities, aggregate
YCB rejection, signal-bearing schema drift, symlink/input-hash/output guards
and repository-output rejection.

The wider Q0-M, YCB capability, source-sufficiency, revision-identity and
source-inventory suite passes `45/45`. `cargo run -p xtask -- content-package`
passes with `123` records and `64` chunks. The mapped
`cargo run -p xtask -- boundary-scan` still fails only on the pre-existing
`SOURCE_LAYOUT_ESCAPE_HATCH` in
`tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`;
Q0-M adds no boundary finding. `git diff --check` and all `141` changed-document
local links pass.

## Decision and next boundary

Q0-M is complete and opens only Q1-M. Q1-M must:

1. join all `109` available identities to the full historical exposure ledger;
2. freeze exact Steel versus broad Metal policy without reading signal;
3. estimate effective project/object clusters and prove each protected
   validator evaluation has at least `35` reject parents, `16` positives and
   multi-project coverage;
4. assign source/project/object-disjoint one-use roles atomically or publish a
   terminal source-power OOD certificate;
5. keep generator and validator roles disjoint and every protected payload
   sealed until the freeze is immutable.

P0 may continue independently. Q2 and real Metal training remain blocked.
