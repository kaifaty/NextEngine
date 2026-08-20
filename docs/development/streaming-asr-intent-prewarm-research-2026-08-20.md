# Streaming ASR and speculative intent prewarm research

| Field | Value |
| --- | --- |
| Date | 2026-08-20 |
| Status | Bounded research; next adapter experiment selected, product adoption unproven |
| Scope | Russian local streaming ASR for early dialogue preparation in the one-character demo |
| Selected experiment | `nvidia/nemotron-3.5-asr-streaming-0.6b`, `ru-RU`, 320 ms initial chunk profile |
| Controls | Current Voxtral Realtime streaming route; current GigaAM-v3 final-only quality reference |

## Decision

The game-facing ASR profile must be natively stateful and emit replaceable
partial transcript revisions while the player is speaking. A final-only model
may remain a quality oracle or optional second pass, but it cannot be the
primary dialogue route because its capture-to-first-text latency includes the
whole utterance.

Evaluate NVIDIA Nemotron 3.5 ASR Streaming 0.6B as the next primary candidate.
It is a multilingual cache-aware FastConformer-RNNT model with `ru-RU` in its
transcription-ready tier. The official card exposes 80, 160, 320, 560 and
1120 ms streaming configurations. The first local profile should use 320 ms:
the official normalized FLEURS Russian WER is 9.87% with the language supplied,
compared with 10.84% at 80 ms and 9.17% at 1120 ms. These figures select an
experiment, not a product-quality claim for whisper or game microphones.

Prefer the official NeMo-Speech.cpp local runtime for the bounded experiment
if its exact revision and dependencies pass local build and license checks. It
offers resident local inference, realtime PCM16 WebSocket serving and a native
C/C++ integration surface. Keep it behind the existing model-neutral
`Transcriber` facade; no NVIDIA type belongs in the consumer contract.

## Why the current GigaAM route is not primary

The current pinned `ai-sage/GigaAM-v3` adapter buffers every PCM chunk and calls
one full `forward` plus RNNT decode in `finish()`. It correctly advertises
`supports_streaming=false`, produces no fabricated partials and is bounded to
25 seconds. The upstream checkpoint uses an RNNT head, but model architecture
alone does not provide a verified streaming state/cache API. The official
repository examples currently expose file/batch transcription and ONNX
inference, not the partial-revision contract needed here.

Therefore:

- keep GigaAM as an offline/final A/B quality reference;
- do not repeatedly decode a growing GigaAM waveform to imitate streaming;
- reconsider it only when a pinned runtime exposes incremental encoder and
  predictor state with bounded revisions and passes the same latency/quality
  corpus.

## Required runtime contract

The facade should distinguish native streaming from buffered emulation:

```text
TranscriberCapabilities {
  supports_streaming: true,
  streaming_kind: "stateful_cache_aware",
  revision_semantics: "replace_all",
  input_chunk_ms,
  algorithmic_lookahead_ms,
  max_session_duration_ms,
  endpointing,
  timing_precision,
  model/runtime/artifact identity
}
```

Every transcript event carries `session_id`, `turn_id`, monotonic
`transcript_revision`, current full text, optional stable prefix, audio frontier
and emission time. A consumer must tolerate correction of the tentative suffix.
An adapter may expose an append-only stable prefix only when measured behavior
supports it; otherwise the facade derives a conservative prefix from agreement
across consecutive revisions.

The model must process bounded new audio plus cached state. Per-chunk compute,
retained memory and event size must not grow with total utterance duration.
Buffered overlapping-window or whole-prefix redecoding is a separate
`buffered_emulation` capability and cannot silently satisfy this gate.

## Speculative dialogue flow

```text
microphone PCM
  -> VAD / speech onset
  -> stateful ASR partial revision N
  -> conservative stable-prefix gate
  -> cancelable speculation keyed by (turn, revision, prefix hash)
       1. prefill fixed persona + bounded dialogue history
       2. classify provisional intent / retrieve immutable context
       3. build an unpresented response-plan or LLM draft
  -> later ASR revision
       compatible prefix -> reuse cache/draft
       incompatible text -> cancel and roll back to longest common prefix
  -> endpoint or explicit button release
  -> final canonical utterance
  -> validate compatible speculative result or discard it
  -> admit response sentence -> subtitle -> TTS
```

Three latency layers remain separate:

1. At dialogue open, prefill the static persona, policy and bounded prior turns.
2. During speech, use stable partial text only for immutable, cancelable work.
3. After final ASR, admit text and any response only through the normal
   validation boundary.

Partial ASR cannot reserve an action, call a tool, mutate world state, publish
an NPC commitment or start TTS. This follows the existing Proposed SPEC-16
prewarm rule and Accepted ADR-005 isolation/fallback boundary. The benefit is
latency hiding, not speculative gameplay authority.

Push-to-talk may remain in the first demo. Press begins continuous PCM and ASR;
release only forces end-of-utterance. It must not be the point where ASR first
starts. Automatic VAD endpointing can later remove the button without changing
the revision/cancellation model.

## Measurements and admission gates

Use the same externally stored, reference-transcribed Russian clips for every
route: normal speech, whisper, background noise and short pauses. Report:

- first non-empty and first useful partial after speech onset, p50/p95;
- partial cadence, revision count and normalized suffix churn;
- endpoint-to-final transcript latency, p50/p95;
- WER/CER and empty-transcript rate by normal/whisper/noise stratum;
- per-chunk queue/inference p50/p95 and end-to-end RTF;
- duration-bucket slope for per-chunk latency and resident memory;
- cancel-to-idle time and stale-result count;
- cold/warm load, RAM/VRAM and remaining physical GPU headroom;
- final-text agreement between streaming and any optional final reference pass;
- speculative work reuse, cancellation and final incompatibility rates;
- final-ASR-to-first-admitted-sentence and final-ASR-to-first-PCM latency with
  speculation disabled versus enabled.

Initial report-only targets for the local single-stream experiment are:

- first useful partial p95 at or below 800 ms after detected speech onset;
- partial update gap p95 at or below 400 ms for the 320 ms profile;
- endpoint-to-final p95 at or below 400 ms;
- paced RTF below 1.0 with no growing queue;
- no positive duration slope suggesting whole-prefix reprocessing;
- at least 1 GiB physical VRAM headroom in the admitted resident profile.

Quality is a paired comparison, not a fixed borrowed WER threshold. Nemotron
must beat or materially complement Voxtral on the frozen whisper/normal/noise
set; official FLEURS numbers cannot substitute for that evidence.

## Smallest experiment

1. Pin an exact Nemotron model/runtime revision and record weight/runtime
   hashes and separate model/runtime license classifications.
2. Run the official runtime's own Russian streaming microphone/WAV smoke at
   320 ms before writing an adapter.
3. Add a `nemotron-3.5-streaming` adapter behind the existing `Transcriber`
   interface, preserving replace-all revisions and cancellation.
4. Do not load Voxtral, GigaAM and Nemotron together merely to keep a UI
   selector. Use resource-admitted startup profiles and replay the same saved
   WAVs across them; the production profile keeps only the selected primary
   ASR resident.
5. Benchmark 80/160/320/560 ms only after the 320 ms correctness control works.
6. Add speculative intent/context/LLM prewarm behind a feature flag and prove
   that disabling it yields the same admitted final dialogue semantics.

## Primary sources

- [NVIDIA Nemotron 3.5 ASR model card](https://huggingface.co/nvidia/nemotron-3.5-asr-streaming-0.6b) — `ru-RU` tier, cache-aware architecture, chunk choices, Russian FLEURS WER, model license and official streaming example.
- [NVIDIA NeMo streaming inference documentation](https://docs.nvidia.com/nemo/speech/nightly/asr/inference.html#streaming-inference) — cache-aware pipelines process each frame once with cached activations and support prompt-conditioned multilingual streaming.
- [NVIDIA NeMo-Speech.cpp ASR models](https://github.com/NVIDIA/NeMo-Speech.cpp/blob/main/docs/asr/models.md) — local GGUF runtime support for Nemotron 3.5 and explicit rejection of non-streaming model families on streaming requests.
- [NVIDIA NeMo-Speech.cpp server](https://github.com/NVIDIA/NeMo-Speech.cpp/blob/main/docs/server.md) — resident engine registry, loopback service and realtime PCM16 WebSocket.
- [NVIDIA NeMo-Speech.cpp ASR configuration](https://github.com/NVIDIA/NeMo-Speech.cpp/blob/main/docs/asr/configuration.md) — cache-aware RNNT context, endpointing and bounded streaming configuration.
- [GigaAM official repository](https://github.com/salute-developers/GigaAM) — current public load/file/ONNX inference surfaces used to bound the GigaAM claim.

