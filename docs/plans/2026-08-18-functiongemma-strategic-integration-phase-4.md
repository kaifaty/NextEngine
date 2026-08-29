# Phase 4: FunctionGemma и strategic neural model tool integration

| Поле | Значение |
| --- | --- |
| Дата | 2026-08-18 |
| Статус | Deferred prerequisite-gated design; implementation is not authorized by Phase 2 or Phase 3 shadow evidence |
| Зависимость | [Phase 3 engine neural-capability integration](2026-08-18-engine-neural-capability-integration-phase-3.md) |
| Candidate | `google/functiongemma-270m-it`; exact base/fine-tuned revisions remain unpinned |
| Причина отдельного этапа | FunctionGemma обучается на frozen consumer-backed strategic catalog, а не на предположительных возможностях движка или модели |
| Архитектурная граница | Optional `ai-host`/developer process; strategic output remains advisory/proposal-only until validated engine command flow exists |

## 1. Решение и целевой поток

FunctionGemma не подключается в Phase 2 и не входит в Phase 3. Phase 4
начинается только после того, как Phase 3 свела реальные neural capabilities к
engine-owned границам и заморозила exact `StrategicSemanticCatalog`.

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
      │ validated typed call from exact catalog revision
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

## 2. Entry gates from Phase 3

Phase 4 не переходит к model work, пока не готовы:

1. exact `NeuralCapabilityCatalog` revision for every participating model;
2. exact consumer-backed `StrategicSemanticCatalog` revision;
3. classification of the strategic model as `CandidateScorer`,
   `AdvisoryQuery` or another explicitly approved role;
4. engine-owned stable IDs, bounded input/output schemas and version policy;
5. distinction between immutable query, advisory result, `AgentIntent`
   proposal and authoritative `WorldCommand`;
6. exact gateway failure, deadline, cancellation and idempotency semantics;
7. Phase 3 shadow results proving zero changes to authoritative roots;
8. representative Russian/engine-domain positive and negative corpus seed;
9. external artifact store, provenance/license policy and evaluation harness;
10. measured joint resource envelope for worker placement;
11. concrete consumer and architecture promotion scope for any new public
    engine contract.

До выполнения этих gates generic base FunctionGemma остаётся research
candidate и не является working product component.

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

Large LLM выдаёт bounded `StrategicRequestDraft`. FunctionGemma получает
только этот draft, selected trusted schemas from the exact Phase 3 catalog and
cited immutable context IDs. Она не получает arbitrary user tool definitions,
callbacks, credentials, filesystem/network access or direct world state.

## 4. Authority and validation

Every model output is untrusted:

- exact tool ID and catalog revision must match;
- JSON types, depth, counts, UTF-8 bytes, enums and numeric ranges are bounded;
- extra fields, non-finite values and unknown context IDs are rejected;
- deadlines, cancellation and idempotency are checked before dispatch;
- result size/type is validated before returning to the LLM;
- mapping uses an explicit stable-ID dictionary, never regex plus dynamic
  execution;
- model result is observation, advisory or an `AgentIntent` proposal according
  to the Phase 3 authority classification;
- gameplay mutation remains possible only through normal validated
  `WorldCommand` transaction and committed `DomainEvent` flow.

Initial FunctionGemma integration is shadow/read-only. Proposal-producing
operations remain disabled until a concrete engine consumer, replay/fault
evidence and normal architecture workflow authorize them.

## 5. Training and evaluation strategy

FunctionGemma is treated as a function-calling base for task-specific
fine-tuning, not a direct dialogue model. Multi-step orchestration is owned by
the supervisor; the first profile supports one compile pass and bounded
independent parallel calls only.

Training assets remain outside the repository and include:

- exact Phase 3 catalog/schema revisions;
- generated and human-reviewed Russian paraphrases;
- engine-domain vocabulary and ambiguous requests;
- positive calls and no-call/irrelevant examples;
- malformed, stale, oversized and injection examples;
- examples for every strategic role/authority class;
- provenance, license, split hashes and generator/reviewer identities;
- base model, tokenizer, runtime and fine-tune hashes.

Minimum held-out metrics:

- correct no-call versus call routing;
- exact operation ID accuracy;
- exact argument match and schema-valid rate;
- context-ID citation validity;
- irrelevance/injection rejection rate;
- duplicate/idempotency behavior;
- Russian paraphrase robustness;
- p50/p95 compile latency and peak RAM/VRAM;
- regression versus deterministic/fake compiler baseline.

Aggregate accuracy alone cannot admit the model. Every operation class with
mutation-adjacent consequences needs its own passing rejection/validation gate.

## 6. Implementation sequence

### Phase 4 Gate A — validate Phase 3 catalog closure

- reopen exact catalog/schema/validator bytes;
- reject missing, speculative or consumer-less operations;
- freeze the training/evaluation catalog revision.

### Phase 4 Gate B — build corpus and evaluation harness

- hash-closed external train/dev/test splits;
- Russian engine-domain positives, no-calls and adversarial cases;
- deterministic exact-match/schema/rejection metrics.

### Phase 4 Gate C — measure base FunctionGemma

- pin exact base artifact/runtime/tokenizer;
- official formatting and strict bounded parser;
- record baseline errors without production quality claim.

### Phase 4 Gate D — fine-tune and select artifact

- train outside runtime/repository;
- compare against base and deterministic alternatives;
- admit only a provenance-complete artifact passing held-out gates.

### Phase 4 Gate E — integrate isolated compiler worker

- resident load/warm, capability handshake and resource preflight;
- LLM draft → FunctionGemma candidate → strict validation;
- cancellation, stale output, malformed tokens and crash coverage.

### Phase 4 Gate F — shadow strategic round trip

- validated call → fake/read-only strategic backend;
- bounded result → LLM response pass → existing TTS;
- compare with direct Phase 3 gateway use and change no gameplay state.

### Phase 4 Gate G — optional proposal integration

- only after a concrete engine consumer and architecture approval;
- strategic output normalizes to declared `AgentIntent` candidate;
- engine validators independently decide whether a `WorldCommand` may be
  committed;
- replay, fallback, privacy, fault and performance checks become mandatory.

## 7. Completion gates

Phase 4 shadow vertical is complete only when:

1. exact Phase 3 catalogs and gateway contract are reopened and validated;
2. FunctionGemma artifact is fine-tuned/evaluated on held-out engine-domain
   data and has complete hashes/provenance;
3. compiler is replaceable behind `StrategicToolCompiler`;
4. candidate validation rejects unknown/malformed/stale/unsafe calls;
5. strategic backend remains replaceable behind `StrategicAgentGateway`;
6. one full `LLM → compiler → shadow strategic model → LLM → TTS` turn works;
7. direct Phase 3 and FunctionGemma-mediated calls have equivalent gateway
   validation semantics;
8. second turn proves worker residency without reload;
9. failure degrades to truthful direct dialogue/text fallback;
10. CPU/GPU/RAM and p50/p95 evidence fits an admitted profile;
11. no component gains direct mutable-world access.

Proposal/engine integration has a separate completion gate and cannot be
inferred from a passing shadow vertical.

## 8. Rollback and reconsideration

- If fine-tuned FunctionGemma does not beat the deterministic compiler on exact
  schema/rejection metrics, keep it disabled.
- If the Phase 3 catalog changes, invalidate affected train/eval results and
  produce a new artifact revision; never silently reuse the old model.
- If catalog churn outpaces evaluation, prefer deterministic typed routing.
- If the compound stack exceeds the local resource budget, keep FunctionGemma
  in a separate measured worker/profile or omit it.
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
