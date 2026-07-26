# SPEC-16: Text-canonical multimodal dialogue и model packs

| Поле | Значение |
|---|---|
| ID | SPEC-16 |
| Статус | Proposed |
| Lifecycle | Deferred Proposed |
| Версия | 0.5 |
| Последняя проверка | 2026-07-26 |
| Нормативные зависимости | [SPEC-01](01-system-architecture.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-06](06-ai-agents-perception-and-memory.md), [SPEC-07](07-rpg-scripting-and-plugins.md), [SPEC-08](08-audio-navigation-and-world-services.md), [SPEC-09](09-tooling-sdk-and-observability.md), [SPEC-11](11-security-licensing-and-governance.md), [ADR-005](adr/005-offline-first-ai-process-boundary.md), [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md) |
| Заменяет | отсутствует |

## Статус предложения: Deferred Proposed

Этот документ имеет lifecycle `Deferred Proposed`: contracts ниже доступны для обсуждения и прототипирования, но не меняют Accepted runtime. Ни одна модель из [RESEARCH-002](research/npc-dialogue-model-landscape.md) не является shipping default.

## Назначение и invariants

SPEC-16 задаёт один dialogue-turn contract для typed text и microphone audio, streaming presentation, per-role model routing, optional local packs и explicit remote providers.

- UTF-8 NFC text MUST быть единственным каноническим содержанием dialogue turn.
- Typed input и finalized ASR MUST создавать один и тот же `CanonicalUtterance`; partial ASR не является gameplay input.
- LLM, embeddings, ASR, TTS и audio-understanding MUST исполняться только в optional separate `ai-host` по ADR-005.
- Model output MUST оставаться untrusted candidate. RPG/world mutation проходит `AgentIntent → validator → WorldCommand → commit`.
- Simulation tick MUST NOT ждать ASR/LLM/TTS. Deadline/cancellation приходят как staged external signals и записываются как replay input, а не вычисляются из wall clock при replay.
- Base game MUST NOT включать generative weights. Authored dialogue, subtitles и deterministic planner являются обязательным `TextOnlyFallback`.
- Local packs MUST быть immutable, content-addressed, separately installed и reconstructible; cloud MUST быть default-off и explicit opt-in.
- Vendor types, raw filesystem paths, credentials, provider session IDs, tokenizer objects и audio-device handles MUST NOT входить в public contracts.

## Source of truth и ownership

| State | Единственный owner/source of truth | Не является source |
|---|---|---|
| Dialogue session, choices, commitments, transcript policy | RPG Framework `Dialogue` aggregate | Prompt, provider thread, TTS playback |
| Canonical player/NPC utterance admitted into session | RPG Framework after turn validation | Partial ASR, token stream, waveform |
| Perception, memory projection, high-level proposal | Agent Intelligence / Memory Service per SPEC-06 | Vendor cache/session |
| Model catalog and installed immutable model resolution | Asset & Tool chain model-pack registry | Downloader temp path, provider model list |
| Generated PCM/playback progress | World Services presentation/audio runtime | Dialogue state or replay gameplay hash |
| Pack/provider grants, voice consent and licensing | Versioned project security policy | Model card claim, local UI toggle alone |
| Gameplay mutation | Owning domain after committed WorldCommand | Dialogue text, function/tool call, speech segment |

Agent Intelligence subsystem владеет turn protocol и deterministic routing policy. RPG Framework владеет authoritative dialogue state. World Services владеет audio capture/playback streams. Asset & Tool chain владеет pack manifests, installation transaction и replay references. Security policy владеет network, data, license и voice-consent rules.

## Public contracts

All contracts are engine-owned versioned values intended for `crates/contracts`; this proposal does not add Rust code.

### DialogueTurnId и CanonicalUtterance

`DialogueTurnId` is an opaque 128-bit nominal ID scoped by dialogue session and idempotency ledger. It MUST NOT be derived from wall clock, provider request ID or audio-device state.

`CanonicalUtterance` contains:

| Field | Contract |
|---|---|
| `schema_version` | Exact turn schema major/minor |
| `dialogue_session_id`, `turn_id` | RPG session identity + nominal `DialogueTurnId` |
| `speaker: PersistentId` | Valid participant in current session |
| `source` | Closed enum `Typed` or `FinalAsr`; audio-native adapter also resolves to `FinalAsr` |
| `locale` | Valid BCP-47 tag, maximum 64 UTF-8 bytes |
| `text` | NFC UTF-8, no NUL/control payload, maximum 4,096 bytes for player input |
| `input_revision` | Monotonic session input revision for stale/cancel checks |
| `provenance` | Local typed-input marker or ASR pack/provider/model/content hashes; no provider secret/session |

ASR confidence/timestamps MAY be attached as non-authoritative annotations. They cannot alter text after final admission; correction creates a new revision/turn according to RPG transcript policy.

### DialogueTurnRequest

`DialogueTurnRequest` contains protocol/schema version, session/turn/speaker IDs, current dialogue revision, locale, response schema ID, `CanonicalUtterance`, capability-filtered `PerceptionFrame`, bounded memory projection, planner-visible `MechanicAffordance` values, allowed intent kinds, content/project/model hashes, deadline token, cancellation token and idempotency key.

V1 hard ceilings before IPC serialization:

- canonical input ≤4,096 UTF-8 bytes;
- total serialized context ≤65,536 bytes;
- ≤128 facts, ≤64 memory records and ≤64 affordances;
- response text ≤16,384 UTF-8 bytes;
- project MAY choose lower limits; larger limits require a new versioned profile and repeated security/performance checks.

Selection of facts/memories/affordances MUST be deterministic for the same snapshot/profile: stable relevance key, then stable ID. The request cannot contain save bytes, raw prompt files, arbitrary tool definitions, filesystem paths or credentials.

### DialogueTurnCandidate

`DialogueTurnCandidate` contains request/turn/idempotency IDs, canonical response text, sentence boundaries, optional bounded `AgentIntent`, memory proposals, cited fact/revision IDs, completion state, selected role/pack/provider IDs, model/content hashes, generation parameters, adapter protocol version and redaction classification.

Candidate validation order:

1. framing, schema, size, UTF-8/NFC and idempotency;
2. dialogue session/turn/revision and cancellation freshness;
3. pack/provider provenance and granted capability;
4. referenced fact/affordance freshness;
5. content/safety policy and allowed intent kinds;
6. RPG dialogue/narrative constraints;
7. optional `AgentIntent` validation and WorldCommand construction;
8. atomic command commit or stable rejection/fallback.

Provider `function_call`/tool output MUST map to a declared `AgentIntent` variant or be rejected. It MUST NOT carry executable code, arbitrary command payload or direct mutable target.

### SpeechSegmentCandidate

`SpeechSegmentCandidate` is presentation-only and contains turn ID, sentence index, monotonic segment sequence, `is_final`, locale, audio encoding/sample rate/channels, byte length, content hash, source text hash, TTS pack/provider provenance and cancellation key. One IPC segment is limited to 262,144 bytes; larger audio is split into ordered chunks.

Unknown, duplicate-with-different-bytes, skipped or post-final sequence MUST be rejected. Exact duplicate chunk MAY be deduplicated. PCM/encoded bytes never enter authoritative state hash.

### AiModelPackManifest

`AiModelPackManifest` is an immutable public manifest:

| Field group | Required content |
|---|---|
| Identity | schema, `pack_id`, semantic version, publisher, manifest ContentHash |
| Roles | One or more closed roles: `asr`, `dialogue`, `tts`, `embeddings`, `audio-understanding`, `narrative-director` |
| Model files | Engine `AssetId`, SHA-256, byte size, media/model format, quantization, adapter ID/protocol range; no runtime path |
| Compatibility | locales, input/output schemas, context/audio limits, streaming/timestamp/structured-output/voice-clone capabilities |
| Resources | supported target triples, CPU features, RAM/VRAM/disk envelope, declared concurrency and warm-up policy |
| Provenance | upstream source URL + immutable revision, build/conversion tool hashes, parent model/content hashes |
| Licensing | code/weights/data/voice SPDX or project classification, redistribution scope, notices and exception reference |
| Validation | compatible check IDs and result hashes; self-reported metrics remain separate |
| Fallback | next compatible pack role or `TextOnlyFallback`; model cannot select it |

Pack MUST NOT contain credentials, provider account, absolute path, mutable cache, raw dataset, training run or voice reference recording. Voice-clone capable pack remains disabled until exact `VoiceConsentRecord` passes `VOICE-L1`.

### AiProviderProfile

`AiProviderProfile` contains profile/adapter ID, protocol range, allowed roles/models, endpoint class, network capability name, transmitted data categories, retention/zero-retention mode, processing region, terms/pricing snapshot timestamp, request size/deadline limits, disclosure text ID, consent revision and local fallback. Endpoint URL MAY be a project configuration reference; credentials MUST live in an external secret store and never enter manifest/save/replay/log.

Revoked or stale consent disables new requests immediately, cancels in-flight work best-effort and selects local/text fallback. Already committed gameplay is not rolled back.

### DialogueCapabilityProfile

`DialogueCapabilityProfile` contains a deterministic ordered route for each role and locale, resource ceilings, concurrency, deadline, fallback and grants. Standard profiles:

| Profile | Allowed routes | Mandatory fallback |
|---|---|---|
| `RussianLocal` | Valid local Russian ASR/dialogue/TTS packs | `TextOnlyFallback` |
| `MultilingualLocal` | Valid local packs matching exact locale | `RussianLocal` when compatible, else `TextOnlyFallback` |
| `HighEndStreaming` | Valid local streaming pack on declared high-end hardware | `MultilingualLocal`, then `TextOnlyFallback` |
| `RemoteOptIn` | Explicit user/project-granted provider per role | Matching local profile, then `TextOnlyFallback` |
| `TextOnlyFallback` | Typed text/authored deterministic dialogue/subtitles and optional authored/default voice | none |

Resolver sorts only by profile order, exact compatibility, granted capability and installed/healthy state. Model output, measured response speed, provider suggestion, random choice or worker completion order cannot change route during one turn. Health transition is staged and takes effect at a declared turn boundary.

## Turn data flow и streaming

```text
typed text ───────────────┐
                         ├→ CanonicalUtterance → DialogueTurnRequest
microphone → partial ASR ┘          │
       (ephemeral UI only)          ↓
                            optional ai-host
                                    ↓
                         DialogueTurnCandidate
                                    ↓
                  schema/freshness/capability/RPG validation
                         ┌──────────┴──────────┐
                         ↓                     ↓
              accepted WorldCommand       rejection/fallback
                         ↓                     ↓
                    commit/event       authored safe response
                         └──────────┬──────────┘
                                    ↓
                         canonical subtitle text
                                    ↓
                        optional TTS → audio scene
```

- Partial ASR, LLM tokens and PCM chunks are ephemeral, cancelable and bounded. They MAY drive provisional UI only.
- A finalized `CanonicalUtterance` is admitted once. Late/cancelled ASR cannot create a second turn with the same idempotency identity.
- Sentence-level subtitle/TTS MAY start after the sentence passes schema/content validation.
- A sentence that states a quest, trade, inventory, relationship or other gameplay commitment MUST be held until every referenced WorldCommand commits. Rejection emits an authored rejection/fallback sentence; speculative state-changing speech is never presented as fact.
- Eligibility prewarm during ASR MAY read only immutable capability/fact views. It cannot reserve, mutate or guarantee an action before finalized input and common validation.
- Barge-in cancels current capture/playback and any uncommitted candidate. It cannot roll back committed commands or DomainEvents.
- Dialogue state MUST NOT wait for TTS completion unless authored content explicitly uses a deterministic timing command independent of generated audio duration.

## Model-pack installation и future CLI

The future tool flow is `download to explicit staging root → size/hash/schema/license/provenance validation → adapter compatibility probe → atomic registry publish`. Failure quarantines staging and preserves the previous active revision. Runtime resolves `AssetId + ContentHash`; it never receives downloader path.

Future CLI/JSON contracts under SPEC-09 semantics:

| Command | Contract |
|---|---|
| `next ai model-pack validate <manifest>` | Read-only schema/hash/license/provenance/resource/adapter validation; no implicit download |
| `next ai model-pack install <manifest> --store <dir>` | Explicit network/write operation, staging + atomic publish, stable JSON/diagnostics |
| `next ai model-pack list [--role <role>]` | Installed immutable revisions, compatibility and health; no credentials |
| `next ai benchmark dialogue --pack <manifest> --profile <profile> --out <dir>` | Exact corpus/hardware/config run producing metrics and fault results |

Base package manifest references no required generative pack. Project MAY require a local pack only if startup has an explicit authored `TextOnlyFallback`; invalid required project policy fails before world mutation rather than silently enabling cloud.

## Persistence и replay

- Replay records finalized player `CanonicalUtterance` as ordered external input and accepted WorldCommand stream. It MAY record accepted NPC canonical text as oracle/presentation input.
- Replay MUST NOT call ASR, dialogue model, TTS or remote provider again. It consumes recorded canonical turn text and optional exact canonical PCM input.
- Save stores authoritative RPG dialogue/session/commitment/transcript-policy state and allowed canonical transcript/provenance. It MUST NOT store weights, provider threads, credentials or TTS playback position as gameplay authority.
- Durable transcript/memory record contains exact admitted text, locale, participant/turn IDs, source fact/event IDs and pack/provider provenance only when transcript/privacy policy permits.
- Headless/game/capture authoritative hashes ignore PCM and device playback. Commands/events/final gameplay hashes remain exact.

## Security, privacy и voice consent

- Microphone capture MUST be explicit, visible and bounded to the active input session. Raw capture is not retained by default.
- Network transmission, provider, data categories, region and retention MUST be disclosed before `RemoteOptIn`; consent is granular and revocable.
- `VoiceConsentRecord` binds speaker/rights-holder identity class, allowed project/purpose, source recording hashes, pack/provider, jurisdictions/expiry, revocation/deletion policy and rights-record reference. It contains no raw voice bytes.
- Missing, expired, revoked or scope-mismatched voice consent MUST disable cloning and use an authored/default licensed voice or subtitles.
- Prompts, canonical dialogue, raw/reference voice and provider responses remain default-redacted in logs and diagnostics.
- Model parser/operator set, pack installer and provider adapter follow the SPEC-11 threat model.

## Stable diagnostics и failure semantics

| Code / failure | Required outcome |
|---|---|
| `AI_MODEL_HASH_MISMATCH` | Reject/quarantine pack before activation; retain previous pack/text fallback |
| `AI_MODEL_LICENSE_UNCLASSIFIED` | Reject install/publish; no warning-only path |
| `AI_MODEL_INCOMPATIBLE` | Reject role/locale/resource/adapter route; select next declared route |
| `AI_PROVIDER_CONSENT_REQUIRED` | No network request; local/text fallback |
| `AI_TURN_SCHEMA_INVALID` | Discard candidate; no partial commit; authored fallback |
| `AI_DIALOGUE_DEADLINE_EXCEEDED` | Discard late response; staged fallback no later than one gameplay tick after deadline signal |
| `AI_STREAM_SEQUENCE_INVALID` | Cancel/reject affected presentation stream; canonical text/gameplay remain valid |
| `AI_TURN_CANCELLED` | Discard uncommitted work; dedupe any late result |
| `AI_VOICE_CONSENT_INVALID` | Disable cloning/reference voice; default licensed voice/subtitle |
| `NONDETERMINISTIC_RESULT` | Replay or product check fails; no retry-to-green or model regeneration |

Process absence/crash/protocol mismatch, missing pack, model OOM, malformed/partial stream, remote offline, consent revocation and late TTS all degrade to the next declared route. No failure may block tick, mutate state directly, partially commit a turn or repeat a committed command.

## Product checks

Wall timings are measurements on declared hardware, never simulation decisions.

| Check ID | Scenario / command | Expected behavior / fallback |
|---|---|---|
| `DIALOGUE-P1` | `next check DIALOGUE-P1 --scenario dialogue-text-audio-parity --turns 1000` | Typed and final-ASR turns produce the same mandatory outcomes with no direct mutation, partial commit or duplicate command; invalid/stale candidates reject, otherwise use authored dialogue and subtitles. |
| `DIALOGUE-P2` | `next check DIALOGUE-P2 --scenario dialogue-stream-faults --injections 1000` | Timeout, restart, cancellation, barge-in, reordering, duplicate and late results never crash or stall the game; select `TextOnlyFallback` within one gameplay tick after the deadline signal. |
| `MODEL-ASR-P1` | `next ai benchmark dialogue --role asr --corpus russian-gameplay-v1 --profile $PROFILE` | Clean Russian WER ≤10%, noisy mix WER ≤20%, final result p95 ≤600 ms, and invalid audio is bounded/rejected; use the next ASR route or typed input. |
| `MODEL-DIALOGUE-P1` | `next ai benchmark dialogue --role dialogue --corpus npc-dialogue-v1 --profile $PROFILE` | First valid sentence p95 ≤1,000 ms, complete candidate p95 ≤2,000 ms, schema validity ≥99%, and stale/forbidden mutations always reject; use the next route or authored dialogue. |
| `MODEL-TTS-P1` | `next ai benchmark dialogue --role tts --corpus russian-voice-v1 --profile $PROFILE` | First PCM p95 ≤500 ms, real-time factor ≤0.5, continuation is bounded, and audio schema limits hold; use the next TTS route, authored voice or subtitles. |
| `MODEL-E2E-P1` | `next ai benchmark dialogue --pack <manifest> --profile $PROFILE --corpus npc-dialogue-v1` | At concurrency 4, first subtitle p95 ≤1,500 ms, first PCM p95 ≤2,500 ms, and gameplay hashes match text-only; use a lower profile or `TextOnlyFallback`. |
| `MODEL-L1` | `next check MODEL-L1 --manifest <pack> --corpus model-license-provenance` | Files, sources, conversions, licenses and redistribution terms are classified and hash-closed; otherwise do not install or distribute the pack. |
| `VOICE-L1` | `next check VOICE-L1 --manifest <pack> --corpus voice-consent-revocation` | Unauthorized, expired, revoked or scope-mismatched cloning is rejected and raw reference voice never enters default logs; use a licensed voice or subtitles. |

## Requirements

| ID | Requirement |
|---|---|
| `REQ-079` | Typed input and finalized ASR use one canonical utterance/session path. |
| `REQ-080` | The engine owns a bounded request/candidate protocol and deterministic per-role routing. |
| `REQ-081` | Generated speech that claims a gameplay change waits for successful `WorldCommand` commit. |
| `REQ-082` | Speech streaming is ordered, cancelable and presentation-only; barge-in never changes committed gameplay. |
| `REQ-083` | Model packs, allowed transcripts and replay inputs are content-addressed; replay never regenerates model output. |
| `REQ-084` | Model-pack validate/install/list/benchmark commands use stable CLI/JSON contracts and atomic publication. |
| `REQ-085` | Remote providers are disclosed, consent-bound and default-off; missing consent fails closed to a local/text route. |
| `REQ-086` | Voice cloning requires valid scoped consent and always has a licensed voice/subtitle fallback. |

## Failure paths

| ID | Trigger | Required result |
|---|---|---|
| `FAIL-025` | ASR/model absent, timed out, crashed or malformed | Select the bounded text fallback without blocking a tick. |
| `FAIL-026` | Stale/invalid intent or gameplay commitment | Commit nothing and present a truthful authored response. |
| `FAIL-027` | Late, reordered or duplicate TTS segment | Cancel the affected audio while preserving the canonical turn. |
| `FAIL-028` | Invalid, hash-mismatched or license-incompatible pack | Quarantine it and retain the prior active revision. |
| `FAIL-029` | Network unavailable or consent revoked | Send no request and use the declared local/text route. |
| `FAIL-030` | `ai-host` restart or late result | Deduplicate by identity and never repeat a committed command. |

`narrative-director` является compatibility role для optional future model-pack
route из [SPEC-31](31-autonomous-quest-lifecycle-and-narrative-director.md).
SPEC-31 владеет request/candidate/validation/fallback semantics и не зависит от
всего speech/dialogue stack. Pack с этой role остаётся Proposed и
optional; отсутствие совместимого pack выбирает `TemplateNarrativeDirector`, а
не блокирует мир и не меняет mandatory outcome.

Authored gods from Accepted ADR-031 are role instances under that same
compatibility role, not new mandatory model-pack kinds. Adapter routing MAY use
different persona/system context per god, but request identity, epistemic fact
ceiling, fixed decision boundary, output schema and whole-candidate per-god
fallback remain engine-owned. One pack call
cannot return an authoritative multi-god council verdict or bypass atomic
pantheon resolution.

Lifecycle остаётся `Deferred Proposed`; эти contracts не выбирают shipping model и не меняют Accepted runtime.
