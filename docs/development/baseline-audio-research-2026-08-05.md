# Research: Baseline audio (roadmap queue #2, R2)

| Поле | Значение |
|---|---|
| Статус | Research note; рабочий материал, не normative architecture |
| Дата | 2026-08-05 |
| Контекст | Roadmap "Ближайшая implementation queue", пакет 2: Baseline audio (`PLANNED`) |

Этот документ фиксирует результат исследования перед реализацией пакета
**Baseline audio**: clips, emitters/listener, priority/voice limits,
attenuation/panning и subtitle fallback. Нормативной authority остаются
Accepted SPEC/ADR; при расхождении действует precedence из
[architecture/README](../architecture/README.md).

## 1. Scope пакета по roadmap

Roadmap R2 объявляет для этого increment: «baseline sample playback,
attenuation/panning, voice limiting и subtitle fallback». Очередь формулирует
пакет как: clips, emitters/listener, priority/voice limits,
attenuation/panning and subtitle fallback.

Явные non-goals (SPEC-08 + scope guard R2):

- Steam Audio propagation adapter (`Proposed`, отдельный track с fallback);
- ASR/TTS и voice services (принадлежат `ai-host`; отсутствие voice —
  text/subtitle fallback);
- navigation/world services из SPEC-08 (отдельные пакеты очереди 4–6);
- zone reverb сверх declared fallback, HRTF, advanced occlusion;
- gameplay hearing/AI perception из acoustic facts (consumers приходят с
  agent/world пакетами; здесь только детерминированный fact contract).

Критерии R2, на которые влияет пакет: «missing device/locale/audio output
использует declared fallback» и «UI/camera/audio/presentation faults не меняют
gameplay hash».

## 2. Нормативная база (прочитана полностью)

Routing-строка: «Audio, navigation, world services» →
[SPEC-08](../architecture/08-audio-navigation-and-world-services.md), ADR
нет, check `play`. Пакет также затрагивает строки «Assets/content catalog»
(SPEC-24, check `content-package`), «Rendering/presentation extraction»
(SPEC-30, cue envelopes), «Platform host» (SPEC-29, device lifecycle) и
«Player interaction/UI/localization» (SPEC-18 + ADR-019, subtitle fallback).

Прочитано полностью: SPEC-08 (v1.8), SPEC-24 (v1.1), SPEC-30 (v2.0),
SPEC-29 (v2.4), SPEC-18 (v2.3), ADR-019 (v1.1). Дополнительно по
inventory: SPEC-22/ADR-025 schema registration применяется через существующий
cook path.

## 3. Что требует архитектура (обязательства для дизайна)

### 3.1. Content (SPEC-24 `nextengine.content.audio`)

`NeutralAudioV1`: sample rate 8 000..=192 000 Hz, channel count 1..=8,
duration ≤ 21 600 s, loop/cue frames (integer, strictly sorted, ≤ 65 535
cues), loudness metadata, один canonical PCM blob. V1 PCM encoding:
`PcmS16Le`, `PcmS24LePacked`, `PcmF32LeCanonical`; float samples finite и в
`[-1, 1]`. Device format, mixer buffer, codec state, voice-service object в
record запрещены. Target encoded streams — presentation-only variants;
gameplay acoustic/timing facts остаются portable.

### 3.2. Audio scene (SPEC-08 «Audio architecture»)

`AudioScene` consumes PresentationSnapshot, cooked clips, emitters/listener,
rooms/portals и world acoustic parameters. Engine-native baseline MUST:
sample playback, streaming, spatial attenuation/panning, priority/voice
limiting, zone reverb fallback — без proprietary SDK.

Mixer/device state — presentation only (SPEC-08 source-of-truth). Gameplay
hearing использует deterministic acoustic facts (source, listener, loudness
class, occlusion zone, tick), не device output. Audio device loss → gameplay
продолжается, mixer reconnects; acoustic facts сохраняются.

### 3.3. Presentation cue boundary (SPEC-30)

`PresentationCueEnvelopeV1` уже резервирует closed rank `Audio = 0`; audio cue
ID — engine-owned `kind_local_key`. В коде `PresentationSnapshotV2.cue_batches`
— stub (`Vec<ContentHash>`, всегда пустой): вся cue machinery SPEC-30
(typed envelopes, consumption state, realization outcomes) ещё не
реализована ни для одного family. Audio будет первым real cue family;
минимальный audio payload должен вписываться в envelope contract, не
реализуя весь VFX/decal пласт.

### 3.4. Displayless check (SPEC-08 AUDIO-02)

Displayless audio check использует тот же deterministic `AudioScene`, sink —
bounded canonical PCM/WAV, не hardware device. Конфигурация фиксирует
listener, buses, sample rate/channels и semantic tick window. PCM
детерминирован на pinned sink, event alignment within one sample, acoustic
facts exact, gameplay hashes не зависят от audio output.

### 3.5. Subtitle fallback (SPEC-18 + SPEC-08)

Voice отсутствует → text/subtitle fallback; dialogue state не ждёт playback
completion. Subtitle/caption settings — `PresentationOnly` поля
`PlayerPreferenceProfile` (SPEC-18). Localized dialogue presentation не меняет
Dialogue node/choice/command payload; subtitles ссылаются stable text ID
(ADR-044 catalogs), не rendered strings.

### 3.6. Platform (SPEC-29)

Audio device — private platform adapter concern; device loss/restore —
typed lifecycle facts, presentation-only; headless не создаёт device objects.
`game`/`headless` делят authoritative substrate; audio sink не входит в
gameplay hashes.

## 4. Текущее состояние кода (инвентаризация 2026-08-05)

- Audio отсутствует полностью: ни одного совпадения `audio` в `crates/`.
- Content patterns: `NeutralRenderRecordV1` (mesh/material/texture,
  CanonicalBinaryV1, owner `nextengine.assets`) и `TextCatalogV1`
  (`crates/contracts/src/localization.rs`, ADR-044) — ближайший шаблон
  «один record = один canonical segment с embedded content_hash».
- Cook: `NeutralProjectSourceV1 { records, render_records, text_catalogs, .. }`
  → `cook_project_v1` публикует entries+blobs в `ContentManifestV1`;
  text catalogs — `PresentationOnly`, регистрация schema ref только при
  non-empty source.
- Activation: `project/src/activation.rs` декодирует blobs по schema ID в
  `ActivatedProjectV2 { neutral_records, text_catalogs, render_content_catalog }`;
  `contracts/src/project/activation.rs` валидирует manifest binding и
  closure (locale chain для text).
- Presentation extraction: scene/camera/semantic-ui batches реализованы;
  `cue_batches` — stub. `PresentationConsumptionStateV1` не реализован.
- Desktop adapter `desktop-sdl-ash`: SDL window/Vulkan/UI overlay; audio
  device path отсутствует.
- Preferences: `PlayerPreferenceProfileV1` — text scale + UI locale;
  subtitle/caption полей пока нет.
- Decode limits по умолчанию: total 16 MiB, field payload 8 MiB — это
  фактический ceiling inline PCM (~48 с stereo S16 44.1 kHz).
- `content_package` check утверждает exact fixture counts
  (`text_catalogs.len() == 2`); fixture changes требуют обновления check.
- Authoritative root fixture (`5e45825e…`) зависит от content manifest:
  добавление fixture clips меняет root ровно один раз, детерминированно.

## 5. Принятые решения (2026-08-05, bounded для baseline)

- **D1. PCM inline в record.** PCM samples хранятся inline в canonical
  record (как texels у texture), bounded decode limits. Effective ceiling —
  field payload 8 MiB при default limits. SPEC-24 duration cap 21 600 s —
  schema ceiling, не текущий decode budget; отдельный streaming payload
  role для длинных clips — поздний пакет (streaming admission, SPEC-23),
  здесь не нужен: baseline clips короткие.
- **D2. Loudness metadata — fixed-point.** `integrated_loudness_q16_16: i32`
  (signed, LU, обычно отрицательное) + `sample_peak_q16_16: u32`
  (`0..=65_536`, т.е. 0..=1.0). Float в public contract запрещён кроме
  declared cases; Q16.16 согласуется с canonical scalar profile SPEC-24.
- **D3. Channel model — count only.** `channel_count: u8` в `1..=8` без
  named speaker layouts. Baseline mixer (A3) объявит deterministic
  mono/stereo path и explicit downmix profile; contract не фиксирует layout
  semantics сверх count.
- **D4. Semantic class `PresentationOnly`.** Clip bytes не влияют на domain:
  gameplay acoustic facts производятся из DomainEvent/world state
  (SPEC-08 data flow), а не из clip payload. Loop/cue/timing metadata
  остаются presentation consumption facts. Если поздний пакет введёт
  authored timing events, класс пересматривается через новую schema
  version (ExactOnly policy).
- **D5. F32 canonicalization.** `PcmF32LeCanonical`: constructor
  нормализует semantic negative zero в `+0.0`; NaN/infinity/`|x| > 1`
  reject. Decode требует byte-exact round trip (уже нормализованные
  bytes), как у остальных records.
- **D6. A1 не трогает reference-game fixture.** Plumbing принимает
  `audio_records: Vec::new()` → manifest/blobs/roots byte-exact прежние.
  Fixture clips приходят с первым consumer (A3 mixer/AUDIO-02), чтобы
  root change был paired с product behavior.
- **D7. Cue payload — минимальный audio-only slice.** A2 определит
  `AudioCueV1` payload schema под существующий envelope rank `Audio = 0`
  без реализации всего VFX/consumption пласта SPEC-30; полный envelope
  batching придёт с первым real consumer и не блокирует baseline mixer.

## 6. Декомпозиция пакета (sub-increments)

- **A1 (этот increment): `NeutralAudioV1` content contract + cook/activation
  plumbing.** Contracts: schema/validation/canonical codec/hash + focused
  tests. Cook: `audio_records` source field, PresentationOnly entries,
  schema registration. Activation: decode + `ActivatedProjectV2.audio_clips`
  binding validation. Корни не меняются.
- **A2: Audio scene contracts.** Emitter/listener snapshot records,
  deterministic acoustic fact values, `AudioCueV1` payload под envelope
  rank 0, extraction из committed events/snapshot в presentation.
- **A3: Baseline software mixer + canonical PCM/WAV sink.** Voice
  admission (priority/limit), attenuation/panning, deterministic summing,
  displayless sink config; AUDIO-02-style check.
- **A4: Desktop adapter.** SDL audio device path, device loss/restore как
  typed facts, gameplay-independent; AUDIO-P1 aspects.
- **A5: Subtitle fallback.** Preference subtitle/caption fields
  (PresentationOnly), dialogue surface subtitle rendering из text
  catalogs, voice-absent fallback.
- **A6: Fixture + checks.** Engine-owned synthesized PCM clips в
  reference-game (provenance ProjectAuthored/Generated), content-package
  counts, `play` integration, roadmap/checkpoint update с root change
  evidence.

Каждый sub-increment — отдельный commit с focused checks; A1 не требует
roadmap status change (пакет остаётся в работе до A6).

## 7. Product checks план

- A1: `fast` (`cargo run -p xtask -- host-check`) + focused contract/cook/
  activation tests. Roots не меняются — `persistence-replay` как
  regression confirmation.
- A2: + `persistence-replay` (snapshot/cue contracts).
- A3: + AUDIO-02-style displayless PCM check (в составе `play` или
  отдельным scenario), `play`.
- A4: + `platform` (device path), `play`.
- A5: + `play`, ACCESS-P1 aspects через pseudo-locale/subtitle profile.
- A6: + `content-package`, `play`, `persistence-replay`; root change
  фиксируется в roadmap.

## 8. Риски и замечания

- `cue_batches` stub: A2 принимает первый real cue family contract;
  удержать его минимальным, не закрывая весь SPEC-30 envelope пласт.
- Inline PCM ceiling (8 MiB field) достаточен для baseline; не
  презентовать как streaming solution.
- SDL audio callback threading: device adapter (A4) обязан сохранять
  mixer state presentation-only и не создавать hidden gameplay input;
  callback не трогает authoritative state.
- Loudness metadata в A1 — authored values; cooker-derived loudness
  analysis — tool concern поздних пакетов.

## 9. Статус реализации

| Sub-increment | Статус |
|---|---|
| A1 NeutralAudioV1 + plumbing | `DONE_LOCAL_WINDOWS` (2026-08-05): contracts `142`+`10` audio tests, project integration test, `host-check`/`play`/`persistence-replay`/`content-package` PASS, authoritative roots byte-exact (fixture не тронут) |
| A2 Audio scene contracts + extraction | `DONE_LOCAL_WINDOWS` (2026-08-05): `AudioSceneSnapshotV1` contracts + deterministic extractor, 13 focused tests, `host-check`/`play`/`persistence-replay` PASS, roots byte-exact; production wiring в live loop приходит с A3 вместе с fixture clips и mixer |
| A3 Baseline mixer + canonical PCM sink + live wiring | `DONE_LOCAL_WINDOWS` (2026-08-05): `AudioMixerV1` (priority admission/preemption, distance/pan/zone в integer math, resampling, loop), canonical WAV sink, 4 engine-owned fixture clips + event/listener bindings, wiring в `ReferenceGameDriverV1`, AUDIO-02-style `audio-scene` check (20 ticks, 5 cues/5 facts, byte-exact PCM в paired runs); одноразовый root change от fixture clips: state roots `88977d5d→34a9bcd6` (play), `d3f6eced→c98bf08e` (persistence-replay), ledger/archive/identity roots byte-exact |
| A4 Desktop device adapter | `DONE_LOCAL_WINDOWS` (2026-08-05): SDL playback stream + bounded ring в desktop-sdl-ash, worker→adapter→game PCM plumbing, device loss/reopen counters, `audio_*` report fields; roots не изменились; real device path `NOT_RUN` headlessly (unit coverage sink'а + full-chain compile) |
| A5–A6 | Не начаты |

### A4 implementation record (2026-08-05)

- `crates/desktop-sdl-ash/src/audio_output.rs`: `DesktopAudioOutputV1` —
  SDL3 playback stream (48 kHz stereo S16LE callback stream) + bounded ring
  (96 000 samples, drop-oldest); states `Disabled | Unavailable | Active`;
  open failure → `Unavailable` silent sink с typed counters (AUDIO-P1
  bounded fallback); `AudioDeviceRemoved/Added` SDL events →
  `note_device_removed/added` с bounded reopen (max 8 attempts);
  `queue_pcm` exact ordering; `callback_underruns`. 4 unit tests.
- Plumbing: `ReferenceGameDriverV1::audio_pcm_shared` →
  `ApplicationCoordinator::reference_game_live_audio`
  (`ApplicationAudioFrameV1`) → worker `latest_audio` RwLock publication
  после каждого fixed-step publication (failures → silence, never worker
  failure) → `read_latest_audio` → apps/game closure queue по
  `audio_sequence` в adapter sink. Adapter: `audio_output_enabled` option
  (default true), новый pub entry
  `run_interactive_with_shared_timed_frame_source_audio_and_finalize`
  (legacy entry сохранён и делегирует), `audio_*` поля в
  `DesktopRunReport` + итоговый eprintln в game.
- Checks: `host-check`, `play`, `persistence-replay`, `content-package`,
  `platform`, `audio-scene` — все PASS, roots byte-exact (fixture не
  тронут). Real audio device path не гонялся headlessly
  (`NOT_RUN_ADAPTER_DISABLED` в platform candidate): sink logic покрыт
  unit tests, full chain компилируется и проходит adapter test suite.

### A3 implementation record (2026-08-05)

- `crates/presentation/src/audio_mix.rs`: `AudioMixerV1` +
  `AudioMixProfileV1` (48 kHz stereo baseline, 1 600 frames/30 Hz tick,
  16 voices, min/max distance 1/20 m, pan range 5 m, occlusion 0.5).
  Voice admission: priority desc → active-first → admission ordinal →
  canonical key; deterministic preemption/drops counters. Integer-only
  math: isqrt distance attenuation, Q1.30 quaternion inverse pan rotation,
  linear pan law, fixed-point resampling, S16/S24/F32 downmix, loop
  regions, saturating clamp counter. `encode_canonical_wav` — 44-byte
  RIFF PCM S16LE sink. 8 focused tests: profile fail-closed, non-spatial
  completion, priority preemption, distance/pan determinism, loop wrap +
  retire, WAV header, zone occlusion, half-rate resample.
- `crates/reference-game/src/audio.rs`: 4 synthesized clips (48 kHz mono
  S16, integer LFSR noise / two-tone / thud, decay envelopes) с
  fixed-point loudness metadata; cue bindings (switch/pickup/melee →
  EventPrincipal, dialogue → Listener); listener = player capsule body.
- Live wiring (`live.rs` + `live/audio_ops.rs`): driver публикует audio
  scene и canonical PCM window на каждом advance через stage→validate→
  commit; restore сбрасывает mixer (omit-never-replay semantics); roots
  gameplay не затрагиваются.
- Verification `player_fixture/audio_check.rs`: scripted interactive
  session (W/Q/R/E/D/F/Return через production control events) гонится
  дважды через `ReferenceGameDriverV1`; byte-exact scenes/PCM/facts/
  final state root; acceptance: ≥4 cues, все 4 clip assets, facts ==
  cues, ≥3 non-silent windows, canonical WAV digest. Команда
  `xtask audio-scene` печатает evidence; `xtask play` теперь включает
  этот gate (routing row audio → `play`).
- Fixture: 26 asset entries (4 audio clips); обновлены
  `content_package` и `content_pipeline` counts. Checks: `host-check`,
  `play`, `persistence-replay`, `content-package`, `platform`,
  `audio-scene` — все PASS.

### A2 implementation record (2026-08-05)

- `crates/contracts/src/presentation/audio_scene.rs`:
  `AudioSceneSnapshotV1` (epoch/sequence/tick-bound, canonical sort
  emitters by key, cues by `(activation_tick, cue_id)`, facts by
  `(tick, source, listener)`; limits 1 024/1 024/1 024),
  `AudioListenerRecordV1`, `AudioEmitterRecordV1`, `AudioCueV1` с
  derived engine-owned `cue_id` (domain hash от source event identity +
  cue slot + clip/emitter binding; re-extraction/replay дают то же
  значение), `AcousticFactV1` (source/listener/loudness class/occlusion
  zone/tick по SPEC-08 world services), closed `AudioLoudnessClassV1` и
  `AudioPriorityClassV1`. JCS + domain hash, как у остальных presentation
  records. 6 focused tests: canonical order independence, duplicate
  emitter/cue reject, cue identity stability/tamper, zero-hash clip и
  invalid orientation reject, epoch/sequence/tick binding, limits.
- `crates/presentation/src/audio_scene.rs`: `extract_audio_scene` —
  committed `DomainEvent` records + exact physics poses + binding profiles
  → immutable snapshot. `AudioEventCueBindingV1` (event schema → clip,
  loudness/priority, zone, `EventPrincipal | Listener` subject),
  `AudioEmitterBindingV1` (continuous sources с physics pose/fallback),
  principal subject resolution по всем `RpgEventV1`/`PhysicalEventV1`
  вариантам; principal без physics pose → cue без emitter record
  (non-spatial на mixer уровне); binding для event schema без principal
  subject — configuration error. 7 focused tests: order stability при
  permuted events, unbound schemas, listener anchoring, principal-missing
  reject, duplicate binding reject, fallback transform, cue identity
  binding.
- Production wiring (reference-game live loop, binding profile из
  fixture content) намеренно отложена в A3: cue bindings требуют
  реальных clip asset revisions, которые появятся с fixture clips и
  mixer vertical. Roots не изменились: persistence-replay
  `d3f6eced…`/`a8d12b0b…`, play state/ledger roots прежние.

### A1 implementation record (2026-08-05)

- `crates/contracts/src/audio.rs`: `NeutralAudioV1` +
  `AudioPcmEncodingV1`/`AudioLoopRegionV1`/`AudioLoudnessMetadataV1`,
  CanonicalBinaryV1 segment `nextengine.audio.v1` с embedded content hash,
  domain-separated `record_sha256`, fail-closed decode с byte-exact round
  trip. 10 focused tests (round trip, все три encoding, scalar bounds,
  loop/cue failures, loudness bound, F32 NaN/inf/range/negative-zero,
  tampered hash, unsupported version, non-canonical cues).
- Cook: `NeutralProjectSourceV1.audio_records`, PresentationOnly entries,
  schema registration при non-empty source, cross-family duplicate reject,
  decode re-validation в `validate_source`. Helpers (`schema_ref`,
  `ensure_unique`, `validate_text_catalog_closure`) вынесены в
  `cook_support.rs` — лимит 1000 строк/файл.
- Activation: audio branch в `activate_project`, sorted/bound
  `ActivatedProjectV2.audio_clips` с manifest hash validation в
  `contracts::project::activation`.
- Integration test `audio_clips_cook_publish_and_activate_through_production_loader`
  в `project/tests/content_pipeline.rs`: positive round trip через
  production loader, PresentationOnly class, duplicate-identity и
  invalid-record reject.
- Reference-game fixture получает `audio_records: Vec::new()` → manifest,
  composition lock и все authoritative roots неизменны.
- Checks: `host-check` PASS (fmt/clippy/tests/file-size), `play` PASS,
  `persistence-replay` PASS (roots `d3f6eced…`/`a8d12b0b…` без изменений),
  `content-package` PASS (manifest hash `548e3a3a…` без изменений).
