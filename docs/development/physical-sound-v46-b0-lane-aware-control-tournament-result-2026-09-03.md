# Physical sound V46 B0 — lane-aware control tournament result

| Field | Value |
| --- | --- |
| Date | `2026-09-03` |
| Status | `COMPLETE / REPEAT_EXACT / NO_USEFUL_TEACHER / CONTRACT_ISOLATED` |
| Decision | `NoUsefulTeacher` |
| Retained real floor | C0R global prototype, parent median RMSE `1.312187716` |
| Product effect | None; M0 remains closed and authored clips remain authoritative |
| Next | E0 real-source power and V0 independent-validator source growth |

## Result

B0 evaluated every comparison that the D1 claim masks permit and refused the
ones that would cross target contracts. Two complete external runs emitted the
same five files byte-for-byte.

The C0R train/development intersection reproduces its already disclosed simple
controls exactly. The unconditional global prototype remains the best real
control. It beats the fixed coarse-material ridge and retrieval copy on both
parent-median and project-balanced summaries:

| Control | Parent median RMSE | Parent mean RMSE | P90 RMSE | Project-balanced mean RMSE |
| --- | ---: | ---: | ---: | ---: |
| Global prototype | `1.312187716` | `1.300542984` | `1.548982183` | `1.326293656` |
| Ridge | `1.412043404` | `1.404286600` | `1.717672061` | `1.414528994` |
| Retrieval copy | `1.504515226` | `1.494768094` | `1.768173882` | `1.446525664` |

Those metrics cover 24 coarse-material-supported development parents in two
projects. Records were first aggregated into physical parents; projects then
received equal weight. The 70 WAV rows never became 70 independent votes.

This real floor does not make a teacher useful. B0 finds zero independently
scoreable non-real teachers:

- D0/D1 contain `0` trusted external modal-teacher rows;
- D1 contains `0` materialized structural-transfer rows;
- the 84 Clatter records still collapse to 36 modal priors, but no independent
  Recipe V3 modal truth exists against which to score them;
- the V31 analytic owner lacks compatible D1 geometry/contact inputs and a
  lossless mapping into the C0R 105-dimensional target;
- the 15 IETeasy NDAC-75 rows are train-only and have no independent
  same-contract development project.

Accordingly B0 returns `NoUsefulTeacher`, not a fabricated cross-contract
winner. The retained C0R number remains a disclosed regression floor; it does
not authorize M0 or model selection.

## Pre-access closure

The original pre-access implementation was committed as `3e219cb8`. Two
official metadata-only preflights then failed before any C0R target object was
opened:

1. D1 access counters were correctly present under the `counters` object, while
   the first adapter checked the top level; commit `c10a0113` corrected the
   parser and its fixture.
2. Two disclosed train parents intentionally span source projects as
   cross-source anchors; commit `bdb00f87` preserved them as one train parent,
   excluded them from project voting and retained the stricter one-project rule
   for every development parent.

Commit `bdb00f87` is therefore the final target-access seal. Neither preflight
reached target decoding, produced an output directory or exposed validator,
PCM, model or protected values.

| Sealed artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| [Profile](../../lab/profiles/physical-sound-v46-b0-lane-aware-control-tournament.v1.json) | `6,975` | `0385113ec983c54e08ce300b8e2f3b2685eb3eba242eda5d2c3bd3df04f5503d` |
| [Owner](../../lab/scripts/physical_sound_v46_b0_lane_aware_control_tournament_v1.py) | `38,252` | `4cd394a983e3a1395624a568116c6fde74fc10c0bdb8475d01757bb4d2fba0d2` |
| [Focused tests](../../lab/tests/test_physical_sound_v46_b0_lane_aware_control_tournament_v1.py) | `12,812` | `9497179771cc7e209cfa4138b8a63a2a215f6caf1e94f6fe9f7e0b57623703a8` |
| [Protocol](physical-sound-v46-b0-lane-aware-control-tournament-protocol-2026-09-03.md) | `7,210` | `82cf67104e2a74ef182faebf7aba77ec3785970f1669129bd40623c63a1a1684` |

## Bounded access

The final process opened exactly the allowed 134 unique C0R pseudo-targets:

| Role | Objects | Bytes |
| --- | ---: | ---: |
| Generator train | `64` | `327,466` |
| Generator development | `70` | `355,267` |
| Total | `134` | `682,733` |

It also read `1,036,602` bytes from six exact external metadata inputs and
rechecked `3,528` already admitted Clatter modal-control scalars. All forbidden
counters are zero: network, PCM/waveform, IETeasy target, validator projection
and target, protected values, models/checkpoints, candidate-training steps and
generated renders.

## Repeat-exact publication

The official external root is
`physical-sound-v46-b0-lane-aware-control-tournament-final-2026-09-03` under
the private experiment store. `run-a` and `run-b` have no recursive byte
difference:

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `access.json` | `675` | `c94c103a0e2cee1ea773af028f23fa940d02d7330e1c4d3954a083adc28bc08a` |
| `control-metrics.json` | `59,737` | `6f9ad7573fe98753dce8126bdd47eab0fe0557a04a1bcf82f51b3302d1cd51b9` |
| `lane-matrix.json` | `1,823` | `f0deeb70635e30f9f99b6c27c45924faec2d42d99df74a53cb80ff83d331f5ed` |
| `profile.json` | `6,975` | `0385113ec983c54e08ce300b8e2f3b2685eb3eba242eda5d2c3bd3df04f5503d` |
| `report.json` | `4,863` | `46db4251459525585f22b5b751db8030cf3fa44f581e33f8a4c9e03eb63b658a` |

Nine positive publication gates pass. The tenth gate,
`teacher_usefulness_independently_scoreable`, is deliberately `false` and
drives the frozen `NoUsefulTeacher` decision; it is not relabelled as a passing
quality claim.

## Verification

- focused B0 suite: `PASS`, `5/5` tests;
- combined V45 R0/T0/C0 plus V46 D0/D1/B0 suite: `PASS`, `43/43` tests;
- final external A/B and recursive byte comparison: `PASS`;
- Ruff `0.14.1` check/format and Python compile: `PASS`;
- profile/hash drift, D1 access nesting, target-contract crossing, Clatter
  mutation, cross-source train-anchor handling, unsafe input/output and
  forbidden-import guards: `PASS`;
- `cargo run -p xtask -- boundary-scan`: `FAIL` only on the pre-existing
  `SOURCE_LAYOUT_ESCAPE_HATCH` in
  `tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`;
  B0 adds no source-layout escape hatch and receives no boundary-scan credit;
- runtime ProductChecks: `NOT_RUN`, because B0 is an external research tool
  under Proposed SPEC-45 and changes no production consumer or public contract.

## Consequence

V46 does not start M0 with zero trusted supervision. The next independent work
packages are E0 and V0: find and qualify published internet sources that add a
second descriptor-to-signal project, close the real-parent deficit and provide
independent validator calibration. A future exact synthetic or structural
source may reopen a fresh teacher preflight, but neither Clatter labels nor a
cross-contract conversion may substitute for it.

Real support remains `71/105`, deficit `34`. PSEL/B1, real fitting, LabWinner,
protected admission, cooker, demo and runtime promotion remain closed. Authored
clips remain mandatory.
