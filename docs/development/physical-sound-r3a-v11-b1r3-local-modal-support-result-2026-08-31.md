# Physical sound R3A V11 B1R3 — local modal-support result

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Protocol | [B1R3 local modal support](physical-sound-r3a-v11-b1r3-local-modal-support-protocol-2026-08-31.md) |
| Status | `REPEAT_EXACT_REJECT / DEVELOPMENT_FAILED / HOLDOUT_UNOPENED` |
| Decision | `REJECT_LOCAL_MODAL_SUPPORT` |
| Product effect | None; authored clips remain authoritative |

## Outcome

B1R3 proves that the local-support direction is materially better than the
previous broad-band mask, but the frozen revision does not pass. Its explicit
six-mode reconstruction predicts twelve unseen-force development responses at
`0.007180` mean NRMSE and `0.011006` maximum NRMSE. It recovers every admitted
frequency within `0.120 Hz` and every admitted decay within `1.15%` relative
error. It also rejects the weak-excitation and uncalibrated low-coherence
controls as intended.

The run loses the `6,643 Hz` truth mode because the complete fit excitation
bank has insufficient local input energy there: median local input SNR is only
`1.207…1.341`, local discovery coverage is `0.008…0.042`, and zero contacts
support the pole. The same incidental hole also makes the positive comb-notch
control report two unsupported modes (`6,643 Hz` and the intentional
`8,000.061 Hz` notch), not the preregistered single unsupported mode.

The development spectrum endpoint is `2.011288 dB`, narrowly above its frozen
`2.0 dB` gate. No threshold is repaired. Development fails, holdout is never
generated, and no real source, ML, validator, atlas or runtime work is opened.

## Exact identity and accounting

Two complete runs are byte-identical for every JSON and NPY output.

| Artifact | SHA-256 |
| --- | --- |
| Frozen manifest | `00961dab43aab921c899759d3e7aeb5938bd6f582bf3e762197e1cfe3d31e638` |
| Repeated preflight report | `672b959cea36a13a32fe29eefaace03f780d4fc44e8ce936cadbf13a66a1b34f` |
| Model | `f841be2e7e9ca74bcc066c83395ed073a8d2dc9aa0136d2dca2ffe0af9de60e5` |
| Run report | `69145ba7db9855e83085fc7cca33ca9a81b5269eceb1176054647ec28ec8912b` |
| GTLS transfer | `8cfcf9f2395efe6512b7b1fcb3be9d8297fde4dba0b0b6f69a652b28b977e21b` |
| Modal transfer | `bd5aa9dac383c9e64f951f56b68586f495b267b343fd4452f71f3d4e4f0a9d6e` |
| Input SNR | `908cb96c5114e9f6a067d2e8771a02943e6a4836825028a10cac6869d220e9db` |
| Corrected coherence | `c910d5332989f3783f77ed2e8548b8fe8d667cf4dbf86c0359c3bca5a3b6a7a4` |
| Discovery-valid mask | `58014bab4f8aa58cdc36e47b3d197cf82600ac1aaa2153cdfcdcd6d8653e254c` |
| Modal reconstruction | `e13147a14f65df5d4f73ba06d4cc5e6b59fb6a2fb86744d28a5b3eb56b29d201` |

External roots are `r3a-v11-b1r3-local-support-freeze`, paired preflights and
`r3a-v11-b1r3-local-support-run-a/-b` under the external physical-sound store.

The run evaluates `96` fit trials, `12` development trials and `72`
development-control trials. It separately accounts for `9,216,000` samples in
each fit sensor-noise channel and `9,216,000` unmeasured-room samples. It
generates zero holdout trials, reads zero real/parent-holdout samples and makes
zero network requests.

## Measurements

| Endpoint | Gate | Observed | Result |
| --- | ---: | ---: | --- |
| Noiseless identity NRMSE | `<= 1e-11` | `0` | pass |
| Supported / unsupported truth modes | `7 / 0` | `6 / 1` | **fail** |
| False-positive modes | `0` | `0` | pass |
| Maximum frequency error | `<= 1 Hz` | `0.119071 Hz` | pass |
| Maximum absolute damping error | `<= 1.5/s` | `0.559205/s` | pass |
| Maximum relative damping error | `<= 0.20` | `0.011412` | pass |
| Mean / maximum held NRMSE | `<= 0.08 / 0.15` | `0.007180 / 0.011006` | pass |
| Mean spectrum RMSE | `<= 2 dB` | `2.011288 dB` | **fail** |
| Candidate / impulse NRMSE | `<= 0.70` | `0.012762` | pass |
| Candidate / raw-output-modal NRMSE | `<= 0.80` | `0.007572` | pass |
| Candidate / raw-H1 NRMSE | `<= 1.50` | `0.244136` | pass |
| GTLS / best corrected H1/H2 | `<= 1.05` | `1.000102` | pass |

The retained truth indices are `0,1,2,3,4,6`; the only missing ordinary mode
is index `5` at `6,643 Hz`. Every retained mode has six supporting contacts and
six confident residues.

## OOD controls

- Weak excitation returns `OOD_UNSUPPORTED_MODAL_BAND`, supports zero modes and
  admits no complete-domain model.
- Uncalibrated dynamic interference returns `OOD_LOW_COHERENCE`, supports zero
  modes and admits no complete-domain model. B1R2's calibration leak is fixed.
- The positive comb-notch control finds indices `0…4`, but both index `5` and
  the intentionally notched index `6` are unsupported. It therefore returns
  the frozen failure label `INVALIDLY_ADMITTED` instead of the expected
  `OOD_UNSUPPORTED_MODAL_BAND`. Its complete-domain admitted mode count remains
  zero; the failure is incorrect support attribution, not a shipped unsafe
  model.

## Bounded conclusion

The estimator is now accurate where the measured force actually identifies a
mode. The remaining failure is not evidence that a larger neural decoder is
needed. It exposes a missing layer in the experiment contract: acquisition
identifiability, learned object modes and query-force relevance were treated as
one support decision.

A successor must use a fresh truth fixture and make those scopes explicit:

1. certify modal-band coverage from the complete calibration-force ensemble
   before fitting or scoring object modes;
2. select fresh truth modes from predeclared covered bands, plus separately
   declared unsupported controls, rather than assume smooth pulses are
   broadband;
3. require recovery of every acquisition-supported mode and zero invented
   modes;
4. evaluate a held query by the energy it can produce through the already
   identified transfer model; a harmless force notch is not itself an object
   model failure;
5. retain the uncalibrated-interference and missing-impact mismatch controls.

Roadmap V12 permits exactly one fresh coverage-certified synthetic revision.
It also allows a zero-decode internet-source inventory in parallel because that
inventory cannot expose PCM or estimator roles. Real waveform access remains
blocked until the oracle passes.

## Verification at result capture

- B1R3 focused Python suite: `8/8 PASS`;
- two complete result roots: byte-identical for all eight artifacts;
- holdout/real/parent-holdout counters: zero;
- broader repository checks are recorded with the V12 roadmap commit.
