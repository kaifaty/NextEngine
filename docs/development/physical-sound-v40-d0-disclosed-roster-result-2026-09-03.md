# Physical sound V40 D0 disclosed-roster result

Date: `2026-09-03`

Result: `COMPLETE / REPEAT_EXACT_DISCLOSED_ROSTER_FROZEN / 5_TRAIN_2_DEVELOPMENT_2_VALIDATOR / ZERO_SIGNAL / C0_AUTHORIZED`

Roadmap authority: [Roadmap V40](../plans/physical-sound-synthesis-roadmap-v40.md)

## Claim

D0 permanently assigns all nine project/revision families disclosed by I0 to
exactly one of three ML-development roles. The assignment is frozen before a
new generator, validator threshold or candidate output exists, so later quality
cannot move a convenient source between training, development and validator
calibration.

D0 does not retrieve payloads, decode signal, compute features, train a model,
release a validator or open a protected role. It authorizes only a separately
profiled C0 disclosed-corpus build and the already-open metadata-only S0 source
search. Admission, cooking, demo integration, runtime inference and public
contracts remain false; authored fallback remains mandatory.

## Frozen implementation

- owner: `lab/scripts/physical_sound_v40_d0_disclosed_roster_v1.py`,
  `25,823 bytes`,
  `sha256=9211c5b11ed4e962ca5a91f1191b788703a7015a8ffa3f66e9cb91de56fb7fda`;
- profile: `lab/profiles/physical-sound-v40-d0-disclosed-roster.v1.json`,
  `6,621 bytes`,
  `sha256=2415b96ceea4252d68e9c8b11365138bd3d8dba725304da6b34629e8c2b03e7a`;
- tests: `lab/tests/test_physical_sound_v40_d0_disclosed_roster_v1.py`,
  `9,925 bytes`,
  `sha256=c901ef1ba3708a3a654687d875406f7404b3ab9e3df03606b27cc2f8475ae205`.

The profile binds seven repository dependencies by exact path, size and
SHA-256: I0 owner/profile/result, V39 F0 owner/profile/result and the historical
project-disjoint split result. It also binds the old partitioned-manifest
receipt `9ddec04d…26c33` without requiring that external corpus to reproduce D0.

## Assignment rule

D0 preserves the historical project split wherever it still has a disclosed
meaning:

- old `dev` remains `generator_development`;
- old `calibration` remains `validator_calibration`;
- former `holdout` and `shadow` are already exposed, so they collapse into
  `generator_train` and can never become protected again;
- REALIMPACT was previously used for development and therefore enters only
  `generator_train`.

This produces the immutable roster:

| Role | Families |
| --- | --- |
| `generator_train` | Freesound wasserbjorn pack 41981; Kaffekrus SoundPacks; REALIMPACT; Kronland; AV-MSF |
| `generator_development` | Freesound ascap pack 14905; YCB Impact Sounds |
| `validator_calibration` | CMU AuditoryLab Impact Events; ObjectFolder Real |

Every descendant object, recording, feature and mutation must inherit its
family role. A cross-family parent alias rejects C0; it cannot be repaired by a
filename, object ID, metadata hash or new adapter. Generator and validator
learned artifacts remain in separate namespaces, candidate outputs are hidden
from calibration and thresholds cannot be shared with generator selection.

## Independence result

| Property | Exact result |
| --- | ---: |
| I0 disclosed families assigned once | 9 |
| Generator train families | 5 |
| Generator development families | 2 |
| Validator calibration families | 2 |
| Protected families opened | 0 |
| Clean I0 families spent | 0 |
| Unknown or unassigned families | 0 |

Each family has a canonical commitment; each role has a root over its ordered
family commitments. The full disclosed-roster artifact has SHA-256
`6d6f29fa2e2292bb5f69acde401284e7fc5a012061f01db15ee1f1d722480de9`.
C0 must bind this exact root before reading any disclosed payload.

## External A/B evidence

External root:

`/home/kaifaty/.codex/experiments/nextengine/physical-sound/physical-sound-v40-d0-2026-09-03`

Fresh `run-a` and `run-b` contain identical four-file trees. Deterministic tree
root:

`cca47d7c425f5f0c42beeb6fddb47a66f7702df1a5614b3b2c1d4c0bed4f576c`

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `access-ledger.json` | 7,791 | `60e3fb2725a8869d9cfaa6113da23fba7f220c73d7db348efc0733c0627747a7` |
| `disclosed-roster.json` | 10,142 | `6d6f29fa2e2292bb5f69acde401284e7fc5a012061f01db15ee1f1d722480de9` |
| `profile.json` | 6,621 | `2415b96ceea4252d68e9c8b11365138bd3d8dba725304da6b34629e8c2b03e7a` |
| `report.json` | 2,975 | `806e2708c3938fa002bb31f299ac464631a7346e64a5a00fbcad592684d832bb` |

Every current-run counter for network, source payload, audio header/preview,
PCM, force, mesh, feature, model target, candidate output, protected signal and
role signal is zero. Historical exposure is carried as lineage metadata, not
misreported as new D0 access.

## Failure and boundary evidence

The combined D0/I0/F1/F0/Q1a Python suites pass `49/49`. D0 covers role-map,
count, authority and dependency drift; clean-family aliasing; unknown,
duplicate and non-canonical input; occupied, in-repository and symlink outputs;
and cleanup after late publication failure. The owner imports no network,
signal or model library.

The existing Rust neural data-plane suite passes `10/10` for evidence-lane
closure, role privacy, repeatability and atomic publication.

`cargo run -p xtask -- boundary-scan` remains red only on the pre-existing
`SOURCE_LAYOUT_ESCAPE_HATCH` in
`tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`.
D0 adds no Rust escape hatch and does not depend on that fixture.

## Decision and next action

D0 closes as repeat-exact `D0_DISCLOSED_ROSTER_FROZEN_C0_AUTHORIZED`. C0 is now
the next sequential milestone: retrieve only these disclosed families into an
external content-addressed store, normalize canonical `48 kHz` segments,
preserve observed-axis masks and derive deterministic modal/transient targets.
C0 must publish immutable train/development/calibration projections against the
D0 roster root or fail atomically. It grants no model-training authority until
its lineage, capability and role-isolation gates pass.
