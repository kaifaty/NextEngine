# Fun-CosyVoice3-0.5B-2512 TTS demo — current task state

| Field | Value |
| --- | --- |
| Status | `COMPLETE` |
| Updated | `2026-08-17` |
| Task key | `fun-cosyvoice3-tts-demo` |
| Scope | Install and evaluate the official Fun-CosyVoice3-0.5B-2512 CUDA inference path on the available RTX 3080, then research faster official and community runtimes without integrating weights or runtime code into NextEngine. |
| Definition of done | A pinned external installation produces valid Russian WAV through a repository-owned wrapper; source/model identity, license, reference-audio boundary, cold/warm latency, RTF, VRAM and listening examples are recorded; a local stage profile and evidence-backed acceleration order identify the smallest next A/B. |
| Authority | Working context only; `AGENTS.md`, Accepted architecture, exact upstream source/model revisions and raw external evidence outrank this file. |

## Resume in 60 seconds

- **Current conclusion:** There is no separate official quantized fast checkpoint. Local profiling assigns 84.25% of warm wall to the autoregressive LLM and only 11.51% to flow, so the next useful A/B is an F16/BF16 llama.cpp LLM hybrid or official vLLM, not TensorRT flow alone. The official FP16/PyTorch quality/correctness baseline remains RTF 0.582; its bundled-reference voice is still childlike.
- **Why:** Five new same-session requests averaged 2.035 s / RTF 0.553 under synchronization instrumentation: LLM 1.715 s, flow 0.234 s, HiFT 0.037 s and other/frontend 0.049 s. An infinitely fast flow has only a calculated 1.13x end-to-end ceiling. An open llama.cpp integration reports 2.6x end-to-end on a T4 while retaining PyTorch token2wav, but is not yet a local result.
- **Next action:** If CosyVoice acceleration is selected, pin the open llama.cpp integration in a separate external environment, test F16/BF16 first, then Q8_0 and Q5_K_M only after quality passes; otherwise continue with the next model.
- **Current blocker:** None.
- **Do not retry:** Do not put model weights, reference speech, generated WAV files, Python environments or raw logs in Git.
- **Reconsider when:** A pinned llama.cpp, official vLLM or TensorRT path is evaluated, a Russian-aware text frontend is needed, or a broader prompt corpus replaces the fixed short benchmark.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| Host probe on 2026-08-17 | `PASS`: RTX 3080 10,240 MiB, compute capability 8.6, driver 610.43.02, CUDA toolkit 13.3, about 253 GiB free storage | A bounded CUDA installation and benchmark are feasible. |
| Official Hugging Face model card | `REPORT_ONLY`: 0.5B model, Apache-2.0, Russian listed among nine languages, zero-shot/cross-lingual/instruct and streaming support | Use the official package as the first correctness baseline; validate locally rather than assuming the published latency. |
| Official CosyVoice repository | `REPORT_ONLY`: recommends `Fun-CosyVoice3-0.5B`, Python 3.10 and its `AutoModel` example; vLLM and TensorRT-LLM are optional acceleration paths | Keep the initial environment isolated and avoid premature optimized-backend complexity. |
| Official source/model external closure | `PASS`: source `074ca6dc…`, Matcha-TTS `dd9105b3…`, model snapshot `29e01c4e…`; required 5.1 GiB base files match official LFS SHA-256 values | Immutable official baseline is installed outside Git. |
| Dedicated Python/CUDA runtime | `PASS`: Python 3.10.20, PyTorch 2.3.1+cu121, CUDA visible, official CUDA-12 ONNX Runtime 1.18 uses `CUDAExecutionProvider` for the speech tokenizer | Baseline uses the intended CUDA path without changing global Python. |
| First Russian cross-lingual probe | `PASS`: 3.68 s mono float32 24 kHz WAV; model load 17.31 s; uncached inference wall 4.99 s; total RTF 1.356; upstream inner synthesis about 2.81 s | Correctness is established; cached-speaker warm and streaming probes remain necessary. |
| Same-session offline benchmark | `PASS`: prewarm RTF 0.730; three warm requests RTF 0.598/0.576/0.573, mean 0.582; 7,981 MiB peak total GPU; repeat mean 0.571 | The official baseline is faster than realtime and is the conservative local speed comparator. |
| Streaming probe | `REPORT_ONLY`: first session emitted 1.36 s PCM after 1.825 s, then mutated upstream `token_hop_len` from 25 to 100; later same-session calls emitted one complete chunk after mean 2.076 s | The published 150 ms headline was not reproduced by this local short-prompt path; do not generalize the stateful result to every serving configuration. |
| Determinism probe | `PASS_WITH_NOTE`: three warm requests had identical PCM SHA-256; WAV SHA-256 differed because torchaudio writes a timestamped `PEAK` metadata chunk | Compare decoded PCM, not whole-file bytes, for deterministic regression checks. |
| External evidence archive | `PASS`: four listening WAVs and canonical/repeat raw summaries/logs under `/home/kaifaty/.local/share/nextengine/tts-model-evaluations/fun-cosyvoice3-0.5b-2512-2026-08-17` | Results are reviewable without adding weights or generated artifacts to Git. |
| Human listening verdict | `QUALITY_LIMITED`: both Russian examples sound childlike with the bundled official reference; no child/adolescent style instruction was used | Preserve timing as valid, but do not score this voice as an adult-character candidate. |
| Instrumented warm stage profile | `PASS`: five fixed-seed requests averaged 2.035 s / RTF 0.553 with identical baseline PCM; LLM 84.25%, flow 11.51%, HiFT 1.81%, frontend and other 2.42% | Optimize the AR LLM first; TensorRT flow alone has only a calculated 1.13x perfect-stage ceiling on this prompt. |
| Official acceleration review | `REPORT_ONLY`: current source supports vLLM and TensorRT; full NVIDIA Triton/TensorRT-LLM reports L20 batch-1 RTF 0.1091 and four-stream mean first chunk 750.42 ms | Treat the server numbers as evidence of potential, not RTX 3080 predictions; GPU, batch, prompt and runtime differ. |
| Community runtime review | `REPORT_ONLY`: open hybrid llama.cpp PR reports T4 RTF 1.17 to 0.45; full GGML and Candle/Rust paths exist but publish no RTX 3080 result | The hybrid F16/BF16 LLM is the smallest quality-preserving candidate; pin community code and measure locally before claims. |
| Acceleration research report | `PASS`: detailed evidence, Amdahl bounds, runtime matrix and test order recorded in `docs/development/tts-model-evaluations/fun-cosyvoice3-0.5b-2512/acceleration-research-2026-08-17.md` | Resume from the report instead of repeating discovery. |

## Decisions that still constrain the work

### D-001 — Official baseline first, external artifacts only

- **Observation:** CosyVoice3 has multiple inference backends and requires a reference voice for the documented zero-shot examples.
- **Evidence:** Official model card and CosyVoice repository usage examples.
- **Decision:** Pin the official source and model snapshot, use the bundled demonstration reference only for a local non-distributed evaluation, and keep all heavy/generated artifacts under an external machine-local root.
- **Rejected alternatives:** Do not start with a community quantization or Rust port because that would confound model quality with runtime parity before an official baseline exists.
- **Consequences:** The first report will characterize the official PyTorch/CUDA path; acceleration can be a later A/B experiment.
- **Uncertainty:** Human Russian quality and performance on a broader prompt corpus remain unmeasured.
- **Reconsider when:** An optimized official backend is benchmarked against the same pinned inputs.

### D-002 — Direct Russian tokens and official CUDA-12 ONNX Runtime

- **Observation:** The official common YAML imports inference-irrelevant training modules; PyPI ONNX Runtime 1.18 expected cuBLAS 11; WeText attempted an unpinned multilingual FST download even with `text_frontend=False`; CosyVoice3 cross-lingual inference asserted without its required `<|endofprompt|>` marker.
- **Evidence:** Import failures at `matcha.utils`, `cosyvoice.dataset.processor`, ONNX provider load failure for `libcublasLt.so.11`, and the first manual generation logs under the external root.
- **Decision:** Retain the exact minimal upstream dependency closure needed by the shared YAML, use the CUDA-12 ORT wheel from the extra index in official `requirements.txt`, omit WeText for this Russian direct-token baseline, and prefix target text with `You are a helpful assistant.<|endofprompt|>` while saving only the synthesized speech.
- **Rejected alternatives:** Do not use the PyPI ORT wheel or wait on WeText's mutable ModelScope download for this fixed Russian benchmark; neither is needed for the requested direct-token path.
- **Consequences:** The speech tokenizer runs on CUDA, initialization is offline after installation, and the benchmark avoids English normalization of Russian input.
- **Uncertainty:** Omitting text normalization may affect numbers and punctuation in a broader corpus; only the fixed plain-text sentence is currently in scope.
- **Reconsider when:** A Russian-aware pinned frontend is evaluated or the official dependency/runtime closure changes.

### D-003 — Accelerate the autoregressive LLM before flow

- **Observation:** The working baseline is already faster than realtime, but a material single-request reduction requires identifying its dominant stage rather than selecting a backend by headline throughput.
- **Evidence:** Five synchronized warm profiles averaged 1.715 s in `llm_job`, 0.234 s in flow, 0.037 s in HiFT and 0.049 s elsewhere out of 2.035 s complete wall. The official loader builds a vLLM engine before deleting original transformer layers; the open llama.cpp PR skips loading the PyTorch LLM and reports a 2.6x T4 end-to-end result.
- **Decision:** Preserve official PyTorch as the comparator. If optimization resumes, test the hybrid F16/BF16 llama.cpp LLM first in a separate pinned environment, then official vLLM; add low-bit LLM quantization only after full-precision listening parity. Defer flow-only TensorRT.
- **Rejected alternatives:** Do not begin with five flow steps, full-pipeline Q4, full Triton or vLLM-Omni: they add quality, deployment or memory variables before the measured LLM bottleneck is isolated.
- **Consequences:** The next experiment has a 1.5x complete-warm speedup gate and must record speech-token/PCM divergence, listening quality and peak VRAM.
- **Uncertainty:** The reported T4 ratio may not transfer to Ampere; official vLLM may hit transient 10 GiB initialization pressure; community runtimes may change sampling or intonation.
- **Reconsider when:** A local full-precision hybrid or official vLLM result falsifies the stage-based ranking, or concurrency rather than single-request latency becomes the primary requirement.

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: Official FP16 inference fits in 10 GiB VRAM | Confirmed: complete benchmark peaked at 8,115 MiB total GPU memory | About 2.1 GiB device headroom is not enough to assume safe multi-request concurrency | Treat one resident model/single request as the proven boundary. |
| H2: Russian zero-shot quality exceeds the rejected Fish audio.cpp example | Russian is officially supported and two valid examples are archived | Not established: user judged the official-reference timbre childlike, while the desired adult reference was not tested | Repeat only with a clean authorized adult reference and compare pronunciation, prosody and artifacts separately. |
| H3: Warm RTF is competitive with Fish Q4/audio.cpp Q8 | Confirmed technically: 0.582 versus Fish Q4 0.874 and audio.cpp Q8 0.659 | Different runtimes and model/reference packages prevent attributing the difference to parameter count alone | Keep comparison observational and use the same fixed text in later evaluations. |
| H4: Replacing the LLM backend can materially reduce local warm RTF without harming quality | LLM is 84.25% of measured wall; an open hybrid reports 2.6x end-to-end on T4 with F16 GGUF | No optimized backend has been measured on this RTX 3080; AR sampling changes can alter intonation | Run pinned F16/BF16 hybrid A/B before any quantized or full-runtime test. |

## Required context

Read these sources before acting:

1. `AGENTS.md`.
2. `docs/development/tts-model-evaluations/fish-s2-pro/README.md` for the comparison protocol only.
3. `docs/development/tts-model-evaluations/fun-cosyvoice3-0.5b-2512/acceleration-research-2026-08-17.md` before selecting an optimized backend.
4. Current official `QwenAudio/CosyVoice` source, `FunAudioLLM/Fun-CosyVoice3-0.5B-2512` model card/files and their licenses.

## Next action

1. Continue with the next model using the same fixed timing protocol, unless CosyVoice acceleration is explicitly selected.
2. If CosyVoice speed is selected, test the pinned F16/BF16 llama.cpp LLM hybrid first; require at least 1.5x complete warm speedup and no new listening defect.
3. If F16/BF16 passes, compare Q8_0 and Q5_K_M. If it fails for non-quantization reasons, test official vLLM in a separate environment.
4. Use a clean authorized adult reference before interpreting optimized-path timbre as production quality.

## Do not retry

- Installing into the repository or global Python environment — it would contaminate unrelated development and violate artifact hygiene; use a dedicated external root.
- Treating HTTP header arrival or generator startup as TTFA — measure the first emitted PCM boundary when streaming is evaluated.

## Handoff

- **Workspace state:** Repository-owned wrapper, focused tests, durable report and task-state are tracked; source, model, venv, logs, raw summaries and audio remain under machine-local external roots.
- **Checks:** All 15 model artifacts, source/submodule revisions, reference hash, CUDA ORT provider, focused unit tests, generated WAV structure, archive copies and same-session offline/streaming benchmarks passed.
- **Remaining risk:** Adult-reference Russian quality, multi-request concurrency, broader text coverage and every optimized backend on the RTX 3080 remain unmeasured; community and L20 results are not local claims.
- **Promotion needed:** None for a bounded lab experiment.
