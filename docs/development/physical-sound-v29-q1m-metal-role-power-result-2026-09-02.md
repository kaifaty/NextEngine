# Physical Sound V29 Q1-M — Metal exposure and role-power result

| Field | Value |
| --- | --- |
| Date | `2026-09-02` |
| Status | `COMPLETE / REPEAT_EXACT / ZERO_SIGNAL / SOURCE_POWER_OOD / NO_ROLE_ASSIGNMENT / PROTECTED_PAYLOADS_SEALED` |
| Decision | `Q1M_SOURCE_POWER_INSUFFICIENT_SOURCE_GROWTH_REQUIRED` |
| Protocol | [Q1-M protocol](physical-sound-v29-q1m-metal-role-power-protocol-2026-09-02.md) |
| Roadmap | [V29 Q1-M](../plans/physical-sound-synthesis-roadmap-v29.md) |
| Product effect | None; no validator, generator, role, payload, content contract or runtime path is authorized. |

## Outcome

Q1-M accounts for all `109` Q0-M identities and proves that the two current
internet projects cannot support an independent exact-Steel validator,
generator and one-shot joint admission. The result is a successful automatic
OOD decision, not a failed implementation and not a request for human review.

No role is assigned. Every source payload remains sealed and authored clips
remain authority.

## Why the source set is insufficient

The protected validator holdout and joint-admission shadow are independent
evaluations. Each requires at least `16` exact-Steel positive groups, `35`
non-Metal reject parents and two publisher/project revisions. Whole projects
cannot be split between roles.

| Necessary power | Available | Required | Result |
| --- | ---: | ---: | --- |
| Distinct project revisions for every frozen role | `2` | `9` | deficit `7` |
| Distinct project revisions for the two protected evaluations alone | `2` | `4` | deficit `2` |
| Exact-Steel groups for protected evaluations alone | `23` | `32` | deficit `9` |
| Non-Metal rejects for protected evaluations alone | `70` | `70` | zero left for validator development/calibration |
| Current freshness-certified exact-Steel groups | `0` | at least protected plus development/generator roles | insufficient |
| Current freshness-certified non-Metal groups | `0` | at least protected plus development/calibration roles | insufficient |

The `39` broad-Metal identities are diagnostic only. Pooling `16`
Iron/Aluminium rows with Steel would change the frozen target and still would
not fix project independence, reject allocation or freshness. Q1-M therefore
does not take that shortcut.

## Exposure accounting

Every available Q0 identity receives one explicit exposure state:

| State | Groups | Consequence |
| --- | ---: | --- |
| `historical_exposed` | `31` | Ineligible for a fresh role |
| `historical_freshness_unassessed` | `14` | YCB capability evidence cannot certify freshness |
| `historical_unexposed_not_currently_certified` | `13` | Old unexposed result predates Q0-M and needs a current re-audit |
| `identity_mapped_but_not_exposure_candidate_unassessed` | `51` | Identity is known, but the historical census did not assess it as a candidate |

The join is complete as accounting (`109/109`) but supplies zero current
freshness certificates. An absent historical record is not converted into a
fresh source claim.

## Atomic role result

The frozen role set was:

```text
validator-development
validator-calibration
validator-holdout
generator-training
generator-development
generator-method-holdout
joint-admission-shadow
```

The project-topology and class-power gates fail before any assignment. The
published role assignment is therefore empty, and all `109` available groups
remain `unassigned_source_power_ood`. No object from one project is moved to a
different role to make the counts look larger.

The exact blockers are:

```text
full_role_project_topology_requires_9_has_2
protected_role_project_topology_requires_4_has_2
exact_steel_protected_minimum_requires_32_has_23
non_metal_power_exhausted_by_protected_minima_leaves_0_for_development_and_calibration
current_freshness_not_certified_for_any_available_group
freshness_qualified_class_power_is_zero
```

## Exact evidence

External root:

```text
/home/kaifaty/.codex/experiments/nextengine/physical-sound/physical-sound-v29-q1m.UVuuOn
```

Run A and run B are recursively byte-identical:

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `metal-role-power-audit.json` | `72,478` | `f98e80c7ad21fd698718d2e31c858b3b4d53f386c69356b5252397a1d3e20944` |
| `report.json` | `2,310` | `1d1e1365667a70efb1834ada282cf0d4426abdf81afde86c0d2bd9eb749842a0` |

Frozen inputs:

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| Q0-M inventory | `172,172` | `9a04c8a2fb9f0c6a3d351dac2796936d933d223e9400afc79a95f06e01665f43` |
| Q0-M report | `2,018` | `a33c0e2eae12663d8c8dc90cca926b1d614bec332a07e4dddea4d75bf403b9d5` |
| V15 S0a exposure census | `2,318,665` | `ce3c5ab861a91bd85fc9a5a90f24629f15dbdc8d9faa936cd04e791efa667365` |
| V15 S0a identity map | `266,858` | `66e9a29e69b72b95836beef9dbd8d7d52cd33b1213c709fc2f57422139d1ea40` |
| V15 S0b YCB capability inventory | `227,019` | `7f1b316e5109122cb9efd1834ab7a6a95cbd9d098d0f8d89971be98e76d158cf` |

Implementation identities at the official run:

| Artifact | SHA-256 |
| --- | --- |
| Runner | `f587b1fa5b153ce836c6d856d895843d362501acd95dab532201ffcffa8a69d5` |
| Focused tests | `9fd47416600630478437a633ed2b08b30579f2098d6b89c3d05ca8debb17721c` |
| Protocol | `0480bce0e19c4f7eb16e3c65357fa04be5b0aa76ccd8e12a1acb030da533991b` |
| Implementation commit | `00e4be2d` |

## Access and verification

Both official runs read `2,986,732` metadata bytes and report zero network
requests, source payload bytes, archive members, audio headers, mesh values,
PCM samples, force samples, protected signal values and role signal values.

The focused Q1-M suite passes `12/12`. It covers exact repeats, project and
protected-evaluation floors, independent positive/reject minima, exact Steel
policy, atomic no-assignment, complete exposure accounting, freshness
non-inference, ambiguous joins, YCB parent mismatch, signal drift, symlink
inputs and output replacement.

The wider Q1-M/Q0-M/YCB capability/source-sufficiency/revision-identity/source-
inventory suite passes `57/57`. `cargo run -p xtask -- content-package` passes
with `123` records and `64` chunks. The mapped `cargo run -p xtask --
boundary-scan` still fails only on the pre-existing
`SOURCE_LAYOUT_ESCAPE_HATCH` in
`tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`;
Q1-M adds no boundary finding.

## Decision and next boundary

Q1-M closes the current two-project release as `FallbackOutOfDomain`. Q2 and
P2 remain blocked; Q0-M's raw identity feasibility is not revoked, but it is
not enough for protected statistical evaluation.

The next Q-lane package is `Q1a-M source growth`:

1. discover and freeze at least seven additional independent internet
   publisher/project revisions, with enough dual-class coverage to populate
   whole-project roles;
2. add at least nine exact-Steel groups before considering unprotected role
   needs, while preserving exact labels;
3. add non-Metal reject capacity beyond the protected `70` floor for
   development and calibration;
4. build a current metadata-only exposure ledger covering every proposed
   identity and certify freshness before role assignment;
5. rerun Q1-M as a fresh release with unchanged `16/35`, project-disjoint and
   one-use gates.

P0/P1 may proceed independently on synthetic known truth. No real Metal signal
training or validator implementation starts until a fresh Q1-M role freeze
passes.
