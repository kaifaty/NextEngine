# Physical sound R3A V5 neural preflight A — 2026-08-31

Status: `REPRODUCIBLE / READY_FOR_TRAINING_ENVIRONMENT_FREEZE /
TRAINING_NOT_AUTHORIZED / DEVELOPMENT_UNREAD / CLIP_FALLBACK_REQUIRED`

## Result

The first V5 boundary is complete. A hash-closed internet impact corpus,
task-specific convolutional encoder/RVQ/decoder, three latent capacities,
loss contract and deterministic CPU controls are frozen. Two external runs
produce byte-identical manifests and reports and return
`READY_FOR_TRAINING_ENVIRONMENT_FREEZE`.

This is infrastructure evidence, not sound-quality credit. No source waveform
sample was decoded into a numeric training tensor, no real optimization step
ran and none of the four representation-development contacts, row `2407`,
method holdout or admission shadow was read. Neural training remains explicitly
unauthorized until the GPU environment and runner controls are frozen.

## Frozen boundary

- implementation commits: `6ffe17f25e59e873c5789e8d92d73b71770c8da7`
  and report-field correction `512b35ddbb6c5a271326efbed1e737fa89c19014`;
- Heller/CMU `Impacts_audio1.zip`, `38,059,995` bytes, SHA-256
  `1d57964b4f48d277b6cf567a0bce938b3b8901f6b43ca6787db1036e8d737783`;
- research-only source manifest SHA-256
  `6377ea4019ca37ada7a9ca1c1a6e8bff71647a83d2b8d9ea39ee074caf31567e`;
- `73` PCM16 stereo 44.1 kHz clips in `17` event groups, used only as
  unlabeled impact-reconstruction evidence;
- fixed group-disjoint split: `56` train clips and `17` internal-validation
  clips; no label is promoted to material or object truth;
- 48 kHz mono convolutional encoder/decoder with periodic activations,
  128-dimensional latent, 1,024-entry codebooks and RVQ capacities of
  `4/8/16` quantizers;
- frozen waveform, SI-SDR, complex-STFT, log-spectrum, log-mel,
  envelope/decay, target-peak and RVQ losses; adversarial training remains
  disabled for the first bounded reconstruction test;
- canonical external JSON is the primary run record. A later MLflow mirror is
  optional, local-file-store only and never the lineage authority;
- runtime inference, model-registry promotion and public content contracts are
  forbidden. Authored clips remain the mandatory fallback.

The group split is derived from a fixed SHA-256 seed. Its four validation
groups are `Hammer on Plywood`, `Knocking over Can on Table`, `Marble on
Mirror` and `Metal Rods`; the other thirteen groups are training-only.

## Exact repeated evidence

The final pair is stored externally as
`r3a-v5-neural-preflight-final-a` and
`r3a-v5-neural-preflight-final-b` under the physical-sound experiment root.
Both files in both runs compare byte for byte.

| Artifact | SHA-256 |
| --- | --- |
| Manifest | `1cc234962963232a1c33c2b7c9973a13788743d07225c77073dbfbfc0b60fb31` |
| Report | `38176a41f1ad31dac6ce188daf3b7a5bdcbd623551290ade7195fb4a71793bf3` |
| Sorted manifest/report checksum list | `3ccea1249728d62547be4c78d5f8150fe8ad72fbfbccda1f4ce0053a946611b3` |

The CPU control environment is CPython `3.11.15`, NumPy `1.26.4`, SciPy
`1.11.4` and PyTorch `2.0.1+cpu`, with OpenBLAS, OMP and MKL thread counts
fixed to one. The full model contains `8,291,169` parameters and repeats exact
codes and output for a fixed synthetic input.

The micro-codec control also repeats exactly. Across each 80-step run its
reconstruction L1 changes from `0.3420518339` to `0.0513037592`, an improvement
ratio of `0.1499882595` against the frozen maximum `0.45`.

| Capacity | Nominal rate | Three-second code bytes | Sidecar |
| --- | ---: | ---: | --- |
| `rvq-6kbps` | `6 kbps` | `2,254` | one float32 output gain |
| `rvq-12kbps` | `12 kbps` | `4,504` | one float32 output gain |
| `rvq-24kbps` | `24 kbps` | `9,004` | one float32 output gain |

These are code-shape bounds, not measured trained quality and not deployable
asset sizes. The external decoder/checkpoint cost does not enter runtime
because the first product route bakes ordinary clip assets offline.

## What remains before real training

Preflight A deliberately does not freeze a GPU/PyTorch environment, decode the
internet waveform payloads, implement the production loss runner or create a
checkpoint. The next boundary must therefore:

1. freeze an external CUDA/PyTorch environment and exact hardware/software
   fingerprint;
2. implement bounded deterministic corpus decoding, group-aware sampling and
   the complete frozen loss without changing the manifest architecture,
   capacities or selection policy;
3. prove finite forward/backward, non-collapsed RVQ use, exact checkpoint
   save/reload and repeat inference on synthetic/tiny-corpus controls;
4. emit canonical JSON lineage and optionally mirror it to a local external
   MLflow file store;
5. keep real long training, development reads and holdout access disabled.

Only a passing training-environment/runner boundary may authorize the three
frozen capacity runs. It still grants no permission to tune on development,
open a new object or add runtime neural inference.

## Rejected interpretations

- The `73` inventoried clips are not `73` training clips; the fixed role split
  is `56/17`.
- Header parsing and payload hashing are not waveform decoding or training.
- Synthetic overfit proves trainability of a small control, not reconstruction
  quality of the 8.29-million-parameter model.
- Nominal latent rate does not prove the current `64 KiB` contact budget or
  spectral/modal quality gate.
- `READY_FOR_TRAINING_ENVIRONMENT_FREEZE` is not authorization for a long run,
  development evaluation, holdout access or engine integration.

## Smallest next action

Freeze the external GPU training environment and V5 runner, then execute only
the finite/reload/codebook/tiny-corpus controls. Do not read the four
development contacts and do not start the three real capacity runs in that
commit.
