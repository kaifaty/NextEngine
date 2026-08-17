# Voxtral Mini 4B Realtime 2602 — current task state

| Field | Value |
| --- | --- |
| Status | `COMPLETE` |
| Updated | `2026-08-17` |
| Task key | `voxtral-mini-4b-realtime-2602` |
| Scope | Determine whether useful quantizations run on the workstation RTX 3080, assess quality evidence, and provide a bounded live-microphone probe |
| Definition of done | Verify one recommended quant locally, distinguish measured quality from inference, and validate a non-persisting microphone streaming client |
| Authority | Working context only; upstream model card, technical report, runtime documentation, and exact local evidence outrank this file |

## Resume in 60 seconds

- **Current conclusion:** Use GGUF Q4_K_M through transcribe.cpp on the 10 GB RTX 3080. The bounded `lab/scripts/voxtral_microphone.py` client now validates an exact ALSA or PipeWire input before loading the model, but this host currently has no usable microphone route.
- **Why:** Local CUDA runs and the Python streaming binding passed, including an exact incremental Russian transcript. On the current host both analog microphone jacks report disconnected, CMF Buds Pro 2 is using playback-only A2DP despite an available HSP/HFP MSBC profile, and the generic `pipewire` alias records digital silence.
- **Next action:** Connect an analog microphone or enable a Bluetooth HFP/HSP capture profile, then select the exact source reported by `--list-inputs` and run `--probe-microphone` before live transcription.
- **Current blocker:** No physical/capture microphone route is currently active; no representative NextEngine Russian speech corpus was in scope for quality evaluation.
- **Do not retry:** Official BF16 vLLM on this 10 GB card; Mistral requires at least 16 GB.
- **Reconsider when:** vLLM gains verified Voxtral quantization support or representative Russian evaluation contradicts Q4 neutrality.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| `docs/development/voxtral-mini-4b-realtime-2602-research-2026-08-17.md` | `REPORT_ONLY` | Bounded source review and local benchmark |
| Local Q4_K_M CUDA run on RTX 3080 10 GB | `PASS` | Q4_K_M fits and exceeds realtime for tested clips |
| `lab/scripts/voxtral_microphone.py --check` with shared CUDA library | `PASS` | The bounded microphone client loads the model and resolves CUDA on this host |
| Python binding incremental stream over `samples/ru.wav` | `PASS` | The same stream surface used by the microphone client produced the exact Russian reference |
| `--list-inputs` plus ALSA/PipeWire route inspection | `PASS` | Analog front/rear mic jacks are unavailable; CMF Buds Pro 2 uses playback-only A2DP and offers an inactive HSP/HFP MSBC capture profile |
| Two-second signal probes | `PASS` | `default`/`pipewire` and the inactive Bluetooth source are silent; the bare analog endpoint produces near-clipping jack noise rather than usable speech |
| Live run through the exact built-in PipeWire source | `PASS` with expected empty transcript | Capture, CUDA inference, finalization, and the no-speech warning execute end to end; no microphone was connected for speech input |
| Published LibriSpeech test-clean quant ladder | `REPORT_ONLY` | English WER is neutral from BF16 through Q4_K_M within reported confidence interval |

## Decisions that still constrain the work

### D-001 — Prefer validated GGUF Q4_K_M

- **Observation:** The official BF16 path requires >=16 GB, while Q4_K_M used approximately 4080 MiB locally and produced exact supplied English and Russian sample transcripts.
- **Evidence:** Linked research report and upstream sources.
- **Decision:** Q4_K_M is the default candidate for this RTX 3080.
- **Rejected alternatives:** BF16 vLLM does not fit the documented memory floor; unvalidated community AWQ/GPTQ/FP8 artifacts lack a confirmed supported runtime/quality gate.
- **Consequences:** Product evaluation should use the GGUF CUDA runtime unless stronger runtime-specific evidence appears.
- **Uncertainty:** Russian domain WER, noisy/multi-speaker behavior, and long-session application latency remain unmeasured.
- **Reconsider when:** A representative corpus shows material Q4 regression or official quantized vLLM support becomes available.

### D-002 — Keep microphone capture outside gameplay runtime

- **Observation:** The research need is interactive human evaluation, while captured speech and transcripts are diagnostic evidence rather than authoritative gameplay input.
- **Evidence:** SPEC-09 tooling/observability boundary and the validated external transcribe.cpp streaming binding.
- **Decision:** Keep the probe under `lab/scripts`; capture raw PCM through `arecord`, feed it in memory, and write no audio files.
- **Rejected alternatives:** Runtime integration would create an unjustified product contract; repeated short CLI invocations reload the model and do not provide true live streaming.
- **Consequences:** The tool requires an external transcribe.cpp checkout/shared library and remains Linux-only development tooling.
- **Uncertainty:** Real microphone quality depends on the selected ALSA device, room, gain, and noise conditions.
- **Reconsider when:** A production voice-input consumer and its deterministic fallback are explicitly scoped.

### D-003 — Fail before model load when the selected input is unusable

- **Observation:** The aliases `default` and `pipewire` can open successfully while returning only zero-valued PCM, and argparse previously accepted repeated `--device` options by silently keeping the last value.
- **Evidence:** Local two-second probes measured digital silence on both aliases; the user's command contained both `--device default` and `--device pipewire` and finished with an empty transcript.
- **Decision:** List exact ALSA/PipeWire sources, reject repeated device selection, and require a two-second level preflight before every live run. Missing, short, or sub-threshold capture fails with a selection hint; sustained clipping and an empty model result produce explicit warnings.
- **Rejected alternatives:** Treating a successfully opened capture process as proof of microphone availability leaves silent routes undetected; automatically changing the desktop or Bluetooth audio profile would mutate user audio state.
- **Consequences:** A live run adds two seconds before model loading and requires the user to resolve the host audio route explicitly.
- **Uncertainty:** A level probe proves signal presence, not speech intelligibility; unusual very quiet microphones may require a lower `--silence-threshold-dbfs` after gain and route checks.
- **Reconsider when:** The wrapper gains a portable device API that can expose route availability without sampling audio.

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| Q4_K_M preserves Russian quality | Exact short Russian smoke sample; English full-split WER is neutral | No Russian quant-vs-BF16 WER study | Paired Russian corpus evaluation |
| 960 ms is preferable for gameplay | Official Russian FLEURS WER improves from 6.02% at 480 ms to 5.56% | Adds 480 ms algorithmic delay | Task-specific latency/quality user study |

## Required context

Read these sources in precedence order before acting:

1. Official Mistral model card and technical report linked from the research report.
2. `docs/development/voxtral-mini-4b-realtime-2602-research-2026-08-17.md`.

## Next action

1. Assemble a license-safe, representative Russian evaluation set.
2. Compare Q4_K_M at 480 and 960 ms using normalized WER/CER, latency, realtime factor, and peak VRAM.
3. Retain the current Q4_K_M default unless the paired result shows material regression.

## Do not retry

- Official BF16 vLLM on this RTX 3080 10 GB — documented minimum is 16 GB; reconsider only if a supported lower-memory path is released.

## Handoff

- **Workspace state:** Research/task-state documentation plus a bounded lab microphone script and focused tests; downloaded model and external runtime build remain only under `/tmp/codex-voxtral-research`.
- **Checks:** Local CUDA Q4_K_M runs, shared-library `--check`, incremental Russian stream, focused Python tests, exact input listing/probes, a full bounded live path, `git diff --check`, and `cargo run -p xtask -- host-check` passed.
- **Remaining risk:** No connected microphone was available for a real speech capture; no representative Russian corpus evaluation and no local Q8/Q6/Q5 comparison.
- **Promotion needed:** None.
