# Physical sound V40 I0 project-exposure result

Date: `2026-09-03`

Result: `COMPLETE / REPEAT_EXACT_PROJECT_EXPOSURE_AUDIT / SOURCE_POWER_OOD / ZERO_SIGNAL / NO_ROLE_AUTHORITY`

Roadmap authority: [Roadmap V40](../plans/physical-sound-synthesis-roadmap-v40.md)

## Claim

I0 is the fail-closed project/revision-family ledger for the current Steel
source frontier. It binds every project carried by V39 F1 to historical access
evidence, quarantines any project whose signal family was already opened and
reruns the unchanged whole-project planner over only metadata-clean projects.

I0 does not assign roles, open payloads, decode signal, train a model, release a
validator or authorize admission, cooking, demo integration, runtime inference,
a public contract or a material-quality claim. It authorizes only V40 D0's
permanent disclosed roster and S0's clean metadata-first source growth.

## Frozen implementation

- owner: `lab/scripts/physical_sound_v40_i0_project_exposure_v1.py`,
  `28,155 bytes`,
  `sha256=fe7895237f939db73c731db473cf3237ee819e09c99ff326e5d76faf0ac9f6ed`;
- profile: `lab/profiles/physical-sound-v40-i0-project-exposure.v1.json`,
  `9,319 bytes`,
  `sha256=874c4bc48ddd1145c7cd6a329d3a85512ead6ed2a658266c2decc9b822d98bfd`;
- tests: `lab/tests/test_physical_sound_v40_i0_project_exposure_v1.py`,
  `10,455 bytes`,
  `sha256=50a9aadb375d288a671b5aacb8c5bf98ee244ecfe67b8d847d9ebc4a28d21660`.

The canonical profile binds thirteen repository dependencies by exact path,
byte count and SHA-256. These include the V39 F1 owner/profile/result, the Q1a
metadata-only result and the historical evidence documents for every disclosed
family. Unknown origins fail closed and a metadata hash never resets family
exposure.

## Exposure result

The ledger accounts for all eleven V39 F1 candidate project revisions. Nine
historically opened families are permanently disclosed. Two of them occur in
the old candidate frontier and are quarantined from protected power:

| Candidate family | Quarantined power | Historical reason |
| --- | ---: | --- |
| ObjectFolder Real | `17 exact-Steel / 63 non-Metal` | PCM and derived signal values were already decoded |
| YCB Impact Sounds `bj5w8` | `6 exact-Steel / 7 non-Metal` | Eight recordings from this revision were already opened |

The remaining nine metadata-clean projects provide only:

| Frontier | Projects | Exact Steel | Non-Metal | Reserved |
| --- | ---: | ---: | ---: | ---: |
| V40 I0 clean pool | 9 | 9 | 7 | 5 |

The exhaustive whole-project partition is infeasible. Its best two protected
roles retain five reserve projects but expose these deficits:

| Role | Projects | Exact Steel | Non-Metal | Deficit |
| --- | ---: | ---: | ---: | ---: |
| A | 2 | 3 | 1 | `13 exact-Steel / 34 non-Metal` |
| B | 2 | 3 | 4 | `13 exact-Steel / 31 non-Metal` |

The clean-project root is
`6b072b22efb26257ff52e926129f96af3d2499be16ef03f6e2c30e78991aa15f`;
the partition root is
`dd83db7dfb744193886bf924a1b497d89aa2914dc9a1acbfa28455e4017e32af`.
Therefore I0 establishes the current frontier as `SourcePowerOOD`; it does not
open either protected role.

## External A/B evidence

External root:

`/home/kaifaty/.codex/experiments/nextengine/physical-sound/physical-sound-v40-i0-2026-09-03`

Fresh `run-a` and `run-b` contain identical four-file trees. Deterministic tree
root:

`1a0fa5cc317cf60e8221d11895048cff4f1ead4a3b813ae2f1384803ded1af37`

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `clean-frontier.json` | 4,624 | `2104bd7c94d769d01bf2990a20ee862d7e99797e31d4b77c425c6b26de097e58` |
| `profile.json` | 9,319 | `874c4bc48ddd1145c7cd6a329d3a85512ead6ed2a658266c2decc9b822d98bfd` |
| `project-ledger.json` | 10,576 | `37c600f40ba3c67c330e76392aa209487cb4b927787be5dd318b38f906dd4381` |
| `report.json` | 2,779 | `e840e5211d8822d3487d7d46111037ef1a6650c23e696b6c40c470b1f73420cc` |

Every counter for network, source payload, audio header/preview, PCM, force,
mesh, feature, model target, candidate output, protected signal and role signal
is zero. Repository/profile metadata reads are reported separately.

## Failure and boundary evidence

The combined V40 I0, V39 F0/F1 and Q1a Python suites pass `41/41`. Coverage
includes unknown-origin quarantine, exposure/material power, minimum/authority/
family/dependency mutations, duplicate and non-canonical JSON, in-repository,
occupied and symlink outputs, and atomic cleanup after late publication failure.
The owner imports no network, signal or model library.

The existing Rust neural data-plane suite passes `10/10` for role privacy,
evidence-lane closure, projection repeatability and atomic publication.

`cargo run -p xtask -- boundary-scan` remains red only on the pre-existing
`SOURCE_LAYOUT_ESCAPE_HATCH` in
`tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`.
I0 adds no Rust escape hatch and does not depend on that fixture.

## Decision and next action

I0 closes as repeat-exact
`I0_PROJECT_EXPOSURE_AUDIT_PASS_SOURCE_POWER_OOD`. V40 D0 may now freeze the
nine disclosed families into parent-disjoint generator-train,
generator-development and validator-calibration roles without returning any of
them to protected evaluation. In parallel, S0 must find genuinely new internet
project families against the exact clean deficits `13/34` and `13/31` before
S1 may assign protected roles.
