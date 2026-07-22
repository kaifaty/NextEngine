# SPEC-16: Text-canonical multimodal dialogue и model packs

| Поле | Значение |
|---|---|
| ID | SPEC-16 |
| Статус | Proposed |
| Версия | 0.1 |
| Владелец | Agent Intelligence Team |
| Требуемые согласующие | Architecture Working Group, RPG Framework Team, World Services Team, Asset & Persistence Team, Developer Experience Team, Security & Governance Team, Verification & Evidence Team |
| Последняя проверка | 2026-07-22 |
| Нормативные зависимости | [SPEC-01](01-system-architecture.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-06](06-ai-agents-perception-and-memory.md), [SPEC-07](07-rpg-scripting-and-plugins.md), [SPEC-08](08-audio-navigation-and-world-services.md), [SPEC-09](09-tooling-sdk-and-observability.md), [SPEC-11](11-security-licensing-and-governance.md), [SPEC-12](12-vertical-slice-conformance.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [ADR-005](adr/005-offline-first-ai-process-boundary.md), [ADR-007](adr/007-identities-persistence-and-replay.md), [ADR-010](adr/010-artifact-first-headless-validation-and-review.md) |
| Заменяет | отсутствует |

## Статус предложения

Этот документ является post-1.5 review proposal и не меняет Accepted packet 1.4 или remediation candidate 1.5. Термины MUST/MUST NOT задают future acceptance contract только для promotion bundle [ADR-017](adr/017-text-canonical-multimodal-dialogue-and-replaceable-model-packs.md). Ни одна модель из [RESEARCH-002](research/npc-dialogue-model-landscape.md) не является Accepted или shipping default.

## Назначение и invariants

SPEC-16 задаёт один dialogue-turn contract для typed text и microphone audio, streaming presentation, per-role model routing, optional local packs и explicit remote providers.

- UTF-8 NFC text MUST быть единственным каноническим содержанием dialogue turn.
- Typed input и finalized ASR MUST создавать один и тот же `CanonicalUtterance`; partial ASR не является gameplay input.
- LLM, embeddings, ASR, TTS и audio-understanding MUST исполняться только в optional separate `ai-host` по ADR-005.
- Model output MUST оставаться untrusted candidate. RPG/world mutation проходит `AgentIntent → validator → WorldCommand → commit`.
- Simulation tick MUST NOT ждать ASR/LLM/TTS. Deadline/cancellation приходят как staged external signals и записываются в verification/replay input, а не вычисляются из wall clock при replay.
- Base game MUST NOT включать generative weights. Authored dialogue, subtitles и deterministic planner являются обязательным `TextOnlyFallback`.
- Local packs MUST быть immutable, content-addressed, separately installed и reconstructible; cloud MUST быть default-off и explicit opt-in.
- Vendor types, raw filesystem paths, credentials, provider session IDs, tokenizer objects и audio-device handles MUST NOT входить в public contracts.

## Source of truth и ownership

| State | Единственный owner/source of truth | Не является source |
|---|---|---|
| Dialogue session, choices, commitments, transcript policy | RPG Framework `Dialogue` aggregate | Prompt, provider thread, TTS playback |
| Canonical player/NPC utterance admitted into session | RPG Framework after turn validation | Partial ASR, token stream, waveform |
| Perception, memory projection, high-level proposal | Agent Intelligence / Memory Service per SPEC-06 | Vendor cache/session |
| Model catalog, installed immutable artifact resolution | Asset & Tool chain model-pack registry | Downloader temp path, provider model list |
| Generated PCM/playback progress | World Services presentation/audio runtime | Dialogue state or replay gameplay hash |
| Pack/provider grants, voice consent and licensing | Project policy + Security & Governance records | Model card claim, local UI toggle alone |
| Gameplay mutation | Owning domain after committed WorldCommand | Dialogue text, function/tool call, speech segment |

Agent Intelligence Team owns the turn protocol and deterministic routing policy. RPG Framework owns admission into authoritative dialogue state. World Services owns audio capture/playback streams. Asset & Persistence Team owns pack manifests, installation transaction and replay references. Developer Experience owns CLI/diagnostics. Security & Governance owns network, data, license and voice-consent policy.

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
- project MAY choose lower limits but cannot raise them without a versioned profile and repeated security/performance gates.

Selection of facts/memories/affordances MUST be deterministic for the same snapshot/profile: stable relevance key, then stable ID. The request cannot contain save bytes, raw prompt files, arbitrary tool definitions, filesystem paths or credentials.

### DialogueTurnCandidate

`DialogueTurnCandidate` contains request/turn/idempotency IDs, canonical response text, sentence boundaries, optional bounded `AgentIntent`, memory proposals, cited fact/revision IDs, completion state, selected role/pack/provider IDs, model/artifact/content hashes, generation parameters, adapter protocol version and redaction classification.

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
| Roles | One or more closed roles: `asr`, `dialogue`, `tts`, `embeddings`, `audio-understanding` |
| Artifacts | Engine `AssetId`, SHA-256, byte size, media/model format, quantization, adapter ID/protocol range; no runtime path |
| Compatibility | locales, input/output schemas, context/audio limits, streaming/timestamp/structured-output/voice-clone capabilities |
| Resources | supported target triples, CPU features, RAM/VRAM/disk envelope, declared concurrency and warm-up policy |
| Provenance | upstream source URL + immutable revision, build/conversion tool hashes, parent artifact hashes |
| Licensing | code/weights/data/voice SPDX or reviewed classification, redistribution scope, notices and exception reference |
| Evidence | gate/run IDs and hashes; self-reported metrics stored separately from Next Engine results |
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
| `next ai model-pack list [--role <role>]` | Installed immutable revisions, compatibility/evidence/health; no credentials |
| `next ai benchmark dialogue --pack <manifest> --profile <profile> --artifact-root <dir>` | Exact corpus/hardware/config run producing RunManifest, metrics and faults |

Base package manifest references no required generative pack. Project MAY require a local pack only if startup has an explicit authored `TextOnlyFallback`; invalid required project policy fails before world mutation rather than silently enabling cloud.

## Persistence, replay и evidence

- Replay records finalized player `CanonicalUtterance` as ordered external input and accepted WorldCommand stream. It MAY record accepted NPC canonical text as oracle/presentation input.
- Replay/capture MUST NOT call ASR, dialogue model, TTS or remote provider again. It consumes recorded canonical turn text and, where audio review is required, exact canonical PCM root.
- Save stores authoritative RPG dialogue/session/commitment/transcript-policy state and allowed canonical transcript/provenance. It MUST NOT store weights, provider threads, credentials or TTS playback position as gameplay authority.
- Durable transcript/memory record contains exact admitted text, locale, participant/turn IDs, source fact/event IDs and pack/provider provenance only when transcript/privacy policy permits.
- Headless/game/capture authoritative hashes ignore PCM and device playback. Commands/events/final gameplay hashes remain exact.
- Dialogue/model/voice changes are `audio` and often narrative-observable impact; ImpactResolver maps them to required automatic gates and HumanReviewRequired evidence under SPEC-15.

## Security, privacy и voice consent

- Microphone capture MUST be explicit, visible and bounded to the active input session. Raw capture is not retained by default.
- Network transmission, provider, data categories, region and retention MUST be disclosed before `RemoteOptIn`; consent is granular and revocable.
- `VoiceConsentRecord` binds speaker/rights-holder identity class, allowed project/purpose, source recording hashes, pack/provider, jurisdictions/expiry, revocation/deletion policy and human/legal evidence reference. It contains no raw voice bytes.
- Missing, expired, revoked or scope-mismatched voice consent MUST disable cloning and use an authored/default licensed voice or subtitles.
- Prompts, canonical dialogue, raw/reference voice and provider responses remain default-redacted in logs/evidence. Required qualitative review uses bounded classified samples approved by project policy.
- Model parser/operator set, pack installer and provider adapter require SPEC-11 threat review before implementation.

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
| `NONDETERMINISTIC_RESULT` | Replay/gate fails; no retry-to-green or model regeneration |

Process absence/crash/protocol mismatch, missing pack, model OOM, malformed/partial stream, remote offline, consent revocation and late TTS all degrade to the next declared route. No failure may block tick, mutate state directly, partially commit a turn or repeat a committed command.

## Proposed gates

All timings are measured wall performance on exact declared hardware but are not simulation decisions. Warm-up, corpus, pack files, adapter/build/config and hardware hashes MUST appear in RunManifest.

| Gate | Owner | Reproducible command/scenario | Pass threshold | Required evidence | Fallback |
|---|---|---|---|---|---|
| `DIALOGUE-P1` | Agent Intelligence + RPG Framework | `next gate DIALOGUE-P1 --scenario dialogue-text-audio-parity --turns 1000` | 1,000 scripted typed/final-ASR turns; exact mandatory outcomes; 0 direct mutation, partial commit or duplicate command; 100% invalid/stale schema rejected | canonical input/candidate corpus, command/event/rejection trace, replay hashes | authored deterministic dialogue + subtitles |
| `DIALOGUE-P2` | Agent Intelligence + World Services | `next gate DIALOGUE-P2 --scenario dialogue-stream-faults --injections 1000` | 1,000 timeout/restart/cancel/barge-in/reorder/duplicate/late injections; 0 game crash/tick stall/partial commit; fallback selected ≤1 gameplay tick after deadline signal | ai-host/stream timeline, fault report, replay/dedupe ledger | circuit-break adapter; `TextOnlyFallback` |
| `MODEL-ASR-P1` | Agent Intelligence | `next ai benchmark dialogue --role asr --corpus russian-gameplay-v1 --profile $PROFILE` | Russian clean WER ≤10%; noisy gameplay-mix WER ≤20%; end-of-utterance→final p95 ≤600 ms; 100% invalid audio bounded/rejected | corpus/provenance, transcripts, WER/latency/resource report | next ASR route or typed input |
| `MODEL-DIALOGUE-P1` | Agent Intelligence + RPG Framework | `next ai benchmark dialogue --role dialogue --corpus npc-dialogue-v1 --profile $PROFILE` | first schema-valid sentence p95 ≤1,000 ms; complete candidate p95 ≤2,000 ms; first-attempt schema validity ≥99%; 100% stale/forbidden mutations rejected | prompts redacted, canonical request/response hashes, validator/persona/commitment metrics | next LLM route or authored dialogue |
| `MODEL-TTS-P1` | World Services | `next ai benchmark dialogue --role tts --corpus russian-voice-v1 --profile $PROFILE` | first PCM p95 ≤500 ms; real-time factor ≤0.5; 0 unbounded continuation across corpus; schema/audio bounds 100% | text/audio roots, latency/RTF/duration/pronunciation report, required HumanReviewDecision | next TTS route or subtitles/authored voice |
| `MODEL-E2E-P1` | Agent Intelligence + World Services | `next ai benchmark dialogue --pack <manifest> --profile $PROFILE --corpus npc-dialogue-v1` | 1,000 warm turns at concurrency 4; end-of-player-utterance→first subtitle p95 ≤1,500 ms and →first PCM p95 ≤2,500 ms; 0 gameplay hash differences with text-only run | full pipeline timeline, resource/queue metrics, replay/audio roots | lower profile or `TextOnlyFallback` |
| `MODEL-L1` | Security & Governance | `next gate MODEL-L1 --manifest <pack> --corpus model-license-provenance` | 100% files/source/conversions/license/redistribution classified and hash-closed; unknown/custom terms without exception rejected | manifest closure, SBOM/notices, provenance graph, legal exceptions | do not install/distribute pack |
| `VOICE-L1` | Security & Governance + World Services | `next gate VOICE-L1 --manifest <pack> --corpus voice-consent-revocation` | 100% unauthorized/expired/revoked/scope-mismatched clones rejected; 0 raw reference voice in default logs/evidence; deletion/revocation fixtures pass | consent records, denial/audit/redaction/deletion report | licensed default voice or subtitles |

Human listening cannot waive automatic waveform, bounds, command/replay, license or consent failure. Missing reviewer/encoder gives `AwaitingCapability`, never `PASS`.

## Promotion contract

Promotion requires one reviewed transaction after required owner/Security approvals:

1. ADR-017 and SPEC-16 become `Accepted`; ADR-005 remains Accepted and is not superseded.
2. SPEC-01/03/06/07/08/09/11/12/15 and glossary receive the accepted contracts without changing ownership or fifteen-gate count.
3. Traceability adds exactly the reserved rows below and maps child gates into VS-03, VS-04, VS-12 and VS-15.
4. Evidence register rows remain `Proposed` until their exact artifacts independently pass all applicable gates; accepting the protocol does not accept a model.

| Reserved row | Primary owner | Future acceptance contract |
|---|---|---|
| `REQ-079` | RPG Framework | Typed/final-ASR parity through one canonical utterance/session path |
| `REQ-080` | Agent Intelligence | Engine-owned bounded request/candidate protocol and per-role routing |
| `REQ-081` | RPG Framework | Stateful generated speech waits for successful WorldCommand commit |
| `REQ-082` | World Services | Ordered/cancelable presentation-only speech streaming and barge-in |
| `REQ-083` | Asset & Persistence | Model packs, transcripts and replay are content-addressed; replay never regenerates |
| `REQ-084` | Developer Experience | Atomic model-pack validation/install/list/benchmark CLI/JSON |
| `REQ-085` | Security & Governance | Remote provider disclosure/consent/default-off and fail-closed licensing |
| `REQ-086` | Security & Governance | Voice-clone consent/revocation/redaction and licensed fallback |
| `FAIL-025` | Agent Intelligence | ASR/model absent, timeout, crash or malformed candidate → bounded text fallback |
| `FAIL-026` | RPG Framework | Stale/invalid intent or commitment → no partial commit and truthful authored response |
| `FAIL-027` | World Services | Late/reordered/duplicate TTS stream → cancel audio, preserve canonical turn |
| `FAIL-028` | Asset & Persistence | Invalid/hash/license-incompatible pack → quarantine, retain prior revision |
| `FAIL-029` | Security & Governance | Network unavailable or consent revoked → no request, local/text fallback |
| `FAIL-030` | Agent Intelligence | ai-host restart/late result → idempotent dedupe, no duplicate command |

Until this transaction is approved, lifecycle state is `AwaitingReview`; implementation may only be an explicitly non-conforming PoC outside Accepted claims.
