# Physical sound PS-2N0 classical baseline export

| Field | Value |
|---|---|
| Date | 2026-08-30 |
| Result | `N0.2_COMPLETE / Q30_STRUCTURED_EXPORT_PASS / TRANSFER_CONTROLS_FROZEN / REAL_DEVELOPMENT_REPEAT_PASS / NO_TRAINING_OR_ADMISSION` |
| Architecture | [SPEC-45](../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md), `Proposed` |
| Plan | [Neural acoustic field implementation plan](../plans/2026-08-30-physical-sound-neural-acoustic-field-implementation-plan.md), N0.2 |
| Runtime effect | None; external research tooling and a read-only lab snapshot |

## Question and answer

N0.2 first had to determine what the phrase “frozen Q30/DCT baseline” can
truthfully mean for a neural benchmark. The repository has two different
classical boundaries:

- a complete, exact 16-mode selected-glass Q30 renderer with a bounded onset
  transient;
- a frozen Python coloured-residual DCT protocol and boundary pipeline, but no
  hash-closed per-row DCT fit or exact cooked renderer output.

The new `xtask physical-sound-registry classical-baseline` command exports the
first boundary completely and reports the second as `FallbackOutOfDomain`.
It does not reconstruct missing DCT coefficients, approximate NumPy FFT
semantics or call a protocol freeze a usable prediction.

## External input and sealing boundary

Schema:

`nextengine.experimental-physical-sound-classical-baseline.manifest.v1`

The manifest binds an exact PS-2N0 `fit-projection.json` plus exactly one
sorted binding for every projected row. Each binding declares:

- the exact row ID;
- the requested frozen baseline profile;
- explicit Q16.16 excitation energy;
- the external reference WAV and its SHA-256.

The command accepts only the unsealed `train/development` projection with task
scope `exact_object_few_shot_impact_listener_field`. Projection and reference
artifacts are rehashed before use; the reference byte count and hash must equal
the row's projected audio identity. Calibration, method-holdout and
admission-shadow projections reject. Output must be an empty external
directory.

No model is trained, no threshold is selected and no shadow contents are
materialized.

## Exported Q30 representation

The presentation lab now exposes a read-only snapshot of the already-cooked
selected profile. Playback still consumes the private constants directly; the
snapshot is not a public contract or content schema.

`classical-profile.json` contains:

- 48 kHz stereo numeric profile identity;
- all 16 modes in frozen source-profile index order;
- exact signed-Q30 recurrence coefficients and initial samples;
- all 144 bounded onset-transient samples;
- exact integer scaling rules;
- source-profile SHA-256;
- the frozen DCT preregistration and boundary-pipeline source hashes plus its
  current fallback state.

Frozen identities:

| Artifact | SHA-256 |
|---|---|
| Structured Q30 profile | `19b051fe85be66847e0b8178c2f602cfec7d0f39bf6b6c2f893e0326f5fef181` |
| Full-energy selected Q30 WAV | `c912806ccbc24e9b2af04b80f9a030f1ec66a6f16321bd3addb3cabc1db9c823` |
| DCT preregistration source | `fe8a12f516ee8f85dc515c23cf2b7f67cc6bcc4d12a2b7f20c7f69d0626cebf5` |
| DCT boundary pipeline source | `34c16185790fac7dbccd00fcc00fd332a0917f6d071cbd70100aaeb4f7081e51` |

The exporter fails if any of these frozen identities changes.

## Row result and feature surface

A supported target row produces:

- a canonical WAV and raw S16LE PCM hash;
- exact row-only coverage metadata;
- sample rate, channel/frame counts and duration;
- peak/RMS and hard signal-failure tags;
- the existing general acoustic-validator feature vector;
- temporal-dynamics and amplitude-envelope feature vectors;
- exact WAV equality plus comparable-mono maximum error, RMS error and
  correlation against the bound reference.

The profile is never inferred from an object or material label. A manifest
must bind it to an exact projection row. Missing modal-field axes,
`reject_parent` rows or the DCT profile emit a row record with
`FallbackOutOfDomain` and no prediction/WAV. This preserves negative evidence
without partial success.

## Synthetic verification

Focused tests cover:

- two independent exports with byte-identical profile, records, report and
  WAV;
- exact preservation of the frozen selected Q30 WAV;
- canonical 16-mode ordering and 144-sample residual export;
- explicit DCT, missing-axis and reject-parent fallback with no emitted WAV;
- stale projection hash rejection;
- sealed-role projection rejection.

Focused result: `3 passed; 0 failed`. The presentation physical-sound suite
passes `16/16`; the complete physical-sound registry suite passes `148/148`.
Formatting and xtask all-target Clippy with warnings denied pass.

The mapped broad `boundary-scan` remains red on the pre-existing tracked
`#[path = "transfer_calibration/dsp.rs"]` escape hatch in
`realimpact_transfer_fixture.rs` (last changed by `a26f070f`). The new source
files contain no layout escape hatch and have physical line counts `845`,
`334` and `382`, all below the 1,000-line limit.

## Real-development transfer coverage

The [R0–R1 real evidence](physical-sound-neural-real-boundary-r0-r1-2026-08-30.md)
adds the separate transfer-domain surface without changing Q30 waveform
semantics. Eight even Green Goblet listener rows are context; seven odd rows
are queries. Nearest-listener and bracketing linear-segment predictions repeat
byte-identically. The better linear aggregate reaches `2.5373 dB` mean
absolute level error, `9.1745 dB` mean gain-matched multi-resolution spectrum
RMSE and `2.4129 dB` mean normalized waveform RMSE.

This is a baseline measurement, not quality credit. Several per-query phase
correlations remain near zero or negative, so direct sample interpolation is
not promoted as the learned representation.

## Allowed claim and remaining work

Allowed now:

`Q30_WAVEFORM_BASELINE_EXPORT_IMPLEMENTED / TRANSFER_LISTENER_CONTROLS_FROZEN`

Not allowed:

- neural training, quality, domain or admission credit;
- method-holdout/admission-shadow access;
- runtime content or production promotion.

N0.2 is complete for the current benchmark tasks: exact Q30 covers its
compatible waveform/synthetic surface, incompatible/missing-axis rows retain
explicit fallback, and the real transfer surface has its own frozen controls.
The missing DCT per-row cooker remains an explicit non-blocking fallback rather
than partial transfer credit.
