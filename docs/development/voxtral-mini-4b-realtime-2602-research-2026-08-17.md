# Voxtral Mini 4B Realtime 2602 on RTX 3080 — research (2026-08-17)

## Conclusion

`Q4_K_M` GGUF through `transcribe.cpp` is a viable local configuration for
the 10 GB RTX 3080 in this workstation. It loaded with the CUDA backend,
peaked at approximately 4080 MiB of process VRAM, and transcribed the supplied
11 s and 35.3 s samples at about 3x realtime. The official BF16 vLLM path is
not viable on this card: Mistral documents a GPU with at least 16 GB memory.

The independently published LibriSpeech `test-clean` evaluation reports no
measurable English WER loss across the available GGUF ladder down to Q4_K_M:
2.07-2.09% versus 2.08% for the same-machine Transformers BF16 reference.
This does not establish quantization neutrality for Russian or noisy,
multi-speaker, domain-specific audio.

## Model and official quality

- Model: approximately 3.4B decoder parameters plus a 970M causal audio
  encoder; Apache-2.0; 13 languages.
- It emits one text token per 80 ms audio slot and supports configurable delay.
  Mistral recommends 480 ms as the quality/latency sweet spot.
- On FLEURS at 480 ms, Mistral reports average WER 8.72% and Russian WER 6.02%.
  At 160/240/960/2400 ms, Russian WER is respectively
  9.53/7.87/5.56/5.41%. This makes delay a larger demonstrated quality lever
  than Q4-vs-BF16 on the published English quantization test.
- At 480 ms, Mistral reports long-form English WER of 5.05% (Meanwhile),
  10.23% (E-21), 12.30% (E-22), and 3.17% (TEDLIUM).

Primary sources:

- [Official model card](https://huggingface.co/mistralai/Voxtral-Mini-4B-Realtime-2602)
- [Technical report, arXiv:2602.11298](https://arxiv.org/abs/2602.11298)

## Quantizations relevant to RTX 3080

The validated `handy-computer` GGUF set is:

| Quant | File size | LibriSpeech test-clean WER | 3080 10 GB assessment |
| --- | ---: | ---: | --- |
| BF16 | 8.87 GB | 2.08% | Too close to physical capacity once runtime buffers are included; official vLLM requires >=16 GB |
| F16 | 8.88 GB | 2.09% | Same risk as BF16; not recommended |
| Q8_0 | 4.73 GB | 2.07% | Expected to fit comfortably in GGUF runtime; not locally measured |
| Q6_K | 3.66 GB | 2.08% | Expected to fit comfortably; not locally measured |
| Q5_K_M | 3.28 GB | 2.08% | Expected to fit comfortably; not locally measured |
| Q4_K_M | 2.83 GB | 2.08% | Locally verified; recommended default |

Source and reproduction details:
[transcribe.cpp runtime repository](https://github.com/handy-computer/transcribe.cpp)
and [GGUF model card](https://huggingface.co/handy-computer/Voxtral-Mini-4B-Realtime-2602-gguf).
The reported WER uses all 2620 LibriSpeech `test-clean` utterances, Whisper-style
English normalization, greedy decoding, batch size 8, and an NVIDIA L40S. The
reported 95% confidence interval is approximately +/-0.18 percentage points.

The official production runtime remains vLLM with BF16. A vLLM request for
AWQ/GPTQ/FP8 support was still an open feature request during this research;
therefore community AWQ/GPTQ/FP8 uploads should not be treated as a supported
3080 deployment path without runtime-specific validation.

## Local RTX 3080 evidence

Environment:

- NVIDIA GeForce RTX 3080, 10240 MiB physical memory (9871 MiB reported usable
  by the runtime), compute capability 8.6;
- NVIDIA driver 610.43.02; CUDA toolkit 13.3.73;
- `transcribe.cpp` commit `9315160`, built with `TRANSCRIBE_CUDA=ON`;
- `Voxtral-Mini-4B-Realtime-2602-Q4_K_M.gguf`.

Results:

| Case | Result |
| --- | --- |
| 11.0 s JFK, offline | exact expected sentence; 3.33 s inference, 3x realtime |
| 11.0 s JFK, streaming, 500 ms feeds, 480 ms model delay | final transcript byte-equal to offline; 6.90 s wall including load and incremental calls |
| 4.5 s Russian sample | exact reference: `Важно различать глаголы и дополнения.`; 1.58 s inference, 3x realtime |
| 35.3 s English sample | correct supplied reference transcript; 10.20 s inference, 3x realtime; peak process VRAM approximately 4080 MiB |

Model load took approximately 1.2-1.4 s. These are smoke/throughput checks,
not statistically meaningful quality measurements. The streaming wall time
also includes an artificial synchronous feed loop and should not be read as
the model's algorithmic 480 ms transcription delay.

## Recommendation

Start with `Q4_K_M`, CUDA, temperature/greedy decoding, and 480 ms delay. It
has the strongest evidence-to-footprint ratio and leaves roughly 5.8 GB of the
runtime-reported GPU memory free for the application. Consider Q8_0 only if a
representative Russian evaluation corpus shows a real Q4 regression; the
published English result gives no evidence that it will.

Before product adoption, evaluate Q4_K_M on representative Russian gameplay
speech: accents, background music/effects, microphone noise, proper nouns,
interruptions, and long sessions. Report normalized WER/CER, first-token and
finalization latency, realtime factor, and peak VRAM. Compare at least 480 ms
and 960 ms delay because the official Russian FLEURS result improves from
6.02% to 5.56% for an additional 480 ms.

## Live microphone utility

`lab/scripts/voxtral_microphone.py` is a bounded Linux development tool, not a
gameplay/runtime integration. It uses `arecord` or `pw-record` to capture 16
kHz mono PCM and feeds the samples directly to the `transcribe.cpp` Python
streaming API. It does not persist microphone audio by default.

Build a CUDA-enabled shared library in the external `transcribe.cpp` checkout:

```bash
cmake -S /path/to/transcribe.cpp -B /path/to/transcribe.cpp/build \
  -DTRANSCRIBE_CUDA=ON \
  -DTRANSCRIBE_BUILD_SHARED=ON \
  -DCMAKE_BUILD_TYPE=Release
cmake --build /path/to/transcribe.cpp/build -j --target transcribe
```

Validate the model/runtime without opening the microphone:

```bash
python lab/scripts/voxtral_microphone.py \
  /path/to/Voxtral-Mini-4B-Realtime-2602-Q4_K_M.gguf \
  --transcribe-root /path/to/transcribe.cpp \
  --check
```

List inputs and start live Russian transcription:

```bash
python lab/scripts/voxtral_microphone.py --list-inputs

python lab/scripts/voxtral_microphone.py \
  --probe-microphone \
  --device 'pw:<exact-node-name-from-list>' \
  --save-wav /tmp/voxtral-probe.wav

python lab/scripts/voxtral_microphone.py \
  /path/to/Voxtral-Mini-4B-Realtime-2602-Q4_K_M.gguf \
  --transcribe-root /path/to/transcribe.cpp \
  --device 'pw:<exact-node-name-from-list>' \
  --delay-ms 480 \
  --language ru
```

Use Ctrl-C to stop and finalize the transcript. `--duration 15` provides a
bounded capture, and `--delay-ms 960` selects the higher-quality official
operating point. Raw audio and model weights remain external artifacts and
must not be committed. Every live run performs a two-second microphone
preflight before loading the model. It fails on missing/short capture or a peak
below -65 dBFS, warns on sustained clipping, and rejects repeated `--device`
options instead of silently selecting the last one. Prefer exact `plughw:` or
`pw:` entries: desktop aliases such as `default` and `pipewire` can resolve to
an unavailable or silent route. `--probe-microphone` only measures input and
does not load or run the model; its success message states this explicitly.
`--save-wav PATH` is an opt-in diagnostic exception: probe mode saves the
measured sample, while live mode saves the exact PCM fed to the model. The path
must be outside the repository and must not already exist.

On the evaluated host, the ALSA card exposes two capture endpoints, but
PipeWire reports the front and rear analog microphone ports as disconnected.
CMF Buds Pro 2 is visible as a source while its current A2DP profile provides
playback only. PipeWire also reports an available HSP/HFP MSBC profile with an
input channel. Consequently, both `default` and `pipewire` aliases produced
digital silence; the wrapper now rejects that condition before spending time
loading the model. Enabling the headset/HFP capture profile or connecting an
analog microphone is a host setup action, not something the probe changes
automatically.

For this workstation, the user selected the opposite tradeoff: keep the
headset microphone immediately active after every connection. On WirePlumber
0.5.13 the persistent setup is:

```bash
wpctl settings --save bluetooth.autoswitch-to-headset-profile false
wpctl set-profile <current-cmf-device-id> <headset-head-unit-msbc-index>
wpctl set-default <current-cmf-sink-id>
wpctl set-default <current-cmf-source-id>
```

The exact numeric object IDs are session-local and must be taken from `wpctl
status` and `pw-dump`. An explicit Bluetooth disconnect/reconnect restored the
MSBC profile and both physical nodes, and a subsequent ten-second capture
through the default `pipewire` alias measured a non-silent -43.7 dBFS peak.
This setup deliberately trades A2DP playback quality for an always-present
mono headset input/output pair. The wrapper's device listing reads the active
PipeWire `Profile` parameter rather than the misleading BlueZ initialization
property when reporting capture availability.
