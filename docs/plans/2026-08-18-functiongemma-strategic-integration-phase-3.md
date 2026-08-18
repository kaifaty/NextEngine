# Phase 3: FunctionGemma и strategic neural model

| Поле | Значение |
| --- | --- |
| Дата | 2026-08-18 |
| Статус | Deferred prerequisite-gated design; implementation is not authorized by Phase 2 |
| Зависимость | [Phase 2 LLM + TTS conversation service](2026-08-18-conversation-service-phase-2.md) |
| Candidate | `google/functiongemma-270m-it`; exact base/fine-tuned revisions remain unpinned |
| Причина отдельного этапа | FunctionGemma должна быть обучена на реальных возможностях движка и strategic model; generic base integration сейчас не даёт полезного contract evidence |
| Архитектурная граница | Optional `ai-host`/developer process; strategic output remains advisory/proposal-only until validated engine command flow exists |

## 1. Решение и целевой поток

FunctionGemma не подключается в Phase 2. Phase 3 начинается только после того,
как известны реальные возможности strategic neural model, stable engine-owned
names/types и authoritative mutation boundary. Тогда целевой поток имеет вид:

```text
UtteranceFinal
      │
      ▼
large LLM planning pass
      │ bounded StrategicRequestDraft
      ▼
fine-tuned FunctionGemma
      │ untrusted StrategicCallCandidate
      ▼
strict StrategicCallValidator
      │ validated typed call
      ▼
StrategicAgentGateway
      │
      ├─ deterministic strategic backend
      ├─ learned strategic neural model
      └─ fake/shadow backend
      │ bounded advisory/proposal result
      ▼
large LLM response pass
      │ admitted text
      ▼
TTS
```

FunctionGemma является replaceable compiler между bounded LLM intent и exact
strategic contract. Она не ведёт диалог, не исполняет функции, не получает raw
world storage и не создаёт `WorldCommand` напрямую.

## 2. Обязательные входные данные до начала реализации

Phase 3 не переходит к model integration, пока не готовы:

1. inventory реальных strategic capabilities и их владельцев;
2. engine-owned stable IDs, bounded input/output types and version policy;
3. distinction между immutable query, advisory result, `AgentIntent` proposal
   и authoritative `WorldCommand`;
4. exact gateway failure, deadline, cancellation and idempotency semantics;
5. representative Russian and engine-domain request corpus;
6. negative/adversarial corpus, включая irrelevant requests and prompt
   injection;
7. external artifact store, provenance/license policy and evaluation harness;
8. measured Phase 2 resource envelope, чтобы выбрать CPU/GPU placement;
9. concrete consumer and architecture promotion scope for any engine-facing
   public contract.

До этих артефактов можно делать только fake contract sketches. Нельзя объявлять
generic FunctionGemma base model рабочей частью продукта.

## 3. Replaceable boundaries

```python
class StrategicToolCompiler(Protocol):
    def capabilities(self) -> StrategicCompilerCapabilities: ...
    async def compile(self, request: StrategicCompileRequest) -> StrategicCompileResult: ...

class StrategicAgentGateway(Protocol):
    def capabilities(self) -> StrategicGatewayCapabilities: ...
    async def invoke(self, call: StrategicCall) -> StrategicResult: ...
```

Facade contracts не содержат `Gemma`, tokenizer, PyTorch/provider types или
конкретную strategic network class. Adapter владеет official chat template,
control tokens and parser. Gateway скрывает deterministic, learned and fake
backends behind one typed contract.

Large LLM получает только high-level declared capability names и выдаёт
bounded `StrategicRequestDraft`. FunctionGemma получает этот draft, selected
trusted schemas и cited immutable context IDs. Она не получает arbitrary user
tool definitions, callbacks, credentials, filesystem/network access or direct
world state.

## 4. Authority and safety boundary

Every model output is untrusted:

- exact tool ID and catalog revision must match;
- JSON types, depth, counts, UTF-8 bytes, enums and numeric ranges are bounded;
- extra fields, non-finite values and unknown context IDs are rejected;
- deadlines, cancellation and idempotency are checked before dispatch;
- result size/type is validated before returning to the LLM;
- mapping uses an explicit stable-ID dictionary, never regex plus dynamic
  execution;
- model result is advisory or an `AgentIntent` proposal;
- gameplay mutation remains possible only through normal validated
  `WorldCommand` transaction and committed `DomainEvent` flow.

Initial integration is shadow/read-only. Proposal-producing operations remain
disabled until a concrete engine consumer, replay/fault evidence and the normal
architecture workflow authorize them.

## 5. Training and evaluation strategy

Official FunctionGemma guidance positions the 270M model as a function-calling
specialist and a base for task-specific fine-tuning, not a direct dialogue
model. Upstream explicitly trains single-turn and independent parallel calls;
multi-step and multi-turn orchestration are not assumed.

Training assets remain outside the repository and include:

- exact catalog/schema revision;
- generated and human-reviewed Russian paraphrases;
- engine-domain vocabulary and ambiguous requests;
- positive single calls and independent parallel calls;
- no-call/irrelevant examples;
- malformed, stale, oversized and injection examples;
- provenance, license, split hashes and generator/reviewer identities;
- base model, tokenizer, runtime and fine-tune hashes.

Minimum held-out metrics:

- correct no-call versus call routing;
- exact tool ID accuracy;
- exact argument match and schema-valid rate;
- context-ID citation validity;
- irrelevance/injection rejection rate;
- duplicate/idempotency behavior;
- Russian paraphrase robustness;
- p50/p95 compile latency and peak RAM/VRAM;
- regression versus deterministic/fake baseline.

Thresholds are set from the concrete catalog and risk classification before
training. Aggregate accuracy alone cannot admit the model; every dangerous
operation class needs a separately passing rejection/validation gate.

## 6. Implementation sequence

### Phase 3 Gate A — inventory strategic capabilities

- document exact query/advisory/proposal operations;
- identify technical source of truth for every input/output;
- remove capabilities without a concrete consumer or safe authority boundary.

### Phase 3 Gate B — define the smallest strategic gateway contract

- versioned types, limits, deadlines, cancellation and idempotency;
- fake/shadow adapter and deterministic validator tests;
- architecture promotion only if a Rust public contract is actually needed.

### Phase 3 Gate C — build corpus and evaluation harness

- hash-closed external train/dev/test splits;
- Russian engine-domain positives, no-calls and adversarial cases;
- deterministic exact-match/schema/rejection metrics.

### Phase 3 Gate D — measure base FunctionGemma

- pin exact base artifact/runtime/tokenizer;
- official formatting and strict bounded parser;
- record baseline errors without production quality claim.

### Phase 3 Gate E — fine-tune and select artifact

- train outside runtime/repository;
- compare against base and deterministic alternatives;
- admit only a provenance-complete artifact that passes held-out gates.

### Phase 3 Gate F — integrate isolated compiler worker

- resident load/warm, capability handshake and resource preflight;
- LLM draft → FunctionGemma candidate → strict validation;
- cancellation, stale output, malformed tokens and crash coverage.

### Phase 3 Gate G — shadow strategic round trip

- validated call → fake/read-only strategic backend;
- bounded result → LLM response pass → existing TTS;
- no gameplay mutation.

### Phase 3 Gate H — optional proposal integration

- only after a concrete engine consumer and architecture approval;
- strategic output normalizes to declared `AgentIntent` candidate;
- engine validators independently decide whether a `WorldCommand` may be
  committed;
- replay, fallback, privacy, fault and performance checks become mandatory.

## 7. Completion gates

Phase 3 shadow vertical is complete only when:

1. exact strategic catalog and gateway contract are versioned;
2. FunctionGemma artifact is fine-tuned/evaluated on held-out engine-domain
   data and has complete hashes/provenance;
3. compiler is replaceable behind `StrategicToolCompiler`;
4. candidate validation rejects unknown/malformed/stale/unsafe calls;
5. strategic backend is replaceable behind `StrategicAgentGateway`;
6. one full `LLM → compiler → shadow strategic model → LLM → TTS` turn works;
7. second turn proves worker residency without reload;
8. failure degrades to truthful direct dialogue/text fallback;
9. CPU/GPU/RAM and p50/p95 evidence fits an admitted profile;
10. no component gains direct mutable-world access.

Proposal/engine integration has a separate completion gate and cannot be
inferred from a passing shadow vertical.

## 8. Rollback and reconsideration

- If fine-tuned FunctionGemma does not beat the deterministic baseline on exact
  schema/rejection metrics, keep the compiler disabled.
- If the strategic catalog changes faster than the model can be evaluated,
  prefer deterministic typed routing until a stable version exists.
- If Phase 2 models exhaust the local resource budget, keep Phase 3 in a
  separate measured worker/profile; do not weaken headroom checks.
- If a future model reliably combines planning and constrained compilation,
  it may replace both adapters only behind the same validator/gateway contracts.

## 9. Primary FunctionGemma sources

- [Google FunctionGemma overview](https://ai.google.dev/gemma/docs/functiongemma)
  — 270M function-calling role and task-specific fine-tuning intent.
- [Google formatting and best practices](https://ai.google.dev/gemma/docs/functiongemma/formatting-and-best-practices)
  — control tokens, single/parallel calls and multi-turn/multi-step limits.
- [Official FunctionGemma model card](https://huggingface.co/google/functiongemma-270m-it)
  — direct-dialogue limitation, evaluation and artifact information.
- [Google full function-calling sequence](https://ai.google.dev/gemma/docs/functiongemma/full-function-calling-sequence-with-functiongemma)
  — formatter/tool/result lifecycle and validation warning.
