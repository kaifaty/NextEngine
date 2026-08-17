# Fun-CosyVoice3-0.5B-2512 local TTS evaluation (2026-08-17)

## Result

This was a bounded local evaluation, not a shipped NextEngine backend.

The pinned official FP16/PyTorch path passed on the RTX 3080. On the fixed
Russian prompt it produced 3.680 s of audio in a conservative warm mean of
2.142 s: RTF 0.582, or about 1.72x realtime. The repeat mean was RTF 0.571.
Peak total GPU memory was 7,981-8,116 MiB across the retained offline runs.

This is technically faster than both retained Fish S2 Pro comparators, but the
quality decision remains a human listening task:

| Candidate | Warm RTF | Approx. realtime speed | Peak total GPU | Human result |
| --- | ---: | ---: | ---: | --- |
| Fun-CosyVoice3 official FP16 | 0.582 | 1.72x | 7,981 MiB | Childlike timbre with the bundled reference; unsuitable as the target adult voice without a new reference |
| Fish `audio.cpp` Q8_0 | 0.659 | 1.52x | 8,189 MiB | Rejected: poor quality |
| Fish `s2.cpp` Q4_K_M | 0.874 | 1.14x | 5,124 MiB | Generally acceptable |

Relative to those fixed-prompt results, CosyVoice had about 11.7% lower RTF
than Fish `audio.cpp` Q8_0 and 33.4% lower RTF than Fish `s2.cpp` Q4_K_M. This
is an observational local comparison, not a model-only attribution: runtimes,
reference-voice paths and output models differ.

The user listening verdict is that both generated examples sound childlike.
No child/adolescent style was requested: the only text prefix was the mandatory
`You are a helpful assistant.<|endofprompt|>`. Speaker timbre came from the
bundled `asset/zero_shot_prompt.wav`. Therefore the measured speed remains a
valid runtime baseline, but the archived voice is not an acceptable adult-voice
quality sample. Re-evaluate quality only with a clean, authorized adult speaker
reference; do not try to fix speaker identity by changing the benchmark text.

## Acceleration research

The follow-up [acceleration research](acceleration-research-2026-08-17.md)
found no separate official quantized fast checkpoint. Official acceleration
replaces runtime stages with vLLM, TensorRT or the full Triton/TensorRT-LLM
stack; community options include a hybrid llama.cpp LLM, complete GGML ports
and a Candle/Rust implementation.

A new five-request local profile measured the warm request at 2.035 s / RTF
0.553 and preserved the baseline decoded PCM hash. Mean wall-time composition
was 84.25% autoregressive LLM, 11.51% flow, 1.81% HiFT and 2.42% frontend plus
other overhead. Consequently, even a perfect flow replacement has only a
1.13x end-to-end ceiling on the fixed prompt, while LLM acceleration can be
material.

The smallest next A/B is the open hybrid `llama-cpp-python` path with an F16
or BF16 LLM GGUF while keeping the official PyTorch flow and HiFT. The PR author
reports a 2.6x end-to-end T4 result, but that number was not reproduced on this
RTX 3080. If F16/BF16 preserves listening quality, compare Q8_0 and Q5_K_M.
Official vLLM is the next alternative; its current initialization order has a
transient-memory risk on the 10 GiB card. TensorRT-only flow, full Triton,
vLLM-Omni, full GGML and Rust remain later experiments.

## Evaluation boundary

| Field | Value |
| --- | --- |
| Date | 2026-08-17 |
| GPU | NVIDIA GeForce RTX 3080, 10,240 MiB |
| Driver | 610.43.02 |
| Compute capability | 8.6 (`sm_86`) |
| Host CUDA toolkit | 13.3 |
| Runtime CUDA | PyTorch 12.1 wheels |
| Benchmark language | Russian, cross-lingual zero-shot path |
| Fixed text | `Привет! Сервер синтеза речи работает локально.` |
| Long text | `Капитан, западные ворота снова открыты. Если мы выйдем до рассвета, стража не успеет перекрыть старую дорогу.` |
| Seed | 1234, reset before every request |
| Precision | Official PyTorch path with FP16 autocast |
| Text frontend | Disabled; direct Russian tokens with required CosyVoice3 `<|endofprompt|>` prefix |
| Reference | Official repository `asset/zero_shot_prompt.wav`, used only for the local non-distributed demo |
| Warm protocol | Load once, prepare/caches speaker once, run one disposable synthesis, then measure three requests in the same process |

RTF is synthesis wall time divided by generated audio duration. Lower is
faster; RTF below 1 is faster than realtime. Process wall includes Python
imports, model loading, reference-speaker preparation and all requests.

## External installation

All third-party source, model files, Python packages, reference speech, raw
logs and generated WAVs remain outside Git:

```text
/home/kaifaty/.local/share/nextengine/fun-cosyvoice3-0.5b-2512
├── model/
├── outputs/
├── source/
└── venv/
```

| Artifact | Immutable identity |
| --- | --- |
| `QwenAudio/CosyVoice` | commit `074ca6dc9e80a2f424f1f74b48bdd7d3fea531cc` |
| bundled `Matcha-TTS` | commit `dd9105b34bf2be2230f4aa1e4769fb586a3c824e` |
| `FunAudioLLM/Fun-CosyVoice3-0.5B-2512` | revision `29e01c4e8d000f4bcd70751be16fa94bf3d85a18` |
| Selected base-model closure | 5,427,041,134 bytes; all 15 expected size/SHA-256 checks passed |
| Official reference WAV | 334,138 bytes; SHA-256 `c7b31d6dbe7cc6a716dded00550db5b50940bf209e424e4ad207b12e657c8ff6` |

The selected closure excludes the separate RL checkpoint, TensorRT flow ONNX
and batch tokenizer because none is used by this baseline. The largest model
artifacts are:

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `llm.pt` | 2,024,669,519 | `69f43bd545131c30e98947fb360ea8b4dc9916d8e83dded7757c7ea4f5a24970` |
| `flow.pt` | 1,329,116,148 | `a6fab32a7825e5b0bc855ddd948f8db9370b0a786fbc249caa4595e95b608e4b` |
| `CosyVoice-BlankEN/model.safetensors` | 988,097,824 | `130282af0dfa9fe5840737cc49a0d339d06075f83c5a315c3372c9a0740d0b96` |
| `speech_tokenizer_v3.onnx` | 969,451,503 | `23236a74175dbdda47afc66dbadd5bcb41303c467a57c261cb8539ad9db9208d` |
| `hift.pt` | 83,202,622 | `b279d7641eb97ae55b3b540cfba4f953c26492a2df758328a89a4d007ab87a65` |
| `campplus.onnx` | 28,303,423 | `a6ac6a63997761ae2997373e2ee1c47040854b4b759ea41ec48e4e42df0f4d73` |

The full SHA closure is encoded in the repository wrapper and rechecked by
`doctor`. Upstream publishes the model under Apache-2.0 and lists Russian among
its supported languages. Voice cloning still requires an authorized reference
recording and an explicit consent-aware evaluation.

## Runtime closure

| Component | Version/result |
| --- | --- |
| Python | 3.10.20 |
| PyTorch / torchaudio | 2.3.1+cu121 / 2.3.1+cu121 |
| ONNX Runtime GPU | 1.18.0 from the upstream CUDA-12 feed |
| Speech tokenizer providers | `CUDAExecutionProvider`, then CPU fallback |
| Transformers | 4.51.3 |
| NumPy / protobuf | 1.26.4 / 4.25.8 |

The isolated runtime deliberately omits WeText. Even with
`text_frontend=False`, it attempted to fetch an unpinned multilingual FST
bundle; this fixed plain-Russian benchmark does not require normalization.
Numbers, abbreviations and mixed-language text need a separate pinned frontend
evaluation before production use.

The PyPI ONNX Runtime 1.18 wheel expected CUDA 11 libraries on this machine, so
the runtime uses the official CUDA-12 wheel referenced by upstream requirements.
The speech tokenizer's selected provider was verified directly, not inferred
from CUDA availability.

## Offline timing

Canonical retained benchmark:

```text
/home/kaifaty/.local/share/nextengine/tts-model-evaluations/fun-cosyvoice3-0.5b-2512-2026-08-17/raw/offline
```

| Phase | Wall | Audio | RTF | Realtime speed |
| --- | ---: | ---: | ---: | ---: |
| Prewarm | 2.687 s | 3.680 s | 0.730 | 1.37x |
| Warm 1 | 2.200 s | 3.680 s | 0.598 | 1.67x |
| Warm 2 | 2.118 s | 3.680 s | 0.576 | 1.74x |
| Warm 3 | 2.107 s | 3.680 s | 0.573 | 1.75x |
| Warm mean | 2.142 s | 3.680 s | 0.582 | 1.72x |

| Initialization/footprint | Canonical result |
| --- | ---: |
| Python/runtime imports | 4.436 s |
| Model load after imports | 8.847 s |
| Reference-speaker preparation | 2.030 s |
| Complete process wall | 25.937 s |
| Peak total GPU memory | 7,981 MiB |
| PyTorch peak allocated / reserved | 4,286 / 5,338 MiB |

An earlier uncached disk probe loaded the model in 17.31 s and spent 4.99 s in
the first complete reference-plus-synthesis call (RTF 1.356 for 3.68 s audio).
It validates the true cold path but is not the steady-state service number.

The retained repeat measured a warm mean of 2.102 s, RTF 0.571 and 1.75x
realtime with 8,116 MiB peak total GPU memory. The conservative comparison uses
the later instrumented RTF 0.582 rather than the fastest observed repeat.

The independent longer dialogue produced 5.960 s audio in 3.763 s, RTF 0.631,
after an 8.848 s load and 2.026 s speaker preparation. Peak total GPU memory
was 8,011 MiB and complete process wall was 20.573 s.

## Streaming probe

Canonical retained probe:

```text
/home/kaifaty/.local/share/nextengine/tts-model-evaluations/fun-cosyvoice3-0.5b-2512-2026-08-17/raw/streaming
```

The first streaming request emitted three chunks of 1.36, 2.00 and 0.32 s.
First returned PCM arrived after 1.825 s; full wall was 3.190 s (RTF 0.867).
During that request the upstream model mutated `token_hop_len` from 25 to 100.

The following same-session requests therefore emitted the complete 3.68 s as
one chunk. Their mean first-chunk time was 2.076 s, wall 2.124 s and RTF 0.577.
Peak total GPU memory was 8,115 MiB; complete process wall was 26.671 s.

Consequently, the upstream headline of latency “as low as 150 ms” was not
reproduced by this short Russian cross-lingual wrapper. The local result is
stateful and serving-configuration-specific; it must not be generalized into a
claim that the model cannot reach lower TTFA elsewhere. Before using streaming,
reset/ownership semantics for `token_hop_len` need an upstream-compatible fix
or one isolated model session per stream.

## Determinism

All three fixed-seed warm requests produced identical decoded PCM bytes:

```text
b0081dadddd2ce253608a7baf965799d6dab6913981bf0b8fe56b16efac2740d
```

Whole WAV SHA-256 values differed because torchaudio writes a `PEAK` metadata
chunk containing a timestamp. This is container metadata, not different audio.
Regression tests should hash decoded contiguous float32 PCM, as the wrapper
does, rather than the complete WAV file.

## Audio archive

The consolidated external artifact archive contains no model weights or
third-party source:

```text
/home/kaifaty/.local/share/nextengine/tts-model-evaluations/fun-cosyvoice3-0.5b-2512-2026-08-17
├── examples/
│   ├── fixed-russian-demo.wav
│   ├── offline-warm-fixed-prompt.wav
│   ├── russian-dialogue.wav
│   └── streaming-first-session-fixed-prompt.wav
└── raw/
    ├── generated/
    ├── offline/
    └── streaming/
```

| Example | Format/duration | WAV SHA-256 | Purpose |
| --- | --- | --- | --- |
| `fixed-russian-demo.wav` | mono float32, 24 kHz, 3.680 s | `444e3248f061ad140c9af45f0f08536f90cfc11246e4407897b0372320abf7e4` | Childlike timbre with official reference |
| `russian-dialogue.wav` | mono float32, 24 kHz, 5.960 s | `e884e3a100e4004decce218c8a1ae999cfaabb13988045c399c41ff688764a19` | Same childlike timbre on a longer RPG-style phrase |
| `offline-warm-fixed-prompt.wav` | mono float32, 24 kHz, 3.680 s | `7cc90aa0d16eac0ca56aad68e4943dc8c477a7fcef2eabbe064f8158672e3537` | Canonical warm request |
| `streaming-first-session-fixed-prompt.wav` | mono float32, 24 kHz, 3.680 s | `2da217662e91d0631522cc2da2623c462d8e88ada9b657e8212a77deda0055f1` | First-session three-chunk stream reassembled as WAV |

The recorded human verdict concerns the speaker timbre. Pronunciation, prosody,
speaker similarity to an adult reference and artifacts remain untested because
the bundled reference is not the desired voice.

## Reproduction

From the repository root:

```bash
python3 lab/scripts/fun_cosyvoice3_demo.py doctor

python3 lab/scripts/fun_cosyvoice3_demo.py generate --force \
  --text 'Привет! Сервер синтеза речи работает локально.'

python3 lab/scripts/fun_cosyvoice3_demo.py benchmark --iterations 3

python3 lab/scripts/fun_cosyvoice3_demo.py benchmark --stream --iterations 3
```

Use `--skip-model-hash-check` only for repeated local runs after `doctor`; it
still checks sizes and pinned source revisions, but avoids rereading 5.1 GiB.
The wrapper rejects model installations and generated outputs inside the Git
repository.

Repository-owned entry points:

- `lab/scripts/fun_cosyvoice3_demo.py`
- `lab/tests/test_fun_cosyvoice3_demo.py`

## Constraints and next experiments

- Treat one resident model and one request as the proven 10 GiB boundary; do
  not infer safe concurrency from approximately 2 GiB remaining VRAM.
- Keep the official PyTorch path as the quality/correctness baseline.
- Compare vLLM or TensorRT only as a pinned A/B experiment with identical text,
  seed, reference and listening protocol.
- Use a broader Russian corpus before making latency-percentile, normalization
  or production-quality claims.
- Keep weights, references, logs and generated audio outside Git.

## Official upstream sources

- <https://huggingface.co/FunAudioLLM/Fun-CosyVoice3-0.5B-2512>
- <https://github.com/QwenAudio/CosyVoice>
- <https://github.com/QwenAudio/CosyVoice/blob/main/requirements.txt>
