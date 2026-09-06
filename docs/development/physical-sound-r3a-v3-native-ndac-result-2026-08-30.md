# Physical sound R3A V3 — native-48 kHz NDAC result

Date: 2026-08-30
Status: `COMPLETE / REPRODUCIBLE / REJECT_LEARNED_CODEC_REPRESENTATION / NO_R3B_AUTHORITY`

## Decision

The corrected R3A V3B oracle returns
`REJECT_LEARNED_CODEC_REPRESENTATION` for the exact native-48 kHz NDAC-75
candidate published with FlowDec. The identity control passes all five
endpoints exactly and both complete runs are byte-identical, so this result is
not confounded by sample-rate conversion.

NDAC preserves absolute level, envelope and decay on the Purple Scoop
development contact, but fails the frozen spectral and modal-frequency limits.
It improves only three of five endpoints over nearest fit, while the gate
requires at least four, every absolute limit and aggregate improvement.

This rejects one general-audio codec at one frozen bitrate and checkpoint. It
does not reject neural physical sound, FlowDec's stochastic postfilter or a
task-specific modal field. It authorizes neither neural contact-field training
nor deterministic distillation. Authored clips remain mandatory.

## Why V3 used NDAC

R3A V2 could not distinguish codec error from its 48→44.1→48 kHz control. The
official [FlowDec repository](https://github.com/facebookresearch/FlowDec)
publishes the underlying deterministic NDAC-75 checkpoint for general audio at
48 kHz and documents ten codebooks as 7.5 kbps. The stochastic FlowDec
postfilter was deliberately excluded. The
[FlowDec paper](https://openreview.net/forum?id=uxDFlPGRLX) is prior art and
dependency provenance, not evidence of this exact physical-transfer claim.

The source is [REALIMPACT](https://github.com/samuel-clarke/RealImpact). Each
selected waveform is its published force-deconvolved transfer response, not a
raw microphone recording and not a universal material exemplar.

## V3A fail-closed infrastructure result

The first native-rate revision froze `73_PlasticBin` before target access:

- preflight manifest `e556ddbc…3801f` and report `0916f8a7…5c9e` repeated
  byte-identically;
- extraction report `b259994c…d3fb` and four-contact array
  `ed152004…168b` repeated;
- extraction stopped before row `2407`; sealed samples decoded: `0`.

The oracle then failed before publishing a report or quality metrics. For an
exact 144,000-sample input, the official decoder produced 143,992 samples and
the shape mismatch stopped evaluation. Plastic Bin development row `1807` is
therefore opened invalid infrastructure evidence, not a codec-quality result.

The repair was derived without a new target: a 144,000-sample synthetic
control showed the same eight-sample deficit. Appending exactly eight zeros
before the official preprocess yields 144,632 decoded samples; slicing the
original 144,000 samples and all ten-codebook arrays repeats exactly. The
repair changed code lineage, so it was not applied post-hoc to Plastic Bin.
V3B froze a new source instead.

## Frozen V3B protocol

- Source: source-disjoint REALIMPACT `23_PurpleScoop`, exact archive headers,
  central directory, metadata members and mesh closed before audio.
- Listener: azimuth `0`, distance offset `0`, microphone `7`.
- Split: rows `7`, `607`, `1207` fit; row `1807` query-seeing representation
  development; row `2407` sealed field holdout.
- Signal: float32 force-deconvolved response at native 48 kHz, absolute peak
  aligned to sample `512`, first three seconds scored, fit-only peak scaling.
- Candidate: FlowDec-release NDAC-75 `800k`, ten 1024-entry codebooks,
  640-sample hop, no sample-rate conversion and no FlowDec postfilter.
- Guard: append eight zero samples before official preprocessing, require
  decoded length at least 144,000, then slice exactly 144,000 samples.
- Baseline: Euclidean nearest fit contact.
- Identity control: the exact float32 target copy must return zero on every
  endpoint.
- Endpoints: absolute RMS level, gain-matched multiresolution spectrum over
  `120–18,000 Hz`, normalized envelope, modal-frequency median and decay T60.
- Reproducibility: CPU, one intra/inter-op thread, deterministic Torch, MKLDNN
  disabled, synthetic warm-up, then exact two-repeat arrays.

No Purple Scoop audio payload was read before two byte-identical preflights
fixed source, roles, code, checkpoint, environment, guard and thresholds.

## Exact evidence

| Artifact | Exact identity |
| --- | --- |
| V3B preflight manifest | `57dcc3b3f56e22582e7e73161812204d1f3d1b03bfbc38611cfa4711525d8959` |
| Repeated preflight report | `845d45983c74b547a2470c3583bab0d867720715f952c80426783b6673a966d5` |
| FlowDec revision/tree | `26ec106281f061a720e0399b0edfd3c3c020d7ab` / `671e689063e0c90d3c9c3a5766192097f4bbcf5b` |
| Official checkpoint archive | `1205002207` bytes; `4f8c72ae264f32be583ed7d4fcc60704af8c9ab200dc0e7d8414dec57f4d163c` |
| NDAC-75 weights | `258100682` bytes; `4eb9d0daf1d5f0efa0c890b8902ac1d40d8d18b331b6b05940d03f172fab08f0` |
| Synthetic guarded control | codes `2eee2b4b…984d`; decoded `4b3afc3e…0a3c` |
| Compressed REALIMPACT member | `2546414608` bytes; `03c5aff94edf03c7c2f97cc80c21cff95ed153fc067ba544c69adc0608ba2228` |
| Repeated extraction report | `7f2d7de4fe8a667de6ae7f11a5a5a342357f35443030dfa0017f053b56907164` |
| Repeated four-contact array | `2dcab65966fc7a09f9cab188eb644af34014e9ed25a2a7a42ddcb1a9fe060b3c` |
| Repeated oracle report | `bcc24ef9b3525aa0420c1c3a68777aea45ae83f975c4a538fd52468693fcf8e1` |
| Repeated NDAC audition WAV | `a6d668fffc692dec6b25bf83abb591bbef5c6edaf2981ff2eb93cbbab3c72b97` |

The extractor decoded `1,670,447,488` uncompressed bytes and stopped before
the sealed boundary at `2,223,875,568`; sealed waveform samples decoded: `0`.
The checkpoint exposed exactly three allowlisted pickle globals before
`torch.load`. The native identity control is exact zero on every endpoint.

## Metrics

Lower is better.

| Endpoint | Nearest fit | NDAC-75 | Candidate limit | Beats nearest |
| --- | ---: | ---: | ---: | --- |
| Absolute RMS level error, dB | 3.8162 | 0.2535 | 0.5 | yes |
| Gain-matched spectrum RMSE, dB | 9.3709 | 12.1198 | 4.0 | no |
| Normalized envelope RMSE | 0.00905 | 0.00374 | 0.20 | yes |
| Modal-frequency median, cents | 426.30 | 560.81 | 100 | no |
| Decay T60 relative error | 0.2950 | 0.03908 | 0.35 | yes |

NDAC encodes the three-second target as `2,260` ten-bit codes: `2,825`
ideal packed bytes and `7,533.33` effective bits/s including the guard frame.
Its shared decoder is `258.1 MB` and remains report-only.

## Interpretation and next discriminator

The clean native-rate result changes the research direction. A general
perceptual waveform codec can preserve macroscopic loudness/envelope/decay yet
move or smear the narrow resonances that define a physical transfer response.
Trying another nearby bitrate or general codec on the opened target would be
the same failed family, not a new hypothesis.

The next Roadmap V6 discriminator is a task-specific modal bottleneck:

1. use only already-opened development contacts to compare at most three fixed
   capacity points for an object-global pole bank, contact-specific complex
   gains and a deterministic learned multiresolution residual basis;
2. require the representation and deterministic inverse together to preserve
   all five endpoints; a neural waveform decoder cannot receive cooker credit;
3. freeze the smallest passing method once, then evaluate it on one new
   unopened object; no new object is spent during method exploration;
4. only a passing sealed representation authorizes an AV-MSF/ObjectFolder-
   shaped neural field that predicts location-dependent gains and residual
   coefficients, not PCM.

This follows the factorization used by
[AV-MSF](https://arxiv.org/abs/2608.05145), where frequencies and damping are
object-global while a neural field predicts spatial gains and an explicit
residual, and by
[ObjectFolder 2.0](https://openaccess.thecvf.com/content/CVPR2022/papers/Gao_ObjectFolder_2.0_A_Multisensory_Object_Dataset_for_Sim2Real_Transfer_CVPR_2022_paper.pdf),
whose AudioNet predicts location-dependent modal quantities instead of direct
spectrograms. [NeuralSound](https://arxiv.org/abs/2108.07425) and
[DiffSound](https://arxiv.org/abs/2409.13486) remain synthetic/physics teachers
and controls; they do not establish the real-data claim by themselves.

## Do not retry

- unguarded NDAC on any exact three-second target;
- guard repair or any quality tuning on opened Plastic Bin row `1807`;
- another bitrate, postfilter, normalization, threshold or endpoint selected
  from opened Purple Scoop row `1807`;
- another general neural codec as if perceptual quality implied modal fidelity;
- opening either row `2407` to rescue a failed representation;
- treating NDAC weights or codes as a deterministic runtime cooker.

## Verification

- Ruff format/check: passed for all V3A/V3B runners and focused tests.
- Focused unittests: V3A `7` passed; V3B `8` passed.
- Python compileall: passed.
- Two V3B preflights, extractions, oracle reports and WAV sets: byte-identical.
- V3B identity control: exact zero on all five metrics.
- `git diff --check`, links and boundary scan are recorded at final handoff.
