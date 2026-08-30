# Physical sound neural real boundary and transfer baseline R0–R1

| Field | Value |
|---|---|
| Date | 2026-08-30 |
| Result | `R0_COMPLETE / R1_TRANSFER_CONTROLS_FROZEN / BYTE_IDENTICAL_REPEATS / R2_NOT_RUN / SHADOW_SEALED` |
| Architecture | [SPEC-45](../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md), `Proposed` |
| Roadmap | [Neural physical sound](../plans/physical-sound-synthesis-roadmap.md), R0–R1 |
| Runtime effect | None; external research tooling and evidence only |

## Question and answer

R0 asked whether published internet data could be projected without confusing
ordinary impact recordings with force-deconvolved transfer responses, filling
missing axes or exposing the admission shadow. R1 asked for a classical
listener-field control that is mathematically compatible with that transfer
signal before any neural training.

Both boundaries now have repeated real-data evidence. The result does not show
that a neural model works. It provides the exact split, inputs, controls and
primary endpoints against which the first model may be judged.

## R0 — real transfer slice

`physical-sound-registry realimpact-neural-slice` consumes the previously
verified Green Goblet 15-listener block at one fixed impact. It validates the
source manifest/report, block and row hashes, finite samples and row peaks
before output creation. One shared raw peak renders every row to mono PCM16,
so listener-relative amplitude is retained while absolute excitation credit
remains unavailable.

Frozen identities:

| Artifact | SHA-256 |
|---|---|
| Source listener manifest | `fcf44d41bdd54ad3bc9df27b6c8ccc4d5ccd470a1f786641e64461e794c850de` |
| Source acquisition report | `cef5d381f5a6d9a6a1666503a8ed7e44c70758cd4e6f9e636f8001a111d57680` |
| Source f32 listener block | `8bcffd0a9f57fd101a803228f7e8aa66d469f82d99ad6e43950318aae0f875ca` |
| Rendered slice manifest | `fbaf79763f67f4e6e04617da733d0de6f8e259571467caa75b7ef32a6ed9cb87` |
| Rendered slice report | `c32b1f4ce40528f5f9b4d919842a893448840f5935bb0fac1ea808067767ae8f` |

Two independent output directories are byte-identical for the manifest,
report and all 15 WAVs. The slice claim is limited to relative multi-listener
force-deconvolved transfer. It grants no absolute-amplitude, physical identity,
quality, model-training, admission or runtime authority.

## R0 — five-role projection V2

The V2 neural data-plane schema adds explicit
`recorded_impact_waveform`/`force_deconvolved_transfer_response` semantics and
retains a published impact point when outward normal is absent. The real
projection combines five independent publisher/source groups:

| Role | Source | Rows | Signal semantics |
|---|---|---:|---|
| train | AV-MSF glass object | 2 | recorded impact waveform |
| development | REALIMPACT Green Goblet | 15 | force-deconvolved transfer response |
| calibration | YCB wineglass | 2 | recorded impact waveform |
| method holdout | SoundPacks drinking glass | 2 sealed | recorded impact waveform |
| admission shadow | Heller/CMU glass vase | 2 sealed | recorded impact waveform |

The projection manifest hash is
`e5615f73e1e926a3fb1e623fbe178aec069ac9d0b32d6bf5be33718cc1f795a8`.
Two projections are byte-identical with these output hashes:

| Output | SHA-256 |
|---|---|
| `fit-projection.json` | `07c7fafaff1fa067647a12a38b910e69fd5e56c11ae0a150eceb3edfb2eb93eb` |
| `calibration-projection.json` | `6c75c11fa2efe08be2f50b64320e9fac46178a6e73ad0c11bb3f42066101d3c1` |
| `report.json` | `920373576302145db6c20308107c72cdb0c5ad8fda32ccc7ebca4936bd31bbbe` |
| `sealed-role-commitments.json` | `02def139881fcb5d18c5d0b64ac647ffefe26af29decbcae8bff7f0125fc0815` |

The leakage audit reports no cross-role object, source, recording-parent,
condition, mutation-parent or identical-audio reuse. Capability counts are
honest: audio `23`, recorded waveforms `8`, transfer responses `15`, material
`8`, impact/listener coordinates `15`, and geometry/support/excitation/complete
modal field `0`. Method-holdout and admission-shadow row contents are not
materialized.

## R1 — frozen listener-field controls

`physical-sound-registry transfer-field-baseline` accepts the unsealed fit
projection plus exact WAV bindings. It requires development target rows with
transfer semantics, the same object/impact, mono PCM16 format and a common
sample grid. Every query listener must be bracketed by two compatible context
listeners before output creation.

The Green Goblet split uses even microphone IDs as eight context rows and odd
IDs as seven queries. Two controls are frozen:

1. nearest known listener, with row-ID tie breaking;
2. sample-synchronous linear interpolation on the shortest bracketing listener
   segment.

Manifest `32353f036c68bf005deb26b4e4f68bf4ad051ba516af55dcfde3b02f5553b751`
and report `ee92855df82c70a61b43c8c99409dd7a581dddd0a1dd61953ace4777ac8f942c`
repeat byte-identically together with all 14 prediction WAVs.

| Control | Mean absolute level error | P95 absolute level error | Mean gain-matched MR log-spectrum RMSE | P95 spectrum RMSE | Mean normalized waveform RMSE |
|---|---:|---:|---:|---:|---:|
| nearest listener | `2.8510 dB` | `8.5090 dB` | `9.9771 dB` | `12.6548 dB` | `3.1073 dB` |
| linear segment | `2.5373 dB` | `4.7094 dB` | `9.1745 dB` | `10.9250 dB` | `2.4129 dB` |

Linear interpolation wins every aggregate listed above, but it does not win
every individual query. Several waveform correlations are near zero or
negative, showing that listener-dependent phase is not captured by direct
sample mixing. This is the intended hard baseline, not evidence that a learned
field will pass.

## Frozen R2 decision rule

Metric profile `transfer-listener-field-r1-v1` was fixed before neural
training. Its five primary endpoints are mean/P95 absolute level error,
mean/P95 gain-matched multi-resolution log-spectrum RMSE and mean normalized
waveform RMSE. R2 may pass only if one frozen candidate is strictly lower than
both controls on every primary endpoint with identical rows, preprocessing and
unweighted per-query aggregation. Per-query distributions remain published;
a pooled mean cannot hide listener failures.

This rule is deliberately conjunctive. Failure produces
`REJECT_LISTENER_FIELD` or `DATA_INSUFFICIENT`, not threshold changes or an
unbounded tuning sweep.

## Allowed claim and next action

Allowed now:

`R0_REAL_DATA_BOUNDARY_COMPLETE / R1_TRANSFER_CONTROLS_FROZEN`

Not allowed:

- trained-model or learned-quality credit;
- unseen impact, geometry, support or excitation claims;
- method-holdout or admission-shadow access;
- validator, admission, public content or runtime promotion.

The smallest next action is R2/N0.3: preregister one compact fixed-impact
listener-field model and its modal-only/residual ablations, then train it on
the eight context rows and evaluate the seven frozen queries without changing
the R1 metric profile.
