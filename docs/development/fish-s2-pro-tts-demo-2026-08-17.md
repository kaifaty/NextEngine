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
| Q6_K | 4,525,266,528 bytes; SHA-256 `84ac904172a2cadb84e8f7f14ea3f1acef0584987635e85f7207fd254eafa235` |
| Q8_0 | 5,630,037,088 bytes; SHA-256 `e2043182234786e7b975547d3bbcb23ff02e4ff684b82f7fa851287e4cb4f267` |
| tokenizer | 12,217,872 bytes; SHA-256 `f24e08099d45a8adf3f52f5f0b03276e433bb9d689bb15fcbcc48ce58744588b` |

The CUDA release build targets the detected RTX 3080 `sm_86`. Weights,
reference voices, logs and generated WAV files stay under this external root
and never enter Git.

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

Use the quality-first Q8_0 comparator with its codec on CPU:

```bash
python3 lab/scripts/fish_s2_pro_demo.py generate --force \
  --quality q8 \
  --text 'Привет! Это проверка качества квантованной модели.'
```

On this 10 GiB card, forcing the Q8 codec to CUDA is not a supported demo
profile: the current runtime allocation probe fails and `s2.cpp` falls back to
CPU. The wrapper therefore avoids the known failed allocation by default.

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

## Measured first probe

Hardware: NVIDIA GeForce RTX 3080 10 GiB, driver 610.43.02, CUDA 13.3. The
wrapper validation runs used short Russian prompts and `max_tokens=384`;
sampling and different prompts mean the output durations differ, so the figures
are local setup observations rather than a quality comparison or product gate.

| Profile | Codec | Audio | Peak total GPU memory | Synthesis RTF from upstream metrics | Cold process wall time |
| --- | --- | ---: | ---: | ---: | ---: |
| Q6_K | CUDA | 5.155 s | 6,205 MiB | 3.035 | 18.14 s |
| Q8_0 | CPU | 6.130 s | 5,701 MiB | 5.650 | 37.88 s |

Q6_K is the practical default on this host. Q8_0 remains useful for listening
comparisons, but neither result satisfies the Proposed SPEC-16 target of
real-time factor at most 0.5. No production-readiness claim is made.

The local Q6 server smoke returned HTTP 200 and a valid mono float32 44.1 kHz
WAV (3.947 s of audio) in 12.43 s. The chunked low-latency request also returned
HTTP 200 and valid mono PCM16 44.1 kHz WAV (2.833 s), but upstream metrics
reported RTF 6.24 because the alpha streaming path repeatedly decodes its
prefix. HTTP header arrival is not a first-audio measurement, so no TTFA claim
is made.

## Voice cloning boundary

The basic demo intentionally does not accept reference audio. Voice cloning
must use a recording whose speaker/rights holder authorized this exact use and
must keep both reference audio and generated profiles outside Git. The direct
upstream flags are `--prompt-audio`, `--prompt-text`, `--voice` and
`--save-voice`; enabling them is a separate consent-aware experiment.

## Upstream sources

- <https://github.com/rodrigomatta/s2.cpp>
- <https://huggingface.co/rodrigomt/s2-pro-gguf>
- <https://huggingface.co/fishaudio/s2-pro>
- <https://github.com/fishaudio/fish-speech/blob/main/docs/en/install.md>
