# Physical sound R3A V5 training runner preflight — 2026-08-31

Status: `REPRODUCIBLE / READY_FOR_FROZEN_CAPACITY_TRAINING /
DEVELOPMENT_UNREAD / HOLDOUT_UNREAD / CLIP_FALLBACK_REQUIRED`

## Result

The external CUDA/PyTorch environment and complete V5 reconstruction runner
controls are frozen and reproducible. Two final runs produce byte-identical
manifests, reports and checkpoints and return
`READY_FOR_FROZEN_CAPACITY_TRAINING`.

The boundary validates explicit PCM decoding, resampling/alignment, every
frozen loss term, finite forward/backward, active RVQ use and exact checkpoint
continuation. It authorizes only the three preregistered `6/12/24 kbps`
capacity runs. It does not authorize development reads, holdout access,
architecture/loss/capacity changes or runtime neural inference.

## Frozen training environment

- implementation commit:
  `90984de202442b316da928165bde8107b0b59a9d`;
- CPython `3.11.15`, NumPy `1.26.4`, SciPy `1.11.4`;
- PyTorch `2.12.1+cu130`, CUDA runtime `13.0`, cuDNN `92000`;
- NVIDIA GeForce RTX 3080, UUID
  `GPU-8a3af62f-30db-ce95-caf9-201a67bf5810`, compute capability `8.6`,
  driver `610.43.02`;
- physical memory reported by `nvidia-smi`: `10,240 MiB`; memory reported by
  PyTorch: `9,871 MiB`. Both values are preserved instead of incorrectly
  requiring two APIs with different semantics to agree;
- installed package inventory SHA-256:
  `31aa8f7eedb9089bb9b9d5fc56b0d2521b44f62d34bf1c3c196bcc166b59ea89`;
- `CUBLAS_WORKSPACE_CONFIG=:4096:8`, deterministic algorithms enabled,
  cuDNN benchmark and TF32 disabled.

The environment choice follows the official
[PyTorch previous-version matrix](https://pytorch.org/get-started/previous-versions/),
which publishes the stable 2.12.1 CUDA 13.0 wheel. The
[PyTorch 2.12 release note](https://pytorch.org/blog/pytorch-2-12-release-blog/)
also identifies CUDA 13.0 as the default binary line; the host driver exposes
CUDA UMD 13.3 and the selected GPU is Ampere.

MLflow is not installed in the critical environment. Canonical external JSON
remains lineage authority; an optional later MLflow mirror may use only an
external local file store and cannot change run identity or promotion.

## Exact repeated evidence

The final pair is stored externally as
`r3a-v5-training-preflight-final-a` and
`r3a-v5-training-preflight-final-b`. All three artifacts compare byte for
byte.

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| Manifest | — | `75ed5e06cff62bc546645314cf1708259b91f2d07bba6972349780b848aa2679` |
| Report | — | `a4e30d87b752331692c30c0aebcaeb11101f73f0b96d1d8271d1c0da7899b1e1` |
| Control checkpoint | `87,140,277` | `74fccfac2a07591bbf7ccf6c0d2c74520a667a32e88225e97b7b5bf91f537659` |
| Sorted three-artifact checksum list | — | `607581c43362d7195d779e2c5e2856ef4833a8243fc531d078f9b8528b2040df` |

The control decodes exactly one train clip and one internal-validation clip
from the frozen Heller archive: `953,344` source int16 scalar samples in total,
then one 48,000-sample float32 segment per role. Development and sealed
waveform reads remain zero. Real capacity training steps remain zero.

The 6 kbps control runs three optimizer steps, saves a checkpoint and executes
one identical reference/resume step in two branches. Results:

| Check | Result |
| --- | --- |
| Initial → final waveform L1 | `0.0933614299 → 0.0709542260` |
| Relative L1 improvement | `0.2400049351`, minimum `0.01` |
| Train unique codes by RVQ stage | `[8, 7, 6, 7]`, minimum per stage `2` |
| Validation unique codes by RVQ stage | `[1, 1, 2, 3]` |
| Active validation stages | `2`, minimum `2` |
| Checkpoint continuation | exact state, metrics, output and codes |

The runner implements the frozen weighted waveform L1, SI-SDR, multiresolution
complex STFT, log spectrum, target-peak emphasis, log-mel, envelope, decay and
RVQ losses. Hann windows, `center=false`, exact hop rules, mel construction and
floor semantics are bound by
`v5-full-loss-center-false-hann-v1`.

## Dead-code counterexample and correction

The first smoke exposed a real training-substrate defect. The model's original
uniform embedding initialization used one code in every active residual stage,
even though reconstruction L1 improved. Three more blind optimizer steps would
not have made that a valid runner.

The correction is frozen as
`first-train-latent-equal-residual-share-v1`: before optimizer creation, the
first authorized train segment seeds each active codebook with the encoder
latent divided equally across residual stages. The same control then uses
`[8,7,6,7]` train codes and passes exact resume. This is a deterministic
training initialization rule, not an architecture, loss, capacity or
development-derived change.

Do not restore the original tiny-uniform codebook start for real V5 runs. Do
not weaken the code-usage gate into reconstruction-only evidence.

## Allowed claim and next action

The allowed claim is only that the frozen full-size runner is finite,
trainable for a few steps, uses multiple train codes and resumes exactly on the
declared host. No acoustic endpoint has passed and no model has been selected.

Next implement and launch the hash-closed `rvq-6kbps`, `rvq-12kbps` and
`rvq-24kbps` training runs on the 56 internet train clips plus the twelve
already-authorized fit contacts, using the 17 internet internal-validation
clips for checkpoint selection. The runner must keep the four development
contacts, every row `2407`, method holdout and admission shadow unread. All
datasets, checkpoints, metrics and generated audio remain external.
