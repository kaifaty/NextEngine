# План реализации resident speech-timeline service

| Поле | Значение |
| --- | --- |
| Дата | 2026-08-18 |
| Статус | Proposed implementation plan; не ADR, не roadmap commitment |
| Целевой профиль | Linux x86_64, RTX 3080 10 GiB, один локальный microphone stream |
| Модели первого профиля | Voxtral Mini 4B Realtime 2602 Q4_K_M + emotion2vec_plus_base |
| Архитектурная граница | Optional separate `ai-host`/developer service по ADR-005; gameplay и Rust public contracts не меняются |
| Research | [Voxtral + emotion2vec facade](../development/voxtral-emotion2vec-facade-research-2026-08-18.md) |
| Durable state | [Audio emotion + ASR timeline](../development/task-state/audio-emotion-asr-timeline.md) |

## 1. Рекомендуемый baseline

Первая реализация — отдельный foreground service process, который запускается
один раз и остаётся resident. Он загружает и прогревает обе модели до открытия
локального WebSocket listener. Подключения создают только bounded session state;
веса между последовательными сессиями не перезагружаются.

```text
microphone client
  └─ localhost WebSocket
       ├─ JSON: hello/start/finish/cancel
       └─ binary: PCM16 mono 16 kHz
                    │
                    ▼
          SpeechTimelineService
             ├─ session state + integer sample clock
             ├─ bounded GPU worker
             │    ├─ VoxtralTranscriberAdapter
             │    └─ Emotion2VecAffectAdapter
             ├─ optional CPU VAD adapter
             └─ TimelineFusion
                    │
                    ▼
       transcript / affect / fusion revisions
       + final canonical text and LLM context
```

Рекомендуемые решения для первой вертикали:

1. loopback WebSocket на `127.0.0.1`, без remote/network режима;
2. один process и один GPU worker thread;
3. одна активная сессия, но любое количество последовательных сессий;
4. explicit `session.start`/`session.finish` как первая endpoint semantics;
5. `timing_precision="utterance"` как честный MVP;
6. автоматический VAD и `model_slot` добавляются отдельными commits после
   базовых residency/latency измерений;
7. structured timeline — authority внутри сервиса; inline emotion tags — только
   derived view при достаточной точности alignment;
8. сервис остаётся lab/tool consumer Deferred Proposed SPEC-16 и не создаёт
   текущий engine IPC/public schema.

## 2. Definition of done

MVP сервиса завершён, когда одновременно выполнено следующее:

1. обе модели загружаются и прогреваются один раз до `service.ready`;
2. вторая последовательная client session не вызывает повторную загрузку
   модели или повторный download;
3. client передаёт PCM16 без временных WAV и получает provisional transcript и
   emotion events во время речи;
4. первая emotion observation появляется после накопления 1 s контекста, а
   последующие stable observations планируются с шагом 250 ms без растущей
   очереди;
5. `session.finish` выдаёт ровно один `utterance.final` с plain final text,
   structured affect track и `alignment_grade="utterance"`;
6. malformed/oversized PCM, неверный protocol version, duplicate start,
   post-final audio, disconnect и cancel завершаются bounded typed outcome;
7. raw audio, transcript и score vectors не попадают в default logs и не
   сохраняются на диск;
8. service absence/crash никак не влияет на deterministic gameplay;
9. joint paced workload имеет RTF `< 1.0`, не теряет ASR chunks и публикует
   combined CPU/GPU memory, queue wait и capture-to-event p50/p95;
10. combined model residency оставляет не менее 1 GiB физического VRAM headroom
    на RTX 3080 либо emotion adapter явно переводится на измеренный CPU fallback;
11. focused tests, fault scenarios, `git diff --check` и финальный risk-scoped
    `host-check` проходят либо получают честный `NOT_RUN` с причиной.

Значения ASR latency, WER и emotion quality на этом этапе report-only. Параметр
Voxtral `delay_ms=480` не считается измеренной end-to-end задержкой.

## 3. Scope guard

### Входит

- resident local service и microphone client;
- model-neutral adapter interfaces и capability handshake;
- Voxtral Q4_K_M через внешний pinned `transcribe.cpp`;
- emotion2vec+ inference из in-memory waveform;
- transcript, raw affect observation, smoothed affect segment и fusion tracks;
- bounded binary PCM ingress, cancellation и typed diagnostics;
- no-model unit/integration tests с fake adapters;
- opt-in live CUDA benchmark с внешними model/runtime paths;
- automatic VAD как следующий bounded increment;
- отдельный experiment для Voxtral model-slot timing.

### Не входит

- Rust `crates/contracts` или production engine IPC;
- promotion SPEC-16/ADR-017;
- direct `CanonicalUtterance` admission в RPG runtime;
- multi-user/multi-GPU/remote service;
- TLS, Internet exposure или cloud provider;
- hot model replacement в активной сессии;
- persistence/replay of PCM or model sessions;
- calibrated claims о mental state;
- обязательный final forced aligner;
- shipping/distribution модели с не закрытыми license/provenance terms.

Если после lab evidence потребуется engine consumer, это отдельный план: он
должен определить engine-owned handshake, deadlines/idempotency, fallback,
`CanonicalUtterance` admission и DIALOGUE fault checks. Текущий план эту
границу не пересекает.

## 4. Подготовка Git lineage

Текущий research HEAD и проверенная Voxtral wrapper history расходятся от
`731d682`. Реализацию не следует начинать копированием файла из `git show`.

Рекомендуемый порядок:

1. создать `codex/speech-timeline-service` от текущего HEAD;
2. перенести только Voxtral-related commits из
   `codex/architecture-foundation-promotion`:
   `941cd40`, `d1e88eb`, `286023e`, `5bcda07`, `e839a38`;
3. не переносить соседние animation commits только ради wrapper;
4. разрешить возможные documentation conflicts, сохранив более новый pair
   research и task-state;
5. до refactor запустить существующие `lab/tests/test_voxtral_microphone.py`.

Это preparation lineage, а не новая функциональная реализация. Если target
branch к моменту работы уже содержит эквивалентные commits, шаг становится
read-only ancestry check.

## 5. Структура Python project

Рекомендуется расширить существующий isolated emotion project и переименовать
его directory/project в model-neutral `tools/speech-timeline`, сохранив
`next-emotion-probe` как backward-compatible console entry point.

```text
tools/speech-timeline/
  pyproject.toml
  uv.lock
  README.md
  src/
    nextengine_emotion_probe/       # сохранённый probe CLI/API
    nextengine_speech_timeline/
      __init__.py
      cli.py                        # serve, microphone, benchmark, doctor
      protocol.py                   # bounded messages/events
      capabilities.py               # model-neutral capability values
      session.py                    # lifecycle and sample clock
      scheduler.py                  # one bounded worker/priority/coalescing
      timeline.py                   # transcript/affect/fusion revisions
      metrics.py                    # monotonic latency/resource counters
      transport_websocket.py        # localhost adapter only
      adapters/
        base.py
        voxtral_transcribe_cpp.py
        emotion2vec.py
        vad.py                      # optional, added later
  tests/
    test_protocol.py
    test_session.py
    test_scheduler.py
    test_timeline.py
    test_websocket.py
    test_voxtral_adapter.py
    test_emotion_adapter.py
```

`lab/scripts/voxtral_microphone.py` остаётся совместимым direct probe. Его
capture-neutral model/session logic переносится в
`VoxtralTranscriberAdapter`; сам script использует тот же adapter и сохраняет
device listing, preflight, direct transcription и opt-in external WAV. Второй
независимый model loader не создаётся.

Новый project pin'ит Python 3.12 и exact runtime dependencies. Внешние GGUF,
Hugging Face cache и `transcribe.cpp` checkout/build остаются вне repository.

### Startup profile and artifact validation

`serve` принимает explicit external profile path, а не набор неограниченных
environment guesses. Profile фиксирует:

- Voxtral GGUF path, expected byte size/SHA-256, adapter ID и delay;
- `transcribe.cpp` checkout/shared-library paths и expected runtime revision;
- emotion model ID, exact Hugging Face revision, cache root and device;
- service bounds, listener port and external ready/token file;
- optional debug/benchmark output root outside repository.

До model allocation service проверяет schema/version, finite numeric ranges,
file existence/size/hash и runtime capability. Public events содержат только
model/runtime IDs and hashes, не raw filesystem paths.

Network остаётся default-deny. Emotion service load использует уже загруженный
pinned snapshot (`local_files_only` semantics); download остаётся отдельной
explicit `next-emotion-probe download`/prepare operation. Missing cache,
hash/revision mismatch или unclassified incompatible profile завершают startup
typed diagnostic до listener publication. Текущий emotion checkpoint может
быть помечен `unclassified_local_only` для PoC, но не distributable/ship-ready.

## 6. Model-neutral adapter contracts

Контракты остаются private/current-only Python values, не `crates/contracts`.

### StreamingTranscriber

```text
capabilities() -> TranscriberCapabilities
load()         -> ModelLoadEvidence
warmup()       -> WarmupEvidence
start(config)  -> TranscriberSession

TranscriberSession.push_pcm(chunk) -> TranscriptRevision | None
TranscriberSession.finish()        -> TranscriptRevision(final=true)
TranscriberSession.cancel()
TranscriberSession.close()
```

`TranscriberCapabilities` содержит:

- adapter/protocol version;
- model ID, immutable revision/content hash и runtime revision;
- native sample rate/PCM encoding;
- locales/language hint support;
- streaming/revision semantics;
- `timing_precision` + `resolution_samples`;
- supported delay values;
- declared duration/resource limits;
- license/provenance classification status.

Voxtral adapter на первом этапе объявляет `timing_precision="utterance"`.
Authoritative current hypothesis берётся из `StreamText.full`, а
`committed/tentative` сохраняются только как best-effort UI fields. Финальный
text берётся после единственного `finalize()`.

### VocalAffectAnalyzer

```text
capabilities() -> AffectCapabilities
load()/warmup()
observe(AudioWindow) -> AffectObservation
```

`AudioWindow` содержит immutable float32 mono samples, exact half-open sample
interval и source session revision. Adapter возвращает полный normalized
nine-label vector, inference time, model identity и semantics
`uncalibrated_observed_expression`.

Существующий `EmotionProbe` получает новый in-memory API:

```text
analyze_waveform(samples: numpy.float32, sample_rate_hz=16000)
```

Установленный FunASR 1.4.2 принимает `numpy.ndarray` raw 16 kHz samples, поэтому
временный WAV не нужен. Текущий `analyze(path)` становится wrapper над тем же
normalization/result builder и сохраняет прежний CLI JSON.

Chinese/English upstream labels нормализуются только в affect adapter:

```text
生气/angry       -> angry
厌恶/disgusted   -> disgusted
恐惧/fearful     -> fearful
开心/happy       -> happy
中立/neutral     -> neutral
其他/other       -> other
难过/sad         -> sad
吃惊/surprised   -> surprised
<unk>            -> unknown
```

Unknown label, non-finite score, wrong vector length или duplicate normalized
label являются typed adapter failure, не best-effort guessing.

## 7. Session state and data structures

### Lifecycle

```text
CONNECTED -> AUTHENTICATED -> ACTIVE -> FINALIZING -> FINAL
                                  └----> CANCELLED
                                  └----> FAILED
```

- one connection owns at most one active session;
- `start` is accepted once;
- binary PCM is accepted only in `ACTIVE`;
- `finish` and `cancel` are idempotent by session identity but emit one terminal
  event;
- post-terminal frames are rejected;
- disconnect implies cancel unless final already emitted;
- a new session gets fresh model session/cache and sample clock, not new weights.

### Audio storage

Для первого bounded turn достаточно `bytearray`:

- PCM16 mono little-endian;
- default hard turn ceiling 30 s (`960,000` bytes);
- hard binary WebSocket frame ceiling 32,000 bytes (1 s);
- recommended client frame 8,000 bytes (250 ms);
- odd byte length, empty frame, overflow and wrong sample format reject before
  conversion/model call;
- `audio_received_samples = total_pcm_bytes / 2` — единственный audio clock;
- per-window float32 conversion выполняется только при постановке inference
  job;
- buffer очищается на terminal state; default path ничего не пишет на диск.

`bytearray` проще ring buffer и bounded менее 1 MiB на 30 s turn. Настоящий
ring/deque нужен только после доказанного long-session consumer.

## 8. Local WebSocket protocol V1

Transport остаётся private lab adapter. JSON control/event frame ограничен
64 KiB и проверяется до allocation-heavy parsing. Audio идёт binary, без
base64. WebSocket уже гарантирует ordering; transport adapter присваивает
каждому binary frame internal monotonic `sequence` и `start_sample`.

### Client -> service

```json
{"schema_version":1,"type":"client.hello","token":"<redacted>"}
{"schema_version":1,"type":"session.start","session_id":"opaque","locale":"ru","sample_rate_hz":16000,"encoding":"pcm_s16le","channels":1}
<binary PCM frames>
{"schema_version":1,"type":"session.finish","session_id":"opaque"}
{"schema_version":1,"type":"session.cancel","session_id":"opaque"}
```

### Service -> client

```text
service.ready
session.started
speech_timeline.update
utterance.final
session.cancelled
error
```

`service.ready` содержит exact service protocol, model/runtime identities,
capabilities, devices, load/warm-up timing и hard bounds. `error` содержит
stable code, terminal flag и bounded details без transcript/audio/secrets.

Service binds only `127.0.0.1`. При запуске он создаёт random 256-bit token в
explicit external token/ready file с mode `0600`; token не передаётся через
command-line args и редактируется из logs. Клиент читает ready file. Remote bind
и `--no-auth` отсутствуют в V1.

## 9. Scheduler and backpressure

`asyncio` обслуживает transport, но все model calls выполняются одним dedicated
worker thread. Обе модели загружаются и прогреваются в этом worker до listener
startup, что исключает случайный CUDA thread affinity и concurrent allocator
pressure.

Очередь имеет stable priority и monotonic enqueue sequence:

1. Voxtral `finish`;
2. Voxtral `push_pcm`;
3. final exact emotion observation;
4. stable 2 s emotion observation;
5. fast 1 s emotion observation.

Правила bounds:

- ASR chunk не дропается и не reorder'ится;
- pending provisional emotion job хранится максимум один на mode;
- новый более свежий window заменяет ещё не начатый obsolete window;
- started job не отменяется посередине CUDA call;
- ASR backlog больше двух chunks или inability to keep paced RTF `< 1` завершает
  session typed `SERVICE_OVERLOADED`, а не растит memory;
- terminal/cancel invalidates queued jobs by session generation;
- completion from stale generation discarded before timeline mutation.

Если combined VRAM не оставляет 1 GiB headroom, первый fallback — измерить
emotion2vec на CPU при Voxtral на CUDA. Разделение на два GPU processes не
считается memory fix: оно добавит CUDA contexts и усложнит scheduling.

## 10. Emotion timeline and fusion V1

### Observation cadence

1. при `audio_end=1.0 s` — fast trailing 1 s observation;
2. до 2 s — новая fast observation каждые 250 ms;
3. с 2 s — trailing 2 s stable observation каждые 250 ms; параллельный 1 s
   pass больше не нужен;
4. при `finish` — exact whole-turn observation в declared max duration;
5. после добавления VAD jobs запускаются только при достаточной speech
   occupancy, но sample intervals остаются исходными.

Raw observation всегда хранит полный vector. Первоначальный deterministic
smoother не обучается и остаётся configurable research policy:

- weighted moving average последних overlapping observations;
- candidate label должен быть top-1 два consecutive hops;
- switch требует finite score threshold и margin над текущим label;
- minimum dwell предотвращает flicker;
- недостаточная evidence публикует `unknown`;
- exact threshold/margin/dwell остаются report parameters до labeled Russian
  transition corpus.

### Independent tracks

Каждый `speech_timeline.update` содержит три independently revisioned части:

- `transcript`: full current text, optional best-effort stable prefix, finality,
  `timing_precision`;
- `vocal_affect`: `replace_from_sample`, raw observation IDs и smoothed
  half-open segments;
- `fusion`: `alignment_grade`, derived spans либо turn-level expression.

При `utterance` grade `fusion.spans=[]`; downstream LLM получает plain text и
turn/region-level `observed_vocal_expression`. Word ranges и inline tags
запрещены до passing timing gate.

## 11. Automatic endpointing increment

После passing explicit-finish vertical добавляется replaceable CPU
`SpeechEndpointDetector` adapter:

- exact pinned Silero VAD artifact/runtime или другой выбранный CPU adapter;
- 16 kHz PCM, declared frame size and bounded state;
- pre-roll, speech start, speech occupancy и endpoint candidate on audio clock;
- initial silence-end seed 400 ms и max utterance 30 s — benchmark parameters,
  не product constants;
- explicit finish/cancel остаются обязательным fallback;
- VAD never edits transcript/emotion and never admits gameplay input;
- false endpoint test включает короткие паузы, фоновый шум и trailing silence.

VAD artifact проходит отдельную hash/license/provenance classification. До
этого service работает с manual finish и не download'ит VAD implicitly.

## 12. Model-slot alignment experiment

Этот этап не блокирует resident MVP.

1. В отдельной pinned `transcribe.cpp` branch/API expose generated token ID,
   control/text kind и decoder output slot.
2. Python binding получает capability `timing_precision="model_slot"`; старый
   runtime продолжает работать как `utterance`.
3. Voxtral adapter группирует lexical units только через model-specific control
   tokens и configured delay mapping.
4. На вручную размеченном Russian corpus измеряются median/p95 word boundary
   error и tagged-span precision.
5. `model_slot` включается только при заранее объявленном gate; иначе adapter
   остаётся `utterance` и рассматривается optional final aligner.

Arrival time, `committed_changed` и `audio_committed_ms` не используются как
acoustic time ни в одном варианте.

## 13. Observability and benchmark report

Каждый model job записывает bounded counters без raw content:

- session/revision/job kind;
- audio window start/end sample;
- enqueue/start/end monotonic timestamps;
- queue wait and inference duration;
- input/output byte/count bounds;
- coalesced/dropped/stale result counters;
- process RSS and GPU used/peak snapshot when available;
- model load/warm-up count and duration.

`benchmark` умеет подать внешний WAV в paced и unpaced режимах и публикует один
versioned JSON report во внешний `--out` path через staging + atomic replace.
Report включает p50/p95, RTF, max queue depth, no-reload count and model hashes;
audio/transcript не включаются по умолчанию.

Required local evidence:

1. cold start and warm readiness;
2. two sequential sessions, load count exactly one per adapter;
3. 30 s paced audio without growing ASR backlog;
4. cancel/disconnect during each job type;
5. combined GPU peak and CPU RSS;
6. microphone capture-to-first-ASR, window-end-to-emotion and
   finish-to-final p50/p95;
7. CPU emotion fallback result if GPU headroom gate fails.

## 14. Commit sequence

### Commit 0 — converge the tested Voxtral lineage

- bring exact Voxtral wrapper/test commits into the implementation branch;
- run the existing focused test before modifications;
- no semantic refactor in the cherry-pick conflict resolution.

### Commit 1 — `refactor(tools): establish speech timeline Python project`

- rename/expand `tools/emotion-probe` to `tools/speech-timeline`;
- preserve `next-emotion-probe` CLI and its output;
- add `nextengine_speech_timeline` package skeleton;
- update exact documentation paths and external venv instructions;
- regenerate the project lockfile only through the package manager.

Checks: old probe tests, import/CLI smoke, lock consistency, `git diff --check`.

### Commit 2 — `refactor(audio): add in-memory emotion inference`

- factor common result construction from file-only `EmotionProbe.analyze`;
- add validated float32 waveform API;
- add normalized label mapping in `Emotion2VecAffectAdapter`;
- preserve raw-score semantics and no-write default.

Checks: array shape/dtype/rate bounds, non-finite/malformed vector failures,
file API parity, fake-model call count, existing probe tests.

### Commit 3 — `refactor(asr): extract resident Voxtral adapter`

- move capture-independent runtime/model/session logic from microphone script;
- expose capabilities, load/warm/start/push/finish/cancel;
- use full hypothesis as current authority and declare `utterance` timing;
- make old direct microphone script consume the same adapter;
- preserve device/preflight/debug-WAV behavior.

Checks: fake transcribe binding lifecycle, exactly one finalize, full versus
display text, two sessions/one model load, all existing microphone tests.

### Commit 4 — `feat(audio): add bounded speech timeline core`

- add protocol values, session state machine and sample clock;
- add bounded PCM buffer and independent track revisions;
- implement scheduler priority/coalescing with fake adapters;
- implement raw affect cadence, smoothing and utterance fusion;
- no socket and no real models required for tests.

Checks: state transitions, frame/turn bounds, monotonic revisions, stale
generation discard, queue overload, exact sample intervals, no word spans at
utterance grade.

### Commit 5 — `feat(audio): serve resident loopback sessions`

- add authenticated localhost WebSocket adapter and ready file;
- load/warm models before listen;
- binary PCM ingress and JSON events;
- disconnect/cancel/error semantics;
- one active session and sequential reuse.

Checks: in-process fake WebSocket integration, auth/version/size failures,
duplicate/post-final frames, client disconnect, second sequential session.

### Commit 6 — `feat(audio): add microphone client and live timeline view`

- reuse existing capture/list/preflight functions;
- add full-duplex microphone sender/event renderer;
- render transcript and emotion timeline without making tags authoritative;
- keep direct Voxtral probe command available.

Checks: fake capture chunks, exact duration, client cancel/final, no default
audio write, opt-in debug path remains outside repository.

### Commit 7 — `test(audio): close resident latency and fault evidence`

- add external WAV benchmark runner and atomic JSON report;
- run actual joint CUDA two-session smoke;
- measure VRAM/RSS/queue/latency/RTF;
- test CPU emotion fallback only if headroom gate fails;
- update task-state with measured decision.

Checks: focused Python suite, actual external model smoke, `git diff --check`,
final risk-scoped `cargo run -p xtask -- host-check`.

### Commit 8 — optional `feat(audio): add bounded VAD endpointing`

- only after Commit 7 evidence;
- pin/classify VAD artifact, keep explicit finish fallback;
- add pause/noise/false-endpoint corpus.

### Separate experiment — Voxtral model-slot timing

- external runtime change and alignment corpus evidence;
- not combined with transport/residency commits;
- promotion to `model_slot` only through a measured task-state decision.

## 15. Test matrix

| Layer | Positive | Failure |
| --- | --- | --- |
| Protocol | valid hello/start/binary/finish | bad version/type, >64 KiB JSON, odd/oversized PCM |
| Session | one final, sequential reuse | duplicate start/final, post-final audio, disconnect/cancel |
| Scheduler | ordered ASR, coalesced affect | backlog overload, stale completion, adapter exception |
| Emotion adapter | in-memory vector and normalized labels | NaN, duplicate/unknown labels, wrong score count |
| Voxtral adapter | one load, multiple sessions, final full text | unsupported stream, load/feed/finalize failure |
| Timeline | exact half-open samples and revisions | regressing interval/revision, word spans without timing |
| Transport | authenticated loopback client | wrong token, remote bind request, malformed control |
| Persistence | no files by default | repository debug path/overwrite rejected |
| Live resource | paced RTF <1 and stable queue | OOM/headroom fail selects measured fallback or rejects profile |
| Fault isolation | service kill/restart | no gameplay dependency; client receives bounded failure |

Tests с fake adapters являются ordinary CI/local suite. Tests с actual weights,
microphone and CUDA opt-in, используют external artifact paths and никогда не
download'ят модель неявно.

## 16. Rollback and failure decisions

- Если one-process joint runtime воспроизводимо падает из-за CUDA/runtime
  interaction, перейти к resident supervisor + two workers behind the same
  adapter interface; transport/timeline schema не менять.
- Если combined VRAM слишком велик, измерить emotion CPU. Не уменьшать Voxtral
  quantization и не вводить parallel GPU processes без quality/resource
  evidence.
- Если service не держит paced RTF `<1`, сначала измерить queue attribution,
  затем изменить chunk/decode cadence; не скрывать backlog дропом ASR.
- Если emotion transition flicker высок, оставить raw observations и
  `unknown`, а thresholds подбирать только на labeled corpus.
- Если model-slot timing не проходит boundary gate, оставить utterance fusion
  и отдельно оценить final aligner.
- Если license/provenance модели остаётся unclassified для distribution,
  сервис остаётся local PoC и не входит в package.

## 17. Вопросы, меняющие scope

План использует рекомендуемые ответы ниже; перед реализацией их желательно
подтвердить.

1. **Граница первого результата**
   - **A — standalone local service (рекомендуется):** завершаем resident
     WebSocket service и microphone client, без Rust/engine integration.
   - B — сразу engine-facing `ai-host`: потребуется отдельный public IPC,
     `CanonicalUtterance` path, play/fault checks и решение по Proposed
     SPEC-16/ADR-017.

2. **Endpointing MVP**
   - **A — explicit start/finish сначала (рекомендуется):** residency, streaming
     и fusion проверяются без третьей модели; automatic VAD идёт следующим
     commit.
   - B — automatic VAD обязателен для первого usable MVP: больше scope и
     artifact/license work, но сразу hands-free dialogue turns.

3. **Точность alignment для определения `done`**
   - **A — `utterance` достаточно для MVP (рекомендуется):** сервис уже
     передаёт LLM текст + честный emotion timeline; model-slot — measured
     follow-up.
   - B — сервис не считается готовым без `model_slot`: понадобится изменение
     внешнего `transcribe.cpp` и русский manually aligned corpus до завершения.

4. **Transport**
   - **A — authenticated localhost WebSocket (рекомендуется):** проще подключать
     Python/Rust/UI clients и поддерживать binary PCM/full duplex events.
   - B — Unix domain socket: уже security surface, но сложнее future cross-
     platform client и browser/tool integration.

Если ответов нет, реализация может безопасно начинаться по вариантам `A/A/A/A`.
