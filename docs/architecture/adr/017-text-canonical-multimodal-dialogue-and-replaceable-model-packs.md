# ADR-017: Text-canonical multimodal dialogue и replaceable model packs

| Поле | Значение |
|---|---|
| ID | ADR-017 |
| Статус | Proposed |
| Версия | 0.1 |
| Владелец | Agent Intelligence Team |
| Требуемые согласующие | Architecture Working Group, RPG Framework Team, World Services Team, Asset & Persistence Team, Developer Experience Team, Security & Governance Team, Verification & Evidence Team |
| Дата предложения | 2026-07-22 |
| Последняя проверка | 2026-07-22 |
| Нормативные зависимости | [SPEC-16](../16-text-canonical-multimodal-dialogue-and-model-packs.md), [SPEC-01](../01-system-architecture.md), [SPEC-06](../06-ai-agents-perception-and-memory.md), [SPEC-07](../07-rpg-scripting-and-plugins.md), [SPEC-08](../08-audio-navigation-and-world-services.md), [SPEC-11](../11-security-licensing-and-governance.md), [ADR-005](005-offline-first-ai-process-boundary.md), [ADR-007](007-identities-persistence-and-replay.md), [ADR-010](010-artifact-first-headless-validation-and-review.md) |
| Заменяет | отсутствует |
| Заменён | не заменён |

## Статус предложения

Этот ADR является отдельным post-1.5 review track. Он не меняет Accepted packet 1.4, не входит в remediation candidate 1.5 и не объявляет ни один model/provider Accepted. ADR-005 остаётся authoritative process boundary до и после возможного принятия ADR-017.

## Контекст

Accepted architecture уже требует optional `ai-host`, deterministic dialogue fallback и validation `AgentIntent`, но оставляет открытыми вопросы:

- являются ли audio и text разными gameplay paths;
- можно ли показывать/озвучивать streaming sentence до authoritative command outcome;
- как model/provider replacement связывается с hashes, licenses, resources и fallback;
- что сохраняется в save/replay и можно ли повторно вызывать model;
- как remote service и voice cloning получают consent;
- какие latency/quality/security gates отделяют model-card claim от Next Engine evidence.

Skyrim AI projects подтверждают практическую ценность role-separated ASR/LLM/TTS, per-role routing, sentence streaming, prewarm и hash-verified downloads. Их in-process DLL/direct-action approaches не соответствуют Next Engine isolation и mutation boundaries.

## Решение

При принятии:

1. **Text is canonical.** Typed input и finalized ASR создают один `CanonicalUtterance`. Audio, partial transcript, model tokens и PCM остаются ephemeral/presentation data.
2. **Per-role replacement.** ASR, dialogue, TTS, embeddings и audio understanding выбираются отдельно детерминированным `DialogueCapabilityProfile`; end-to-end vendor lock-in не является public contract.
3. **Isolated execution.** Все generative/speech roles остаются в optional `ai-host` по ADR-005. In-process generative model запрещён; small motor-policy exception ADR-005 к dialogue не применяется.
4. **Untrusted candidate.** Provider tool/function output может стать только declared `AgentIntent`. Gameplay state меняется исключительно после common validator и atomic WorldCommand commit.
5. **Truthful streaming.** Validated presentation-only sentence можно показывать/озвучивать сразу. Quest/trade/relationship/inventory commitment удерживается до corresponding command commit; rejected proposal получает authored response.
6. **Replay does not regenerate.** Replay использует recorded canonical utterances, accepted commands and exact PCM root when audio evidence is required. Provider/model is never called during replay/capture.
7. **Optional packs.** Base game ships no generative weights. Local models arrive as separately installed immutable `AiModelPackManifest` closures with hash, license, provenance, compatibility, resources, evidence and fallback.
8. **Remote is opt-in.** `AiProviderProfile` declares data categories, retention, region, terms/pricing snapshot and consent revision; credentials remain external. Revocation selects local/text fallback.
9. **Voice cloning is denied by default.** Exact scoped consent/provenance must pass `VOICE-L1`; otherwise use licensed default voice or subtitles.
10. **Models remain hypotheses.** Every exact model artifact stays `Proposed` until role, end-to-end, license and consent gates pass on declared profiles. Protocol acceptance never auto-accepts a model.

## Рассмотренные варианты

- **Separate text and voice gameplay paths** — Rejected: diverging semantics, duplicate quest handling and replay ambiguity. Both converge at `CanonicalUtterance`.
- **Audio-native end-to-end model as authority** — Rejected: hides intermediate text, weakens validation/replay and creates vendor lock-in. It MAY be an adapter only if it emits the same engine-owned contracts.
- **In-process generative runtime like native Skyrim DLL integrations** — Rejected by ADR-005: crash/memory/network isolation and optional fallback are mandatory.
- **Direct LLM actions/tool calls** — Rejected: capability/rule/freshness/transaction validation cannot be delegated to a model.
- **Cloud-required quality baseline** — Rejected: account, pricing, network and retention cannot determine offline correctness.
- **Bundled universal model in base game** — Rejected for v1: hardware, language, license and distribution envelopes vary; optional packs preserve replacement and small base distribution.
- **Regenerate dialogue during replay** — Rejected: model/provider drift makes past behavior unrecoverable.
- **Start TTS only after complete turn** — Rejected as universal rule: sentence streaming reduces perceived latency. Stateful sentences still wait for commit.

## Consequences

- Public contracts gain turn, pack, provider, capability, stream and consent values defined by SPEC-16.
- RPG Framework remains authoritative for dialogue/commitments; Agent Intelligence owns proposals/routing; World Services owns audio presentation; Asset/Persistence owns immutable model catalog/replay references.
- Tooling must eventually provide atomic validate/install/list/benchmark operations and structured diagnostics.
- Project authors must provide authored text fallback for every required dialogue outcome.
- Model/voice changes resolve `audio` and possibly narrative impact; automatic evidence precedes human review.
- Local quality may vary by pack/hardware, but mandatory gameplay outcome remains identical.

## Gates и fallback

The acceptance contract is the complete `DIALOGUE-P1/P2`, `MODEL-ASR-P1`, `MODEL-DIALOGUE-P1`, `MODEL-TTS-P1`, `MODEL-E2E-P1`, `MODEL-L1` and `VOICE-L1` matrix in SPEC-16. A candidate failing any applicable gate remains `Proposed` or becomes `Rejected`; fallback is the next declared role route and ultimately `TextOnlyFallback`. Thresholds are not weakened to promote a preferred vendor.

ADR acceptance itself requires documentation/schema review, ownership/security approval and exact promotion closure; it does not claim model runtime PASS, Windows/Linux shipping evidence or `vertical-v1` conformance.

## Promotion и supersession

Approval is one atomic transaction described by SPEC-16: synchronize SPEC-01/03/06/07/08/09/11/12/15, glossary, traceability and evidence references; add reserved REQ-079…086 and FAIL-025…030; preserve fifteen VS gates and map dialogue coverage into VS-03/04/12/15.

ADR-017 does not supersede ADR-005, ADR-007 or ADR-010. A future decision allowing mandatory cloud, in-process generative dialogue, direct model mutation, non-text authoritative dialogue or model regeneration during replay requires a new superseding ADR.
