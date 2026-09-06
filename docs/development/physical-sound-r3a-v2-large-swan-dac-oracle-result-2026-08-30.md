# Physical sound R3A V2 — Large Swan learned-codec oracle result

Date: 2026-08-30
Status: `COMPLETE / REPRODUCIBLE / INCONCLUSIVE_RESAMPLING_CONTROL / NO_R3B_AUTHORITY`

## Decision

R3A V2 returns `INCONCLUSIVE_RESAMPLING_CONTROL`. It neither supports nor
rejects learned compact representation in general, does not satisfy
`READY_FOR_EXACT_OBJECT_FIELD` and authorizes no neural field training.

The exact Descript Audio Codec (DAC) candidate is also a negative diagnostic
on this opened slice: after the same sample-rate conversion it passes only the
envelope absolute threshold and beats nearest contact only on decay. Because
the no-codec resampling control already fails two primary endpoints, those
candidate failures cannot become a representation-gate conclusion.

Authored clips remain mandatory. Large Swan development row `1807` is now
opened negative/inconclusive evidence; row `2407`, the method holdout and the
admission shadow remain sealed.

## Why this experiment

R3A V1 showed that a hand-written 32-mode plus sparse-DCT record and an
equal-budget sparse DCT do not carry a real development contact. A bounded
research cycle therefore tested a materially different learned codec on a new
object whose payload had never been opened in this project.

Primary-source findings frozen before target audio access:

- [AV-MSF](https://arxiv.org/html/2608.05145v2) factors object impact sound
  into object-global frequencies/damping, contact-conditioned gains and a
  residual, but its project code was still unavailable; it is prior art, not a
  reproducible dependency here.
- The official [DAC repository](https://github.com/descriptinc/descript-audio-codec)
  publishes a universal 44.1 kHz 8 kbps model and weights. Revision
  `c7cfc5d2647e26471dc394f95846a0830e7bec34`, tag `0.0.1` weights and CPU
  execution were selected as a query-seeing representation ceiling.
- [REALIMPACT](https://github.com/samuel-clarke/RealImpact) provides the
  official 50-object roster and 48 kHz force-deconvolved transfer responses.
  `79_LargeSwanCeramic` was absent from all prior project access manifests.
- [ObjectFolder Real](https://objectfolder.stanford.edu/objectfolder-real-download)
  remains the richer 30–50-impact source, but its audio is distributed in
  large ten-object archives and was not needed for this discriminator.

Inference: a passing DAC result could only show waveform representation
capacity. Its 306.7 MB neural decoder is neither a bounded deterministic
cooker nor runtime authority under SPEC-45.

## Frozen protocol

- Source: REALIMPACT `79_LargeSwanCeramic`, exact archive identity and central
  directory hash closed before audio.
- Listener: azimuth `0`, distance offset `0`, microphone `7`.
- Contacts: source-order rows `7`, `607`, `1207` are fit; row `1807` is
  query-seeing representation development; row `2407` is sealed field holdout.
- Signal: published float32 force-deconvolved transfer response at 48 kHz,
  absolute peak aligned to sample `512`, first three seconds scored, fit-only
  peak scale applied unchanged to development.
- Candidate: DAC 44.1 kHz 8 kbps, nine 1024-entry codebooks; exact polyphase
  `147/160` downsample and `160/147` upsample with Kaiser beta `5`.
- Control: the identical roundtrip resampling without DAC.
- Baseline: Euclidean nearest fit contact.
- Endpoints: absolute RMS level, gain-matched multiresolution spectrum in
  `120–18,000 Hz`, envelope, modal-frequency median and decay T60.
- Reproducibility: CPU, one intra/inter-op thread, deterministic Torch,
  MKLDNN disabled, one synthetic warm-up discarded, then exact two-repeat
  arrays required.

No target waveform was read before two byte-identical preflight reports fixed
the source, split, implementation, environment, model and thresholds.

## Exact evidence

| Artifact | Exact identity |
| --- | --- |
| Preflight manifest | `6821c49b428e37abf0903f89fc6a2bdddbdb3ad7b11dc82463c9122abaff0a4c` |
| Repeated preflight report | `2b0663506ce0a766c43e348b534542b39cd58b30ce861957898d32d85c526e3b` |
| DAC code revision/tree | `c7cfc5d2647e26471dc394f95846a0830e7bec34` / `30fa80b4387fe5cd53a7c85d0bea945acb4f5dca` |
| DAC weights | `306717287` bytes; `a88eed82a7024ccc1facdb1e605c4c2f99281c8118c22c9895ffa846d8fb61aa` |
| Compressed REALIMPACT member | `2317727372` bytes; `93fafa3d9a3808781c35cd85315b63a170206610af4814b29e609f567403d636` |
| Repeated extraction report | `79dc70ba358b3085184f3ebe30f057d95bb961e0ff6691841a787944fca17f1d` |
| Repeated four-contact array | `f0ce1461642f9cf20a6cc89d58479f4162cd8665881e605c109583fde9aecb8d` |
| Repeated oracle report | `c1bb54dd9d392f98717f2904ad46ea3b5a1195338f38be2aa04366be0d7711f2` |
| Repeated DAC audition WAV | `1b576ffcf67080b7d9b761e65faadd7669ae9ac98d9d7e953cc400926d9c26de` |

The extraction decoded `1,513,028,544` uncompressed bytes and stopped before
the sealed boundary at `2,014,302,892`; sealed waveform samples decoded: `0`.
The checkpoint archive exposed exactly three allowlisted pickle globals before
`torch.load`. The synthetic control and both target reconstructions repeat
exactly after warm-up.

## Metrics

Lower is better.

| Endpoint | Nearest fit | Resample only | DAC | Candidate limit |
| --- | ---: | ---: | ---: | ---: |
| Absolute RMS level error, dB | 1.6308 | 1.8956 | 5.1562 | 0.5 |
| Gain-matched spectrum RMSE, dB | 8.2418 | 0.0646 | 9.5356 | 4.0 |
| Normalized envelope RMSE | 0.01761 | 0.01166 | 0.02697 | 0.20 |
| Modal-frequency median, cents | 72.59 | 0.00 | 257.25 | 100 |
| Decay T60 relative error | 16.2579 | 5.8438 | 15.5598 | 0.35 |

The resampling control fails its tighter level and decay limits. DAC encodes
the three-second target as `2,331` ten-bit codes (`2,914` ideal packed bytes,
`7,770` effective bits/s), but its decoder weights are a shared 306.7 MB neural
dependency. DAC is worse than nearest fit on four of five endpoints and fails
four candidate absolute thresholds.

## Interpretation and negative knowledge

The protocol declared a `120–18,000 Hz` evaluation band for spectral/modal
comparisons while level and T60 still measured the full waveform. The real
transfer response contains enough out-of-band/high-frequency tail energy that
44.1 kHz conversion alone changes those full-band endpoints materially. This
is a preflight design error exposed by the successful control, not permission
to relax thresholds or redefine endpoints after seeing row `1807`.

Do not retry:

- DAC 44.1 kHz on the opened Large Swan development row with post-hoc
  band-limited level/decay or relaxed gates;
- another bitrate, resampler beta, normalization or model selected from this
  opened target;
- treating a learned codec reconstruction as the deterministic runtime cooker;
- opening Large Swan row `2407` to rescue the hypothesis.

## Verification

- Ruff format/check: passed for the four V2 runners and focused test.
- Focused unittest: `6` passed.
- Python compileall: passed.
- `git diff --check` and direct changed-link/path/identifier validation: passed.
- Two preflights, extractions and oracle reports: byte-identical.
- `cargo run -q -p xtask -- boundary-scan`: failed only on the pre-existing,
  unchanged `SOURCE_LAYOUT_ESCAPE_HATCH` in
  `tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`;
  this change adds no source-layout escape hatch.

## Roadmap V5 discriminator

R3A V3 must use another unopened development projection and freeze one of:

1. the deterministic underlying NDAC-75 codec from the official
   [FlowDec](https://github.com/facebookresearch/FlowDec) release, subject to
   code/checkpoint/environment and exact-repeat preflight; its stochastic
   FlowDec postfilter is excluded from this representation gate; or
2. if no such dependency is reproducible, a band-limited target domain whose
   filter, all five band-limited endpoints and a separate out-of-band energy
   loss are fixed before audio access.

The preferred NDAC path is trained for general 48 kHz audio and preserves the
source sample-rate contract. A passing learned ceiling still authorizes only a
separate compact deterministic modal/multiresolution distillation gate. R3B
remains blocked until that bounded cooker representation passes on a new
development contact.
