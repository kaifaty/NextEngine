# Physical sound V34 H0 method-holdout result

| Field | Value |
| --- | --- |
| Date | `2026-09-02` |
| Decision | `REPEAT_EXACT_METRIC_REJECT / V34_FAMILY_CLOSED` |
| Owner commit | `93f43a41b04391f81c53631c780e78bafb3083cd` |
| Owner SHA-256 | `0c0a7230abe2598ebc7eb6988cead8848f135cf2d7c6e770527c1250f7f1bae0` |
| Frozen candidate weights SHA-256 | `831fd0a78d0d470f6ba54605d52eb0cff0b88aac483b1a35de4564f679fd80bd` |
| Claim | `SYNTHETIC_FROZEN_CANDIDATE_ONE_SHOT_METHOD_HOLDOUT_ONLY / NO_RETRAINING_REAL_MATERIAL_QUALITY_VALIDATOR_RELEASE_ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY` |

## Outcome

The frozen H0 owner loaded the exact D0 candidate/raw-control weights, rebuilt
the disclosed train role byte-identically to its D0 manifest and opened the
method holdout once. Two fresh CPU processes publish the same seven artifacts
and stdout byte-for-byte. Both return `MetricReject`; no H0 candidate freeze is
published and V34 closes.

The result is narrow but binding. Eleven of twelve holdout metric gates pass.
The sole failure is contact transfer to unseen geometry at contacts already
represented in train:

| Geometry-only contact | RMSE / ratio | Gate |
| --- | ---: | ---: |
| Frozen V34 candidate | `0.00117399` | — |
| Nearest train row | `0.00066832` | best control |
| Candidate / best control | `1.756639x` | `<=0.98x` — **fail** |

The candidate remains absolutely accurate in this stratum (`0.001174 <=
0.050`) but the preregistered rule requires it to beat the best frozen control.
The miss cannot be averaged away by stronger results elsewhere.

## Passing holdout evidence

| Metric | Candidate / best relevant control | Gate |
| --- | ---: | ---: |
| Aggregate | `0.616078x` | `<=0.90x` |
| Decay | `0.537618x` | `<=0.95x` |
| Global gain | `0.719425x` | `<=0.95x` |
| Contact aggregate / best non-neural | `0.523720x` | `<=0.95x` |
| Contact aggregate / raw MLP | `0.359954x` | `<=0.95x` |
| Contact-only / best control | `0.525600x` | `<=0.98x` |
| Joint / best control | `0.519231x` | `<=0.98x` |

Candidate RMSE is `0.00069120` decay, `0.01046028` global gain and
`0.01108693` contact. Contact-only/joint RMSE is `0.01561904 / 0.01565150`;
all aggregate/stratum absolute bounds pass. The nearest control's unusual
strength is localized: its geometry-only contact error is `0.00066832`, while
its aggregate contact error is `0.02116958`.

All nine hard gates pass with `666` exact nodal rows and maximum corrected
render peak `0.03988737 < 0.95`. Branch isolation, hash closure, resource gates
and atomic terminal publication pass. Thus the failure is scientific, not an
execution/protocol defect.

## Access, no-retraining and resources

- exactly `6,480` disclosed train rows are deterministically rebuilt for
  ridge/nearest controls;
- exactly `4,320` method-holdout rows open;
- `3,302` frozen candidate/raw-control parameters load from D0 artifacts;
- model training steps remain exact zero;
- real/protected signal and network access remain exact zero;
- Run A uses `68.63 s / 766,700 KiB`; Run B uses
  `67.51 s / 777,456 KiB`; owner wall/RSS gates pass.

## Exact artifact closure

| Artifact | SHA-256 |
| --- | --- |
| `control-report.json` | `4155e5c99de9e5b54b4974c73815740846b7839ac3fc79ff35d246150808b783` |
| `evidence.json` | `4208cb951a1aa2dfadcf13ff1455fc40fbdc8ad83d2049f18243674f135a879a` |
| `h0-decision.json` | `4c0ead9191a613366cd41e1f8d86619ae70f0b2c688c8bdde646b4dde42a6bc5` |
| `method-holdout-manifest.json` | `833ec8d666d93ad93edfb1914fedd665b71e0d3f2a15627ae2bd31aa29827674` |
| `method-holdout-predictions.bin` | `1c4f4ddd39c141d41e1dac653ca01999a76c188559c83ee500d93899c953c46a` |
| `owner-evidence.json` | `3c9edbd63948317f509a24ca373b70ae4fc11aa23c411e6426aa35054b0ce3a7` |
| `report.json` | `91d6006acb8ec0d84266e57c11c65b3f5d2ac0e101a6dfac215f3821d8b64ec6` |

The deterministic closure is `383,166` bytes. Stdout is the canonical report
and has the same SHA-256 as `report.json`.

## Conclusion and next boundary

V34 is permanently closed. Its train, development and method-holdout roles are
spent; no contact basis, width, seed, step, loss, threshold or geometry split
may be changed and tested against them. D0's development pass remains valid
historical evidence, but it does not authorize real training after H0 rejects.

The repeated failure cluster is now specific enough for bounded research:
V32 lost contact development to nearest on fresh geometry, while V34's spectral
candidate fixed development and new-contact/joint holdout but still lost the
geometry-only holdout to nearest. A successor must explain geometry-conditioned
contact transfer rather than add another generic capacity/seed variant. It
requires a materially distinct preregistered representation and fresh roles.

P1 and authored clips remain the complete product fallback. The independent
internet-source lane may continue, but V1/M2/M3/A0/K0/D1 remain blocked.
SPEC-45 stays `Proposed`; no real-material, naturalness, validator, cooker,
runtime or ProductCheck claim is created.

## Verification

- Ruff format/check and Python compile: `PASS`;
- focused V34 H0 Python suite: `PASS`, `5/5` tests;
- official process A/B: `PASS / REPEAT_EXACT_TERMINAL_REJECT`, `7/7` artifacts
  and stdout;
- focused xtask physical-sound registry suite: `PASS`, `161/161` tests;
- `git diff --check` and direct documentation/link/task-state checks: `PASS`;
- `cargo run -p xtask -- boundary-scan`: `FAIL`, the known unchanged
  `SOURCE_LAYOUT_ESCAPE_HATCH` in
  `tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`;
- no ProductCheck applies because no production consumer or promoting ADR
  exists.
