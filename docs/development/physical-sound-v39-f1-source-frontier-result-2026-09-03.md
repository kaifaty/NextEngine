# Physical sound V39 F1 source-frontier result

Date: `2026-09-03`

Result: `COMPLETE / REPEAT_EXACT_IMPROVED_FRONTIER / 6_EXACT_STEEL_23_NON_METAL_REMAIN / ZERO_SIGNAL / NO_ROLE_AUTHORITY`

Roadmap authority: [Roadmap V39](../plans/physical-sound-synthesis-roadmap-v39.md)

## Claim

F1 turns the V30 baseline plus the V38 IETeasy increment into one portable,
hash-closed, metadata-only source-frontier replay. It proves the unchanged
whole-project planner still moves from deficit `6 exact-Steel / 27 non-Metal`
to `6 / 23` and leaves five projects unassigned.

F1 does not assign data roles, open source payloads, decode signal, train a
model, release a validator or authorize admission, cooking, demo integration,
runtime inference, a public contract or a material-quality claim.

## Frozen implementation

- owner: `lab/scripts/physical_sound_v39_f1_source_frontier_v1.py`,
  `32,360 bytes`,
  `sha256=4363cc70b92f54a0646ca2727533a8f6f76216b809b0813aa8623effb291e725`;
- profile: `lab/profiles/physical-sound-v39-f1-source-frontier.v1.json`,
  `11,831 bytes`,
  `sha256=a0e752a85d4bddcca5e2ae8d171d69aa90df1e2c99db6257b000e8ae95e8a82a`;
- unchanged planner owner:
  `lab/scripts/physical_sound_v29_q1a_source_growth_v1.py`,
  `sha256=88574b4a89fb65f537cac5ff8f31cba732446808c8e63874352b058eb9144cac`;
- target policy remains `exact_unqualified_publisher_steel_v1` with protected
  minima `2 projects / 16 exact-Steel / 35 non-Metal` per role and five
  reserved projects.

The profile binds F0, V30/V38 evidence and the previous profile/planner by
exact path, byte count and SHA-256. Its compact ten-project baseline projection
matches the external V30 audit/report exactly: both receipts, every project
row, combined power, decision, partition and audit-to-report binding pass.

## IETeasy classification

Primary evidence is the [Data in Brief article](https://pmc.ncbi.nlm.nih.gov/articles/PMC8567360/),
[Mendeley Data revision 1](https://data.mendeley.com/datasets/srfp7x6wxm/1)
and the [supporting IETeasy article](https://pmc.ncbi.nlm.nih.gov/articles/PMC9123443/).
The source contains ten recordings for each of fifteen physical samples. F1
therefore records `150` repetitions but counts only `15` parent groups.

The exact power is:

- `0 exact-Steel`: AISI 304, AISI 316, Fe37, X150 and C45E are explicit grades,
  not the frozen unqualified `Steel` label;
- `4 non-Metal`: Nylon 6, Polizene/HDPE, Pom-C and Teflon;
- `6 ineligible other-Metal`: aluminium 6082, B10 bronze, B12 bronze, BrAl,
  copper and CW614 brass.

The complete sample roster, dimensions, masses, relations, source revision and
three-lead batch have independent canonical roots. Relabelling a grade as
exact Steel, crediting an ineligible lead, changing physical identity or
counting recording repetitions as groups fails closed.

## Replayed frontier

| State | Projects | Exact Steel | Non-Metal | Best role-B deficit | Reserved |
| --- | ---: | ---: | ---: | ---: | ---: |
| V30 baseline | 10 | 32 | 73 | `6 / 27` | 5 |
| V30 + IETeasy | 11 | 32 | 77 | `6 / 23` | 5 |

IETeasy is one indivisible project revision and joins role B only in the
planning frontier. `feasible=false`; no role opens. RSAudio stays
`METADATA_UNAVAILABLE` and DiffImpact ASMR stays `MISSING_SOURCE_IDENTITY`, so
both receive zero source power.

## External A/B evidence

External root:

`/home/kaifaty/.codex/experiments/nextengine-physical-sound-v39-f1-2026-09-03`

Fresh `run-a` and `run-b` contain identical four-file trees. Deterministic tree
root:

`b12dda3dd44f44f34e9e55c48bbf7bdd21cbe04cbe92d3b5b13e5c1b8896db0f`

| Artifact | SHA-256 |
| --- | --- |
| `classification.json` | `d50e69092d07a7bd54a2e88aad66d370599323342a07d3126cc2ec71e9df1b6f` |
| `frontier.json` | `f09dc9d32f8e27549df8e1a160808d9734a925d4a5ecdaafbd2c981fb8efca4a` |
| `profile.json` | `a0e752a85d4bddcca5e2ae8d171d69aa90df1e2c99db6257b000e8ae95e8a82a` |
| `report.json` | `11ea486671a11fbb1bbe1503f8ad46acfdf88aad3426dd42892e3ef8287c07d6` |

Every F1 counter for network, payload, audio header/preview, PCM, force, mesh,
feature, model target, candidate output, protected signal and role signal is
zero. Repository/profile metadata reads are reported separately.

## Failure and boundary evidence

The combined Q1a/F0/F1 Python suites pass `32/32`. F1 covers material
inflation, duplicate sample/project parents, repetition inflation, ineligible
lead credit, batch growth, minima/authority/receipt/dependency drift,
non-canonical and duplicate-key JSON, in-repository/occupied/symlink output and
late atomic-publication failure.

The hash-bound Rust neural data-plane suite passes `10/10` for role privacy,
evidence-lane closure, projection repeatability and atomic publication.

`cargo run -p xtask -- boundary-scan` remains red only on the pre-existing
`SOURCE_LAYOUT_ESCAPE_HATCH` in
`tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`.
F1 adds no Rust escape hatch and imports no network, signal or model library.

## Decision and next action

F1 closes as repeat-exact `ImprovedFrontier`. Metadata-only S0 discovery
continues against exact deficit `6/23`; it remains the prerequisite for a
protected role freeze. The implementation queue may proceed independently to
V39 D0: freeze a permanent disclosed roster and prove that no disclosed parent
can ever re-enter a protected pool. Scheduling D0 grants no payload, training,
validator or product authority by itself.
