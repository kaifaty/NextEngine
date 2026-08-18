# Phase 3: интеграция имеющихся neural capabilities в границы движка

| Поле | Значение |
| --- | --- |
| Дата | 2026-08-18 |
| Статус | Phase 3A scope confirmed: one-character simple-dialogue demo first; later 3B-3D remain consumer-driven |
| Зависимость | [Phase 2 LLM + TTS conversation service](2026-08-18-conversation-service-phase-2.md) |
| Следующий этап | [Phase 4 FunctionGemma and strategic-model tool integration](2026-08-18-functiongemma-strategic-integration-phase-4.md) |
| Первый профиль | Local Linux x86_64, один active game/conversation session, network default-deny |
| Архитектурная граница | Accepted ADR-005 process/fallback rules plus current deterministic Strategic Agent; learned and multimodal tracks remain optional/Proposed until consumer evidence exists |

## 1. Назначение этапа

Phase 3 — большой этап, разбитый на несколько независимо проверяемых
подэтапов. Он начинается не с абстрактного каталога возможностей, а с
минимального пользовательского vertical slice: в demo scene игрок подходит к
одному персонажу, начинает разговор, говорит в микрофон и получает текстовый и
озвученный ответ. Только после этого proven consumer служит основанием для
обобщения engine-facing boundaries и подключения внутренних моделей.

Для каждой модели Phase 3 отвечает на четыре вопроса:

1. какую конкретную роль и для какого production consumer она выполняет;
2. какие immutable данные может читать и какой bounded candidate возвращает;
3. кто валидирует result и какой subsystem остаётся authority;
4. какой deterministic/authored fallback сохраняет игру без модели.

Это отдельный этап между разговорным прототипом и FunctionGemma, потому что
FunctionGemma нельзя качественно обучить на предположительных функциях. Сначала
движок должен получить реальный, versioned и проверяемый каталог возможностей
существующих моделей и semantic operations. Только этот catalog становится
training/integration input Phase 4.

Phase 3 не создаёт generic AI framework заранее. Контракт появляется только у
модели с exact artifact, concrete consumer и проверяемым fallback согласно
ADR-046.

## 2. Декомпозиция Phase 3

| Подэтап | Результат | Что намеренно не входит |
| --- | --- | --- |
| **3A — one-character dialogue demo** | Играбельная сцена: микрофон → ASR/affect → LLM → субтитры/TTS, минимум три последовательных хода | Контекст мира, память NPC, gameplay consequences, внутренние strategic models, tools и FunctionGemma |
| **3B — capability boundary hardening** | Проверенные на 3A model-neutral handshake, роли, validators, fallbacks и `NeuralCapabilityCatalog` для speech/conversation stack | Универсальный registry для исследовательских моделей без consumers |
| **3C — internal neural capability integration** | Exact audit и shadow-only подключение существующей strategic neural model за engine-owned boundary | Активное принятие решений нейросетью и прямой доступ к world state |
| **3D — Phase 4 preparation** | Consumer-backed `StrategicSemanticCatalog`, corpus/evaluation seed и замороженные schema hashes | Загрузка, fine-tuning или runtime-интеграция FunctionGemma |

Phase 3A является первым обязательным этапом. 3B обобщает только те seams,
которые реально понадобились demo consumer. 3C и 3D не являются условием для
запуска или демонстрации 3A.

## 3. Phase 3A — demo scene с одним разговорным персонажем

### 3.1 Пользовательский vertical slice

3A переиспользует clean `reference-alpha`, существующего relay keeper, его idle
presentation, proximity targeting и штатный semantic `Interact`. Текущий live
dialogue уже открывается как presentation-only modal до выбора `Accept`.
AI-demo profile переиспользует этот open/close path, но не входит в
`AcceptPending` и не пересылает synthetic `Interact` в runtime. Новая
art/animation pipeline не нужна.

1. Игрок подходит к relay keeper и нажимает `Interact`.
2. Открывается bounded dialogue panel с явным состоянием микрофона.
3. Игрок удерживает push-to-talk, говорит одну реплику и отпускает кнопку.
4. Provisional ASR отображается как заменяемый текст и не считается финальным
   вводом.
5. Финальный transcript и vocal-affect observation образуют один
   `ConversationTurnRequest`.
6. LLM возвращает простой разговорный ответ от фиксированной demo persona.
7. Валидированные предложения сразу появляются в субтитрах, затем
   воспроизводятся TTS через presentation/audio boundary.
8. Игрок может провести минимум три последовательных хода без перезагрузки
   моделей. Закрытие диалога отменяет незавершённый ход и удаляет session-local
   history.

Push-to-talk выбран как первый режим, потому что даёт явную границу реплики и
не делает качество VAD условием demo. Автоматический endpointing остаётся
последующим измеряемым улучшением.

### 3.2 Разрешённый контекст

В 3A LLM получает только:

- immutable `DemoPersonaProfile`: stable persona ID, display-name text ID,
  короткую role instruction, locale, voice profile и authored fallback text ID;
- bounded историю текущей dialogue session;
- финальный текст текущей реплики;
- отдельное uncertainty-tagged описание наблюдаемой vocal expression;
- технические ограничения ответа: locale, maximum size и формат plain text.

Рекомендуемый первый предел — не более 8 последних turns и 32 KiB полного
request context; при переполнении старейшие complete turns удаляются до
формирования immutable request. История не сохраняется после закрытия сцены,
не входит в save/replay и не становится памятью персонажа.

Фиксированная persona описывает только манеру общения и имя demo-персонажа.
Это не world context. В 3A запрещены:

- quests, inventory, relationships, faction/world facts и dynamic scene state;
- `PerceptionFrame`, Agent memory, belief state, goals и affordances;
- запросы к внутренней strategic neural model;
- tool/function calls, arbitrary structured actions и FunctionGemma;
- любые обещания об изменении мира или прямые `WorldCommand`/ECS writes.

LLM output parser принимает только bounded textual response. Tool/function
fields, executable payloads и gameplay commitments получают typed rejection и
authored fallback.

### 3.3 Runtime flow и state machine

```text
semantic Interact
      ↓
presentation-owned DemoDialogueSession
      ↓ push-to-talk PCM
SpeechTimelineService ──→ final transcript + observed affect
      ↓
ConversationService ──→ validated text sentences
      ├─→ dialogue subtitles
      └─→ TTS worker ──→ AudioScene playback

No world snapshot, Agent memory, strategic model or tool gateway enters 3A.
```

```text
DialogueOpen
  → Listening
  → FinalizingSpeech
  → GeneratingReply
  → Speaking
  → DialogueOpen
  → Closed
```

Cancel/scene exit is accepted in every async state. A late ASR, LLM or TTS
result carries session/turn/revision identity and is discarded after cancel or
close. Dialogue input context captures movement/interact while the panel is
active; it does not silently change gameplay pause policy or session lifecycle.
Starting and closing this presentation demo do not mutate authoritative world
state. In particular, the existing authored `Accept` transition is not exposed
or triggered by AI-demo responses.

### 3.4 Engine-facing boundary for the demo

The game/application side depends on one model-neutral `ConversationClient`
and versioned events, not on Voxtral, emotion2vec, a specific LLM or TTS SDK.
The client performs an authenticated localhost handshake with the Phase 1/2
resident services and exposes:

- exact capability/model/profile identities;
- start/append/finish/cancel turn operations;
- transcript, affect, response-text and response-audio revisions;
- explicit ready/degraded/unavailable states;
- bounded deadlines, sizes and terminal failure dispositions.

Provider/device/runtime types stay outside Rust engine contracts. Raw
microphone audio is streamed to the optional host and is neither logged nor
persisted by default. Only admitted text is rendered as canonical subtitle;
TTS audio remains a reconstructible presentation result.

### 3.5 Degraded and offline behavior

- `ai-host` absent at scene open: show the authored greeting and keep the game
  playable; disable the microphone action with a stable diagnostic.
- ASR/affect unavailable: reject that voice turn cleanly; do not invent a
  transcript or emotion.
- LLM failure/timeout/malformed output: show a bounded authored response.
- TTS unavailable or interrupted: retain complete subtitles and allow the next
  turn after cancellation cleanup.
- character presentation asset unavailable: use the existing missing-content
  presentation path; the service must not become a gameplay dependency.

Lip sync, facial emotion animation, gestures, long-term memory and authored
branch consequences are explicitly not completion gates for 3A.

## 4. Целевой поток для Phase 3B-3D

```text
engine/application session
      │
      ├─ immutable revision-bound inputs
      │
      ▼
NeuralCapabilityResolver
      │ exact project profile + capability descriptor
      ├────────────────────────────────────────────────────┐
      ▼                                                    ▼
external ai-host roles                             bounded policy roles
ASR / affect / dialogue / TTS                      strategic model candidate
      │ observation/presentation/proposal                 │ score/advisory
      └──────────────────────┬─────────────────────────────┘
                             ▼
                   role-specific validator
                             │
               ┌─────────────┴─────────────┐
               ▼                           ▼
      admitted presentation       typed AgentIntent/advisory
                                           │
                                           ▼
                              deterministic owner validation
                                           │
                              WorldCommand commit or rejection

Every failure ──→ declared authored/deterministic fallback
```

Worker/model completion order и wall time не выбирают gameplay outcome. Async
result получает request identity, source revisions and deadline/cancellation;
late или stale result отбрасывается до owner publication.

## 5. Initial capability inventory

Inventory начинается только с существующих artifacts и adapters. Speech and
conversation entries ниже нужны уже 3A; strategic entry открывается только в
3C и не блокирует demo:

| Capability | First consumer stage | Engine interpretation | Authority |
| --- | --- | --- | --- |
| Streaming ASR | 3A via Phase 1 Voxtral adapter | candidate/final canonical text | demo dialogue admits only validated final text |
| Vocal affect | 3A via Phase 1 emotion2vec adapter | uncertainty-tagged observed-expression annotation | conversation context validator decides whether annotation is admitted |
| Dialogue generation | 3A via Phase 2 LLM adapter | untrusted plain-text response candidate | dialogue presentation validator and authored fallback |
| Speech synthesis | 3A via Phase 2 TTS adapter | presentation-only audio chunks | canonical text remains authority; playback owns no gameplay |
| Existing strategic neural model | 3C; exact artifact identity pending | shadow advisory or score over engine-built candidates | deterministic Strategic Agent and domain owners |

Motor/tactical/deformer/world-solver models do not enter this first profile
merely because related research exists. They get separate inventory entries
only when exact runnable artifacts and concrete consumers are supplied. This
prevents Phase 3 from silently promoting SPEC-33/34 or other Proposed tracks.

## 6. Role and authority classification

Every capability receives one closed role class:

| Class | Allowed output | Forbidden output |
| --- | --- | --- |
| `ObservationCandidate` | bounded fact/annotation with provenance, uncertainty and freshness | authoritative fact or direct state write |
| `PresentationCandidate` | text/audio/media segment tied to admitted source | gameplay event, commitment or owner mutation |
| `IntentCandidate` | typed bounded `AgentIntent` proposal | arbitrary function, `WorldCommand` or executable code |
| `CandidateScorer` | score/selection over canonical engine-built candidates | new hidden candidate, raw action or mutation |
| `AdvisoryQuery` | bounded non-authoritative analysis/result | claim that advisory result committed gameplay |

The existing strategic network must be classified before integration:

- if it scores engine-built goals/affordances, use `CandidateScorer` and retain
  deterministic Utility + bounded GOAP as the complete fallback;
- if it answers dialogue-facing strategic queries, use `AdvisoryQuery` behind
  a typed gateway and never treat its result as a committed agent decision;
- if it produces another shape, Phase 3 first defines the exact consumer and
  authority mapping rather than forcing it into an unsuitable class.

## 7. Capability descriptor

Phase 3 uses a bounded evidence/activation descriptor. It remains a working
profile until an actual production consumer justifies the smallest public
Rust contract.

```json
{
  "schema_version": 1,
  "capability_id": "nextengine.ai.dialogue.local.v1",
  "role_class": "PresentationCandidate",
  "adapter_protocol": "nextengine.ai-host.dialogue/1",
  "artifact": {
    "content_hash": "sha256:...",
    "runtime_hash": "sha256:...",
    "tokenizer_or_schema_hash": "sha256:..."
  },
  "input_schema_hash": "sha256:...",
  "output_schema_hash": "sha256:...",
  "source_revision_policy": "exact",
  "execution_class": "external_ai_host",
  "authority": "presentation_only",
  "resource_profile_id": "linux-rtx3080-local-v1",
  "fallback_id": "nextengine.dialogue.authored-text.v1",
  "network": "deny"
}
```

Descriptor also closes:

- stable capability/role/adapter IDs;
- exact artifact, runtime, schema and conversion hashes;
- locale, tensor/text/audio shapes and hard byte/count/depth limits;
- streaming, revision and finality semantics;
- deterministic/correspondence/statistical guarantee class;
- resident/peak CPU RAM, VRAM, disk and warm-up/concurrency envelope;
- deadline, cancellation, idempotency and stale-result policy;
- provenance, license, redistribution and privacy classification;
- declared validator, owner and fallback IDs.

It contains no filesystem path, provider session, ECS handle, callback,
credential, mutable cache or trainer object.

## 8. Runtime/process placement

ADR-005 governs generative and speech roles:

- ASR, affect/audio understanding, dialogue and TTS remain in optional isolated
  `ai-host` workers;
- network is default-deny;
- game/headless never block a simulation tick waiting for a worker;
- worker restart performs handshake and cannot repeat an admitted result.

Policy roles are classified separately. Small pinned motor policies may use
the existing in-process exception, but that exception is not generalized to
LLM/speech. A strategic model may use an isolated evaluator or a bounded
in-process policy adapter only after its exact scheduling, state, parity and
fallback requirements are demonstrated. Provider/device/runtime types never
cross the engine-owned boundary.

The initial implementation should reuse the Phase 1/2 resident workers and add
one engine-side coordinator, not merge Python/CUDA runtimes into the Rust
simulation process.

## 9. Engine-facing seams

Phase 3 separates three layers:

```text
model adapter
  → role-neutral candidate wire
  → engine-side role validator
  → existing domain owner/fallback
```

Recommended internal seams:

```rust
trait AiHostClient {
    fn capabilities(&self) -> BoundedCapabilitySet;
    fn submit(&self, request: AiHostRequest) -> RequestReceipt;
    fn poll_ready(&self, boundary: ExternalResultBoundary) -> Vec<AiHostResult>;
}

trait StrategicCandidateEvaluator {
    fn capabilities(&self) -> StrategicEvaluatorCapabilities;
    fn evaluate(
        &self,
        input: &StrategicEvaluationInput,
    ) -> StrategicEvaluationCandidate;
}
```

These are plan-level shapes, not authorized public APIs. Exact Rust types are
introduced only with their production consumer. `AiHostClient` returns staged
external candidates and never mutable owner references. Strategic evaluator
receives only engine-built canonical candidates and an epistemic view bounded
to the subject; it does not read raw ECS/world truth.

## 10. Integration lanes

### Lane A — 3A demo and 3B conversation hardening

3A connects the Phase 1/2 service to the bounded demo consumer described in
section 3. Then 3B extracts and hardens only the model-neutral seams proven by
that vertical:

1. engine starts without `ai-host` and selects authored text/subtitle fallback;
2. optional `ai-host` handshake reports exact ASR/affect/LLM/TTS capabilities;
3. finalized ASR becomes a candidate canonical utterance;
4. affect remains a separate uncertainty-tagged observation;
5. LLM text is validated before presentation or intent admission;
6. TTS consumes only admitted canonical text and remains presentation-only;
7. because 3A owns no gameplay state and is not save/replay input, replay does
   not rerun models; any later gameplay consumer needs a separate recorded
   canonical input/result design.

This lane is experimental relative to Deferred Proposed SPEC-16 until a
concrete production consumer and normal architecture promotion exist.

### Lane B — 3C existing strategic neural model

First perform an exact interface audit:

- artifact/runtime/state identities;
- observation and output shapes;
- whether output is score, selection, intent or advisory;
- hidden/recurrent state and reset/save rules;
- execution cadence and maximum batch;
- deterministic fallback and correspondence expectations;
- provenance/license and hardware envelope.

The first execution is shadow-only: model result is compared against the
deterministic Strategic Agent but cannot change selected goal, plan, command or
owner root. An active scorer/advisory profile is a later gate. A learned scorer
must receive canonical engine candidates; a dialogue-facing advisory receives
only a bounded query and immutable cited context.

### Lane C — 3D capability catalog for Phase 4

Generate two related but distinct versioned catalogs from actual consumers:

1. `NeuralCapabilityCatalog` — what each installed model role accepts,
   produces, costs and how it fails;
2. `StrategicSemanticCatalog` — which stable engine/strategic operations are
   actually callable as queries/proposals, with exact schemas and authority.

The second catalog contains no executable callbacks. Each operation maps to an
explicit validator/gateway implementation and declares whether its output is
observation, advisory or `AgentIntent` proposal. This exact revision plus
positive/negative examples becomes the FunctionGemma corpus seed in Phase 4.

## 11. State machine and failure rules

```text
Unresolved
  → DescriptorValidated
  → ArtifactValidated
  → WorkerReady
  → ShadowEnabled
  → EvidenceCollected
  → OptionalProfileAdmitted

Any failure → DisabledWithDeclaredFallback
```

Stable rules:

- invalid hash/schema/license rejects before worker/model use;
- capability/profile mismatch rejects before session start;
- timeout/crash/OOM/malformed result produces no partial candidate admission;
- stale revision or cancellation discards the complete result;
- late result cannot reopen a closed turn or cognition boundary;
- shadow result never changes gameplay state, RNG or canonical roots;
- optional model absence never disables the deterministic/authored path;
- no runtime training, weight mutation or active-session hot swap.

## 12. Resource and observability envelope

Every active profile records:

- cold load and warm-up per model;
- first and second request latency;
- per-role queue wait, execution, validation and delivery latency;
- resident and peak process RAM/VRAM;
- GPU allocator peak and at least 1 GiB remaining physical VRAM headroom;
- model reload count, worker restart count and fallback count;
- stale/cancelled/malformed/timeout/OOM dispositions;
- exact artifact/profile hashes in the measurement record.

Metrics and traces contain IDs, timing, sizes and dispositions by default, not
raw microphone audio, prompts, transcripts, embeddings, logits or generated
voices.

## 13. Implementation sequence

### Phase 3A Commit A1 — `feat(demo): add bounded dialogue session shell`

- reuse the clean `reference-alpha` relay keeper and the production semantic
  targeting/open path, without entering `AcceptPending`;
- add the presentation-owned state machine and a fake `ConversationClient`;
- cover open, three-turn loop, cancel/close and stale-result rejection without
  adding authoritative world state.

### Phase 3A Commit A2 — `feat(demo): add push-to-talk dialogue controls`

- explicit microphone permission/active/error indication;
- provisional/final transcript replacement and accessible subtitles;
- input-context capture and cancellation through existing UI/application
  boundaries.

### Phase 3A Commit A3 — `feat(ai): connect resident conversation service`

- authenticated localhost handshake to Phase 1/2 services;
- final ASR plus affect → bounded `ConversationTurnRequest`;
- fixed demo persona and session-only history; inspectable proof that world,
  memory, strategic-model and tool context are absent.

### Phase 3A Commit A4 — `feat(audio): present streamed character replies`

- validate plain-text LLM sentences before subtitles;
- stream admitted text to TTS and playback through `AudioScene`;
- authored response, subtitle-only and unavailable-service fallbacks.

### Phase 3A Commit A5 — `test(demo): close simple dialogue evidence`

- one real microphone-to-character run with at least three turns;
- cold/warm latency, queue time, model reload count and joint RAM/VRAM record;
- disconnect/cancel/stale/malformed/worker-crash matrix;
- prove the same authoritative world roots before and after the dialogue.

### Phase 3B Commit B1 — `docs(ai): inventory proven capabilities`

- inventory the exact ASR/affect/LLM/TTS artifacts and 3A consumer;
- assign role/authority/fallback classification from observed behavior;
- explicitly exclude research-only models without artifacts/consumers.

### Phase 3B Commit B2 — `feat(ai): harden bounded capability handshake`

- extract the smallest engine-neutral descriptor proven by 3A;
- add hash/schema/resource/provenance validation;
- close fake-worker, malformed, mismatch and second-session residency tests;
- emit the conversation subset of `NeuralCapabilityCatalog`.

### Phase 3C Commit C1 — `test(ai): audit strategic model interface`

- exact artifact/input/output/state/cadence/resource report;
- choose `CandidateScorer`, `AdvisoryQuery` or reject as incompatible;
- deterministic fake/golden corpus before real model execution.

### Phase 3C Commit C2 — `feat(ai): add shadow strategic adapter`

- run the strategic model over exact immutable engine-built inputs;
- validate and record candidate/advisory without applying it;
- collect fault, stale, restart, correspondence and resource evidence.

### Phase 3D Commit D1 — `docs(ai): freeze Phase 4 corpus seed`

- emit exact `NeuralCapabilityCatalog` and `StrategicSemanticCatalog` revision;
- bind stable IDs/schemas, validators, authority and failure semantics;
- prepare external positive/no-call/adversarial example manifest;
- do not load or train FunctionGemma.

An active learned strategic scorer or proposal route is a separate promotion
after shadow evidence; it is not implied by 3D.

## 14. Verification matrix

| Layer | Positive | Failure/adversarial |
| --- | --- | --- |
| Demo interaction | existing character opens/closes a dialogue session via semantic `Interact` | double open, close during every async state, late event after close |
| Demo context | fixed persona + bounded session turns + final text/affect only | world/memory/goal/tool field reaches LLM request |
| Demo presentation | three voice turns produce replaceable transcript, subtitle and TTS | missing microphone/model/TTS, malformed response, cancellation |
| Inventory | exact artifact/consumer/role mapping | model claimed from docs but no runnable artifact |
| Handshake | compatible schema/hash/resources | wrong version/hash, oversized limits, unknown role |
| Conversation | canonical text + affect → validated response/TTS | stale ASR, malformed LLM, TTS crash, ai-host absent |
| Strategic shadow | exact bounded input and output record | hidden state, non-finite/extra output, stale revision, OOM |
| Authority | candidate reaches declared validator only | direct WorldCommand/ECS/tool callback attempt |
| Fallback | complete authored/deterministic path | model absent/crashed at every boundary |
| Authority/replay | dialogue leaves authoritative roots unchanged | model output, timing or replay mutates gameplay state |
| Privacy | metadata-only default diagnostics | audio/prompt/transcript/embedding leakage |
| Resources | admitted joint profile has headroom | preflight rejection without partial startup |
| Catalog | exact callable consumer-backed operation set | speculative/unimplemented tool or mutable callback |

Documentation-only design uses the cheap check path. Executable 3A integration
requires focused checks plus `play`; content-scene/package changes also require
`content-package`, and material model-runtime work conditionally requires
`performance`. `persistence-replay` becomes required only if a later scope adds
authoritative state/commands; doing so accidentally in 3A is a design failure,
not a reason to normalize the coupling.

## 15. Definition of done

### 15.1 Phase 3A exit gate

Phase 3A завершена, когда одновременно:

1. the clean `reference-alpha` relay keeper can be approached and opened
   through semantic `Interact` in AI-demo profile;
2. push-to-talk produces provisional then final transcript with an explicit
   microphone state;
3. final text and separately structured vocal affect start exactly one bounded
   conversation turn;
4. the LLM request contains only the fixed demo persona, current session
   history, current utterance/affect and response constraints;
5. no world context, NPC memory, internal model, tool schema or FunctionGemma
   data reaches the request;
6. validated character text appears in subtitles and admitted sentences reach
   TTS/`AudioScene` without giving audio gameplay authority;
7. at least three sequential turns complete without model reload;
8. leaving/cancelling in every async state prevents late output from reopening
   the dialogue;
9. `ai-host`, ASR/affect, LLM and TTS failure paths have visible bounded
   fallbacks and never make the game unplayable;
10. authoritative world roots and the authored dialogue offer node are
    identical before and after the demo conversation;
11. cold/warm latency, queue, reload, RAM and VRAM measurements are reported
    against the exact model/profile identities;
12. raw audio, prompts, transcripts and generated voice are not persisted or
    logged by default, and risk-scoped checks pass or have explicit `NOT_RUN`
    reasons.

Lip sync, expressive animation, world-aware answers, persistent memory and
strategic/tool integration cannot be used to delay or claim completion of 3A.

### 15.2 Full Phase 3 exit gate

Phase 3 is complete only after 3A and all of the following:

1. every included model has an exact artifact, adapter, role and consumer;
2. every descriptor names bounds, hashes, authority, resources and fallback;
3. engine starts and completes the mandatory path without any model/`ai-host`;
4. existing strategic neural model has an exact interface/state/resource audit
   and runs shadow-only over engine-bounded inputs;
5. crash/timeout/OOM/stale/malformed cases select typed fallback;
6. the joint local profile has explicit latency/resource admission evidence;
7. consumer-backed `NeuralCapabilityCatalog` and
   `StrategicSemanticCatalog` revisions plus Phase 4 corpus seed are frozen;
8. no generic registry, active learned-policy promotion or FunctionGemma
   integration is claimed from shadow evidence.

## 16. Remaining scope choices before Phase 3C

1. **Post-demo inventory breadth**
   - **A — Phase 1/2 speech stack plus the existing strategic model
     (recommended):** every entry has a direct relationship to the planned
     conversation/FunctionGemma path.
   - B — every neural experiment in the repository: much broader and would
     mix motor/world research without concrete consumers.

2. **Strategic model first mode**
   - **A — shadow-only (recommended):** validates interface, capability catalog
     and resource behavior without changing gameplay decisions.
   - B — active candidate scorer/advisory immediately: requires exact consumer,
     state/fallback and corresponding architecture/product checks first.

These choices do not block 3A. If no other preference is supplied before 3C,
use `A/A`.

## 17. Relationship to normative architecture

- ADR-005 requires optional isolated `ai-host`, default-off network and complete
  deterministic/authored fallback.
- SPEC-18 and ADR-019 keep player interaction semantic and presentation
  non-authoritative; model callbacks do not become gameplay mutation paths.
- SPEC-29 and ADR-047 keep application-session lifecycle explicit; opening the
  demo dialogue cannot silently replace pause, close or save policy.
- SPEC-30 and ADR-028 keep extracted render/presentation state
  non-authoritative, while SPEC-08 keeps TTS/`AudioScene` playback subordinate
  to admitted canonical subtitle text.
- ADR-044 keeps authored greeting, fallback and subtitle strings behind neutral
  text IDs and locale fallback rather than embedding provider-owned text.
- SPEC-06/32 and ADR-056/073/074 keep the deterministic Strategic Agent and
  domain owners authoritative; neural output is only observation, advisory,
  intent or score candidate.
- SPEC-16/ADR-017 dialogue/model-pack details remain Deferred Proposed.
- SPEC-33/34 and ADR-050/053/054 learned behavior/training remain optional
  Proposed R8 work; shadow integration does not promote them.
- ADR-046 forbids a public generic framework before a production consumer.

Therefore this plan changes no Accepted architecture or roadmap status. A
future production engine-facing contract or active learned route follows the
normal consumer-driven ADR/SPEC workflow independently.
