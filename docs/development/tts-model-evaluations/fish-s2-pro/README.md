# Fish S2 Pro local TTS evaluation (2026-08-17)

## Final result

This was a bounded local evaluation, not a shipped NextEngine backend.

| Candidate | Measured warm result | Human listening result | Decision |
| --- | --- | --- | --- |
| `s2.cpp` Q4_K_M | RTF 0.874; 1.14x realtime; 5,124 MiB peak total GPU | Generally acceptable quality | Keep only as a recorded baseline: quality is usable, but latency misses the RTF 0.5 experiment target. |
| `audio.cpp` Q8_0 | RTF 0.659; 1.52x realtime; 8,189 MiB peak total GPU | Poor/insufficient quality in `fish-s2-pro-audio-cpp-q8-demo.wav` | Reject as the next integration candidate despite its higher speed. |

Fish S2 Pro evaluation is complete for the pinned runtimes below. Continue with
other TTS models. Do not infer quality from the quantization label alone:
`audio.cpp` uses a different model package, inference implementation and output
format from `s2.cpp`, and its Q8 example sounded worse than the Q4_K_M example.

## Evaluation boundary

| Field | Value |
| --- | --- |
| Date | 2026-08-17 |
| GPU | NVIDIA GeForce RTX 3080, 10,240 MiB |
| Driver | 610.43.02 |
| Compute capability | 8.6 (`sm_86`) |
| CUDA toolkit | 13.3 |
| Benchmark language | Russian |
| Fixed benchmark text | `Привет! Сервер синтеза речи работает локально.` |
| audio.cpp sampling | seed 1234; maximum 384 tokens |
| Warm protocol | Load once, run one disposable complete synthesis, measure the next three requests in the same session. |

The outputs have different durations because generation is sampled. The tables
are local observations on one GPU and one short prompt, not latency percentiles
or a corpus-level quality result. RTF is wall time divided by generated audio
duration; a lower value is faster, and RTF below 1 is faster than realtime.

The consolidated external artifact archive is:

```text
/home/kaifaty/.local/share/nextengine/tts-model-evaluations/fish-s2-pro-2026-08-17
├── examples/       # seven listening examples
└── raw/
    ├── audio-cpp/  # canonical benchmark, repeats, tuning and stage profile
    └── s2-cpp/     # cold logs, GPU samples and exact warm CSV
```

It contains no model weights or third-party source. The original runtime output
directories are also retained under the installation roots documented below.

## Pinned external installations

Model weights, third-party source, logs, raw benchmark payloads and generated
WAV files remain outside Git.

### s2.cpp

External root:

```text
/home/kaifaty/.local/share/nextengine/fish-s2-pro
```

| Artifact | Immutable identity |
| --- | --- |
| `rodrigomatta/s2.cpp` | commit `2c33261938da1a41d713768b1b391b4d368d7d2c` |
| bundled `ggml` | commit `57ea0bc119d722d74594196cc5b494a34dd87be4` |
| `rodrigomt/s2-pro-gguf` | revision `a7320690b5585b03b20ed6484b55926f3015f48d` |
| Q4_K_M | 3,566,165,088 bytes; SHA-256 `83963e1b7cec980b41eb2163d617e2b6241bfd1564dd880e5b43fc4834807bd9` |
| Q5_K_M | 4,031,183,968 bytes; SHA-256 `e445b0c8f32ed0ff584b906098f0fe53a67c0691249bfcccde569544f7d72cb9` |
| Q6_K | 4,525,266,528 bytes; SHA-256 `84ac904172a2cadb84e8f7f14ea3f1acef0584987635e85f7207fd254eafa235` |
| Q8_0 | 5,630,037,088 bytes; SHA-256 `e2043182234786e7b975547d3bbcb23ff02e4ff684b82f7fa851287e4cb4f267` |
| tokenizer | 12,217,872 bytes; SHA-256 `f24e08099d45a8adf3f52f5f0b03276e433bb9d689bb15fcbcc48ce58744588b` |

The CUDA release build targets `sm_86`. Q4_K_M, Q5_K_M and Q6_K keep all 36
transformer layers, KV cache and codec on CUDA. Q8_0 offloads the transformer,
but its 5,369.16 MiB CUDA codec allocation failed on this 10 GiB card, so the
measured Q8_0 profile decoded on CPU.

### audio.cpp

External root:

```text
/home/kaifaty/.local/share/nextengine/audio-cpp-fish-s2-pro
```

| Artifact | Immutable identity |
| --- | --- |
| `0xShug0/audio.cpp` | commit `980bd4164b9de744b618a6b0d5e6e515de94999a` |
| `audio-cpp/audio.cpp-gguf` | revision `c3857f1ec35cfea8993924e7c2a6f682b5dc060b` |
| audio.cpp Q8_0 package | 6,317,911,232 bytes; SHA-256 `4ffc169447b7a26df8bf49e8637adb4000bfa763a22c018b6c03968564259d0b` |

This GGUF is not interchangeable with the `s2.cpp` GGUF files. The custom
deployment build includes `fish_audio` and its required VAD dependencies,
targets CUDA architecture 86 and enables CUDA Graphs. The retained profile uses
native Q8 weights, `fish_audio.mem_saver=false` and four helper threads.

The audio.cpp source is Apache-2.0. Fish S2 Pro weights use the Fish Audio
Research License: research/non-commercial use is permitted by those terms;
commercial use requires a separate Fish Audio license.

## s2.cpp timing results

### Initial standalone probes

| Profile | Codec | Audio | Synthesis RTF | Cold process wall | Peak total GPU |
| --- | --- | ---: | ---: | ---: | ---: |
| Q4_K_M | CUDA | 3.390 s | 3.053 | 12.800 s | 5,124 MiB |
| Q5_K_M | CUDA | 3.297 s | 3.088 | 12.559 s | 5,499 MiB |
| Q6_K | CUDA | 5.155 s | 3.035 | 18.14 s | 6,205 MiB |
| Q8_0 | CPU | 6.130 s | 5.650 | 37.88 s | 5,701 MiB |

These cold values include different initialization paths and differently sampled
outputs. They are retained as installation validation, not the primary speed
comparison.

### Fixed-text same-session benchmark

Model initialization alone was not enough to warm the runtime. Request 0 built
or cached single-token step graphs and was discarded; requests 1-3 are the
steady-warm sample.

| Profile | Request | Phase | Frames | Audio (s) | Generate (ms) | Decode (ms) | Upstream total (ms) | RTF | HTTP wall (s) |
| --- | ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Q4_K_M | 0 | first after init | 69 | 3.204 | 9,415.025 | 416.307 | 9,832.413 | 3.068 | 9.864 |
| Q4_K_M | 1 | warm | 72 | 3.344 | 2,568.234 | 341.771 | 2,911.338 | 0.871 | 2.941 |
| Q4_K_M | 2 | warm | 71 | 3.297 | 2,544.706 | 337.911 | 2,883.815 | 0.875 | 2.912 |
| Q4_K_M | 3 | warm | 70 | 3.251 | 2,508.740 | 336.803 | 2,846.803 | 0.876 | 2.876 |
| Q5_K_M | 0 | first after init | 71 | 3.297 | 9,963.452 | 422.079 | 10,386.536 | 3.150 | 10.417 |
| Q5_K_M | 1 | warm | 70 | 3.251 | 2,642.981 | 334.379 | 2,978.507 | 0.916 | 3.008 |
| Q5_K_M | 2 | warm | 77 | 3.576 | 2,919.522 | 354.616 | 3,275.327 | 0.916 | 3.305 |
| Q5_K_M | 3 | warm | 80 | 3.715 | 3,054.251 | 363.857 | 3,419.270 | 0.920 | 3.448 |
| Q6_K | 0 | first after init | 72 | 3.344 | 10,533.544 | 444.217 | 10,978.913 | 3.283 | 11.007 |
| Q6_K | 1 | warm | 66 | 3.065 | 2,806.008 | 341.573 | 3,148.908 | 1.027 | 3.179 |
| Q6_K | 2 | warm | 73 | 3.390 | 3,150.228 | 371.375 | 3,523.026 | 1.039 | 3.554 |
| Q6_K | 3 | warm | 64 | 2.972 | 2,792.588 | 338.501 | 3,132.531 | 1.054 | 3.163 |

| Profile | First request after init | Warm wall mean | Warm audio mean | Warm RTF mean | Approx. realtime speed |
| --- | ---: | ---: | ---: | ---: | ---: |
| Q4_K_M | 9.864 s | 2.910 s | 3.297 s | 0.874 | 1.14x |
| Q5_K_M | 10.417 s | 3.254 s | 3.514 s | 0.918 | 1.09x |
| Q6_K | 11.007 s | 3.299 s | 3.142 s | 1.040 | 0.96x |

On this prompt Q4_K_M was about 16% faster than Q6_K by warm RTF; Q5_K_M
was about 12% faster. None reached the experimental RTF target of 0.5.

Raw rows:

```text
/home/kaifaty/.local/share/nextengine/tts-model-evaluations/fish-s2-pro-2026-08-17/raw/s2-cpp/q456-warm-benchmark.csv
```

### HTTP probes

| Path | Result |
| --- | --- |
| Finalized `/generate` | HTTP 200; mono float32 44.1 kHz WAV; 3.947 s audio. |
| Chunked low-latency request | HTTP 200; mono PCM16 44.1 kHz WAV; 2.833 s audio; upstream RTF 6.24. |

Chunked streaming repeatedly decoded its prefix. HTTP header arrival was not a
first-PCM-byte measurement, so no TTFA claim was made.

## audio.cpp Q8_0 timing results

### Canonical instrumented benchmark

The repository wrapper ran one prewarm plus three measured requests in one
process. All fixed-seed outputs were byte-identical.

| Request | Wall (ms) | Audio (ms) | RTF | Realtime speed |
| --- | ---: | ---: | ---: | ---: |
| prewarm | 3,007.00 | 4,272.47 | 0.703809 | 1.42084x |
| warm 1 | 2,788.43 | 4,272.47 | 0.652650 | 1.53221x |
| warm 2 | 2,821.54 | 4,272.47 | 0.660400 | 1.51423x |
| warm 3 | 2,837.71 | 4,272.47 | 0.664184 | 1.50561x |
| warm mean | 2,815.89 | 4,272.47 | 0.659078 | 1.51735x |

Peak total GPU memory was 8,189 MiB. The complete benchmark process took
24.389 s including model load, prewarm and all three measured requests.

Compared on the fixed prompt, audio.cpp Q8_0 had about 25% lower RTF and 1.33x
the throughput of `s2.cpp` Q4_K_M, while consuming about 3,065 MiB more peak
total GPU memory. This speed advantage did not compensate for the worse human
listening result.

Raw benchmark closure:

```text
/home/kaifaty/.local/share/nextengine/tts-model-evaluations/fish-s2-pro-2026-08-17/raw/audio-cpp/
```

### Repeat and tuning probes

Every row is the mean of three warm requests producing 4.272 s audio.

| Probe | Warm wall mean | Warm RTF mean | Result |
| --- | ---: | ---: | --- |
| Earlier direct repeat | 2.680 s | 0.627207 | Fastest observed process, retained as an uninstrumented repeat rather than the canonical comparison. |
| Four-thread repeat | 2.804 s | 0.656402 | No material change from the canonical profile. |
| One helper thread | 2.808 s | 0.657140 | No improvement. |
| Eight helper threads | 2.836 s | 0.663731 | Slightly worse. |
| `GGML_CUDA_GRAPH_OPT=1` | 2.807 s | 0.657025 | No improvement; experimental override rejected. |

The conservative reproducible number remains the instrumented wrapper mean of
RTF 0.659. Differences among the small tuning probes are close enough that they
must not be treated as latency percentiles.

### Warm stage profile

The profiled fixed request generated 92 frames with 10 codebooks. Each request
executed one prefill graph, 92 slow-AR step graphs and 920 fast-AR graphs.

| Warm request | Prompt build (ms) | AR generator (ms) | Codec decode (ms) | Session wall (ms) |
| --- | ---: | ---: | ---: | ---: |
| 1 | 0.690 | 2,683.185 | 117.123 | 2,801.940 |
| 2 | 0.701 | 2,692.378 | 117.159 | 2,811.201 |
| 3 | 0.682 | 2,667.872 | 115.659 | 2,785.863 |
| Mean | 0.691 | 2,681.145 | 116.647 | 2,799.668 |

Representative warm GPU/transfer totals were:

| Stage | Observed range |
| --- | ---: |
| Slow step graphs | 1,259-1,262 ms |
| Fast graphs | 917-924 ms |
| Fast mask uploads | 291-295 ms |
| Fast input uploads | 38-45 ms |
| Slow step mask uploads | 26-32 ms |
| Prefill graph | 20-35 ms |

Approximately 95.8% of the profiled warm wall time was inside the
autoregressive generator. Codec decoding was about 4.2%; prompt construction
was below 1 ms. Therefore helper-thread changes, wrapper-language changes and
codec-only tuning cannot materially fix single-request latency. A Rust host
around the same CUDA/ggml backend was estimated at roughly 0-2% latency change;
even eliminating all work outside AR would cap the theoretical gain near 5%.

Full timing trace:

```text
/home/kaifaty/.local/share/nextengine/tts-model-evaluations/fish-s2-pro-2026-08-17/raw/audio-cpp/q8-profile-timing.log
```

## Audio examples

The following files are external artifacts and are intentionally not committed.

| Example | Format and duration | SHA-256 | Listening note |
| --- | --- | --- | --- |
| `/home/kaifaty/.local/share/nextengine/tts-model-evaluations/fish-s2-pro-2026-08-17/examples/s2-q4-k-m.wav` | mono float32, 44.1 kHz, 3.390 s | `819e3d413f371e59af0f5524e180c0757142a45b0aad077c3b6dcc2cb4c3188c` | Q4_K_M quality judged generally acceptable. |
| `/home/kaifaty/.local/share/nextengine/tts-model-evaluations/fish-s2-pro-2026-08-17/examples/s2-q5-k-m.wav` | mono float32, 44.1 kHz, 3.297 s | `0687fa488e04ce94186588225cf6c511d31516be30b06fdebb675c8cdc5aba43` | Comparator; no final human verdict recorded. |
| `/home/kaifaty/.local/share/nextengine/tts-model-evaluations/fish-s2-pro-2026-08-17/examples/s2-q6-k.wav` | mono float32, 44.1 kHz, 5.155 s | `0870a0f64df183c0bc1c1b7ed020f41c233df665b7333b4d64befc97ae08788d` | Original wrapper default; no final human verdict recorded. |
| `/home/kaifaty/.local/share/nextengine/tts-model-evaluations/fish-s2-pro-2026-08-17/examples/s2-q8-0-cpu-codec.wav` | mono float32, 44.1 kHz, 6.130 s | `299d0369aff0972d22dfddfcb55e7e906843f4635de075dfacda2982454ace6d` | CPU-codec quality comparator. |
| `/home/kaifaty/.local/share/nextengine/tts-model-evaluations/fish-s2-pro-2026-08-17/examples/audio-cpp-q8-0.wav` | mono PCM16, 44.1 kHz, 4.272 s | `fcab0a2649a76d4f78a1da13be49efba01c771b6571b83bdfcd8420912c1d180` | Quality judged poor/insufficient; this is the decisive rejection example. |
| `/home/kaifaty/.local/share/nextengine/tts-model-evaluations/fish-s2-pro-2026-08-17/examples/s2-http-final.wav` | mono float32, 44.1 kHz, 3.947 s | `d68076375c3882dc0ec4873ef82cc0be72468c1b440d70a5dfa186ddd29855ff` | Finalized HTTP example. |
| `/home/kaifaty/.local/share/nextengine/tts-model-evaluations/fish-s2-pro-2026-08-17/examples/s2-http-stream.wav` | mono PCM16, 44.1 kHz, 2.833 s | `f5e5b261aee8d4cb6eb598772ddcaa28cddfb48c87a47b5f33ba87c1f8690dc7` | Transport-only streaming example; not realtime. |

Additional prewarm and per-request WAV files are retained under the two
external `outputs/` directories. The exact s2.cpp warm rows are in
`q456-warm-benchmark.csv`; audio.cpp request definitions, summary and logs are
inside its timestamped benchmark directory.

## Reproduction commands

From the repository root:

```bash
python3 lab/scripts/fish_s2_pro_demo.py doctor

python3 lab/scripts/fish_s2_pro_demo.py generate --force --quality q4 \
  --text 'Привет! Сервер синтеза речи работает локально.'

python3 lab/scripts/fish_s2_pro_demo.py generate --force --quality q5 \
  --text 'Привет! Сервер синтеза речи работает локально.'

python3 lab/scripts/fish_s2_pro_audio_cpp_demo.py doctor

python3 lab/scripts/fish_s2_pro_audio_cpp_demo.py generate \
  --skip-model-hash-check --force \
  --text 'Привет! Сервер синтеза речи работает локально.'

python3 lab/scripts/fish_s2_pro_audio_cpp_demo.py benchmark \
  --skip-model-hash-check --iterations 3
```

Repository-owned entry points:

- `lab/scripts/fish_s2_pro_demo.py`
- `lab/scripts/fish_s2_pro_audio_cpp_demo.py`
- `lab/tests/test_fish_s2_pro_demo.py`
- `lab/tests/test_fish_s2_pro_audio_cpp_demo.py`

## Constraints for future TTS comparisons

- Run one complete disposable synthesis before measuring warm latency.
- Use the same Russian prompt, or record a new shared corpus and seed explicitly.
- Record cold process time, warm wall time, audio duration, RTF, peak total VRAM,
  sample format and human listening verdict separately.
- Measure TTFA at the first PCM-byte boundary; HTTP header arrival is not TTFA.
- Keep weights, reference voices, logs and generated audio outside Git.
- Voice cloning requires an authorized recording and a separate consent-aware
  evaluation.
- Do not retry this exact audio.cpp Q8_0 candidate merely for its RTF. Reconsider
  it only after a pinned runtime/model-package change or a controlled reference
  voice/sampling experiment that materially improves quality.

## Upstream sources

- <https://github.com/rodrigomatta/s2.cpp>
- <https://huggingface.co/rodrigomt/s2-pro-gguf>
- <https://huggingface.co/fishaudio/s2-pro>
- <https://github.com/fishaudio/fish-speech/blob/main/docs/en/install.md>
- <https://github.com/0xShug0/audio.cpp>
- <https://github.com/0xShug0/audio.cpp/blob/main/model_specs/fish_audio.json>
- <https://github.com/0xShug0/audio.cpp/blob/main/docs/reports/gguf_q8_performance.md>
- <https://huggingface.co/audio-cpp/audio.cpp-gguf/tree/main>
