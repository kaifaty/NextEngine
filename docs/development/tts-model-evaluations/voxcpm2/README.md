# VoxCPM2 local TTS evaluation (2026-08-17)

## Result

This was a bounded local evaluation, not a shipped NextEngine backend.

Both the pinned official VoxCPM2 2.0.3 compiled PyTorch path and the separately
pinned Nano-vLLM-VoxCPM 2.0.3 optimized path pass on the RTX 3080. Nano-vLLM
with CUDA Graphs generated 3.680 s of 48 kHz Russian audio in a warm mean of
0.917 s: RTF 0.249, or 4.01x realtime. Its first complete 160 ms PCM chunk
arrived in 149.8 ms and peak total GPU memory was 9,271 MiB.

The same PyTorch compiled request measured RTF 0.707 at 8,286 MiB. Nano-vLLM
therefore has 64.8% lower RTF and 2.84x the single-request throughput, but uses
about 985 MiB more peak total GPU and has 2.63x the streaming TTFA of the
PyTorch path (149.8 versus 56.9 ms). In direct listening, the user judged the
PyTorch output to have noticeably better intonation. PyTorch therefore remains
the quality baseline; Nano-vLLM is retained as a throughput-only alternative.

| Candidate | Warm RTF | Realtime speed | Peak total GPU | Quality status |
| --- | ---: | ---: | ---: | --- |
| VoxCPM2 Nano-vLLM CUDA Graphs, 10 steps | 0.249 | 4.01x | 9,271 MiB | Noticeably weaker intonation than PyTorch; throughput-only alternative |
| VoxCPM2 compiled, 10 steps | 0.707 | 1.41x | 8,286 MiB | Noticeably better intonation; preferred quality baseline |
| VoxCPM2 compiled, 4 steps | 0.647 | 1.55x | 8,347 MiB | Faster candidate; human degradation check pending |
| CosyVoice3 compiled baseline | 0.582 | 1.72x | 7,981 MiB | Childlike bundled-reference timbre |
| Fish `audio.cpp` Q8_0 | 0.659 | 1.52x | 8,189 MiB | Rejected: poor quality |
| Fish `s2.cpp` Q4_K_M | 0.874 | 1.14x | 5,124 MiB | Generally acceptable |

Nano-vLLM has 57.2% lower RTF than CosyVoice, 62.2% lower RTF than Fish
audio.cpp Q8 and 71.5% lower RTF than Fish Q4 on the fixed prompt. These are
local runtime observations, not model-only attributions; output models,
conditioning paths and implementations differ.

## Evaluation boundary

| Field | Value |
| --- | --- |
| Date | 2026-08-17 |
| GPU | NVIDIA GeForce RTX 3080, 10,240 MiB |
| Driver / compute capability | 610.43.02 / 8.6 (`sm_86`) |
| Host CUDA toolkit | 13.3 |
| Runtime CUDA | PyTorch baseline cu128; Nano-vLLM cu130 |
| Fixed Russian text | `Привет! Сервер синтеза речи работает локально.` |
| Long Russian text | `Капитан, западные ворота снова открыты. Если мы выйдем до рассвета, стража не успеет перекрыть старую дорогу.` |
| Primary control | `A mature adult man with a low, calm, natural voice and clear articulation` |
| Secondary control | `A mature adult woman with a warm, calm, natural voice and clear articulation` |
| Seed / CFG | 1234 / 2.0 |
| Primary diffusion steps | 10; additional quality/speed samples at 6 and 4 |
| Text normalization / denoiser / bad-case retry | Disabled / disabled / disabled |
| Precision | BF16 language/acoustic model; float32 AudioVAE |
| Output | Mono float32 WAV, 48 kHz |
| Warm protocol | One disposable complete synthesis, then three measured fixed-seed requests in one resident process; the PyTorch model also runs its built-in load warm-up |

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

/home/kaifaty/.local/share/nextengine/voxcpm2-nanovllm
├── outputs/
├── runtime-site/
├── source/
├── triton-cache/
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
| `a710128/nanovllm-voxcpm` | release 2.0.3 commit `0ef61b0ba634dbf2fad9e916bc4fb696a3c0f51f` |
| Inference-only FlashAttention extension | 24,988,520 bytes; SHA-256 `b85683e47a0583b48f294633bfbcc17ab28289cd8cdb8279ec70ab68296f9ff1` |

The PyTorch wrapper encodes and verifies the remaining six small-file hashes.
The Nano-vLLM wrapper verifies its source revision, four large model-file sizes,
runtime versions and the FlashAttention extension hash. The latter was built
from FlashAttention 2.8.3.post1 for Ampere BF16 inference with head dimension
128 and forward causal/non-causal/split-KV kernels only; it is not a general
training wheel. Official VoxCPM source and weights are Apache-2.0; Nano-vLLM is
MIT and FlashAttention is BSD-3-Clause. The model is approximately 2B
parameters and outputs native 48 kHz audio.

## Runtime closure

| Component | PyTorch baseline | Nano-vLLM optimized |
| --- | --- | --- |
| Python | 3.11.15 | 3.11.15 |
| Runtime | VoxCPM 2.0.3 | Nano-vLLM-VoxCPM 2.0.3 |
| PyTorch / CUDA wheel | 2.10.0+cu128 | 2.10.0+cu130 |
| Transformers | 5.3.0 | 5.15.0 |
| Triton | 3.6.0 | 3.6.0 |
| FlashAttention | Not used | 2.8.3.post1, pinned inference build |
| GPU BF16 | Supported and validated | Supported and validated |

The PyTorch upstream frozen lock contains 160 packages and occupies about 7.9
GiB. Nano-vLLM is isolated in a second environment and reuses the immutable
model directory instead of downloading another 4.96 GB snapshot. Both workers
are forced offline and receive only local paths. Denoising and text
normalization are disabled in the PyTorch baseline because the fixed plain
Russian strings and reference-free design path do not need mutable external
models; Nano-vLLM directly consumes the same designed target text.

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

## Nano-vLLM optimized benchmark

Canonical raw evidence:

```text
/home/kaifaty/.local/share/nextengine/tts-model-evaluations/voxcpm2-2026-08-17/raw/nanovllm-cuda-graph-steps10
```

This uses one visible GPU, `max_num_seqs=1`, `max_model_len=4096`, CFG 2.0,
10 diffusion steps and Nano-vLLM CUDA Graphs. Full RTF includes the first chunk;
the additional `RTF after first PCM` matches the upstream Nano-vLLM convention
but is not used for the cross-runtime headline.

| Request | First PCM | Wall | Audio | Full RTF | RTF after first PCM |
| --- | ---: | ---: | ---: | ---: | ---: |
| Prewarm / graph capture | 4.942 s | 5.754 s | 3.680 s | 1.564 | 0.221 |
| Warm 1 | 150.7 ms | 0.917 s | 3.680 s | 0.249 | 0.208 |
| Warm 2 | 139.1 ms | 0.908 s | 3.680 s | 0.247 | 0.209 |
| Warm 3 | 159.6 ms | 0.925 s | 3.680 s | 0.251 | 0.208 |
| Warm mean | 149.8 ms | 0.917 s | 3.680 s | 0.249 | 0.208 |

| Initialization/footprint | Result |
| --- | ---: |
| Runtime imports | 1.444 s |
| First retained model load / server readiness | 18.896 s |
| Later cached model load | 9.162 s |
| Canonical complete process wall | 31.374 s |
| Peak total GPU memory | 9,271 MiB |

The first complete request is not a serving latency number: it performs CUDA
Graph capture and remaining Triton/Torch compilation. A resident process is
mandatory. Once warm, Nano-vLLM lowers full RTF by 64.8% and provides 2.84x
the throughput of compiled PyTorch. It adds about 985 MiB peak total GPU and
increases first-PCM latency from 56.9 to 149.8 ms. The speedup does not preserve
perceived intonation quality: the user preferred PyTorch by a noticeable margin.

### Nano-vLLM CUDA Graph A/B

The eager control uses identical model, text, Voice Design control, seed, CFG,
step count and single-request scheduler bounds:

| Runtime | Warm wall | Full RTF | First PCM | Peak total GPU | Model load |
| --- | ---: | ---: | ---: | ---: | ---: |
| Nano-vLLM CUDA Graphs | 0.917 s | 0.249 | 149.8 ms | 9,271 MiB | 18.896 s first / 9.162 s cached |
| Nano-vLLM eager | 3.254 s | 0.884 | 157.5 ms | 9,310 MiB | 9.831 s |
| PyTorch `torch.compile` offline | 2.601 s | 0.707 | Not applicable | 8,286 MiB | 24.956 s cached |
| PyTorch streaming | 2.743 s | 0.745 | 56.9 ms | 8,303 MiB | 24.956 s cached |

CUDA Graphs lower Nano-vLLM RTF by 71.8% and increase throughput about 3.55x.
Without graphs, Nano-vLLM is about 5.8% slower by RTF than the PyTorch eager
control. Fixed-seed graph and eager PCM are each internally deterministic but
not identical to each other.

A separate `gpu_memory_utilization=0.80` probe measured RTF 0.248 and 9,283 MiB
peak total GPU, effectively unchanged from the canonical 0.90 run. Reducing the
KV-cache target therefore did not reduce the observed graph/model peak and is
not adopted as a memory optimization.

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

The PyTorch adult-female RPG phrase produced 8.000 s audio in 5.738 s, RTF
0.717, at 8,455 MiB peak total GPU. Nano-vLLM produced 6.560 s audio in a warm
mean of 1.551 s: RTF 0.236, 4.23x realtime, first PCM in 146.2 ms and 9,284 MiB
peak total GPU.

The Nano-vLLM waveform is 1.44 s shorter despite identical text, Voice Design,
seed, CFG and steps. This can be a prosody/rate difference or a content omission;
the listening review must explicitly check that every Russian word is present.
Safe multi-request concurrency is not established on the 10 GiB GPU.

## Determinism and waveform checks

All three warm requests in every PyTorch and Nano-vLLM benchmark had identical
decoded float32 PCM at fixed seed. Whole float-WAV hashes may differ through the
libsndfile `PEAK` timestamp field; regression checks therefore use PCM SHA-256.
Graph, eager and PyTorch PCM differ from one another because their numeric
execution paths differ.

All nine listening files are mono float32 48 kHz with no samples at or above
absolute 0.999 and negligible DC offset. The Nano-vLLM male graph/eager files
have peaks 0.813/0.791 and RMS 0.0879/0.0879. The PyTorch 10-step male baseline
has peak 0.896 and RMS 0.139, so the Nano-vLLM sample is about 4.0 dB quieter by
RMS. The Nano-vLLM female file has peak 0.743 and RMS 0.142. These checks
establish valid unclipped files, not perceptual quality.

## Human listening verdict

The user directly compared the retained PyTorch and Nano-vLLM examples and
judged PyTorch to have noticeably better intonation. This is the decisive
quality result for this bounded evaluation: Nano-vLLM CUDA Graphs wins the
single-request throughput measurement, but it is not quality-equivalent to the
PyTorch path and is not the default serving candidate.

Pronunciation, adult-timbre adherence, long-text completeness and broader-corpus
behavior were not separately scored. In particular, the 6.560 s Nano-vLLM
female dialogue still needs a word-completeness check against the 8.000 s
PyTorch result if Nano-vLLM is reconsidered later.

## Emotion and delivery comparison

A reproducible PyTorch `emotion-suite` command now exercises seven
natural-language Voice Design descriptions in one resident compiled streaming
session. The first six use the same Russian line, seed 1234, CFG 2.0 and 10
LocDiT steps; `amused-chuckle` uses a separate line whose wording suits the
requested delivery. These are prompt-controlled attempts, not Fish-style
canonical emotion tags.

Common text:

```text
Капитан, западные ворота снова открыты. Если мы выйдем до рассвета, стража не успеет перекрыть старую дорогу.
```

| Variant | Audio | Wall | RTF | First PCM |
| --- | ---: | ---: | ---: | ---: |
| Neutral | 6.400 s | 5.034 s | 0.787 | 58.6 ms |
| Restrained anger | 6.400 s | 5.044 s | 0.788 | 59.1 ms |
| Sad / tired | 6.880 s | 5.513 s | 0.801 | 59.0 ms |
| Joyful / excited | 7.040 s | 5.534 s | 0.786 | 64.0 ms |
| Tense whisper | 8.160 s | 6.415 s | 0.786 | 59.2 ms |
| Dry sarcasm | 6.880 s | 5.374 s | 0.781 | 61.5 ms |
| Amused / requested chuckle | 4.800 s | 3.728 s | 0.777 | 58.9 ms |
| **Mean / range** | **4.800-8.160 s** | — | **0.787 mean** | **60.0 ms mean** |

The complete process took 71.663 s including 3.620 s of imports, 25.848 s of
model load/cached compilation and one disposable prewarm. Peak total GPU memory
was 7,491 MiB; PyTorch peak allocated/reserved was 5,761/6,298 MiB. All seven
outputs are mono float32 48 kHz, finite and unclipped. Their peaks range from
0.737 to 0.980 and RMS from 0.115 to 0.195. These checks establish usable audio
files and timing only. Whether each prompt preserves the intended adult voice
and conveys the named emotion remains a human-listening decision.

## Audio archive

```text
/home/kaifaty/.local/share/nextengine/tts-model-evaluations/voxcpm2-2026-08-17
├── examples/
│   ├── adult-female-dialogue-steps10.wav
│   ├── adult-female-dialogue-nanovllm-cuda-graph-steps10.wav
│   ├── adult-male-eager-steps10.wav
│   ├── adult-male-nanovllm-cuda-graph-steps10.wav
│   ├── adult-male-nanovllm-eager-steps10.wav
│   ├── adult-male-steps10.wav
│   ├── adult-male-steps4.wav
│   ├── adult-male-steps6.wav
│   ├── adult-male-stream-steps10.wav
│   └── emotion-suite-stream-steps10/
│       ├── _prewarm.wav
│       ├── amused-chuckle.wav
│       ├── dry-sarcasm.wav
│       ├── joyful-excited.wav
│       ├── neutral.wav
│       ├── restrained-anger.wav
│       ├── runtime.log
│       ├── sad-tired.wav
│       ├── summary.json
│       └── tense-whisper.wav
└── raw/
    ├── compiled-steps10/
    ├── compiled-steps4/
    ├── compiled-steps6/
    ├── eager/
    ├── generated/
    ├── nanovllm-cuda-graph-female-dialogue-steps10/
    ├── nanovllm-cuda-graph-gpu-util-080/
    ├── nanovllm-cuda-graph-steps10/
    ├── nanovllm-eager-steps10/
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
| `adult-male-nanovllm-cuda-graph-steps10.wav` | 3.680 s | `495bf7dc7ee3d79714d58b5b0a3da0cc2068eb96a36e9ce6f573c23073462685` | Primary Nano-vLLM optimized quality comparator |
| `adult-male-nanovllm-eager-steps10.wav` | 3.680 s | `df9d0658cabb3b9e5ecc64a20678661ef1745d76cb2726ba1e41a1d4b8ddda66` | Nano-vLLM CUDA Graph/eager numeric comparator |
| `adult-female-dialogue-nanovllm-cuda-graph-steps10.wav` | 6.560 s | `970d0f5815552d27aa9403023f178df33cf3e15c5d67525605e07c31b08bca29` | Longer Nano-vLLM completeness/prosody check |

The retained intonation verdict concerns Nano-vLLM versus PyTorch. A broader
review would still need to score adult-timbre adherence, Russian pronunciation,
audible diffusion noise, complete long-text delivery and whether 4/6 PyTorch
steps materially degrade the 10-step result.

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

python3 lab/scripts/voxcpm2_demo.py emotion-suite \
  --skip-model-hash-check --stream --steps 10 \
  --output-dir /path/outside/repository/emotion-suite

python3 lab/scripts/voxcpm2_nanovllm_demo.py doctor

python3 lab/scripts/voxcpm2_nanovllm_demo.py benchmark \
  --iterations 3 --steps 10

python3 lab/scripts/voxcpm2_nanovllm_demo.py benchmark \
  --iterations 3 --steps 10 --enforce-eager
```

Use `--skip-model-hash-check` only after a full `doctor`; sizes and pinned source
still validate. The wrapper rejects installations and outputs inside Git and
runs the external worker offline.

Repository-owned entry points:

- `lab/scripts/voxcpm2_demo.py`
- `lab/tests/test_voxcpm2_demo.py`
- `lab/scripts/voxcpm2_nanovllm_demo.py`
- `lab/tests/test_voxcpm2_nanovllm_demo.py`

## Constraints and next experiments

- Keep the 10-step compiled PyTorch run as the quality/correctness baseline; its
  intonation was judged noticeably better than Nano-vLLM.
- Do not select Nano-vLLM solely from its RTF 0.249 result. Retain it only for
  workloads that explicitly accept the observed quality tradeoff for 2.84x
  throughput.
- Prefer PyTorch streaming when minimum first-PCM latency matters more than
  completion time: 56.9 ms versus 149.8 ms for Nano-vLLM.
- Treat one resident model and one active request as the proven 10 GiB boundary;
  Nano-vLLM leaves less than 1 GiB at the observed 9,271 MiB peak.
- Use a resident process; do not pay model load, graph capture or compilation per
  line.
- Treat natural-language emotion descriptions as content data and retain the
  exact control string with each request. Human-review the emotion and speaker
  consistency before promoting any description to a gameplay preset.
- Voice cloning needs a clean authorized reference and a separate consent-aware
  evaluation.
- Keep model/runtime/generated artifacts outside Git.

## Official upstream sources

- <https://github.com/OpenBMB/VoxCPM>
- <https://github.com/OpenBMB/VoxCPM/releases/tag/2.0.3>
- <https://huggingface.co/openbmb/VoxCPM2>
- <https://github.com/a710128/nanovllm-voxcpm>
