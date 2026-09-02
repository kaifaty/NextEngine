# Physical sound V32 M1 known-truth tournament result

| Field | Value |
| --- | --- |
| Date | `2026-09-02` |
| Decision | `REPEAT_EXACT_DEVELOPMENT_REJECT` |
| Scientific consequence | `COMPACT_PHYSICS_LOCKED_RESIDUAL_FAMILY_CLOSED` |
| Owner commit | `f81e6de04acd64f934fcecc053ffd4e2b4397134` |
| Owner SHA-256 | `8475767e8c9fd085a95bbeeefe4c6ffca767f9b201b1b6ecd2f6fc73e4f10407` |
| M1 mechanics profile SHA-256 | `789e222aeb1107335632e85a931347fe4c236e372e7cef20e16b00f8ce895cc0` |
| Repeat comparison SHA-256 | `97e7232e4fa2eda8aba60d4e185af4082f8db223dd1ec86a04a043e430969a2e` |
| Claim | `SYNTHETIC_KNOWN_TRUTH_TOURNAMENT_ONLY / NO_REAL_MATERIAL_QUALITY_VALIDATOR_RELEASE_ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY` |

## Outcome

The frozen M1 owner ran in two fresh CPU processes against the same committed
owner and profile. All six published artifacts match byte-for-byte, including
candidate weights, development predictions, corpus/control reports, evidence
and the terminal report. Both runs return `REJECT_DEVELOPMENT`.

The candidate passes every hard physics-locked gate and 13 of 14 development
metric gates. The only failure is the contact branch against nearest-train-row:

| Development comparison | Result | Gate |
| --- | ---: | ---: |
| Aggregate / identity | `0.064124x` | `<= 0.85x` |
| Aggregate / ridge | `0.136037x` | `<= 0.85x` |
| Aggregate / nearest | `0.164946x` | `<= 0.85x` |
| Decay / ridge | `0.058240x` | `<= 0.90x` |
| Decay / nearest | `0.286361x` | `<= 0.90x` |
| Global gain / ridge | `0.176732x` | `<= 0.90x` |
| Global gain / nearest | `0.139861x` | `<= 0.90x` |
| Contact / ridge | `0.073138x` | `<= 0.90x` |
| Contact / nearest | `1.112179x` | `<= 0.90x` — **fail** |
| Material-zero ablation | `10.324437x` | `>= 1.05x` |
| Contact-zero ablation | `30.840003x` | `>= 1.05x` |

Candidate development RMSE is `0.00061238` decay, `0.00904536` global gain
and `0.00228381` contact. Nearest contact RMSE is lower at `0.00205345`.
Although the absolute contact error is small and the candidate strongly beats
identity/ridge, the preregistered tournament requires every branch to beat both
ridge and nearest. The result therefore cannot be promoted by averaging away
the local failure.

## Hard and access gates

- finite output, exact correction bounds, positive decay and all P1 frequency/
  ordinal bit identities pass;
- nodal zero and non-zero signed-gain sign are preserved;
- every common-contact remesh comparison and exact `0.5/1/2` impulse scaling
  pass;
- maximum corrected development render peak is `0.03747392 < 0.95`;
- T0 mutation/reason and V0a parent identities remain hash-exact;
- exactly `3,600` train and `720` development modal rows materialize;
- method holdout remains unopened at exact zero rows;
- network requests, real signal and protected signal values remain exact zero;
- no `candidate-freeze.json` or `method-holdout-report.json` is published.

The absence of holdout access is part of the result, not missing work. Once
development rejected, the owner had no authority to inspect holdout.

## Repeatability and resources

All six artifacts repeat exactly:

| Artifact | SHA-256 |
| --- | --- |
| `candidate-weights.bin` | `dbf39977c04062f9d2eafdebbd289efb1199b3c036fc83555e8eab825c68946f` |
| `control-report.json` | `66b7b9c80beba891e223b45f03257e9df45885b26e3b9559eda52e06fed61b93` |
| `corpus-manifest.json` | `bc5e518b595561ab0b8a9c0ca000915e23dede40efb6e54c769f3450381a9361` |
| `development-predictions.bin` | `a7abba88b1509599b901dd40fc942ac9d395859cd97687745f8456cca07fa2de` |
| `evidence.json` | `f3a20a9efee0d11228f4e8149616d419c196b5d6ec2b41acc92025a666924732` |
| `report.json` | `36349532caf65baa9f810b213d238ecd293f907f4febe43e873d2745a92b1ad9` |

Run A uses `28.89 s` outer wall and `878,832 KiB` maximum RSS. Run B uses
`28.67 s` and `871,984 KiB`. Each deterministic artifact closure is `59,936`
bytes. Both processes remain below `300 s`, `1 GiB` and `64 MiB`.

## Conclusion and next boundary

The failure is narrow but scientifically binding: the compact generic contact
MLP does not beat a local retrieval control on fresh geometry cells. The
opened development role cannot choose another seed, width, step count,
threshold, contact feature or loss. M0/M1 is closed and M2 real training is not
authorized by this lineage.

P1 remains valid classical physics research machinery and authored clips
remain the complete product fallback. A successor requires a new
preregistered hypothesis and fresh synthetic roles. The smallest justified
research question is whether contact correction needs a geometry/mode-local
spectral representation rather than another generic coordinate MLP. The
independent S0 internet-source lane may continue, but source growth alone does
not repair this representation reject or authorize M2.

SPEC-45 remains `Proposed`. This result creates no public schema, real-material
quality, validator release, admission, cooker, demo, runtime model or
ProductCheck claim.

## Verification

- `uv run --project lab python -m unittest
  lab.tests.test_physical_sound_v32_m0_physics_locked_residual_v1
  lab.tests.test_physical_sound_v32_m1_known_truth_tournament_v1`: `PASS`,
  `9/9` tests;
- official process A: completed with terminal scientific reject;
- official process B: completed with the same terminal scientific reject;
- owner repeat comparator: `PASS / REPEAT_EXACT`, `6/6` artifacts;
- `cargo run -p xtask -- boundary-scan`: `FAIL`, the known pre-existing
  `SOURCE_LAYOUT_ESCAPE_HATCH` in unchanged
  `tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`;
- no current ProductCheck is earned because SPEC-45 has no production
  consumer or promoting ADR.
