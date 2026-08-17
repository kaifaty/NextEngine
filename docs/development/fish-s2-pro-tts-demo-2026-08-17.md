# Fish S2 Pro local TTS demo (RTX 3080)

This is a bounded local experiment, not a shipped NextEngine TTS backend. The
community `s2.cpp` engine is alpha software. Its source and Fish S2 Pro weights
use the Fish Audio Research License: research/non-commercial use is allowed by
the license terms, while commercial use requires a separate Fish Audio license.

## Installed external closure

The machine-local root is:

```text
/home/kaifaty/.local/share/nextengine/fish-s2-pro
```

| Artifact | Immutable identity |
| --- | --- |
| `rodrigomatta/s2.cpp` | commit `2c33261938da1a41d713768b1b391b4d368d7d2c` |
| bundled `ggml` submodule | commit `57ea0bc119d722d74594196cc5b494a34dd87be4` |
| `rodrigomt/s2-pro-gguf` | revision `a7320690b5585b03b20ed6484b55926f3015f48d` |
| Q4_K_M | 3,566,165,088 bytes; SHA-256 `83963e1b7cec980b41eb2163d617e2b6241bfd1564dd880e5b43fc4834807bd9` |
| Q5_K_M | 4,031,183,968 bytes; SHA-256 `e445b0c8f32ed0ff584b906098f0fe53a67c0691249bfcccde569544f7d72cb9` |
| Q6_K | 4,525,266,528 bytes; SHA-256 `84ac904172a2cadb84e8f7f14ea3f1acef0584987635e85f7207fd254eafa235` |
| Q8_0 | 5,630,037,088 bytes; SHA-256 `e2043182234786e7b975547d3bbcb23ff02e4ff684b82f7fa851287e4cb4f267` |
| tokenizer | 12,217,872 bytes; SHA-256 `f24e08099d45a8adf3f52f5f0b03276e433bb9d689bb15fcbcc48ce58744588b` |

The CUDA release build targets the detected RTX 3080 `sm_86`. Weights,
reference voices, logs and generated WAV files stay under this external root
and never enter Git.

### audio.cpp alternative closure

The faster alternative is installed separately at:

```text
/home/kaifaty/.local/share/nextengine/audio-cpp-fish-s2-pro
```

| Artifact | Immutable identity |
| --- | --- |
| `0xShug0/audio.cpp` | commit `980bd4164b9de744b618a6b0d5e6e515de94999a` |
| `audio-cpp/audio.cpp-gguf` | revision `c3857f1ec35cfea8993924e7c2a6f682b5dc060b` |
| audio.cpp Q8_0 package | 6,317,911,232 bytes; SHA-256 `4ffc169447b7a26df8bf49e8637adb4000bfa763a22c018b6c03968564259d0b` |

This is a different GGUF package with audio.cpp-specific metadata and tensor
names; it is not interchangeable with the `s2.cpp` files above. The custom
deployment build includes only `fish_audio` plus required VAD dependencies,
targets CUDA architecture 86, and has CUDA Graphs enabled. The audio.cpp source
is Apache-2.0; the Fish S2 Pro model remains subject to the Fish Audio Research
License and is not cleared here for commercial use.

## Quick demo

From the NextEngine repository root:

```bash
python3 lab/scripts/fish_s2_pro_demo.py doctor

python3 lab/scripts/fish_s2_pro_demo.py generate --force \
  --text 'Привет! Это демонстрация Fish S2 Pro на локальном компьютере.'
```

The wrapper verifies the pinned code, tokenizer and selected GGUF hash before
running. It defaults to Q6_K with all 36 transformer layers and the codec on
CUDA. The generated file is placed outside the repository at:

```text
/home/kaifaty/.local/share/nextengine/fish-s2-pro/outputs/fish-s2-pro-q6-demo.wav
```

Select either lower-bit comparator explicitly:

```bash
python3 lab/scripts/fish_s2_pro_demo.py generate --force --quality q4 \
  --text 'Привет! Сервер синтеза речи работает локально.'

python3 lab/scripts/fish_s2_pro_demo.py generate --force --quality q5 \
  --text 'Привет! Сервер синтеза речи работает локально.'
```

Q4_K_M and Q5_K_M keep the codec on CUDA on this host. Q6_K remains the
wrapper default until a human listening comparison decides whether the Q4
quality trade-off is acceptable.

Use the quality-first Q8_0 comparator with its codec on CPU:

```bash
python3 lab/scripts/fish_s2_pro_demo.py generate --force \
  --quality q8 \
  --text 'Привет! Это проверка качества квантованной модели.'
```

On this 10 GiB card, forcing the Q8 codec to CUDA is not a supported demo
profile: the current runtime allocation probe fails and `s2.cpp` falls back to
CPU. The wrapper therefore avoids the known failed allocation by default.

## Faster audio.cpp demo

Validate the pinned external installation once, then generate without repeating
the 6.3 GB hash pass on every request:

```bash
python3 lab/scripts/fish_s2_pro_audio_cpp_demo.py doctor

python3 lab/scripts/fish_s2_pro_audio_cpp_demo.py generate \
  --skip-model-hash-check --force \
  --text 'Привет! Сервер синтеза речи работает локально.'
```

The generated mono PCM16 44.1 kHz file is external at:

```text
/home/kaifaty/.local/share/nextengine/audio-cpp-fish-s2-pro/outputs/fish-s2-pro-audio-cpp-q8-demo.wav
```

Run the reproducible warm benchmark with one disposable prewarm request and
three measured requests in the same model session:

```bash
python3 lab/scripts/fish_s2_pro_audio_cpp_demo.py benchmark \
  --skip-model-hash-check --iterations 3
```

The wrapper keeps `fish_audio.mem_saver=false`, Q8 weights native, CUDA Graphs
enabled and helper threads at 4. Increasing helper threads to 8 and enabling
the experimental `GGML_CUDA_GRAPH_OPT=1` did not improve the bounded local
probe, so neither override is part of the profile.

## Local HTTP demo

Start the server on loopback only:

```bash
python3 lab/scripts/fish_s2_pro_demo.py server
```

In another terminal, request a finalized WAV:

```bash
curl --fail --silent --show-error \
  -X POST http://127.0.0.1:3030/generate \
  -F 'text=Привет! Сервер синтеза речи работает локально.' \
  -F 'params={"max_new_tokens":384,"stream":false}' \
  --output /home/kaifaty/.local/share/nextengine/fish-s2-pro/outputs/http-demo.wav
```

The upstream server also exposes experimental chunked streaming:

```bash
curl --fail --silent --show-error --no-buffer \
  -X POST http://127.0.0.1:3030/generate \
  -F 'text=Это экспериментальная потоковая генерация.' \
  -F 'params={"max_new_tokens":384,"stream":true,"chunked":true,"low_latency":true,"output_format":"wav"}' \
  --output /home/kaifaty/.local/share/nextengine/fish-s2-pro/outputs/http-stream-demo.wav
```

Do not expose the alpha server beyond loopback without reviewing its network
boundary. The wrapper requires `--allow-non-loopback` for such a bind.

## Measured probes

Hardware: NVIDIA GeForce RTX 3080 10 GiB, driver 610.43.02, CUDA 13.3. The
wrapper validation runs used short Russian prompts and `max_tokens=384`;
sampling and different prompts mean the output durations differ, so the figures
are local setup observations rather than a quality comparison or product gate.

| Profile | Codec | Audio | Peak total GPU memory | Synthesis RTF from upstream metrics | Cold process wall time |
| --- | --- | ---: | ---: | ---: | ---: |
| Q6_K | CUDA | 5.155 s | 6,205 MiB | 3.035 | 18.14 s |
| Q8_0 | CPU | 6.130 s | 5,701 MiB | 5.650 | 37.88 s |

Q8_0 remains useful only as a listening comparator on this host. The lower-bit
profiles were then compared with the exact text `Привет! Сервер синтеза речи
работает локально.` and finalized, non-streaming WAV output. Each server was
started fresh, request 0 was used to construct/cache the single-token step
graphs, and requests 1–3 were measured as steady warm runs:

| Profile | Codec | First request after init | Warm wall time, mean | Warm audio, mean | Warm upstream RTF, mean |
| --- | --- | ---: | ---: | ---: | ---: |
| Q4_K_M | CUDA | 9.864 s | 2.910 s | 3.297 s | 0.874 |
| Q5_K_M | CUDA | 10.417 s | 3.254 s | 3.514 s | 0.918 |
| Q6_K | CUDA | 11.007 s | 3.299 s | 3.142 s | 1.040 |

The first synthesis after model initialization is not representative of a
steady warm server: it remains roughly three times slower even though model
loading is already complete. A service intended for interactive use must run
one disposable synthesis during prewarm. Under this small fixed-text probe,
Q4_K_M is about 16% faster than Q6_K by upstream warm RTF and uses less VRAM;
Q5_K_M is about 12% faster. These three-sample observations are not latency
percentiles or a corpus quality result, and none satisfies the Proposed
SPEC-16 target of RTF at most 0.5.

The exact per-request rows and external WAV files are retained outside Git at
`/home/kaifaty/.local/share/nextengine/fish-s2-pro/outputs/`, including
`q456-warm-benchmark.csv`.

The audio.cpp Q8_0 package was measured with the same text, seed 1234 and
`max_tokens=384`. Its repository-owned benchmark entry point reported:

| Engine/profile | Warm wall, mean | Warm audio | Warm RTF, mean | Speed vs realtime | Peak total GPU memory |
| --- | ---: | ---: | ---: | ---: | ---: |
| `audio.cpp` Q8_0 | 2.816 s | 4.272 s | 0.659 | 1.52x | 8,189 MiB |

On this one fixed-text probe, audio.cpp Q8_0 has about 25% lower RTF and 1.33x
the realtime throughput of the `s2.cpp` Q4_K_M result. It also uses about
3.1 GiB more peak total GPU memory. Two uninstrumented repeat processes placed
audio.cpp warm RTF between 0.627 and 0.656; the instrumented wrapper result
above is retained as the conservative reproducible comparison. Fixed-seed
audio.cpp outputs were byte-identical.

Detailed timing attributes about 2.67-2.69 s of a warm request to the
autoregressive generator and only 116-117 ms to codec decode. The remaining
speed ceiling is therefore the 92 slow plus 920 fast autoregressive graph
executions, not model loading, the codec or CPU helper thread count. The result
still misses the Proposed SPEC-16 RTF target of at most 0.5 and does not prove
subjective Russian quality.

The chunked low-latency request returned HTTP 200 and valid mono PCM16 44.1 kHz
WAV (2.833 s), but upstream metrics reported RTF 6.24 because the alpha
streaming path repeatedly decodes its prefix. HTTP header arrival is not a
first-audio measurement, so no TTFA claim is made.

## Voice cloning boundary

The basic demo intentionally does not accept reference audio. Voice cloning
must use a recording whose speaker/rights holder authorized this exact use and
must keep both reference audio and generated profiles outside Git. The direct
`s2.cpp` flags are `--prompt-audio`, `--prompt-text`, `--voice` and
`--save-voice`; audio.cpp uses `--voice-ref` and `--reference-text`. Enabling
either path is a separate consent-aware experiment.

## Upstream sources

- <https://github.com/rodrigomatta/s2.cpp>
- <https://huggingface.co/rodrigomt/s2-pro-gguf>
- <https://huggingface.co/fishaudio/s2-pro>
- <https://github.com/fishaudio/fish-speech/blob/main/docs/en/install.md>
- <https://github.com/0xShug0/audio.cpp>
- <https://github.com/0xShug0/audio.cpp/blob/main/model_specs/fish_audio.json>
- <https://github.com/0xShug0/audio.cpp/blob/main/docs/reports/gguf_q8_performance.md>
- <https://huggingface.co/audio-cpp/audio.cpp-gguf/tree/main>
