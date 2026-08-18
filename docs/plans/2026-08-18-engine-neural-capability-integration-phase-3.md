# Phase 3: интеграция имеющихся neural capabilities в границы движка

| Поле | Значение |
| --- | --- |
| Дата | 2026-08-18 |
| Статус | User-requested intermediate-stage design; optional and consumer-driven; two scope choices remain pending |
| Зависимость | [Phase 2 LLM + TTS conversation service](2026-08-18-conversation-service-phase-2.md) |
| Следующий этап | [Phase 4 FunctionGemma and strategic-model tool integration](2026-08-18-functiongemma-strategic-integration-phase-4.md) |
| Первый профиль | Local Linux x86_64, один active game/conversation session, network default-deny |
| Архитектурная граница | Accepted ADR-005 process/fallback rules plus current deterministic Strategic Agent; learned and multimodal tracks remain optional/Proposed until consumer evidence exists |

## 1. Назначение этапа

Phase 3 сводит фактически имеющиеся нейросети в engine-owned typed boundaries.
Он отвечает на четыре вопроса для каждой модели:

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

## 2. Целевой поток

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

## 3. Initial capability inventory

Inventory начинается только с существующих artifacts и adapters:

| Capability | Source stage | Engine interpretation | Authority |
| --- | --- | --- | --- |
| Streaming ASR | Phase 1 Voxtral adapter | candidate/final canonical text | RPG dialogue admits only validated final text |
| Vocal affect | Phase 1 emotion2vec adapter | uncertainty-tagged observed-expression annotation | perception/context owner decides whether annotation is admitted |
| Dialogue generation | Phase 2 LLM adapter | untrusted response/intent candidate | RPG/Agent validators and authored fallback |
| Speech synthesis | Phase 2 TTS adapter | presentation-only audio chunks | canonical text remains authority; playback owns no gameplay |
| Existing strategic neural model | existing engine/R&D artifact, exact identity pending | shadow advisory or score over engine-built candidates | deterministic Strategic Agent and domain owners |

Motor/tactical/deformer/world-solver models do not enter this first profile
merely because related research exists. They get separate inventory entries
only when exact runnable artifacts and concrete consumers are supplied. This
prevents Phase 3 from silently promoting SPEC-33/34 or other Proposed tracks.

## 4. Role and authority classification

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

## 5. Capability descriptor

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

## 6. Runtime/process placement

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

## 7. Engine-facing seams

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

## 8. Integration lanes

### Lane A — conversation capabilities

Connect the Phase 1/2 service to an engine/application consumer:

1. engine starts without `ai-host` and selects authored text/subtitle fallback;
2. optional `ai-host` handshake reports exact ASR/affect/LLM/TTS capabilities;
3. finalized ASR becomes a candidate canonical utterance;
4. affect remains a separate uncertainty-tagged observation;
5. LLM text is validated before presentation or intent admission;
6. TTS consumes only admitted canonical text and remains presentation-only;
7. replay consumes recorded canonical input/result and does not rerun models.

This lane is experimental relative to Deferred Proposed SPEC-16 until a
concrete production consumer and normal architecture promotion exist.

### Lane B — existing strategic neural model

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

### Lane C — capability catalog for Phase 4

Generate two related but distinct versioned catalogs from actual consumers:

1. `NeuralCapabilityCatalog` — what each installed model role accepts,
   produces, costs and how it fails;
2. `StrategicSemanticCatalog` — which stable engine/strategic operations are
   actually callable as queries/proposals, with exact schemas and authority.

The second catalog contains no executable callbacks. Each operation maps to an
explicit validator/gateway implementation and declares whether its output is
observation, advisory or `AgentIntent` proposal. This exact revision plus
positive/negative examples becomes the FunctionGemma corpus seed in Phase 4.

## 9. State machine and failure rules

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

## 10. Resource and observability envelope

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

## 11. Implementation sequence

### Phase 3 Commit A — `docs(ai): inventory actual neural capabilities`

- locate exact runnable artifacts/adapters and consumers;
- assign role/authority/fallback classification;
- explicitly exclude research-only models without artifacts/consumers.

### Phase 3 Commit B — `feat(ai): define bounded capability handshake`

- engine-neutral descriptor and `ai-host` handshake;
- hash/schema/resource/provenance validation;
- fake worker plus malformed/mismatch tests.

### Phase 3 Commit C — `feat(ai): connect conversation candidates to engine`

- finalized ASR, affect observation, LLM candidate and admitted TTS flow;
- authored/text/subtitle fallback with `ai-host` absent;
- no direct gameplay mutation and no replay-time regeneration.

### Phase 3 Commit D — `test(ai): audit strategic model interface`

- exact artifact/input/output/state/cadence/resource report;
- choose `CandidateScorer`, `AdvisoryQuery` or reject as incompatible;
- deterministic fake/golden corpus before real model execution.

### Phase 3 Commit E — `feat(ai): add shadow strategic adapter`

- run the strategic model over exact immutable inputs;
- validate and record candidate/advisory without applying it;
- fault, stale, restart and resource evidence.

### Phase 3 Commit F — `docs(ai): freeze Phase 4 capability corpus seed`

- emit exact `NeuralCapabilityCatalog` and `StrategicSemanticCatalog` revision;
- bind stable IDs/schemas, validators, authority and failure semantics;
- prepare external positive/no-call/adversarial example manifest;
- do not load or train FunctionGemma.

An active learned strategic scorer or proposal route is a separate promotion
after shadow evidence; it is not implied by Commit F.

## 12. Verification matrix

| Layer | Positive | Failure/adversarial |
| --- | --- | --- |
| Inventory | exact artifact/consumer/role mapping | model claimed from docs but no runnable artifact |
| Handshake | compatible schema/hash/resources | wrong version/hash, oversized limits, unknown role |
| Conversation | canonical text + affect → validated response/TTS | stale ASR, malformed LLM, TTS crash, ai-host absent |
| Strategic shadow | exact bounded input and output record | hidden state, non-finite/extra output, stale revision, OOM |
| Authority | candidate reaches declared validator only | direct WorldCommand/ECS/tool callback attempt |
| Fallback | complete authored/deterministic path | model absent/crashed at every boundary |
| Replay | recorded canonical inputs/results | model rerun or wall-time-selected outcome |
| Privacy | metadata-only default diagnostics | audio/prompt/transcript/embedding leakage |
| Resources | admitted joint profile has headroom | preflight rejection without partial startup |
| Catalog | exact callable consumer-backed operation set | speculative/unimplemented tool or mutable callback |

Documentation-only design uses the cheap check path. Executable engine
integration later inherits `fast` and `play`; state/command changes also require
`persistence-replay`, artifact/content changes `content-package`, and material
model-runtime work conditional `performance`.

## 13. Definition of done

Phase 3 shadow vertical is complete when:

1. every included model has an exact artifact, adapter, role and consumer;
2. every descriptor names bounds, hashes, authority, resources and fallback;
3. engine starts and completes the mandatory path without any model/`ai-host`;
4. Phase 1/2 models connect through staged typed candidates, not direct owner
   access;
5. TTS remains presentation-only and model output never mutates world directly;
6. existing strategic neural model has an exact interface/state/resource audit;
7. strategic model runs in shadow over engine-bounded inputs without changing
   goals, commands, RNG or authoritative roots;
8. crash/timeout/OOM/stale/malformed cases select typed fallback;
9. second session proves worker/model residency without accidental reload;
10. joint resource/latency evidence admits or rejects an exact local profile;
11. `NeuralCapabilityCatalog` and consumer-backed
    `StrategicSemanticCatalog` revisions are frozen;
12. the external Phase 4 corpus seed binds those exact catalog/schema hashes;
13. no generic registry, learned-policy promotion or FunctionGemma integration
    is claimed from shadow evidence.

## 14. Scope choices before implementation

1. **First inventory breadth**
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

If no other preference is supplied, use `A/A`.

## 15. Relationship to normative architecture

- ADR-005 requires optional isolated `ai-host`, default-off network and complete
  deterministic/authored fallback.
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
