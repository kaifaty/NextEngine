# Physical sound V37 E0 full-surrogate seal result

| Field | Value |
| --- | --- |
| Date | `2026-09-03` |
| Result | `PASS / FULL_COUNT_D0_H0_REPEAT_EXACT / EXECUTION_SEALED` |
| Owner commit | `67802b1c` |
| Seal commit | `953a6a6f` |
| Owner | `lab/scripts/physical_sound_v37_e0_full_surrogate_seal_v1.py`, SHA-256 `0ec78599fad0619f87c410d8c5646bf54eeab5b1ed9049c630ece8cf2e93fbe5` |
| Profile | `lab/profiles/physical-sound-v37-e0-full-surrogate-seal.v1.json`, SHA-256 `f95ccc622430a6dd27661d34526c5970a722dbd40a717230d24200c62dd62f2a` |
| Seal | `lab/profiles/physical-sound-v37-e0-execution-seal.v1.json`, file SHA-256 `3057d9643d2101ff12b381b811306904f70656b885314e1f730213edc92484cf`, internal seal `608f901938c4031e63353a9231e005bb1040ac21c50ed3c7746e01525e211d7f` |
| External evidence | `/tmp/nextengine-v37-e0-official.Y7cfj1` |
| Authority | Full-count discarded zero-target execution only; no official, scientific-quality, real-material, validator-release, cooker, demo or runtime claim |

## Outcome

E0 executed the frozen full QSO shape through the V37 lifecycle and atomic
publisher. D0 materialized `6,480` train and `4,320` development rows, trained
the candidate, four QSO ablations and the V36-shaped pointwise control for the
frozen `96` steps each, and exercised all three non-neural controls. H0 then
rebuilt the identical `6,480` train rows, loaded the exact D0 bundle, opened
`4,320` method-holdout rows and evaluated all nine paths with zero training.

All role targets were artificial constant zeros used only to exercise cost,
gradient, serialization, prediction, metrics and gate paths. No F0 truth
formula was evaluated. Consequently E0 proves that the complete experiment is
executable and reproducible; it does not prove that QSO predicts physical
sound or beats any control on scientific values.

## Closed execution shape

| Item | Frozen/executed value |
| --- | ---: |
| total fresh cases represented | `1,512` |
| total canonical fields | `9,720` |
| total query rows | `15,120` |
| logical batch / gradient microbatch | `1,024 / 512` |
| neural paths / steps each | `6 / 96` |
| non-neural paths | `3` |
| combined neural parameters | `64,074` |
| D0 target-row receipt | `10,800` discarded zeros |
| H0 target-row receipt | `10,800` discarded zeros |
| H0 training steps | `0` |

D0 and H0 both returned natural surrogate `Pass`. D0/H0 train structural and
zero-target roots matched exactly. The binary bundle round-tripped every named
float64 tensor byte-for-byte, and H0 validated the D0 decision root, owner,
profile, freeze and weight hash before candidate loading.

## Independent A/B evidence

Two fresh full processes recursively matched across all output files, stdout
and stderr:

- elapsed: `39.73 s` / `40.36 s`;
- peak RSS: `806,372 KiB` / `825,816 KiB`;
- frozen ceiling: `300 s / 1 GiB / 64 MiB output`;
- top-level output: four files, `709,269` bytes;
- top-level tree:
  `e91aae7b7c0613aab4135c27739469a6ad38dfc6552074c422a97422f828d36b`;
- D0 rehearsal root:
  `b167ee9c0f71c4223adf2301aaa1ac8bb379c6a8a952d32bee702eb097303aa5`;
- H0 rehearsal root:
  `a846ed7d93508236e6b8122b498ea6210c4e05fe07a944af8ec727512827ae04`;
- top terminal:
  `369319b0be41a77ac526cae0b3e7c571c978c1e685e2021f5a3aa0a2330be75e`;
- stdout:
  `350dd21170e6e3f53e63feec6ce937f18252c9e5c031404b7f0c9cf9b3839dfd`;
- stderr:
  `dd7241fb9634d51d4b90a8782d9a26dd3fe2ada28ad873374b2d9b28b76827ef`.

Payload identities were:

| Artifact | SHA-256 |
| --- | --- |
| `candidate-bundle.json` | `c34ec6f7198b07adf15965c5ba1e18cf7464239220708601b7e201f184d055ce` |
| `candidate-freeze.json` | `aef543209600e2ddf421f3d9a131bd9fea5e6aed3ef6c25e8b0e13d56729a23c` |
| `evidence.json` | `047c8231a173a70f09fdb12bbbd93e0c061ee8b10b0fbb3267b4f34dc915521f` |
| `terminal.json` | `369319b0be41a77ac526cae0b3e7c571c978c1e685e2021f5a3aa0a2330be75e` |

## Seal and authorization

The checked-in seal is byte-identical to the output of the seal builder. It
binds owner, profile, publisher, Python/NumPy/Torch environment, D0/H0 topology,
both rehearsal roots, two-run exactness and forbidden-access count `0`.

Every run recorded zero official capabilities, official D0/H0 rows, F0 truth
evaluations, prior-generation numeric reads, network requests and
real/protected signal decodes. Generated weights and rehearsal trees remain
outside Git; only the compact seal is checked in.

Verification passed with Ruff `0.16.3` format/check, Python compile, strict
mypy `2.0.0`, `40/40` combined V37 A0–E0 tests and `161/161` focused xtask
physical-sound registry tests. `boundary-scan` reported only the known unchanged
`SOURCE_LAYOUT_ESCAPE_HATCH` in
`tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`;
E0 introduced no new boundary finding. No Cargo ProductCheck applies because
no cooker, demo or production consumer changed.

E0 closes Commit 5 and authorizes only implementation of a seal-verifying,
one-shot D0 provider/runner. It does not authorize H0 access: H0 may run only
after official D0 returns a repeat-exact natural `Pass` freeze. Independent S0
internet-source growth remains open at the unchanged `6 Steel / 27 non-Metal`
frontier.
