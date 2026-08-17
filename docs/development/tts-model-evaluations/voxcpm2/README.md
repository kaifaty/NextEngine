# VoxCPM2 local TTS evaluation (2026-08-17)

## Result

This was a bounded local evaluation, not a shipped NextEngine backend.

The pinned official VoxCPM2 2.0.3 compiled PyTorch path passes on the RTX 3080.
At the default 10 LocDiT steps it generated 3.680 s of 48 kHz Russian audio in
a warm mean of 2.601 s: RTF 0.707, or 1.41x realtime. Peak total GPU memory was
8,286 MiB. Streaming returned its first complete 160 ms PCM chunk in a warm
mean of 56.9 ms and completed at RTF 0.745.

VoxCPM2 is slower offline than the previous CosyVoice and audio.cpp Q8 tests,
but it provides substantially better measured TTFA and reference-free adult
Voice Design. Human quality is intentionally pending user listening.

| Candidate | Warm RTF | Realtime speed | Peak total GPU | Quality status |
| --- | ---: | ---: | ---: | --- |
| VoxCPM2 compiled, 10 steps | 0.707 | 1.41x | 8,286 MiB | Adult male/female examples ready; human verdict pending |
| VoxCPM2 compiled, 4 steps | 0.647 | 1.55x | 8,347 MiB | Faster candidate; human degradation check pending |
| CosyVoice3 compiled baseline | 0.582 | 1.72x | 7,981 MiB | Childlike bundled-reference timbre |
| Fish `audio.cpp` Q8_0 | 0.659 | 1.52x | 8,189 MiB | Rejected: poor quality |
| Fish `s2.cpp` Q4_K_M | 0.874 | 1.14x | 5,124 MiB | Generally acceptable |

At 10 steps VoxCPM2 has 21.5% higher RTF than CosyVoice, 7.3% higher RTF than
Fish audio.cpp Q8 and 19.1% lower RTF than Fish Q4. At 4 steps it is still
11.1% slower by RTF than CosyVoice, about 1.9% faster than audio.cpp Q8 and
26.0% faster than Fish Q4. These are local runtime observations, not model-only
attributions; output durations and implementations differ.

## Evaluation boundary

| Field | Value |
| --- | --- |
| Date | 2026-08-17 |
| GPU | NVIDIA GeForce RTX 3080, 10,240 MiB |
| Driver / compute capability | 610.43.02 / 8.6 (`sm_86`) |
| Host CUDA toolkit | 13.3 |
| Runtime CUDA | PyTorch 12.8 wheels |
| Fixed Russian text | `Привет! Сервер синтеза речи работает локально.` |
| Long Russian text | `Капитан, западные ворота снова открыты. Если мы выйдем до рассвета, стража не успеет перекрыть старую дорогу.` |
| Primary control | `A mature adult man with a low, calm, natural voice and clear articulation` |
| Secondary control | `A mature adult woman with a warm, calm, natural voice and clear articulation` |
| Seed / CFG | 1234 / 2.0 |
| Primary diffusion steps | 10; additional quality/speed samples at 6 and 4 |
| Text normalization / denoiser / bad-case retry | Disabled / disabled / disabled |
| Precision | BF16 language/acoustic model; float32 AudioVAE |
| Output | Mono float32 WAV, 48 kHz |
| Warm protocol | Model's built-in 10-token warm-up, one disposable full synthesis, then three measured fixed-seed requests in one process |

The age/gender descriptions use the official reference-free Voice Design path.
No voice reference was downloaded or cloned. RTF is synthesis wall divided by
generated audio duration; lower is faster and RTF below 1 is faster than
realtime.

## Pinned external installation

Third-party source, weights, Python packages, compile cache, raw logs and WAVs
remain outside Git:

```text
/home/kaifaty/.local/share/nextengine/voxcpm2
├── model/
├── outputs/
├── source/
├── torch-cache/
└── venv/
```

| Artifact | Immutable identity |
| --- | --- |
| `OpenBMB/VoxCPM` | release 2.0.3 commit `19b6bf7590025418821a86dcb817504e0ad7e5df` |
| `openbmb/VoxCPM2` | snapshot `bffb3df5a29440629464e5e839f4d214c8714c3d` |
| Complete model closure | 4,960,731,703 bytes; all 9 size/SHA-256 checks passed |
| `model.safetensors` | 4,580,080,592 bytes; SHA-256 `f7f964cfa9da23653baec6e6f7750719977ad944ed9f95fe52fe3a620506891d` |
| `audiovae.pth` | 376,951,122 bytes; SHA-256 `94b5d51e107e0507d4acc976cfdadb64edd6fd06d1f751dadbf2fd1594274bf1` |
| `tokenizer.json` | 3,676,772 bytes; SHA-256 `f8984687e4a92a3503d521396d454b7d68e9fdaab2a0288eb3536c7c1aa4bc20` |

The wrapper encodes and verifies the remaining six small-file hashes. Official
source and weights are Apache-2.0; Russian is one of the documented 30
languages. The model is approximately 2B parameters and outputs native 48 kHz
audio.

## Runtime closure

| Component | Version/result |
| --- | --- |
| Python | 3.11.15 |
| VoxCPM | 2.0.3 from the pinned local source checkout |
| PyTorch / torchaudio | 2.10.0+cu128 / 2.10.0+cu128 |
| Transformers | 5.3.0 |
| Triton | 3.6.0 |
| GPU BF16 | Supported and validated |

The full upstream frozen lock contains 160 packages and occupies about 7.9 GiB
in the dedicated environment. Runtime is forced offline and receives only the
pinned local model path. Denoising and text normalization are disabled because
the fixed plain Russian strings and reference-free design path do not need
their mutable external models.

## Canonical offline benchmark

Raw evidence:

```text
/home/kaifaty/.local/share/nextengine/tts-model-evaluations/voxcpm2-2026-08-17/raw/compiled-steps10
```

| Request | Wall | Audio | RTF | Realtime speed |
| --- | ---: | ---: | ---: | ---: |
| Prewarm | 2.599 s | 3.680 s | 0.706 | 1.42x |
| Warm 1 | 2.601 s | 3.680 s | 0.707 | 1.41x |
| Warm 2 | 2.600 s | 3.680 s | 0.706 | 1.42x |
| Warm 3 | 2.603 s | 3.680 s | 0.707 | 1.41x |
| Warm mean | 2.601 s | 3.680 s | 0.707 | 1.41x |

| Initialization/footprint | Result |
| --- | ---: |
| Runtime imports | 2.324 s |
| Model load + cached compile warm-up | 24.956 s |
| Complete process wall | 39.821 s |
| Peak total GPU memory | 8,286 MiB |
| PyTorch peak allocated / reserved | 5,761 / 6,272 MiB |

The first uncached TorchInductor process spent 111.094 s in model load and
compile warm-up, then synthesized 3.52 s in 2.514 s (RTF 0.714). Subsequent
compiled processes used the external cache and loaded in 23.757-31.802 s.
Startup is therefore unsuitable for per-line processes; a resident service is
required.

## Step-count speed/quality sweep

Every row is the mean of three fixed-seed warm requests. Fewer LocDiT steps
change the generated waveform and may change when the autoregressive stop head
terminates, so absolute wall time is not monotonic with step count.

| Steps | Warm wall | Audio | RTF | Realtime speed | Peak total GPU |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 10 | 2.601 s | 3.680 s | 0.707 | 1.41x | 8,286 MiB |
| 6 | 2.442 s | 3.680 s | 0.664 | 1.51x | 8,289 MiB |
| 4 | 2.794 s | 4.320 s | 0.647 | 1.55x | 8,347 MiB |

Six steps reduce RTF by 6.1% and four steps by 8.5% relative to 10. The modest
gain shows that autoregressive generation and 48 kHz AudioVAE work dominate
enough of the request that LocDiT steps alone are not a large optimization.
Choose 4 or 6 only after listening for noise, pronunciation and prosody loss.

## Torch compile A/B

The eager control uses identical text, voice, seed, CFG and 10 steps:

| Runtime | Warm wall | RTF | Peak total GPU | Cached model load |
| --- | ---: | ---: | ---: | ---: |
| `torch.compile` | 2.601 s | 0.707 | 8,286 MiB | 24.956 s |
| Eager | 3.075 s | 0.836 | 7,651 MiB | 14.543 s |

Compile lowers RTF by 15.4% but adds about 635 MiB total GPU and substantial
startup/compile cost. Compiled and eager PCM are individually deterministic but
not identical to one another because their numeric execution paths differ.

Release 2.0.3 has a relevant device-selection trap: `optimize()` compares the
device value exactly to `"cuda"`. Passing the otherwise supported `"cuda:0"`
prints a warning and silently leaves the model eager. The wrapper selects the
physical device through `CUDA_VISIBLE_DEVICES` and passes `device="cuda"`.

## Streaming

Raw evidence:

```text
/home/kaifaty/.local/share/nextengine/tts-model-evaluations/voxcpm2-2026-08-17/raw/streaming
```

| Metric | Result |
| --- | ---: |
| Prewarm first PCM | 92.4 ms |
| Warm first PCM mean | 56.9 ms |
| Warm TTFA range | 56.1-58.0 ms |
| Chunk count / duration | 23 / 160 ms each |
| Warm full wall | 2.743 s |
| Warm full RTF | 0.745 |
| Peak total GPU | 8,303 MiB |

This is measured at the first returned NumPy PCM chunk, not generator creation
or an HTTP header. Streaming adds about 5.5% full-request RTF versus offline but
starts playback far earlier. Stream-reassembled PCM is deterministic within the
stream mode and differs slightly from offline PCM.

## Longer Voice Design sample

The adult-female RPG phrase produced 8.000 s audio in 5.738 s, RTF 0.717.
Cached model load was 23.806 s, complete process wall 33.839 s and peak total
GPU memory 8,455 MiB. The maximum observed memory across retained runs is thus
8,455 MiB; safe multi-request concurrency is not established on the 10 GiB GPU.

## Determinism and waveform checks

All three warm requests in each benchmark had identical decoded float32 PCM at
fixed seed. Whole float-WAV hashes differ only at byte 61 in the libsndfile
`PEAK` timestamp field; regression checks therefore use PCM SHA-256.

All six listening files are mono float32 48 kHz, have zero samples at or above
absolute 0.999, peak amplitudes 0.700-0.896 and negligible DC offset. The
4-step sample is materially quieter by RMS (0.066 versus 0.139 at 10 steps).
These checks establish valid unclipped files, not perceptual quality.

## Audio archive

```text
/home/kaifaty/.local/share/nextengine/tts-model-evaluations/voxcpm2-2026-08-17
├── examples/
│   ├── adult-female-dialogue-steps10.wav
│   ├── adult-male-eager-steps10.wav
│   ├── adult-male-steps10.wav
│   ├── adult-male-steps4.wav
│   ├── adult-male-steps6.wav
│   └── adult-male-stream-steps10.wav
└── raw/
    ├── compiled-steps10/
    ├── compiled-steps4/
    ├── compiled-steps6/
    ├── eager/
    ├── generated/
    └── streaming/
```

| Example | Duration | WAV SHA-256 | Listening purpose |
| --- | ---: | --- | --- |
| `adult-male-steps10.wav` | 3.680 s | `9c4af1c79c507522c30b892d9547c72e07562d138ccff4c5a8fab1aebcc038e6` | Primary quality baseline |
| `adult-male-steps6.wav` | 3.680 s | `df954f9f28608050d9c39974a40b1d34d4e97898c0322fd3c96a32f66f802c05` | Six-step quality tradeoff |
| `adult-male-steps4.wav` | 4.320 s | `0b2db6f228f790ff42dbc58518c3d11e0e815f1ca7184d5678c5f23368142def` | Four-step quality tradeoff |
| `adult-male-stream-steps10.wav` | 3.680 s | `0a7434a23e893ca4033a766900f42cfcf2a0a178a740cf6a915df9ab6d5d6e3c` | Stream-reassembled quality check |
| `adult-male-eager-steps10.wav` | 3.680 s | `44d149178bf5a2f3ce0a6efea4ca28acb0cbbc87d0b022c9fc8bda829246baff` | Compile/eager numeric comparator |
| `adult-female-dialogue-steps10.wav` | 8.000 s | `db6aedd91ce5a95580fb576976ce4483c960e11259dd003d86f5759267bc364e` | Adult-female and longer-text quality |

The human review should score adult-timbre adherence, Russian pronunciation,
naturalness/prosody, audible diffusion noise and whether 4/6 steps materially
degrade the 10-step result.

## Reproduction

From the repository root:

```bash
python3 lab/scripts/voxcpm2_demo.py doctor

python3 lab/scripts/voxcpm2_demo.py generate --force --voice adult-male

python3 lab/scripts/voxcpm2_demo.py benchmark \
  --skip-model-hash-check --iterations 3 --steps 10

python3 lab/scripts/voxcpm2_demo.py benchmark \
  --skip-model-hash-check --iterations 3 --steps 6

python3 lab/scripts/voxcpm2_demo.py benchmark \
  --skip-model-hash-check --iterations 3 --steps 4

python3 lab/scripts/voxcpm2_demo.py benchmark \
  --skip-model-hash-check --iterations 3 --stream
```

Use `--skip-model-hash-check` only after a full `doctor`; sizes and pinned source
still validate. The wrapper rejects installations and outputs inside Git and
runs the external worker offline.

Repository-owned entry points:

- `lab/scripts/voxcpm2_demo.py`
- `lab/tests/test_voxcpm2_demo.py`

## Constraints and next experiments

- Keep the 10-step compiled run as the quality/correctness baseline until the
  user accepts a lower-step example.
- Treat one resident model and one active request as the proven 10 GiB boundary.
- Use a resident process; do not pay 24-32 s cached startup per line.
- If quality is acceptable, compare official Nano-vLLM on the same text,
  control, seed, CFG and step count; do not compare only upstream RTX 4090 data.
- Voice cloning needs a clean authorized reference and a separate consent-aware
  evaluation.
- Keep model/runtime/generated artifacts outside Git.

## Official upstream sources

- <https://github.com/OpenBMB/VoxCPM>
- <https://github.com/OpenBMB/VoxCPM/releases/tag/2.0.3>
- <https://huggingface.co/openbmb/VoxCPM2>
