# Roadmap V12: автономная база физического звука

| Поле | Значение |
| --- | --- |
| Дата rebaseline | `2026-08-31` |
| Статус | `CLOSED / C4A_REPEAT_DATA_INSUFFICIENT_FORCE_COVERAGE / V13_REBASELINED / RUNTIME_NOT_AUTHORIZED` |
| Предыдущий roadmap | [V11](physical-sound-synthesis-roadmap-v11.md), закрыт после B1R3 |
| Exact основание | [C1 repeat-exact result](../development/physical-sound-r3a-v12-c1-acquisition-coverage-oracle-result-2026-08-31.md) |
| Архитектура | [SPEC-45](../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md), `Proposed` |
| Текущее состояние | [Physical sound task state](../development/task-state/physical-sound-synthesis.md) |
| Product fallback | Обычные authored clips обязательны для любого reject/OOD/fault |

> V12 закрыт повторяемым отрицательным результатом C4a. Активная программа
> продолжена в [Roadmap V13](physical-sound-synthesis-roadmap-v13.md), который
> отделяет canonical-impact modal field от недоказанного measured transfer.

## Куда мы идём

Цель — не одна формула «как звучит стекло» и не генератор waveform по слову.
Мы строим пополняемую, автоматически проверяемую базу физических моделей:

```text
измеренная сила удара
        ×
моды exact-объекта (частота, затухание, участие точки контакта)
        ×
акустическое представление
        =
детерминированно испечённый набор обычных audio clips
```

Нейросеть появится только после того, как эта факторизация проходит на реальных
данных. Её первая задача — предсказывать участие уже проверяемых мод по
геометрии и точке контакта, а не заменять физику непрозрачным waveform latent.

## Что изменил B1R3

B1R3 repeat-exact отклонён, но локализовал проблему:

- шесть наблюдаемых мод восстановлены с `0.007180` mean NRMSE;
- максимум ошибки частоты — `0.119071 Hz`, относительного затухания — `1.15%`;
- weak excitation и unmeasured interference корректно дают OOD;
- мода `6,643 Hz` не восстанавливается, потому что сам набор force pulses почти
  не содержит энергии в этой полосе: local SNR около `1.2…1.34`;
- намеренный notch на `8 kHz` нельзя изолировать от этого случайного hole.

Значит, следующий слой — не ещё один порог и не более крупная сеть, а явный
**сертификат наблюдаемости входного возбуждения**.

## Что доказал C1

C1 прошёл дважды побитово на новой known-truth revision:

- force-only certificate покрывает `200…9,500 Hz` минимум тремя профилями;
- все пять leave-one-profile-out fits восстанавливают `7/7` мод без false
  positives;
- holdout достигает `0.004961` mean NRMSE и `0.021430 dB` spectrum RMSE;
- acquisition notch, weak excitation, dynamic interference и missing impact
  получают правильные OOD-решения;
- notch только в query не уничтожает уже доказанную object model;
- network, real payload и parent holdout reads равны нулю.

Это закрыло synthetic identifiability prerequisite, но не стало доказательством
качества реального звука. Затем C2 подтвердил интернет-источник, C3 заморозил
роли, а C4a остановил ветку на недостаточном force coverage до чтения response.

## Три разных домена, которые нельзя снова смешивать

```mermaid
flowchart LR
    F["Calibration force ensemble"] --> C["Acquisition coverage certificate"]
    C --> M["Identifiable modal object model"]
    M --> Q["Known query force / energy profile"]
    Q --> R["Predicted response and relevance mask"]
    R --> V{"Independent validator"}
    V -->|Pass| A["Baked clip atlas"]
    V -->|Reject / OOD / fault| B["Authored clip fallback"]
```

1. **Acquisition domain:** какие полосы вообще возбуждены совокупностью
   опубликованных измеренных ударов достаточно сильно, чтобы идентифицировать
   объект.
2. **Object-model domain:** какие poles и residues реально доказаны внутри
   acquisition domain.
3. **Query domain:** какой вклад уже известная модель даст для конкретного
   профиля силы/энергии. Ноль входной энергии не создаёт новую моду и не обязан
   автоматически инвалидировать весь объект.

## Программа работ

| ID | Этап | Статус | Exit criterion |
| --- | --- | --- | --- |
| C0 | B1R3 exact result и V12 rebaseline | `COMPLETE / REPRODUCIBLE` | Два запуска и все артефакты byte-identical; reject и нулевой holdout зафиксированы. |
| C1 | Coverage-certified known-truth oracle | `COMPLETE / REPEAT_EXACT_PASS` | Все acquisition-supported truth modes восстановлены, unsupported controls не изобретены, held responses и OOD gates проходят дважды побитово. |
| C2 | Internet-source zero-decode inventory | `COMPLETE / NARROWED_SOURCE_PASS` | Найден хотя бы один stable paired force+mic source с доказуемыми axes/lineage; PCM не читается. |
| C3 | Source role freeze и bounded importer | `COMPLETE / REPEAT_EXACT_PASS` | Parent-disjoint fit/development/holdout/validator/shadow roles и exact read counters заморожены до decode. |
| C4 | Real transfer-response model | `CLOSED / REPEAT_EXACT_DATA_INSUFFICIENT_FORCE_COVERAGE` | Fit-force certificate дважды показал нулевое общее покрытие; microphone и protected roles не открыты. |
| C5 | Physical Sound Record V1 | `REBASELINED_TO_V13_M1` | Новая experimental record-схема сначала различает canonical-impact и measured-transfer claims. |
| C6 | Exact-object contact ML | `REBASELINED_TO_V13_M3_M5` | Geometry/contact model проверяется как modal-gain field на canonical-impact данных. |
| C7 | Independent Validator V1 | `REBASELINED_TO_V13_M7` | Frozen validator остаётся обязательным независимым gate. |
| C8 | Deterministic atlas cooker | `REBASELINED_TO_V13_M8` | Byte-identical clips и complete fallback map остаются product boundary. |
| C9 | One-shot exact-object admission | `REBASELINED_TO_V13_M9` | Shadow admission перенесён без открытия ролей V12. |
| C10 | Material-family expansion | `REBASELINED_TO_V13_M10` | Material families остаются агрегацией exact domains. |
| C11 | Один production impact prop | `REBASELINED_TO_V13_M11 / ADR_REQUIRED` | Runtime/public promotion по-прежнему не разрешена. |

## C1 — последний bounded synthetic oracle

Это одна свежая revision, а не серия подстроек B1R3. Она завершена решением
`PASS_KNOWN_TRUTH_FRF`; точный результат и hashes находятся в
[C1 result](../development/physical-sound-r3a-v12-c1-acquisition-coverage-oracle-result-2026-08-31.md).

Протокол [V12-C1](../development/physical-sound-r3a-v12-c1-acquisition-coverage-oracle-protocol-2026-08-31.md)
и его [bounded research](../development/physical-sound-r3a-v12-c1-coverage-and-source-research-2026-08-31.md)
зафиксированы до runner. Force-only prefreeze подтверждает полную
`200…9,500 Hz` coverage минимум тремя профилями и не читает object response.

### Fixture и blind separation

- Новые truth frequencies, damping, residues, phases и RNG seeds.
- Сначала создаётся force ensemble и его band-coverage certificate без доступа
  к transfer truth.
- Обычные truth modes детерминированно выбираются только из заранее
  сертифицированных непрерывных bands, а не рядом с максимумами конкретных FFT.
- Отдельные negative fixtures получают заранее объявленные unsupported bands,
  comb notch, weak excitation, unmeasured interference и missing impact.
- Discovery/fit не читает truth; truth доступна только финальному scorer.

### Обязательные проверки

- `supported truth recall = 100%`, false positives `= 0`;
- frequency/damping, NRMSE, spectrum и compatible-control gates остаются не
  слабее B1R3;
- leave-one-force-family-out проверяет, что сертификат принадлежит ансамблю, а
  не одному удачному pulse;
- query с нулевой энергией в известной полосе не изобретает output energy;
- acquisition hole даёт `FallbackOutOfDomain`, dynamic interference —
  `OOD_LOW_COHERENCE`, missing force event — `OOD_MODEL_MISMATCH`;
- development полностью проходит до генерации holdout;
- два полных запуска повторяют manifest/model/arrays/report byte-for-byte;
- real/network/parent-holdout reads равны нулю.

### Жёсткий stop rule

- Полученный `PASS_KNOWN_TRUTH_FRF` вместе с C2 source pass разрешает только C3
  source/role freeze; он сам по себе не разрешает decode.
- Valid failure на восстановлении **сертифицированной** моды закрывает текущую
  Gabor/common-pole ветку. Следующий выбор делается только после отдельного
  bounded comparison с local-rational/vector-fitting control; B1R3 fixture и
  пороги не ремонтируются.
- Failure только потому, что force ensemble не может сертифицировать полезную
  полосу, закрывает источник/experiment design как `DATA_INSUFFICIENT`.

## C2–C3 — интернет вместо домашней записи

Пользователь не записывает микрофон и не ударяет по предметам. Поиск и importer
должны работать с опубликованными данными.

Zero-decode inventory проверяет до чтения waveform:

- paired raw force и microphone с общим timebase;
- stable object/trial identity и impact coordinates;
- geometry/scale, support condition и listener/microphone pose;
- sample rate, units, calibration и saturation metadata;
- parent grouping, stable download, provenance и redistribution boundary.

[C2 exact inventory](../development/physical-sound-r3a-v12-c2-internet-source-zero-decode-inventory-2026-08-31.md)
принимает ObjectFolder-Real как stable paired source с узким claim. Raw
`mic.wav + Force.wav` и coordinate/geometry lineage доказаны без нового decode;
live endpoint сохранил length/ETag. Numeric listener pose, per-object support и
SI calibration не опубликованы, поэтому первая модель ограничена canonical
setup и normalized instrument counts. Старые object-51 contacts являются
structural witness, но не fresh C3/C4 target.

Кандидат без paired force+mic может быть validator/control source, но не учит
абсолютный force→response transfer. Кандидат без geometry/contact axes не
получает contact-ML claim. Отсутствующий axis никогда не выводится из material
label.

После C1+C2 один source freeze заранее назначает шесть непересекающихся ролей:
estimator fit, generator development, representation holdout, validator
calibration, validator method holdout и admission shadow. Decode начинается
только после hash-closed manifest и exact budgets.

[C3 protocol](../development/physical-sound-r3a-v12-c3-object41-source-role-freeze-protocol-2026-08-31.md)
выбирает fresh `41 / Wrench_Large / Steel`, 35 contacts и signal-blind
SHA-256 partition `16/5/4/4/3/3`. Raw scan ограничен первыми `4 GiB`; неполный
объект закрывает revision как `DATA_INSUFFICIENT_RAW_PREFIX` без расширения.

[C3 exact result](../development/physical-sound-r3a-v12-c3-object41-source-role-freeze-result-2026-08-31.md)
проходит дважды побитово: `35/35` paired headers, `35/35` raw/compact microphone
identities, complete geometry/scale и нулевой PCM decode. Это открывает только
C4 fit protocol на том checkpoint; итоговый C4a затем закрыл V12 без открытия
microphone и protected roles.

## C4–C6 — от реального отклика к формулам и ML

Одна запись превращается не в «эталонный WAV», а в проверяемый record:

```text
Object/geometry revision
Contact and listener descriptors
Force ensemble coverage certificate
Shared frequencies and damping
Per-contact modal residues and uncertainty
Unexplained residual budget
Source lineage and protected role
```

C4 остановлен до чтения microphone: повторяемый fit-force certificate показал,
что удары high-SNR, но спектрально узкие и почти не перекрываются. Точный
результат находится в [C4a result](../development/physical-sound-r3a-v12-c4a-object41-real-frf-fit-result-2026-08-31.md).

Этот источник не поддерживает заявленную arbitrary-force transfer-модель.
Object `41` остаётся acquisition-OOD fixture, а record/contact ML/validator/
atlas продолжаются в V13 для более узкого canonical-impact claim. Runtime
weights запрещены: любой accepted model используется offline.

Если classical interpolation выигрывает у ML, база и atlas продолжают работать
без ML. Это успешный инженерный fallback, а не повод подменить метрики.

## C7 — кто валидирует автоматически

Validator — отдельный frozen ensemble, который не участвует в обучении:

- hard checks: hashes, lineage, units, counts, determinism, finite/bounded data;
- physics checks: force-response consistency, poles, damping, residues,
  unexplained energy and OOD calibration;
- acoustic checks: spectrum, decay, transient/envelope and artifact specialists;
- corpus checks: source/object/contact-disjoint failures and controlled
  mutations;
- risk layer: верхняя confidence bound false-pass rate плюс measured coverage.

Решение всегда tri-state: `Pass`, `Reject`, `FallbackOutOfDomain`. Learned
metric или audio-language model может быть только одним specialist vote и не
имеет права единолично пропускать asset. Человеческое прослушивание остаётся
необязательным report-only анализом.

## C8–C11 — продуктовый путь

Offline cooker сворачивает admitted modal record с ограниченным семейством
force/energy profiles и печатает обычные `48 kHz` clips, hashes, coverage и
fallback map. Clips идут по существующему audio path; симуляция от них не
зависит.

Первый one-shot admission относится только к одному exact объекту и canonical
listener condition. Затем тот же immutable pipeline повторяется для других
glass objects, wood и metal. Material-family claim допускается только как
агрегация доказанных exact domains, никогда как одна магическая формула.

Runtime/public contract появляется лишь при concrete visible consumer,
engine-owned committed contact projection, прошедшем ProductCheck и отдельном
Accepted ADR. До этого SPEC-45 остаётся `Proposed` и authored clips —
авторитетный путь.

## Ближайшие commit boundaries

1. `C0–C1c`: `COMPLETE`; B1R3 result/rebaseline, frozen C1 protocol, runner,
   paired preflights and byte-identical run A/B evidence are in Git or external
   evidence roots as appropriate.
2. `C2a`: `COMPLETE`; official URLs, versions, axis matrix and zero-decode
   structural witness are recorded.
3. `C3a`: `COMPLETE`; fresh target, six roles and `4 GiB` stop rule are
   preregistered before runner.
4. `C3b`: `COMPLETE`; bounded importer and focused synthetic tests are
   implemented before real inventory.
5. `C3c`: `COMPLETE`; freeze, paired zero-read preflights and inventory A/B
   pass byte-for-byte.
6. `C4a`: `COMPLETE`; fit-only force/FRF protocol and one-shot limitations are
   frozen before PCM decode.
7. `C4b`: `COMPLETE`; runner, synthetic/no-access tests and repeated zero-read
   preflight passed and opened only the exact sixteen-contact force-first fit
   budget at that historical checkpoint.
8. `C4c`: `COMPLETE / REPEAT_EXACT_DATA_INSUFFICIENT`; force A/B совпадают,
   microphone/development/holdout/validator/shadow decode остаётся нулевым.
9. `C4d–C4e`: `NOT_RUN / CLOSED_WITH_V12`; дальнейшее исполнение перенесено в
   V13 без переноса stronger measured-transfer claim.

## Definition of done

### Research MVP

- Один fresh internet exact object проходит C1–C9 или получает reproducible
  fallback-only result без ручного перебора звуков.
- Force→transfer decomposition и modal record подтверждены held groups.
- ML, если он нужен, выигрывает у всех compatible classical controls.
- Independent validator самостоятельно выдаёт tri-state decision с bounded
  false-pass risk и coverage.
- Accepted output печатает byte-identical clips; любой unsupported case имеет
  deterministic authored fallback.

### Physical Sound Base V1

- Один и тот же hash-closed pipeline повторён для glass, wood и metal.
- База хранит exact-domain формулы, uncertainties, OOD boundaries и provenance,
  а не субъективные material presets.
- Ни один dataset, checkpoint, WAV, capture или generated atlas не попадает в
  Git.
- Production runtime не обучается, не загружает neural weights и не влияет на
  authoritative simulation.
