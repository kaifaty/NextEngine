# Physical sound V41 D1 alias-component roster result

Date: `2026-09-03`

Result: `COMPLETE / REPEAT_EXACT_ALIAS_REPAIR / BLUE_BOWL_COMPONENT_COLOCATED / 5_TRAIN_2_DEVELOPMENT_2_VALIDATOR / ZERO_SIGNAL / C0_AUTHORIZED`

Roadmap authority: [Roadmap V41](../plans/physical-sound-synthesis-roadmap-v41.md)

## First failing boundary

The V40 D0 roster assigned `samuel-clarke--realimpact` to
`generator_train` and `stanford-objectfolder--objectfolder-real` to
`validator_calibration`. Exact pre-payload lineage already proves that
REALIMPACT `6_Bowl` transfer rows and ObjectFolder object `6 / Blue_Bowl`
recordings are observations of the same physical Blue Bowl. A validator split
at family scope would therefore receive another signal from a generator-train
physical parent.

D0 remains immutable evidence: its source roster SHA-256 is
`6d6f29fa…80de9` and root is `5038c54d…af7bd`. D1 supersedes only that roster's
role projection. It does not reinterpret D0 as a pass and it does not read an
audio header, payload, PCM sample, feature or target.

## Repair

D1 promotes physical-parent alias component to the split unit before any C0
payload access:

| Component/family | D0 role | D1 role |
| --- | --- | --- |
| REALIMPACT + ObjectFolder Blue Bowl component | train + validator | `generator_train` for both families |
| `zisen-shao--av-msf` | generator train | `validator_calibration` |
| Every other disclosed family | unchanged | unchanged |

AV-MSF is an independent publisher/project/revision family and has no declared
physical-parent alias to the REALIMPACT/ObjectFolder component. The repaired
roster retains exactly five train, two development and two validator families,
but every physical-parent component now has exactly one role. Descendants
inherit the component role and no further role movement is allowed after D1.

## Frozen implementation

- owner: `lab/scripts/physical_sound_v41_d1_alias_component_roster_v1.py`,
  `26,168 bytes`,
  `sha256=9214bc651e4bce3d987d7f53ebfce2808eef5cc0f229c6bcb59a0d4c69ea56fb`;
- profile:
  `lab/profiles/physical-sound-v41-d1-alias-component-roster.v1.json`,
  `6,355 bytes`,
  `sha256=fd76c7f628f4bf74668ec435fbad48cf14bef173cc320c483f7448f46ee6e907`;
- tests:
  `lab/tests/test_physical_sound_v41_d1_alias_component_roster_v1.py`,
  `9,764 bytes`,
  `sha256=94ebc6cd41fa7cf109dc9ceb292a71db895903c368c6251387419bb2154b45cb`.

The profile binds D0 owner/profile/result, I0 result, the V24 X0 exact Blue Bowl
lineage result and Roadmap V41 by path, size and SHA-256. It separately binds
the exact external D0 roster bytes and root. Any dependency, alias, source role,
assignment or authority drift rejects before output publication.

## External A/B evidence

External root:

`/home/kaifaty/.codex/experiments/nextengine/physical-sound/physical-sound-v41-d1-2026-09-03`

`run-a` consumes the D0 `run-a` roster and `run-b` consumes the independently
reproduced D0 `run-b` roster. Their complete output trees are byte-identical.
Deterministic tree root:

`f5a0b672c35436e41dca3aaf49c944bdf8070dd0473b8c8eee251ff761f13d3f`

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `access-ledger.json` | 7,568 | `39a70259e2421675d7ccd6b55f4c1cf735c49a058c3f889a5e4bd243afab6290` |
| `disclosed-roster.json` | 14,236 | `362bddc6e02808fea89d06b0809e8d9e05b675181dead86da408a8944f9ad7a8` |
| `profile.json` | 6,355 | `fd76c7f628f4bf74668ec435fbad48cf14bef173cc320c483f7448f46ee6e907` |
| `report.json` | 2,991 | `f739fb87f4a2a09314f711e94df6614cb5e840a93b35bbfc3530246e2edae4f0` |

The repaired roster root is
`778cae737a2d94aa06af894c58ea0a11d85bd74534775bad5bd9dad715dd775e`.
C0 must bind both this root and the exact roster file SHA-256 before reading any
disclosed payload.

## Gates and access

All eight conjunctive gates pass in both runs:

- all nine D0 families are projected exactly once;
- the exact X0 alias evidence is hash-bound;
- D0's one cross-role physical-parent component is detected;
- D1 has zero cross-role components;
- role counts remain `5/2/2`;
- clean and protected project spend remain zero;
- protected roles remain empty;
- every forbidden access counter is zero.

Each run reads `10,142` D0 roster bytes and `66,947` repository-binding bytes.
Network requests, source payload bytes, audio headers/previews, PCM, force,
mesh, features, model targets, candidate outputs, protected signal and role
signal are all exactly zero.

## Verification

- focused D1 suite: `PASS`, `8/8` tests;
- combined D0+D1 suites: `PASS`, `16/16` tests;
- the historical D1 Roadmap V41 binding is replayed as frozen evidence while
  the living roadmap advances to later completed stages;
- external CLI A/B and recursive byte comparison: `PASS`;
- mutations cover role/alias/dependency/authority drift, source-D0 mutation,
  noncanonical JSON, repository/occupied/symlink outputs and late atomic
  publication failure;
- repository boundary scan: expected pre-existing `SOURCE_LAYOUT_ESCAPE_HATCH`
  in `realimpact_transfer_fixture.rs`; D1 adds no Rust source or escape hatch;
- runtime ProductChecks: `NOT_RUN`, because D1 is external zero-signal research
  tooling under SPEC-45 `Proposed` and changes no production consumer.

## Decision and next action

D1 returns `D1_ALIAS_COMPONENT_ROSTER_REPAIRED_C0_AUTHORIZED`. C0 may now open
only disclosed payloads assigned by the D1 roster. It must preserve the Blue
Bowl component as one generator-train parent component, assign AV-MSF only to
validator calibration, emit immutable role projections and fail before partial
publication on any additional cross-role physical-parent alias.

D1 grants no model training, validator release, admission, cooking, demo,
runtime or public-contract authority. Authored clips remain mandatory.
