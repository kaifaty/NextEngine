# Physical sound V34 D0 development tournament result

| Field | Value |
| --- | --- |
| Date | `2026-09-02` |
| Decision | `PASS / REPEAT_EXACT / CANDIDATE_FROZEN / H0_AUTHORIZED` |
| Owner commit | `5232098f2744177be43b953554a8ace8df2e0f54` |
| Owner SHA-256 | `1b0f0968e693fc0716861989f2ccd89407864e2912a15cb45658bde6d47e6add` |
| F0 profile SHA-256 | `7be7e96a2899d0e4ef7aad668368669886d7f45adac34ec6676afbbe6b45e429` |
| Frozen candidate weights SHA-256 | `831fd0a78d0d470f6ba54605d52eb0cff0b88aac483b1a35de4564f679fd80bd` |
| Claim | `SYNTHETIC_FRESH_V34_TRAIN_DEVELOPMENT_TOURNAMENT_ONLY / NO_METHOD_HOLDOUT_REAL_MATERIAL_QUALITY_VALIDATOR_RELEASE_ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY` |

## Outcome

The frozen V34 D0 owner ran in two fresh CPU processes. Both return `Pass`,
publish the same ten artifacts and stdout byte-for-byte, and freeze exactly one
candidate for H0. Train/development access is complete and immutable; method
holdout remains exact zero.

This is the first valid positive result for the mode-local spectral residual
hypothesis. V33 never measured it because the protocol failed before artifact
publication. V34 closes both protocol defects before training and then shows
the fixed candidate beats every preregistered contact control without changing
the V33 topology, optimizer, seeds, steps, controls, metrics or thresholds.

## Development metrics

| Metric | Candidate | Gate/result |
| --- | ---: | --- |
| Aggregate normalized RMSE | `0.03127291` | all identity/nearest/raw-ridge ratios pass |
| Decay RMSE | `0.00079584` | absolute and both control ratios pass |
| Global-gain RMSE | `0.00594602` | absolute and both control ratios pass |
| Contact RMSE | `0.01158893` | `<=0.035`, pass |
| Contact / raw MLP | `0.326978x` | `<=0.80x`, pass |
| Contact / nearest | `0.447549x` | `<=0.85x`, pass |
| Contact / raw ridge | `0.275813x` | `<=0.85x`, pass |
| Contact / spectral ridge | `0.292571x` | `<=0.90x`, pass |

| Contact stratum | RMSE | Ratio to best frozen control | Gate |
| --- | ---: | ---: | ---: |
| Contact-only | `0.01630689` | `0.442101x` | `<=0.045` and `<=0.95x` |
| Geometry-only | `0.00159053` | `0.690928x` | `<=0.045` and `<=0.95x` |
| Joint | `0.01631682` | `0.450654x` | `<=0.045` and `<=0.95x` |

Aggregate ratios are `0.107846x` identity, `0.263412x` nearest and `0.191935x`
raw ridge. Decay is `0.061032x / 0.296570x` raw ridge/nearest; global gain is
`0.159758x / 0.165132x`. Every frozen comparison passes without averaging away
a failed branch or stratum.

## Ablations and hard gates

All declared ablations worsen the intended branch:

- full spectral/mode-local lift zero: `4.014073x`;
- local stencil zero: `3.174720x`;
- contact context zero: `4.184861x`;
- material context zero: `12.315522x`.

All nine P1 hard checks pass: finite/bounded corrections, exact frequency/order,
exact impulse ratios, positive decay, remesh identity, sign preservation,
render peak and nodal zero. Development contains `585` exact nodal modal rows;
maximum corrected render peak is `0.03910257 < 0.95`. Branch isolation is exact
for contact-vs-decay/gain and material-vs-contact.

## Access, training and resources

- `6,480` train and `4,320` development target/feature rows materialize;
- exactly `3,302` candidate plus raw-control parameters initialize;
- candidate/raw-control each train for `1,200` full-batch CPU steps with seeds
  `3301/3302`;
- candidate loss moves `0.07661233 -> 0.0000200143`; raw MLP moves
  `0.28228081 -> 0.00359560`;
- method-holdout target rows, real/protected signal and network requests remain
  exact zero;
- Run A uses `96.05 s / 886,892 KiB`; Run B uses
  `95.45 s / 888,356 KiB`; owner wall and RSS gates pass.

## Exact artifact closure

| Artifact | SHA-256 |
| --- | --- |
| `candidate-freeze.json` | `2bf800a99c8d9a42e2555c6fa818c6ed4149ee23dcf380e7ec3e10c616d605f7` |
| `candidate-weights.bin` | `831fd0a78d0d470f6ba54605d52eb0cff0b88aac483b1a35de4564f679fd80bd` |
| `control-report.json` | `04dd843aa9e6112fe1f14691ed8649541074a9e300358de9df96142956fb0a70` |
| `corpus-manifest.json` | `20c1555f2320a69dc68695589edda072edd0315cdc664c64ea2124b3d15fb6d4` |
| `d0-decision.json` | `ae74ccc23e6f973afc9e7aad5580d717b84915630072133f8560a5f307ecdeca` |
| `development-predictions.bin` | `e71f7de4bf58e8888fc8eec1506c1ddd2bed14868420184a1c64dd0d07aed0b2` |
| `evidence.json` | `7770efe58a83245dd7a80a86aa1a1af03a1d4651859cacb1a7085c6233bcdcd4` |
| `owner-evidence.json` | `db5b3cd41c56a0c246d2ece78de704ed5933a3fcb0b511a9a0bd3e1bb59c5e0b` |
| `raw-mlp-control-weights.bin` | `220bf5c18b3436030771b97bb05865eeea0f904d1e825da986e07528ec9f3ba6` |
| `report.json` | `e4fd015c02b755078f6b5597b2f919d30b6205bd91176af3ceb9d75c68807b83` |

The deterministic artifact closure is `395,437` bytes. Stdout is the canonical
report and has the same SHA-256 as `report.json`.

## Conclusion and next boundary

D0 passes and authorizes exactly one H0 method-holdout opening. H0 must bind the
candidate freeze, weights, controls, owner/profile and D0 result hashes; it may
not retrain, alter preprocessing or inspect holdout before its own owner commit.
The complete holdout runs twice only to prove deterministic reproduction of the
same one-shot decision. Any H0 miss closes V34 permanently.

This remains synthetic representation evidence, not a Steel/Glass/Wood or
naturalness claim. Real validator/source gates, offline cooking and authored
fallback remain unchanged. SPEC-45 stays `Proposed` and no ProductCheck credit
is earned.

## Verification

- Ruff format/check and Python compile: `PASS`;
- focused V34 D0 Python suite: `PASS`, `5/5` tests;
- official process A/B: `PASS / REPEAT_EXACT`, `10/10` artifacts and stdout;
- focused xtask physical-sound registry suite: `PASS`, `161/161` tests;
- `git diff --check` and direct documentation/link/task-state checks: `PASS`;
- `cargo run -p xtask -- boundary-scan`: `FAIL`, the known unchanged
  `SOURCE_LAYOUT_ESCAPE_HATCH` in
  `tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`;
- no ProductCheck applies because no production consumer or promoting ADR
  exists.
