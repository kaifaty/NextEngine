# RESEARCH-002: Модели и практики text/audio NPC dialogue

| Поле | Значение |
|---|---|
| ID | RESEARCH-002 |
| Статус | Draft |
| Версия | 0.1 |
| Владелец | Agent Intelligence Team |
| Последняя проверка | 2026-07-22 |
| Нормативные зависимости | отсутствуют |
| Заменяет | отсутствует |

> Этот документ ненормативен. Он фиксирует внешний research snapshot и рекомендации для [SPEC-16](../16-text-canonical-multimodal-dialogue-and-model-packs.md) и [ADR-017](../adr/017-text-canonical-multimodal-dialogue-and-replaceable-model-packs.md), но не принимает technology backend и не заявляет прохождение Next Engine gates.

## Вопрос исследования

Для разговора с NPC нужны два равноправных входа — typed text и microphone audio — и один заменяемый pipeline `ASR → dialogue model → TTS`. Исследование сравнивает open-weight/local и commercial/cloud варианты по русскому языку, multilingual coverage, streaming, resource envelope, лицензированию и пригодности к optional isolated `ai-host` из [ADR-005](../adr/005-offline-first-ai-process-boundary.md).

Слова `быстрый`, `realtime`, WER и latency ниже означают заявления upstream на его corpus/hardware, если явно не сказано иначе. `Next Engine result` для всех кандидатов остаётся `NotRun`; self-reported result не закрывает gate.

## Open-weight и локальные кандидаты

### ASR

| Exact candidate/revision | License | Languages и streaming | Upstream факт на 2026-07-22 | Пригодность | Ограничение / Next Engine result |
|---|---|---|---|---|---|
| [`ai-sage/GigaAM-v3`, revision `e2e_rnnt`](https://huggingface.co/ai-sage/GigaAM-v3) | MIT | Russian; RNN-T; 16 kHz; punctuation/capitalization in end-to-end variants | 220M-class Conformer/RNN-T family; upstream публикует CTC/RNN-T и export paths | Первый кандидат `RussianLocal` из-за русского языка и bounded size | Runtime/latency/WER на gameplay corpus не измерены; exact commit и model-file SHA-256 выбираются только в PoC manifest |
| [`ai-sage/GigaAM-Multilingual`, revisions `ctc`/`large_ctc`](https://huggingface.co/ai-sage/GigaAM-Multilingual) | MIT | RU/KZ/KG/UZ/EN output; 220M/600M; CTC | Pretraining 2M hours across 70+ languages; upstream reports Russian FLEURS WER 4.4%/3.0% | Новый Sber-кандидат для `MultilingualLocal`, особенно Central Asia | Не realtime baseline: CTC deployment и endpointing обязаны отдельно пройти `MODEL-ASR-P1`; result `NotRun` |
| [`Qwen/Qwen3-ASR-0.6B`](https://huggingface.co/Qwen/Qwen3-ASR-0.6B) | Apache-2.0 | 52 languages/dialects; offline и streaming tooling | Official model card reports multilingual language identification and ASR | Главный non-Sber comparator для `MultilingualLocal` | Russian gameplay WER, memory и first-final latency не измерены; result `NotRun` |
| [`mistralai/Voxtral-Mini-4B-Realtime-2602`](https://huggingface.co/mistralai/Voxtral-Mini-4B-Realtime-2602) | Apache-2.0 | 13 languages including Russian; native streaming | Configurable delay 80 ms…2.4 s; recommended 480 ms; BF16 deployment requires one GPU with at least 16 GiB according to model card | `HighEndStreaming` comparator | Слишком тяжёл для baseline consumer profile; exact 16 GiB+ GPU profile и Russian gameplay corpus не проверены; result `NotRun` |

### Dialogue LLM

| Exact candidate/revision | License | Size/deployment | Upstream capability | Пригодность | Ограничение / Next Engine result |
|---|---|---|---|---|---|
| [`ai-sage/GigaChat3.1-10B-A1.8B`](https://huggingface.co/ai-sage/GigaChat3.1-10B-A1.8B), GGUF Q4 candidate | MIT | MoE 10B total / 1.8B active; FP8/BF16/GGUF variants | Multilingual dialogue, structured output and function calling are declared by upstream | Первый Russian-local dialogue candidate | Function call остаётся только untrusted `AgentIntent`; selected GGUF file, checksum, RAM, tokens/s и schema-valid latency должны быть pinned/измерены; result `NotRun` |
| [`Qwen/Qwen3.5-4B`](https://huggingface.co/Qwen/Qwen3.5-4B) | Apache-2.0 | 4B-class multimodal model; Transformers/vLLM/SGLang | Multilingual instruction/dialogue model with published local serving paths | Compact multilingual comparator | Не принимается только по general benchmark; Russian NPC consistency, JSON/schema compliance and p95 latency `NotRun` |
| [`ai-sage/GigaChat3.5-432B-A28B`](https://huggingface.co/ai-sage/GigaChat3.5-432B-A28B) | MIT | 432B total / 28B active; server-class multi-GPU | Large Sber quality/reference model | Offline benchmark oracle или remote server experiment | Не является downloadable player profile и не включается в Proposed shipping candidate rows |

### TTS

| Exact candidate/revision | License | Languages/streaming | Upstream факт | Пригодность | Ограничение / Next Engine result |
|---|---|---|---|---|---|
| [`Qwen/Qwen3-TTS-12Hz-0.6B-Base`](https://huggingface.co/Qwen/Qwen3-TTS-12Hz-0.6B-Base) | Apache-2.0 | 10 languages including Russian; streaming; reference-audio cloning | Upstream claims first-packet latency down to 97 ms in its setup and 3-second voice cloning | Первый `RussianLocal`/`MultilingualLocal` TTS PoC | Upstream latency is not portable evidence; pronunciation, bounded continuation, VRAM and consent path `NotRun` |
| [`ResembleAI/Chatterbox-Multilingual V3`](https://github.com/resemble-ai/chatterbox) | MIT | 23+ languages including Russian; 500M; cross-language cloning | Upstream describes improved speaker similarity and fewer unintended continuations than prior multilingual release | Expressive multilingual comparator | Native mixed-language switching and latency require measurement; voice cloning disabled without `VOICE-L1`; result `NotRun` |
| [`ekwek/Soprano-1.1-80M`](https://github.com/ekwek1/soprano) | Apache-2.0 | English only; streaming; CPU/CUDA/MPS | Upstream reports 80M, under 1 GiB memory, under 250 ms CPU / 15 ms GPU streaming latency | Лучший lightweight English-only comparator | На 2026-07-22 нет Russian/multilingual и voice cloning; не рекомендуется для Russian-first pack; result `NotRun` |

### Audio-native understanding

[`ai-sage/GigaChat3.1-Audio-10B-A1.8B`](https://huggingface.co/ai-sage/GigaChat3.1-Audio-10B-A1.8B) имеет MIT license и принимает audio в том же 10B/1.8B-active family. Его следует исследовать только как optional `audio-understanding` role для emotion/time-aware interpretation. Он не генерирует TTS и не отменяет text-canonical boundary: downstream всё равно получает finalized `CanonicalUtterance` либо bounded non-authoritative annotation. Result — `NotRun`.

## Commercial/cloud snapshot

Cloud adapters не являются fallback для offline correctness. Они требуют `RemoteOptIn`, explicit data disclosure и revalidation terms/price до каждого release.

| Service | Роль | Проверенный факт на 2026-07-22 | Архитектурный вывод |
|---|---|---|---|
| [GigaChat API tariffs for individuals](https://developers.sber.ru/docs/ru/gigachat/tariffs/individual-tariffs) | Dialogue LLM | Документация от 2026-07-20 публикует 1,000,000 freemium text tokens на 12 месяцев и платные packages; endpoint lifecycle отдельно versioned | Полезный Russian `RemoteOptIn` adapter, но quotas, account и network исключают shipping baseline |
| [SaluteSpeech corporate tariffs](https://developers.sber.ru/docs/ru/salutespeech/tariffs/legal-tariffs) | ASR/TTS | Документация от 2026-03-05 публикует prepay/pay-as-you-go: 0.01 RUB/s ASR и 0.000186 RUB/character TTS, с отдельным corporate onboarding | Возможен enterprise comparator; цена, retention, region и terms входят в `AiProviderProfile`, не в gameplay contract |
| [ElevenLabs streaming TTS](https://elevenlabs.io/docs/api-reference/text-to-speech/stream) | TTS | Streaming API, multilingual models и voice IDs; некоторые formats/zero-retention features зависят от paid tier | Quality/latency comparator для `RemoteOptIn`; voice library/clone никогда не становится engine identity или authority |

Любой другой OpenAI-compatible endpoint может подключаться через ту же engine-owned provider boundary, но compatibility по HTTP shape не означает одинаковые retention, cancellation, schema или safety semantics.

## Что используют Skyrim AI projects

### Mantella

[`art-from-the-machine/Mantella`](https://github.com/art-from-the-machine/Mantella) использует явный `speech-to-text → LLM → text-to-speech` pipeline и заменяемые providers: Moonshine/Whisper для STT, OpenAI-compatible/local LLM endpoints, Piper/xVASynth/XTTS для TTS. Полезный вывод — role adapters и возможность вынести inference на другой host. Next Engine сохраняет более строгий engine-owned IPC и не переносит provider session как save truth.

### SkyrimNet

[`MinLL/SkyrimNet-GamePlugin`](https://github.com/MinLL/SkyrimNet-GamePlugin) документирует несколько важных latency patterns:

- разные LLM для action selection и dialogue;
- TTS первого предложения, пока генерируется второе;
- prewarm action eligibility во время ASR;
- async heavy work и отсутствие network wait на game thread;
- local/cloud TTS selection и SHA-verified model downloads.

Его topology — native DLL внутри Skyrim с прямым чтением game memory и расширяемыми actions — несовместима с ADR-005 и Next Engine ownership. Next Engine заимствует pipeline optimizations, но выполняет generative work только в optional `ai-host`, передаёт capability-filtered snapshots и не разрешает model-defined direct mutation.

### CHIM / HerikaServer

[`abeiro/HerikaServer`](https://github.com/abeiro/HerikaServer) является external bridge для STT/TTS/chat providers, long-term memory и function-call actions. Полезны provider separation и memory lifecycle. Произвольный function call не может стать world action: он маппится только в declared `AgentIntent`, затем проходит deterministic validator.

## Рекомендуемые profiles

| Profile | Routing order | Цель | Fallback |
|---|---|---|---|
| `RussianLocal` | GigaAM-v3 `e2e_rnnt` → GigaChat 3.1 Lightning GGUF Q4 → Qwen3-TTS 0.6B; Chatterbox V3 comparator | Первый reproducible PoC для Russian-first game | `TextOnlyFallback` |
| `MultilingualLocal` | GigaAM Multilingual or Qwen3-ASR → Qwen3.5-4B → Qwen3-TTS; Chatterbox V3 comparator | RU + additional project locales | `RussianLocal` when locale permits, otherwise `TextOnlyFallback` |
| `HighEndStreaming` | Voxtral Realtime ASR → project-selected dialogue pack → validated TTS pack | Проверка native streaming на declared 16 GiB+ GPU | `MultilingualLocal`/`TextOnlyFallback` |
| `RemoteOptIn` | Project/user-granted provider per role | Managed quality or low local compute | соответствующий local profile, затем `TextOnlyFallback` |
| `TextOnlyFallback` | Typed input or authored text recognizer → deterministic dialogue/rules → subtitles/authored voice | Mandatory offline correctness | отсутствует; это baseline |

## Рекомендация

1. Начать PoC с `RussianLocal`, но не включать weights в base distribution.
2. Сравнить GigaAM-v3 и новый GigaAM Multilingual на одном noisy gameplay corpus; multilingual release не принимать автоматически как realtime replacement.
3. Проверять LLM по first schema-valid sentence, complete candidate latency, Russian persona/commitment consistency и 100% rejection stale/forbidden mutations, а не по general leaderboard.
4. Проверять TTS по first PCM, real-time factor, pronunciation corpus, unintended continuation и licensed voice-consent lifecycle.
5. Сохранять per-role interchangeability. Audio-native model может оптимизировать отдельные стадии, но не создаёт end-to-end vendor authority.
6. Перенести candidates в [EVIDENCE-001](../evidence-register.md) только как `Proposed`; любой Accepted выбор потребует exact artifacts, Next Engine evidence и reviewed promotion.

## Revalidation checklist

- exact repository/model ID, immutable revision and every file SHA-256;
- code license отдельно от weights/data/voice license;
- supported languages, context/audio limits и actual output schema;
- Windows/Linux `ai-host` support, CPU/GPU/RAM/VRAM envelope;
- network, retention, region, account, quota и price snapshot для cloud;
- ASR/TTS voice data consent, revocation и deletion semantics;
- все `MODEL-*`, `DIALOGUE-*`, `MODEL-L1` и `VOICE-L1` artifacts.
