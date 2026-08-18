# Phase 2: LLM и TTS поверх speech timeline

| Поле | Значение |
| --- | --- |
| Дата | 2026-08-18 |
| Статус | User-requested stage-2 design; FunctionGemma explicitly deferred to Phase 4; two scope choices remain pending |
| Зависимость | [Phase 1 resident speech-timeline service](2026-08-18-speech-timeline-service-implementation.md) |
| Следующий этап | [Phase 3 engine neural-capability integration](2026-08-18-engine-neural-capability-integration-phase-3.md) |
| Первый профиль | Local Linux x86_64, один conversation session, network default-deny |
| LLM/TTS | Role adapters first; exact artifacts selected by local resource/quality measurements |
| Архитектурная граница | Optional developer/`ai-host` service under ADR-005; no direct gameplay mutation and no current Rust public-contract change |

## 1. Решение

Второй этап добавляет отдельный model-neutral `ConversationService` поверх
`SpeechTimelineService`. Speech service продолжает владеть только
microphone/ASR/affect timeline. Conversation service получает finalized turn,
генерирует неавторитетный текст ответа через replaceable LLM, валидирует его и
передаёт admitted sentences в replaceable TTS.

```text
microphone client
      │
      ▼
SpeechTimelineService                       Phase 1
  Voxtral + emotion2vec
      │ UtteranceFinal
      │ canonical text + structured observed expression
      ▼
ConversationSupervisor                      Phase 2 facade
      ├─ ConversationContextAssembler
      ├─ DialogueGenerator
      │     └─ replaceable large LLM
      ├─ ResponseValidator / commitment gate
      └─ SpeechSynthesizer
            └─ replaceable TTS adapter
                    │
                    ▼
        subtitle revisions + ordered audio chunks
```

FunctionGemma, tool schemas и вызов внутренней strategic neural model в этот
этап не входят. Facade не должен зашить предположение, что LLM всегда отвечает
без дополнительного контекста: `ConversationContextAssembler` принимает
bounded immutable context blocks. В Phase 3 validated observation/advisory от
engine-integrated model сможет стать ещё одним typed context block без
изменения speech/TTS contracts, но никакой speculative tool API в Phase 2 не
создаётся. FunctionGemma использует consumer-backed catalog только в Phase 4.

## 2. Scope второй вертикали

Первая usable vertical Phase 2 включает:

1. один authenticated localhost WebSocket facade;
2. один active conversation session и последовательные turns;
3. finalized `UtteranceFinal` из Phase 1 как единственный audio-derived input;
4. одну replaceable local LLM role;
5. structured affect context отдельно от canonical user text;
6. streamed subtitle revisions с explicit finality;
7. одну replaceable non-cloning TTS role;
8. sentence-by-sentence ordered audio chunks;
9. explicit start/finish/cancel, без обязательного VAD/barge-in;
10. no network, no gameplay mutation и no transcript/audio persistence by
    default;
11. fault injection и joint CPU/GPU/RAM/latency measurements.

Не входят в эту вертикаль:

- FunctionGemma, function calling или произвольные tools;
- strategic neural model и engine-facing strategic gateway;
- direct engine-facing IPC или новый Rust public contract;
- arbitrary plugins, MCP, shell, Python eval, filesystem или URLs;
- voice cloning/reference voices;
- automatic VAD/barge-in correctness as a completion gate;
- model training inside runtime;
- remote/cloud providers;
- утверждение, что SPEC-16/ADR-017 уже shipped.

## 3. Service topology

Phase 2 не должен загружать все модели в один Python/CUDA process. Рекомендуемая
topology:

```text
ConversationSupervisor
  ├─ Speech worker        GPU, existing Phase 1 service boundary
  ├─ Dialogue worker      placement from selected capability profile
  └─ TTS worker           placement from selected capability profile
```

В Phase 2 profile supervisor является единственным владельцем внешнего client
listener. Он переиспользует framing/auth/session semantics Phase 1 и передаёт
PCM во внутреннюю границу resident speech worker; отдельное клиентское
microphone connection к speech process не открывается.

Каждый worker:

- стартует из explicit external model/runtime path;
- проверяет exact identity/hash before ready;
- загружает и прогревает модель один раз;
- публикует versioned capabilities/resource envelope;
- принимает bounded request с deadline/cancellation/idempotency;
- может быть перезапущен отдельно;
- не получает arbitrary filesystem, credentials или world mutation authority.

Listener становится ready только после preflight выбранного capability
profile. Если declared resident RAM/VRAM + measured allocator peak + headroom
не помещаются, profile отвергается до session start. Случайный unload/reload
моделей внутри turn не используется как скрытый OOM fallback.

На текущей RTX 3080 10 GiB нельзя заранее считать, что Voxtral, emotion2vec,
large LLM и TTS одновременно поместятся в VRAM. Exact LLM/TTS placement
остаётся model-selection gate. Минимальный GPU headroom после measured joint
peak — 1 GiB.

## 4. Model-neutral contracts

Consumer contract не содержит provider class, tokenizer или runtime-specific
request type.

```python
class DialogueGenerator(Protocol):
    def capabilities(self) -> DialogueCapabilities: ...
    async def respond(self, request: DialogueRequest) -> AsyncIterator[DialogueEvent]: ...

class SpeechSynthesizer(Protocol):
    def capabilities(self) -> TtsCapabilities: ...
    async def synthesize(self, plan: SpeechRenderPlan) -> AsyncIterator[SpeechChunk]: ...
```

Capabilities включают adapter protocol version, exact model/runtime identity
and artifact hash, locales, input/output limits, streaming/revision semantics,
RAM/VRAM envelope, warm-up policy, provenance, cancellation and concurrency.

### `DialogueRequest`

```json
{
  "schema_version": 1,
  "session_id": "opaque",
  "turn_id": "opaque",
  "turn_revision": 4,
  "locale": "ru-RU",
  "user_text": "Нам сейчас лучше отступить?",
  "speech_context": {
    "alignment_grade": "utterance",
    "observed_vocal_expression": "fearful"
  },
  "context_blocks": [],
  "max_response_utf8_bytes": 16384,
  "deadline_id": "opaque",
  "cancellation_id": "opaque"
}
```

`context_blocks` — bounded, typed, immutable и revision-bound. Phase 2
разрешает только явно объявленные authored/session/game-snapshot categories.
Неизвестная category отвергается. Поле является общей границей расширения, а
не скрытым FunctionGemma/tool contract.

Structured emotion остаётся отдельным observation block. Оно не вставляется в
canonical user text и описывается как observed expression, а не внутреннее
психическое состояние пользователя.

### `DialogueEvent`

```json
{
  "schema_version": 1,
  "turn_id": "opaque",
  "candidate_revision": 3,
  "event_kind": "sentence_final",
  "sentence_index": 0,
  "text": "Сначала оценим безопасный путь отхода.",
  "model_hash": "sha256:..."
}
```

Только `sentence_final`, прошедшее `ResponseValidator`, становится admitted
presentation text. Provisional deltas можно показывать как replaceable subtitle
preview, но нельзя отправлять в TTS или считать gameplay fact.

### `SpeechRenderPlan`

```json
{
  "schema_version": 1,
  "turn_id": "opaque",
  "sentence_index": 0,
  "text": "Сначала оценим безопасный путь отхода.",
  "locale": "ru-RU",
  "voice_profile_id": "licensed-default-ru",
  "style": "neutral",
  "rate_permille": 1000,
  "pitch_cents": 0,
  "cancellation_id": "opaque"
}
```

Free-form SSML/model control tokens запрещены в facade. Style, rate и pitch —
closed bounded values из adapter capabilities. Voice cloning disabled.

## 5. Turn state machine

```text
Idle
  → Listening
  → InputFinalized
  → ContextAssembling
  → DialogueGenerating
  → ResponseValidating
  → Speaking
  → Completed
```

Terminal alternatives: `Cancelled`, `Rejected`, `FallbackCompleted` и
`FailedBeforeAdmission`.

Rules:

- только `UtteranceFinal` запускает LLM;
- partial ASR MAY prewarm a selected worker but cannot publish assistant text;
- turn/candidate revision cannot regress;
- stale worker result is discarded;
- TTS starts sentence-by-sentence only after sentence validation;
- cancel stops pending LLM/TTS work and discards late results;
- starting a new listening turn stops previous TTS presentation;
- worker completion/wall time never chooses a gameplay outcome.

## 6. WebSocket facade extension

Phase 2 reuses Phase 1 loopback framing, authentication and session semantics,
но внешний listener запускается только supervisor. Speech worker находится за
его внутренней adapter/worker boundary.

Additional client controls:

- `conversation.configure` — locale, capability profile and safe presentation
  preferences;
- `turn.respond` — generate response for last finalized utterance when
  auto-response is disabled;
- `turn.cancel` — cancel uncommitted work and playback;
- `playback.ack` — bounded progress diagnostic only.

Additional server events:

- `turn.accepted`;
- `assistant.delta`;
- `assistant.sentence.final`;
- `assistant.final`;
- `speech.chunk.header` followed by one binary PCM/encoded frame;
- `speech.done`;
- `turn.completed`, `turn.cancelled` or `turn.failed`.

Prompts, hidden reasoning and raw model tokens are not protocol fields. One
binary speech frame is bounded by the existing 262,144-byte candidate ceiling;
format, sample rate, channels, sequence and exact byte length are declared in
the preceding header.

## 7. LLM integration

LLM input состоит из canonical finalized user text, structured affect summary,
bounded immutable context blocks, locale, response schema and size limits.
Prompt/template, KV cache and provider session remain adapter-private. Public
events expose only candidate text, finality, model identity/provenance and
validation outcome.

Every final sentence passes UTF-8/NFC, size, content and stale-turn validation.
Sentence claiming a gameplay commitment cannot be presented as fact before a
future engine-side command commit. Phase 2 has no tool execution and therefore
must phrase strategic/game outcomes only from supplied immutable facts or as
non-authoritative dialogue.

Failures degrade from selected LLM to declared local fallback/authored response
and finally to a typed text-only failure without a fabricated answer.

## 8. TTS integration

TTS consumes admitted `SpeechRenderPlan`, not arbitrary LLM token stream. It
publishes monotonically sequenced audio chunks with exact format and hashes.
Capabilities declare locales, voice IDs, streaming behavior, audio format,
bounded style controls, first-chunk/RTF measurements, limits, cancellation,
provenance and disabled voice cloning.

TTS failure never invalidates canonical assistant text. Client retains
subtitles and receives typed `speech.failed`; subtitle-only fallback is always
available.

## 9. Scheduling, backpressure and targets

Priority order:

1. microphone/Voxtral streaming while user speaks;
2. cancellation and stale-result cleanup;
3. LLM generation after finalized input;
4. validation/admission of earliest complete sentence;
5. TTS for earliest admitted sentence;
6. remaining LLM/TTS continuation;
7. optional metrics/debug work.

Queues are bounded per role. ASR final, admitted sentence and TTS chunk are
never silently dropped. Queue overflow terminates the turn with a typed outcome
instead of accumulating unbounded latency.

Initial one-session targets are measurements, not product promises:

| Metric | Initial target |
| --- | --- |
| LLM request → first admitted sentence p95 | ≤1,000 ms |
| LLM complete candidate p95 | ≤2,000 ms |
| admitted sentence → first PCM p95 | ≤500 ms |
| TTS real-time factor | ≤0.5 |
| utterance final → first PCM end-to-end p95 | ≤1,500 ms |
| resource headroom | ≥1 GiB physical VRAM and bounded RAM profile |

Cold start, first warm turn and second warm turn are reported separately.
Failure to meet a target blocks only the selected model profile, not the
adapter/facade implementation.

## 10. Configuration and package layout

```toml
[conversation]
locale = "ru-RU"
max_active_sessions = 1
network = "deny"
dialogue = "local_dialogue_adapter"
tts = "local_tts_adapter"
fallback = "text_only"

[conversation.dialogue]
endpoint = "local-worker"
artifact_hash = "sha256:..."
max_context_utf8_bytes = 65536
max_response_utf8_bytes = 16384

[conversation.tts]
endpoint = "local-worker"
artifact_hash = "sha256:..."
voice_profile = "licensed-default-ru"
voice_cloning = false
```

Startup never downloads missing weights. Recommended separation:

```text
tools/speech-timeline/
  src/nextengine_speech_timeline/

tools/conversation-service/
  pyproject.toml
  src/nextengine_conversation/
    contracts.py
    supervisor.py
    state.py
    protocol.py
    resources.py
    context.py
    adapters/
      dialogue.py
      tts.py
    workers/
      protocol.py
      process.py
    cli.py
  tests/
```

`conversation-service` depends on the stable Phase 1 worker boundary, not on
Voxtral or emotion2vec implementation modules.

## 11. Implementation sequence

### Phase 2 Commit A — `feat(ai): define conversation role contracts`

- bounded values/protocols for turn, context, dialogue and speech output;
- fake adapters and deterministic state-machine tests;
- no model/runtime dependency.

### Phase 2 Commit B — `feat(ai): supervise isolated dialogue and TTS workers`

- worker handshake, readiness, cancellation, crash and resource preflight;
- reuse Phase 1 listener implementation/protocol and session identity while
  making supervisor the sole external connection owner;
- second sequential turn proves no worker reload.

### Phase 2 Commit C — `feat(ai): add replaceable dialogue adapter`

- first selected local LLM adapter;
- structured user-text/affect/context input;
- bounded streaming revisions and final sentences;
- no tool or strategic-model call.

### Phase 2 Commit D — `feat(ai): validate and admit dialogue responses`

- UTF-8/NFC, size, stale revision and false-commitment checks;
- provisional subtitle replacement and admitted final text;
- authored/text-only fallback.

### Phase 2 Commit E — `feat(audio): stream replaceable TTS responses`

- selected licensed non-cloning TTS adapter;
- admitted sentence → ordered chunks;
- cancellation, subtitle fallback and audio format validation.

### Phase 2 Commit F — `test(ai): close microphone-to-response evidence`

- one WebSocket session from PCM through final ASR, LLM and TTS;
- two-turn residency, CPU/GPU/RAM and p50/p95 latency;
- crash/OOM/malformed/stale/cancel fault matrix;
- update durable task state with admitted/rejected model profiles.

FunctionGemma work is not an extra Phase 2 commit. Phase 3A first consumes this
service in a bounded one-character demo scene with no world context, internal
models or tools. Later Phase 3 subphases harden the proven boundaries and
shadow-integrate strategic capabilities; FunctionGemma starts only through the
separate Phase 4 gates.

## 12. Test matrix

| Layer | Positive | Failure/adversarial |
| --- | --- | --- |
| Context | exact finalized text + affect + immutable blocks | stale/unknown/oversized block |
| Dialogue | ordered revisions and final sentences | malformed UTF-8, oversized response, false commitment |
| TTS | ordered audio and final marker | skipped/duplicate sequence, wrong format, crash, cancellation |
| State machine | two sequential complete turns | duplicate finish, post-final delta, disconnect at every state |
| Workers | one load/warm per process | bad handshake, incompatible artifact, exit/OOM/restart |
| Security | local typed inputs | prompt requesting shell/path/network/arbitrary code |
| Privacy | no payload persistence by default | transcript/prompt/audio/voice leakage scanner |
| Resources | admitted profile has headroom | preflight reject and subtitle-only fallback |
| Russian quality | held-out dialogue sample corpus | mixed affect, long turn and ambiguous request |

Actual weights/CUDA/microphone/TTS tests are opt-in, use external immutable
artifacts and never download implicitly.

## 13. Definition of done

Phase 2 завершена, когда одновременно:

1. `UtteranceFinal` starts exactly one bounded conversation turn;
2. LLM loads once and a second turn does not reload it;
3. structured vocal affect reaches LLM as observation, not inline fact;
4. validated final sentences stream as subtitles;
5. TTS loads once and streams ordered audio only for admitted sentences;
6. subtitle-only fallback survives TTS failure;
7. swapping fake/real LLM or TTS is configuration plus adapter, not facade
   rewrite;
8. no Phase 2 component executes tools or mutates world state;
9. malformed/stale/late/cancelled results have bounded typed outcomes;
10. raw audio, prompts, transcripts, model tokens and voices are not persisted
    or logged by default;
11. two-turn cold/warm latency, queue, CPU RAM and GPU VRAM evidence is emitted;
12. selected local model profile either meets declared targets/headroom or is
    explicitly rejected without weakening the facade;
13. focused tests and risk-scoped checks pass or have an explicit `NOT_RUN`
    reason.

## 14. Scope choices before implementation

1. **Process topology**
   - **A — supervisor + isolated workers (recommended):** explicit residency,
     resource and fault boundaries.
   - B — one Python process: simpler launch, but allocator/OOM/runtime failures
     couple all models.

2. **Exact LLM/TTS selection**
   - **A — contracts first, then pin models after local benchmark
     (recommended):** avoids choosing artifacts that cannot coexist on 10 GiB.
   - B — pin exact models now: requires names, revisions, license, Russian
     quality and target placement before Commit A.

If no other preference is supplied, use `A/A`.
