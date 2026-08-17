# Fun-CosyVoice3-0.5B-2512 TTS demo — current task state

| Field | Value |
| --- | --- |
| Status | `COMPLETE` |
| Updated | `2026-08-17` |
| Task key | `fun-cosyvoice3-tts-demo` |
| Scope | Install and evaluate the official Fun-CosyVoice3-0.5B-2512 CUDA inference path on the available RTX 3080, with Russian listening examples and reproducible cold/warm measurements, without integrating weights or runtime code into NextEngine. |
| Definition of done | A pinned external installation produces valid Russian WAV through a repository-owned wrapper; source/model identity, license, reference-audio boundary, cold/warm latency, RTF, VRAM and listening examples are recorded. |
| Authority | Working context only; `AGENTS.md`, Accepted architecture, exact upstream source/model revisions and raw external evidence outrank this file. |

## Resume in 60 seconds

- **Current conclusion:** The pinned official FP16/PyTorch baseline passes technically on the RTX 3080 at warm RTF 0.582 (1.72x realtime), but the user judged the bundled-reference voice childlike. Retain the speed result; do not treat the archived voice as an acceptable adult-voice quality sample.
- **Why:** The benchmark did not prompt an age/style: only the mandatory CosyVoice3 marker was added. Speaker timbre came from the official `asset/zero_shot_prompt.wav`, so a clean authorized adult reference is required for a meaningful adult-voice quality comparison.
- **Next action:** Continue with the next model, or rerun CosyVoice quality only when an authorized adult reference WAV is available.
- **Current blocker:** None.
- **Do not retry:** Do not put model weights, reference speech, generated WAV files, Python environments or raw logs in Git.
- **Reconsider when:** A pinned official vLLM/TensorRT path is evaluated, a Russian-aware text frontend is needed, or a broader prompt corpus replaces the fixed short benchmark.

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

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: Official FP16 inference fits in 10 GiB VRAM | Confirmed: complete benchmark peaked at 8,115 MiB total GPU memory | About 2.1 GiB device headroom is not enough to assume safe multi-request concurrency | Treat one resident model/single request as the proven boundary. |
| H2: Russian zero-shot quality exceeds the rejected Fish audio.cpp example | Russian is officially supported and two valid examples are archived | Not established: user judged the official-reference timbre childlike, while the desired adult reference was not tested | Repeat only with a clean authorized adult reference and compare pronunciation, prosody and artifacts separately. |
| H3: Warm RTF is competitive with Fish Q4/audio.cpp Q8 | Confirmed technically: 0.582 versus Fish Q4 0.874 and audio.cpp Q8 0.659 | Different runtimes and model/reference packages prevent attributing the difference to parameter count alone | Keep comparison observational and use the same fixed text in later evaluations. |

## Required context

Read these sources before acting:

1. `AGENTS.md`.
2. `docs/development/tts-model-evaluations/fish-s2-pro/README.md` for the comparison protocol only.
3. Current official `QwenAudio/CosyVoice` source, `FunAudioLLM/Fun-CosyVoice3-0.5B-2512` model card/files and their licenses.

## Next action

1. Continue with the next model using the same fixed timing protocol.
2. If CosyVoice is reconsidered, obtain a clean authorized adult reference and rerun only the quality comparison first.
3. Optimize CosyVoice only as a separately pinned A/B task after voice quality is acceptable.

## Do not retry

- Installing into the repository or global Python environment — it would contaminate unrelated development and violate artifact hygiene; use a dedicated external root.
- Treating HTTP header arrival or generator startup as TTFA — measure the first emitted PCM boundary when streaming is evaluated.

## Handoff

- **Workspace state:** Repository-owned wrapper, focused tests, durable report and task-state are tracked; source, model, venv, logs, raw summaries and audio remain under machine-local external roots.
- **Checks:** All 15 model artifacts, source/submodule revisions, reference hash, CUDA ORT provider, focused unit tests, generated WAV structure, archive copies and same-session offline/streaming benchmarks passed.
- **Remaining risk:** Adult-reference Russian quality, multi-request concurrency, broader text coverage and optimized backends remain unmeasured.
- **Promotion needed:** None for a bounded lab experiment.
