# ADR-017: Text-canonical multimodal dialogue and replaceable model packs

| Поле | Значение |
|---|---|
| ID | ADR-017 |
| Статус | Proposed |
| Lifecycle | Deferred Proposed |
| Версия | 0.4 |
| Дата предложения | 2026-07-22 |
| Последняя проверка | 2026-08-17 |
| Нормативные зависимости | [SPEC-16](../16-text-canonical-multimodal-dialogue-and-model-packs.md), [SPEC-36](../36-streaming-tts-and-spatial-speech-presentation.md), [SPEC-01](../01-system-architecture.md), [SPEC-06](../06-ai-agents-perception-and-memory.md), [SPEC-07](../07-rpg-scripting-and-plugins.md), [SPEC-08](../08-audio-navigation-and-world-services.md), [SPEC-11](../11-security-licensing-and-governance.md), [ADR-005](005-offline-first-ai-process-boundary.md), [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-030](030-product-first-development-and-lightweight-validation.md), [ADR-075](075-bounded-streaming-tts-through-ai-host-and-audio-scene.md) |
| Заменяет | отсутствует |
| Заменён | не заменён |

## Process baseline ADR-030

[ADR-030](030-product-first-development-and-lightweight-validation.md)
определяет обычный repository workflow и risk-based product checks. Technical
proposal и `Deferred Proposed` status этого ADR сохраняются; ни один
model/provider не становится default до отдельного принятого решения.

## Статус предложения

ADR остаётся `Deferred Proposed`. [ADR-005](005-offline-first-ai-process-boundary.md)
остаётся текущей authority для optional `ai-host`, process isolation и
deterministic in-process fallback. SPEC-16 подробно описывает candidate
contracts, но пока не превращает их в shipped baseline.

[ADR-075](075-bounded-streaming-tts-through-ai-host-and-audio-scene.md) и
[SPEC-36](../36-streaming-tts-and-spatial-speech-presentation.md) выделяют
implementation-ready bounded local TTS specialization. Они не меняют статус
этого ADR и не принимают ASR, LLM dialogue, remote providers либо весь
model-pack stack.

## Контекст

Accepted architecture уже требует optional `ai-host`, deterministic dialogue
fallback и validation `AgentIntent`, но оставляет открытыми вопросы:

- являются ли audio и text разными gameplay paths;
- можно ли показывать или озвучивать streaming sentence до authoritative
  command outcome;
- как model/provider replacement связывается с hashes, licenses, resources и
  fallback;
- что сохраняется в save/replay и можно ли повторно вызывать model;
- как remote service и voice cloning получают explicit opt-in;
- какие latency, quality и security checks отделяют model-card claims от
  наблюдаемого поведения Next Engine.

Role-separated ASR/LLM/TTS, per-role routing, sentence streaming, prewarm и
hash-verified downloads полезны как reference patterns. In-process
DLL/direct-action integrations не соответствуют Next Engine isolation и
mutation boundaries.

## Предлагаемое решение

1. **Text is canonical.** Typed input и finalized ASR создают один
   `CanonicalUtterance`. Audio, partial transcript, model tokens и PCM остаются
   ephemeral/presentation data.
2. **Per-role replacement.** ASR, dialogue, TTS, embeddings и audio
   understanding выбираются отдельно deterministic
   `DialogueCapabilityProfile`; end-to-end vendor lock-in не входит в public
   contract.
3. **Isolated execution.** Generative/speech roles остаются в optional
   `ai-host` по ADR-005. In-process generative model запрещён; small
   motor-policy exception ADR-005 к dialogue не применяется.
4. **Untrusted candidate.** Provider tool/function output становится только
   declared `AgentIntent`. Gameplay state меняется после common validator и
   atomic `WorldCommand` commit.
5. **Truthful streaming.** Validated presentation-only sentence можно показать
   или озвучить сразу. Quest, trade, relationship и inventory commitment
   удерживаются до corresponding command commit; rejected proposal получает
   authored response.
6. **Replay does not regenerate.** Replay использует recorded canonical
   utterances, accepted commands и recorded PCM root, когда session сохраняет
   exact audio. Provider/model во время replay не вызывается.
7. **Optional packs.** Base game не содержит generative weights. Local models
   устанавливаются отдельно как immutable `AiModelPackManifest` closure с hash,
   license, provenance, compatibility, resource limits и fallback.
8. **Remote is opt-in.** `AiProviderProfile` declares data categories,
   retention, region, terms/pricing snapshot и local opt-in revision;
   credentials остаются external. Revocation выбирает local/text fallback.
9. **Voice cloning is denied by default.** Exact scoped consent и provenance
   проверяются до use; иначе используются licensed default voice или subtitles.
10. **Models remain hypotheses.** Exact model artifact остаётся `Proposed`, пока
    role, end-to-end, license, consent и resource checks не пройдут на declared
    profiles. Принятие protocol не выбирает model автоматически.

## Рассмотренные варианты

- **Separate text and voice gameplay paths** — `Rejected`: diverging semantics,
  duplicate quest handling и replay ambiguity. Both converge at
  `CanonicalUtterance`.
- **Audio-native end-to-end model as authority** — `Rejected`: скрывает
  intermediate text, ослабляет validation/replay и создаёт vendor lock-in.
- **In-process generative runtime** — `Rejected` по ADR-005: crash, memory и
  network isolation должны сохраняться.
- **Direct LLM actions/tool calls** — `Rejected`: capability, rule, freshness и
  transaction validation нельзя делегировать model.
- **Cloud-required quality baseline** — `Rejected`: account, pricing, network и
  retention не определяют offline correctness.
- **Bundled universal model in base game** — `Rejected` для v1: hardware,
  language, license и distribution envelopes различаются.
- **Regenerate dialogue during replay** — `Rejected`: model/provider drift
  делает прошлое поведение невоспроизводимым.

## Product checks

| Сценарий | Ожидаемый результат | Fallback |
|---|---|---|
| Typed input и finalized ASR выражают одну semantic utterance | Оба пути создают одинаковый bounded `CanonicalUtterance`; replay использует recorded value и не вызывает provider | Authored text input и authored response |
| ASR, dialogue и TTS adapters заменяются по одной роли, включая timeout/crash/bad version | Engine-owned contracts и committed gameplay outcome сохраняются; failure изолирован от tick loop | Следующий declared local role adapter, затем `TextOnlyFallback` |
| Remote provider получает запрос после explicit local opt-in | Передаются только declared data categories; credentials не входят в manifest/log; offline gameplay остаётся рабочим | Local model либо authored text |
| Voice pack без exact license, provenance или scoped consent | Pack отвергается до playback/installation mutation | Licensed default voice или subtitles |
| Model pack превышает bounds, несовместим со schema/profile или меняет output при replay | Pack не выбирается; stable diagnostic указывает role, artifact и incompatible field | Следующий compatible immutable pack либо authored text |

## Последствия

- Public contracts при принятии получат turn, pack, provider, capability, stream
  и consent values из SPEC-16.
- RPG Framework сохраняет authority над dialogue commitments; Agent
  Intelligence создаёт proposals/routing; World Services воспроизводит audio;
  Asset/Persistence хранит immutable model catalog и replay references.
- Project authors предоставляют authored text fallback для каждого обязательного
  dialogue outcome.
- Base gameplay outcome не зависит от local model quality или hardware.

## Supersession

ADR-017 не заменяет ADR-005, ADR-022, ADR-030 или ADR-075. Принятие широкого proposal требует
короткого нового решения либо смены статуса с синхронным обновлением SPEC-16 и
lightweight traceability. Mandatory cloud, in-process generative dialogue,
direct model mutation, non-text authoritative dialogue или regeneration during
replay требуют отдельного superseding ADR.
