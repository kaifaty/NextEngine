# Physical Sound V15-S0b — YCB capability and cost result

| Field | Value |
| --- | --- |
| Date | `2026-09-01` |
| Status | `COMPLETE / REPEAT_EXACT / ZERO_SIGNAL / ROLES_UNFROZEN` |
| Decision | `S0B_YCB_CAPABILITY_PASS_S0C_ROLE_FREEZE_NEXT` |
| Protocol | [S0b protocol](physical-sound-v15-s0b-ycb-capability-cost-protocol-2026-09-01.md) |
| Roadmap | [V15 S0b](../plans/physical-sound-synthesis-roadmap-v15.md) |
| Product effect | None; SPEC-45 remains `Proposed` and authored clips remain authoritative |

## Outcome

S0b now provides a deterministic, source-blind view of the complete public
YCB-impact metadata surface. It binds the exact object/material workbook, the
official YCB model routes, both OSF component trees and the acquisition facts
published in the paper without opening any recording, video, mesh, archive or
signal value.

The useful result is narrower than the publisher's material totals:

| Material | Workbook objects | Exact robot object parents | Opaque files / declared bytes | Non-ambiguous route + parent | S0b role credit |
| --- | ---: | ---: | ---: | ---: | --- |
| Metal | `12` | `9` | `45 / 16,996,164` | `9` | None; freshness and missing axes remain unresolved. |
| Wood | `3` | `3` | `15 / 4,917,987` | `2` | None; all three objects are already referenced by the repository adapter. |
| Glass | `3` | `0` | `0 / 0` | `0` | None; horizontal clips are material aggregates, not object parents. |

All `77` workbook rows remain `training_usable=false` and
`evaluation_complete=false`. The adapter reports `39` exact robot vertical
parents across all materials, but only the target-domain rows above are
relevant to V15.

## Metal scope

The exact Steel object parents are:

```text
27 Skillet                 repository adapter referenced
30 Fork                    freshness unassessed
31 Spoon                   freshness unassessed
32 Knife                   freshness unassessed
37 Scissors                freshness unassessed
38 Padlock                 repository adapter referenced
42 Adjustable wrench       freshness unassessed
43 Philips screwdriver     freshness unassessed
48 Hammer                  repository adapter referenced
```

The six rows absent from the repository adapter are not called fresh. S0c must
merge their exact OSF parent IDs with the historical exposure ledger before
assigning a role. YCB objects `39` Keys, `44` Flat screwdriver and `45` Nails
have no exact vertical recording parent in this publisher revision; `45` also
has no official model route.

Metal therefore remains the first-domain candidate, but S0b does not prove the
unchanged `4/1/1/1/1` role shape. It proves that a nine-object structural pool
exists before freshness and full-axis filtering.

## Wood and Glass consequences

Wood has exact vertical parents for object `36`, `70` and `71`. Object `70`
maps to multiple official geometry variants without a recording-level variant
identity, leaving only two non-ambiguous route-plus-parent rows. All three were
already opened by the repository YCB adapter, so Wood remains
`PENDING_SOURCE_CERTIFICATE`.

Glass has no object-named vertical parent. The robot horizontal tree is arranged
as `material / publisher split / speed`, and the paper confirms that horizontal
clips are collected per material. It contains no object folder that can bind a
clip parent to Wineglass, Skillet lid or Marbles. In addition, Wineglass has no
official model, while Marbles has unresolved variants. Glass remains
`FallbackOnly / SourceGrowth`.

Manual hit/scratch/drop data likewise remains aggregate at the OSF parent level
in the acquired tree and receives no object-level credit.

## Missing axes

The [official paper](https://upcommons.upc.edu/entities/publication/41911ed4-77c5-4886-b606-760ebc39b741)
proves the robot, gripper, microphone, `44.1 kHz` rate, table support, top/side
directions, fixed vertical object-to-microphone relation and horizontal speeds
of `14 mm/s` and `25 mm/s`. It does not publish exact contact coordinates,
force/energy, microphone coordinates, support material/fixture or a
contact-to-mesh correspondence.

The [official YCB model index](https://ycb-benchmarks.s3.amazonaws.com/index.html)
provides stable archive routes and quality footnotes, not archive hashes,
member manifests or proved metric scale. Consequently the maximum S0b credit
for excitation, geometry, listener, support and recorded response is
`partial`; contact geometry and scale stay `unknown`.

## Exact evidence

External root:

```text
/home/kaifaty/.codex/experiments/nextengine/physical-sound/physical-sound-v15-s0b-research.L75yqv
```

Frozen inputs:

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| Object/material workbook | `7,994` | `27672ecfdfaf2a1ecfc8926127ab9ab962e9cb59adb593b141caadc0459ae0fd` |
| Official model index | `105,935` | `d90b97268e9fd4eaf501efc22d17e27d419e9d94f498c6f6b6f63defd2a49891` |
| Open-access paper | `7,130,560` | `13fcf1fe0adce22a9e108c95baaeaf0c0f9a06e2fbd04cb60ee810d10254554b` |
| Normalized OSF snapshot | `520,018` | `0efe649df362398ac068977baa126d42bfd67a32b6f52fc5c6e46f5270e41645` |

The snapshot contains `926` entries (`801` files and `125` folders): `893`
from robot component `bj5w8` and `33` from manual component `hjdby`. Acquisition
used `130` JSON requests and read `1,897,519` metadata bytes. A second live
acquisition produced the same `520,018` bytes and SHA-256.

Run `e` and run `f` are byte-identical:

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `capability-inventory.json` | `227,019` | `7f1b316e5109122cb9efd1834ab7a6a95cbd9d098d0f8d89971be98e76d158cf` |
| `cost-census.json` | `1,030` | `03a60ecd98105bcaaa992a4c80af7cf0fd12a99ff4b8385c19ca058e48be1055` |
| `report.json` | `1,518` | `9febc5094e8063626bfcb2e4f03f2eff54ab41b4255c132bc17e702f1b4e6d44` |

Exact implementation identities:

| Artifact | SHA-256 |
| --- | --- |
| Runner | `41bd1f7d09439ebabde23879cdafebddbc9288d442123c4dacf86eb6570b5251` |
| Focused tests | `ffd63df793f224546e8d52c76106feed8def9e3a66ba7e201fd3e18a8bb42841` |
| Protocol | `b240aeca0802b43a1864275b8e985d897d2c45847ebb368284a7a51e49b5c01e` |

## Access ledger and verification

Acquisition and both builds report zero audio/video/mesh/archive/force body
bytes, zero opened payload members and zero decoded signal values. Build-time
network requests are also zero.

The focused suite passes `11/11` tests. It covers repeat-exact acquisition and
build, real input identities, object-parent aliases, vertical/horizontal
separation, numeric-only false matches, model variants, route/footnote
contradictions, JSON canonicality, duplicate IDs/paths, host/path escape,
forbidden access counters and external fresh-output guards.

The wider affected dataset-contract, exposure, revision-identity, source-
inventory and YCB metadata suite passes `50/50` tests. `content-package`
passes with `123` records and `64` chunks. The mapped `boundary-scan` still
reports the pre-existing `SOURCE_LAYOUT_ESCAPE_HATCH` in
`tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`;
S0b adds no new boundary finding.

## Decision and next boundary

S0b is complete and opens only S0c. S0c must:

1. merge exact YCB parent IDs with the S0a exposure census;
2. combine YCB and current-source candidates without counting aliases twice;
3. prove or reject every required contact/geometry/scale/listener/support axis;
4. publish one immutable Metal `4/1/1/1/1` role descriptor with exact hashes,
   or close Metal as `SOURCE_INSUFFICIENT`;
5. publish Wood pending and Glass fallback certificates in the same result.

No recording or mesh body may be opened until that role/certificate decision
is frozen.
