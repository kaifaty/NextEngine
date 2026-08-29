# Fun-CosyVoice3-0.5B-2512 acceleration research (2026-08-17)

## Verdict

There is no separate official quantized "fast Fun-CosyVoice3" checkpoint. The
official project accelerates the same model by replacing inference stages:
vLLM for the autoregressive speech-token LLM, TensorRT for the DiT flow
estimator, or a complete Triton/TensorRT-LLM serving stack.
The separately published `llm.rl.pt` is a same-size quality/post-training
variant, not a smaller or faster architecture.

For this RTX 3080 and single-request workload, the next useful experiment is
the LLM backend, not the flow backend. A new local stage profile measured the
LLM at 84.25% of warm request wall time, flow at 11.51%, HiFT at 1.81%, and
frontend plus other overhead at 2.42%. Even an infinitely fast flow could
improve the complete request by only 1.13x. The LLM is the only stage with
enough weight to produce a material single-request gain.

Recommended A/B order:

1. Test the unmerged hybrid `llama-cpp-python` path with an F16 or BF16 LLM
   GGUF while retaining the official PyTorch flow and HiFT. Its author reports
   end-to-end RTF improving from about 1.17 to 0.45 on an NVIDIA T4, a 2.6x
   result, and the implementation avoids loading the original PyTorch LLM.
2. If quality matches, compare Q8_0 and then Q5_K_M for the LLM only. Do not
   quantize flow in the first experiment: autoregressive token drift can alter
   pronunciation and intonation, while flow quantization adds another quality
   variable for little local speed potential.
3. Test official vLLM in a separate environment if the hybrid path does not
   hold quality or if multi-request throughput matters. The current official
   loader has a 10 GiB transient-memory risk because it builds the vLLM engine
   before deleting the original transformer layers.
4. Defer TensorRT-only flow, full Triton/TensorRT-LLM, vLLM-Omni, full GGML and
   Candle/Rust until the first two LLM-focused A/B tests are understood.

This document records research and a new instrumented baseline. It does not
claim that an optimized backend was installed or measured on the RTX 3080.

## Scope and comparison boundary

The local comparator remains the pinned official FP16/PyTorch environment:

| Field | Value |
| --- | --- |
| GPU | NVIDIA GeForce RTX 3080, 10,240 MiB |
| Official source | `QwenAudio/CosyVoice` commit `074ca6dc9e80a2f424f1f74b48bdd7d3fea531cc` |
| Model snapshot | `29e01c4e8d000f4bcd70751be16fa94bf3d85a18` |
| Runtime | Python 3.10.20, PyTorch 2.3.1+cu121, FP16 autocast |
| Fixed target | `Привет! Сервер синтеза речи работает локально.` |
| Reference | Official `asset/zero_shot_prompt.wav`, cached before timing |
| Output | 3.680 s, mono float32, 24 kHz |
| Existing conservative warm result | 2.142 s wall, RTF 0.582, 1.72x realtime |
| Existing peak total GPU memory | 7,981-8,116 MiB |

External benchmark numbers below are not normalized to this prompt, GPU,
output duration, precision, batching or concurrency. They are evidence that a
path exists, not predictions for the RTX 3080.

## Local stage profile

The model and cached reference speaker were kept resident. One disposable run
was followed by five measured fixed-seed requests. CUDA was synchronized at
the boundaries of frontend preparation, `llm_job`, `flow.inference`,
`hift.inference` and the complete request. Instrumentation did not write audio
or alter the pinned installation. All five outputs had the same decoded PCM
SHA-256 as the retained baseline:

```text
b0081dadddd2ce253608a7baf965799d6dab6913981bf0b8fe56b16efac2740d
```

| Run | Wall | RTF | LLM | Flow | HiFT | Frontend | Other |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Warm 1 | 2.009 s | 0.546 | 1.700 s | 0.224 s | 0.036 s | 0.0004 s | 0.049 s |
| Warm 2 | 2.043 s | 0.555 | 1.708 s | 0.250 s | 0.036 s | 0.0005 s | 0.049 s |
| Warm 3 | 2.028 s | 0.551 | 1.708 s | 0.234 s | 0.037 s | 0.0004 s | 0.049 s |
| Warm 4 | 2.042 s | 0.555 | 1.729 s | 0.227 s | 0.037 s | 0.0004 s | 0.049 s |
| Warm 5 | 2.053 s | 0.558 | 1.728 s | 0.237 s | 0.038 s | 0.0004 s | 0.049 s |
| **Mean** | **2.035 s** | **0.553** | **1.715 s** | **0.234 s** | **0.037 s** | **0.0004 s** | **0.049 s** |

The instrumented mean is slightly faster than the retained conservative mean,
so the existing RTF 0.582 remains the headline baseline. The stage proportions
are the new result:

| Stage | Mean wall share | Consequence |
| --- | ---: | --- |
| Autoregressive LLM | 84.25% | Primary target for latency and TTFA |
| DiT flow | 11.51% | Secondary target; limited end-to-end upside locally |
| HiFT vocoder | 1.81% | Not worth replacing in isolation |
| Frontend | 0.02% | Already removed from the warm critical path by speaker caching |
| Unattributed wrapper/cleanup | 2.40% | Small operational overhead |

### Amdahl bounds

These are calculations from the measured stage shares, not backend results:

| Hypothetical stage improvement | Complete-request speedup | Projected wall from 2.035 s | Projected RTF |
| --- | ---: | ---: | ---: |
| Flow 2x | 1.06x | 1.918 s | 0.521 |
| Flow infinitely fast | 1.13x | 1.801 s | 0.489 |
| LLM 2x | 1.73x | 1.178 s | 0.320 |
| LLM 2.6x | 2.08x | 0.980 s | 0.266 |
| LLM 4x | 2.72x | 0.749 s | 0.204 |
| LLM infinitely fast | 6.35x | 0.321 s | 0.087 |

The 2.6x row applies the T4 author's reported end-to-end backend ratio as if it
were an LLM-stage ratio on this host. It is a planning estimate only; hardware,
runtime overhead and output token sequences differ.

## Candidate paths

### 1. Official in-process vLLM: high-value, medium feasibility risk

The official README supports vLLM 0.11.x+ with the V1 engine and vLLM 0.9.0
legacy, explicitly recommends a separate environment, and marks 0.10.x as
untested. The current source exports the CosyVoice LLM, creates an `LLMEngine`
with `gpu_memory_utilization=0.2`, and only then deletes the original Qwen
transformer layers.

Advantages:

- It replaces the measured 84% bottleneck while retaining official flow and
  HiFT.
- BF16 weights avoid the extra quality variable of low-bit quantization.
- It is the official lightest route to better concurrency/continuous batching.

Risks on this machine:

- The proven baseline already peaks at 7.98-8.12 GiB. Constructing a vLLM pool
  sized to 20% of a 10 GiB GPU before deleting the original layers can approach
  or exceed the remaining headroom during initialization.
- vLLM has a separate version closure (`transformers==4.57.1` in the current
  official 0.11 example) from the pinned PyTorch baseline.
- The source prohibits bi-stream input text with vLLM. Audio-output streaming
  remains possible, but incremental input text does not.
- Sampling execution changes. Identical weights do not guarantee identical
  speech-token sequences or intonation; PCM and listening parity must be
  checked.

Decision: useful second experiment, or first official experiment, but not an
in-place modification of the working environment. Record initialization peak
VRAM before attempting repeated timing.

### 2. Official TensorRT flow estimator: valid but low local priority

`load_trt=True` replaces only `flow.decoder.estimator`. It needs the official
`flow.decoder.estimator.fp32.onnx` and builds a GPU-specific TensorRT plan. That
ONNX file is not in the pinned 5.1 GiB local closure; the current official file
is about 1.33 GB with SHA-256
`9b51b9533a55937762b262bf2cf9c6220ce40760f76d6532cb16a6a6d84059a8`.

The current loader warns that the DiT TensorRT FP16 engine has a performance
issue and should be used with caution. The official combined example therefore
uses `fp16=False`, which is unattractive on a 10 GiB card. A newer
NVIDIA-contributor package supplies separate offline and streaming autocast
FP16 ONNX estimators, but it belongs to the full Triton path rather than the
pinned baseline.

Decision: do not spend the next experiment on TensorRT alone. Even a perfect
replacement cannot exceed 1.13x end-to-end on the local fixed prompt. It becomes
worthwhile after the LLM is accelerated or for longer token2wav-heavy requests.

### 3. Official full Triton + TensorRT-LLM: fastest published server path

The NVIDIA-contributed official runtime serves the LLM with TensorRT-LLM and
places audio tokenizer, speaker embedding, token2wav and vocoder behind Triton.
On one NVIDIA L20 it reports:

| Mode | Published result |
| --- | ---: |
| Streaming, four concurrent tasks | 750.42 ms mean first chunk; 740.31/941.05/977.55/1002.37 ms p50/p90/p95/p99 |
| Offline batch 1 | RTF 0.1091 |
| Offline batch 2 | RTF 0.0822 |
| Offline batch 4 | RTF 0.0630 |
| Offline batch 8 | RTF 0.0562 |
| Offline batch 16 | RTF 0.0501 |

These numbers are not comparable to the local 3.68 s sentence: the official
offline table uses much longer aggregate audio, and the GPU, batch and server
stack differ. The current launch scripts are tuned for batch 64, large token
limits and a substantial KV-cache pool. Docker is not installed on the local
host, and fit on 10 GiB is unproven.

Decision: best production-throughput lead for a larger GPU, not the next
single-user RTX 3080 experiment.

### 4. Hybrid llama.cpp LLM + official PyTorch token2wav: best next A/B

An open upstream PR adds a `llama-cpp-python` backend only for CosyVoice3. It
keeps the existing inference APIs and PyTorch token2wav path, supports output
streaming, and skips loading the PyTorch LLM weights. The PR author reports on
an NVIDIA T4:

| Backend | Reported end-to-end average RTF |
| --- | ---: |
| PyTorch FP16 | about 1.17 |
| llama-cpp-python F16 GGUF | about 0.45 |

That is a reported 2.6x end-to-end improvement on a different system. The PR
is still open, so its fork and GGUF must be pinned as community evidence rather
than treated as official CosyVoice behavior. The associated LLM-only model
repository offers F16/BF16, Q8_0, Q6_K, Q5_K_M and Q4_K_M among other variants.
Its generic Hugging Face llama.cpp commands output speech tokens, not WAV; the
CosyVoice fork is required to complete synthesis.

Decision: start with F16/BF16 to isolate runtime improvement. Then compare Q8_0
and Q5_K_M only if PCM/token divergence and human listening stay acceptable.
This path targets the proven bottleneck and saves VRAM without replacing flow
or vocoder.

### 5. Full C++/GGML runtimes: promising memory footprint, unproven RTX RTF

Two independent community directions are active:

- `Lourdle/cosyvoice.cpp` is a complete C++/GGML port with CUDA, streaming,
  flash attention, prompt-feature reuse, DiT streaming KV cache and GGUF
  variants from F16 down to Q2. The model publisher describes Q8_0 as the
  default near-F16 choice, Q5_K_M as usable, and Q4_K_M as occasionally muffled.
  No reproducible RTX end-to-end RTF is published. The project also documents
  noisy output with some prebuilt GGML CUDA libraries and recommends building
  the runtime and GGML from source when that occurs.
- CrispASR exposes a different full GGML CosyVoice3 pipeline. Its smallest
  baked-voice combination is 745 MB: Q4_K LLM, Q8_0 flow, F16 HiFT and a voice
  bank. The model card reports 0% ASR WER on a very short English/German smoke
  prompt but notes punctuation/intonation drift from Q4 LLM. Q8 flow is claimed
  perceptually indistinguishable from F16. No RTX latency benchmark is
  published. A five-step flow mode exists, but halving the locally measured
  flow stage would improve the complete request by only about 6% while changing
  the diffusion-quality setting.

Decision: useful low-memory deployment research, not a drop-in proof of quality
or speed. If tested, compare F16 first, then Q8, then mixed Q5/Q8. Do not jump
straight to whole-pipeline Q4 after the earlier Fish C++ quality failure.

### 6. Candle/Rust: real implementation, no RTX evidence yet

`SpenserCai/cosyvoice3.rs` is a community Candle/PyO3 implementation with CUDA
and Metal feature flags, zero-shot/cross-lingual/instruct paths, converted
weights and prompt-feature reuse. Its README reports RTF about 0.3-0.5 on an
Apple M1 Pro with Metal and about 2-4 on unspecified x86_64 CPU hardware. It
publishes no RTX CUDA number and no same-input parity corpus against the current
official PyTorch release.

Decision: Rust may reduce runtime footprint and packaging complexity, but the
language itself does not imply faster GPU kernels. Treat it as an exploratory
port after the LLM-focused A/B, not as a predicted speed win on the RTX 3080.

### 7. vLLM-Omni: emerging unified serving path

Current vLLM-Omni documentation lists CosyVoice3 as a two-stage 24 kHz TTS
pipeline with voice cloning and streaming. It is architecturally attractive
for a common multimodal server, but no official CosyVoice3 RTF table for this
hardware was found. Recent issue history includes full-model/streaming voice
clone correctness failures and a text-only route that could trigger a CUDA
device-side assertion; the supported `/v1/audio/speech` reference-audio path
was not the failing text-only path.

Decision: monitor, but do not use it as the first quality comparator while the
official in-repository vLLM integration is smaller and easier to isolate.

### 8. Full ONNX community ports: portability lead, not a speed result

Community projects export LLM, flow and HiFT to ONNX and can remove PyTorch.
The available published timing found for the complete port is CPU-only and very
slow (LLM 100-200 s, flow 40-100 s); it merely states that CUDA is faster and
does not provide an RTX RTF. The NVIDIA FP16 ONNX package is more credible, but
it is intended for the Triton/TensorRT runtime.

Decision: not the next local path. ONNX becomes relevant for portable service
packaging or when paired with measured TensorRT/CUDA graph work.

## Practical optimization conclusions

- Keep one resident model and pre-extract/cache the reference speaker. The
  measured warm frontend cost is already negligible; repeating prompt analysis
  cannot produce a meaningful win in the current wrapper.
- Accelerate the AR LLM first. This improves complete-request latency and also
  the rate at which the first streaming token window becomes available.
- Do not count batching as single-request latency. vLLM and Triton are most
  compelling when concurrent throughput is part of the requirement.
- Reset or isolate the upstream mutable `token_hop_len` between streaming
  sessions. Otherwise a later request can inherit a larger first chunk and lose
  TTFA even if token generation is faster.
- Do not reduce flow steps before measuring an optimized LLM. On this prompt,
  five instead of ten idealized half-cost steps have only about a 6% complete
  wall-time ceiling and intentionally alter quality.
- Keep the existing decoded-PCM hash and human listening test. Autoregressive
  token quantization can preserve words while still changing punctuation,
  rhythm and intonation.

## Smallest next experiment

Create a separate external environment pinned to the exact open llama.cpp PR
fork and its F16 LLM GGUF. Run the existing fixed Russian sentence and longer
dialogue with the same official reference, seed and one-prewarm/three-warm
protocol. Record:

1. model-load wall and peak total GPU memory;
2. LLM, flow, HiFT and complete wall time;
3. offline RTF and first emitted PCM for streaming;
4. generated speech-token sequence, decoded PCM SHA-256 and audio duration;
5. side-by-side human pronunciation, prosody, timbre and artifact verdict.

Pass criterion: at least 1.5x complete warm speedup, no new pronunciation or
intonation defect, stable output duration, and peak total GPU memory below the
10,240 MiB device boundary. If F16 passes, repeat with Q8_0 and Q5_K_M. If F16
does not pass, do not attribute the failure to quantization; compare official
vLLM next.

## Sources

- [Official CosyVoice README and vLLM versions](https://github.com/QwenAudio/CosyVoice#vllm-usage)
- [Official CosyVoice3 loader](https://github.com/QwenAudio/CosyVoice/blob/main/cosyvoice/cli/cosyvoice.py)
- [Official vLLM/TensorRT example](https://github.com/QwenAudio/CosyVoice/blob/main/vllm_example.py)
- [Official Triton/TensorRT-LLM CosyVoice3 benchmark](https://github.com/QwenAudio/CosyVoice/blob/main/runtime/triton_trtllm/README.Cosyvoice3.md)
- [Official model and flow estimator ONNX](https://huggingface.co/FunAudioLLM/Fun-CosyVoice3-0.5B-2512/blob/main/flow.decoder.estimator.fp32.onnx)
- [NVIDIA-contributor FP16 ONNX package](https://huggingface.co/yuekai/Fun-CosyVoice3-0.5B-2512-FP16-ONNX)
- [Open llama-cpp-python integration PR and T4 result](https://github.com/QwenAudio/CosyVoice/pull/1872)
- [LLM-only GGUF variants used by that integration](https://huggingface.co/Ferraronp/CosyVoice3-qwen2.5-0.5b-speech-gguf)
- [cosyvoice.cpp runtime](https://github.com/Lourdle/cosyvoice.cpp)
- [cosyvoice.cpp model variants](https://huggingface.co/Lourdle/Fun-CosyVoice3-0.5B-2512-GGUF)
- [CrispASR CosyVoice3 GGUF package and quant notes](https://huggingface.co/cstr/cosyvoice3-0.5b-2512-GGUF)
- [Candle/Rust implementation](https://github.com/SpenserCai/cosyvoice3.rs)
- [vLLM-Omni CosyVoice3 documentation](https://docs.vllm.ai/projects/vllm-omni/en/latest/user_guide/examples/offline_inference/text_to_speech/#cosyvoice3)
- [Community full ONNX port](https://huggingface.co/ayousanz/cosy-voice3-onnx)
- [Open RTX 4090 concurrency optimization proposal](https://github.com/QwenAudio/CosyVoice/issues/1892)
