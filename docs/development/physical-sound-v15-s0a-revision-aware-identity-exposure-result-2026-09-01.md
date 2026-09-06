# Physical Sound V15-S0a — revision-aware identity and exposure result

| Field | Value |
| --- | --- |
| Date | `2026-09-01` |
| Status | `COMPLETE / REPEAT_EXACT / ZERO_SIGNAL / METAL_SCOPE_REMAINS_POTENTIAL` |
| Decision | `S0A_IDENTITY_EXPOSURE_PASS_METAL_SCOPE_REMAINS_POTENTIAL` |
| Protocol | [S0a protocol](physical-sound-v15-s0a-revision-aware-identity-exposure-protocol-2026-09-01.md) |
| Roadmap | [V15 S0a](../plans/physical-sound-synthesis-roadmap-v15.md) |
| Product effect | None; roles remain unfrozen, SPEC-45 remains `Proposed` and clips remain authoritative |

## Outcome

S0a removes the cross-revision false-fresh risk without opening source signal.
It resolves the 100 historical ObjectFolder rows, 100 current rows and 50
RealImpact names into 130 physical groups:

- 70 current rows join the same-name/material historical group;
- 30 current rows in the `71…100` revision-shift range remain distinct;
- all 50 RealImpact rows join only their pinned historical ObjectFolder group
  using the frozen paper-origin plus publisher filename-prefix evidence.

The exposure census then unions the existing M1c numeric/path-token evidence
with exact token-bounded RealImpact names found in prior experiment JSON. This
correctly marks `93_GreenGoblet` and the previously used RealImpact metal set
as exposed even where no legacy numeric `object_id` exists.

Metal still has enough source-blind groups to continue:

| Material | Candidate physical groups | Exposed | Unexposed | V15 consequence |
| --- | ---: | ---: | ---: | --- |
| Glass | `9` | `7` | `2` | Remains `FallbackOnly / SourceGrowth`. |
| Wood | `18` | `18` | `0` | Remains pending a genuinely independent source. |
| Metal | `28` | `16` | `12` | Unchanged eight-group gate remains achievable. |

These are metadata candidates, not `training_usable` or
`evaluation_complete` objects. S0a does not assign any dataset role or
authorize audio/member decode.

## Remaining unexposed Metal groups

All twelve are current/historical ObjectFolder identities outside the revision
drift and have a current T2 metadata route:

```text
39 Wrench_Small
40 Wrench_Middle
42 Pestle
44 Sculpture
45 Ladle
46 Spatula
47 Decorative_Cast
52 Fork_Small
53 Fork_Large
54 Spoon_Small
55 Spoon_Large
69 Display_Stand
```

This is a candidate pool, not a ranking. S0c must later prove the required
geometry/contact/listener/support axes and freeze exactly `4/1/1/1/1` without
using waveform content.

## Repeat-exact evidence

External root:

```text
/home/kaifaty/.codex/experiments/nextengine/physical-sound/physical-sound-v15-s0a-final.rkar3U
```

Run A and Run B are byte-identical:

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `identity-map.json` | `266,858` | `66e9a29e69b72b95836beef9dbd8d7d52cd33b1213c709fc2f57422139d1ea40` |
| `exposure-census.json` | `2,318,665` | `ce3c5ab861a91bd85fc9a5a90f24629f15dbdc8d9faa936cd04e791efa667365` |
| `report.json` | `1,698` | `da0792fbf06cda39f3555049b9679ed2d0702038c86bb26d7c7f4c0953d74f7c` |

Exact implementation identities:

| Artifact | SHA-256 |
| --- | --- |
| Runner | `179cd905a2d39469173e7e0e1002b4a75c3e8b51e15ff21f58721e37b279a382` |
| Focused tests | `2bd1b27880edb0af2f59720228e82cf83e6b0fbe579514a1753a7ecb4c8d78a8` |
| Protocol | `d1e3da41a54a1514bbcceb7d543204c378bf7fdc4cfba55a621918bf12bf0ce7` |

## Census and access facts

| Fact | Value |
| --- | ---: |
| Included historical JSON files | `1,340` |
| JSON bytes read | `116,989,018` |
| JSON scalar values parsed | `3,278,438` |
| M1c numeric/path-token records | `2,960` |
| Exact RealImpact-name evidence records | `5,034` |
| Physical groups | `130` |
| Alias edges | `120` |
| Revision discrepancies | `30` |
| Network requests/archive body bytes | `0 / 0` |
| Source payload bytes/members | `0 / 0` |
| WAV/NPY headers parsed | `0 / 0` |
| PCM/force/protected values decoded | `0 / 0 / 0` |

Source listing, dependency, N1b, N1c research and S0a output prefixes are
excluded by frozen rule. Hidden dry runs, failed experiments, manifests,
reports and MLflow JSON remain in the census. Therefore run A cannot expose an
object merely by creating S0a output, while earlier experimental role and
object references remain visible.

## Focused verification

`lab.tests.test_physical_sound_revision_identity_v1` passes eight test methods.
They cover repeat identity, zero-signal accounting, current/historical drift,
RealImpact-to-historical mapping, exact `93_GreenGoblet` exposure, source-
listing exclusion, numeric/path-token ambiguity propagation, schema/claim/
roster mutation, duplicate JSON, symlink, size, real-input hash and external
fresh-output guards.

The wider affected physical-sound metadata suite passes `57/57` tests and the
new runner/tests compile with Python 3.11. `content-package` passes with `123`
records and `64` chunks. `git diff --check` and direct validation of changed
local Markdown links pass.

The mapped `boundary-scan` still reports the pre-existing
`SOURCE_LAYOUT_ESCAPE_HATCH` at
`tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`.
S0a adds no new boundary finding; this known repository-level finding remains
outside the metadata resolver's scope.

## Decision and next boundary

S0a is complete. It opens only V15-S0b, the YCB metadata/cost capability
adapter. S0b may inspect publisher metadata and archive identities, but no
audio body or signal. S0c role freeze remains blocked until both S0a and S0b
are complete.

Do not reuse the old N1b `1/7/18` unexposed counts: those were conservative
pre-alias source rows, not revision-aware physical groups. The current exact
metadata counts are `2/0/12` Glass/Wood/Metal.
