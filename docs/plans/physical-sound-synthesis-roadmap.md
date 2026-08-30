# Roadmap V3: self-validating neural physical sound

| Поле | Значение |
| --- | --- |
| Дата rebaseline | 2026-08-30 |
| Статус | `ACTIVE_R&D / R0_COMPLETE / R1_CONTROLS_FROZEN / R2C_REJECTED / R2D_V1_FIXED_STEP_REJECTED / R2D_V2_DECAY_NEXT / PASS_DISABLED / P1_BLOCKED` |
| Архитектура | [SPEC-45](../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md), `Proposed` |
| Стратегия | [Neural acoustic field strategy](../development/physical-sound-neural-acoustic-field-strategy-2026-08-30.md) |
| Исполнение | [Neural acoustic field implementation plan](2026-08-30-physical-sound-neural-acoustic-field-implementation-plan.md) |
| Текущее состояние | [Physical sound task state](../development/task-state/physical-sound-synthesis.md) |
| Продуктовый статус | Изолированный post-v1 experiment; текущий clip-based audio baseline не меняется |

## North star

Построить автономный цикл, который по опубликованным internet data обучает
модель физического звука, автоматически проверяет её на независимых данных и
превращает принятый результат в компактную детерминированную модель для движка.

Первый целевой результат — один конкретный стеклянный объект с несколькими
позициями удара и слушателя. Для допустимых условий звук синтезируется из
cooked physical model; для неизвестных, ошибочных или out-of-domain условий
всегда используется authored clip.

Работа не считается завершённой, пока нет одновременно:

1. обученной модели с замороженным lineage;
2. преимущества над честным classical baseline на неиспользованных условиях;
3. независимого автоматического validator decision;
4. детерминированного cooker и byte-identical reference PCM;
5. одного exact admitted domain с обязательным fallback;
6. отдельного product decision перед любой runtime-интеграцией.

## Что меняется в V3

Ручной поиск общей формулы `material -> sound` остаётся закрытой основной
веткой. Q30 modal renderer, DCT residual, FEM/BEM и предыдущие real-data
эксперименты сохраняются как baseline, teacher, controls и negative knowledge.

V2 доказал, что наличие большой нейросети и физического loss само по себе не
решает задачу. Первый dense complex field воспроизводимо схлопнулся к почти
нулевому сигналу: обе модели проиграли даже trivial context predictors, хотя
context-only rank-96 oracle сохраняет `99.64%` энергии. Поэтому V3 меняет
порядок программы:

1. сначала objective/optimizer/cooker проходят context-only trainability gate;
2. затем сеть учит только spatial coefficients над замороженным low-rank
   complex basis;
3. grouped query открывается один раз только для прошедшей context revision;
4. physics regularization возвращается как ablation лишь после того, как
   data-only путь доказал сохранение энергии и обучаемость;
5. расширение к impact/object/material axes запрещено до победы над простыми
   интерполяционными controls на одном fixed-impact объекте.

Целевой кандидат остаётся offline neural transfer field, но training boundary
становится двухступенчатым:

```text
published transfer field
  -> context-only complex basis + energy-preserving normalization
  -> neural coordinate-to-coefficient field
  -> bounded modes/gains/residual
  -> deterministic cooker
  -> excitation convolution -> 48 kHz PCM
```

Force-deconvolved transfer response и обычный recorded impact waveform —
разные типы сигнала. Они не сравниваются sample-to-sample и не смешиваются в
одной loss без явной excitation model:

- transfer responses обучают отклик объекта и пространственное поле;
- recorded impacts проверяют итоговую perceptual identity и envelope;
- synthetic FEM/BEM дают физические controls и дополнительный teacher signal;
- direct waveform generation используется только как report-only upper bound
  либо как источник authored assets.

Первая модель не входит в runtime. Она работает во внешнем research pipeline,
а движок в будущем может получить только проверенные и канонически quantized
coefficients.

## Неизменяемые ограничения

- Training data, recordings, generated WAVs, weights, optimizer state и
  feature caches остаются вне Git.
- Пользователь ничего не записывает и не бьёт по объектам; real evidence
  добывается только из опубликованных internet sources.
- Отсутствующая geometry, material, support, excitation или listener metadata
  остаётся отсутствующей и сужает claim.
- Generator не читает validator calibration, method holdout или admission
  shadow.
- Validator не обучается на candidate outputs той же revision и не меняет
  thresholds после открытия shadow.
- Один pooled similarity score, одна audio-language model и человеческое
  прослушивание не могут самостоятельно выдать `Pass`.
- Невалидный, неопределённый или OOD результат всегда означает
  `FallbackOutOfDomain`.
- Physics и gameplay остаются authoritative. Audio presentation не меняет
  world state, replay, save или deterministic `AcousticFactV1`.
- Runtime neural inference, public schema и production integration требуют
  отдельного consumer-driven architecture decision.

## Артефакты конечного контура

| Артефакт | Что фиксирует | Где хранится |
| --- | --- | --- |
| `CorpusRevision` | source hashes, provenance, signal semantics, published axes и пять split roles | Manifest/report в Git; payloads снаружи |
| `ExperimentRevision` | preprocessing, features, model, environment, seeds, baseline и checkpoint lineage | Code/report в Git; weights/features снаружи |
| `ValidatorRelease` | hard gates, specialists, thresholds, OOD и grouped risk policy | Версионированный report/contract |
| `CookedAcousticModel` | bounded modes, damping, spatial gains, residual, domain envelope и fallback | Сначала external research artifact |
| `AdmissionRecord` | immutable `Pass`, `Reject` или `FallbackOutOfDomain` для exact domain | Reviewable evidence |

Каждое изменение corpus, model, preprocessing, cooker или validator создаёт
новую revision. Неудачные результаты не переписываются и остаются в базе как
запрет на повторение уже опровергнутой гипотезы.

## Карта программы

```mermaid
flowchart LR
    R0["R0. Real data boundary"] --> R1["R1. Honest baselines"]
    R1 --> R2D["R2D. Trainability gate"]
    R2D --> R2E["R2E. Low-rank neural field"]
    R2E --> R3["R3. Impact/listener few-shot model"]
    R3 --> R4["R4. Shared-object transfer"]
    R3 --> R5["R5. Independent validator"]
    R4 --> R5
    R5 --> R6["R6. Cooker and one-shot admission"]
    R6 --> R7["R7. Formula base V1"]
    R7 --> R8["R8. Production impact vertical"]
```

R4 является условной веткой. Если exact-object few-shot model работает, а
object-disjoint transfer нет, программа продолжает работу с exact-object
models и не маскирует провал обещанием универсального материала.

## Milestones

| ID | Статус | Размер | Проверяемый результат |
| --- | --- | ---: | --- |
| R0 | `COMPLETE` | S | Первый real transfer slice и five-role projection повторяются byte-identical; signal semantics и missing axes честно сохранены. |
| R1 | `COMPLETE / FROZEN_CONTROLS` | M | Transfer-domain и recorded-waveform baselines покрывают только совместимые rows; метрики и fallback заморожены. |
| R2C | `COMPLETE / REJECTED / REPRODUCIBLE` | M | Dense separable complex field и Helmholtz ablation завершены без выбранного candidate; silence-collapse локализован до generalization. |
| R2D | `V1_FIXED_STEP_REJECTED / V2_DECAY_NEXT` | S–M | Objective/cooker проходят; одна decayed-step revision должна закрыть неизменный log-energy gate без чтения query audio. |
| R2E | `BLOCKED_BY_R2D` | M | Coordinate network над frozen context-only low-rank basis лучше всех classical controls на grouped held listeners. |
| R3 | `BLOCKED_BY_R2_AND_DATA` | L | Few-shot model предсказывает новые impact/listener conditions exact объекта и cooks в exact PCM. |
| R4 | `CONDITIONAL` | L–XL | Shared geometry-conditioned model либо проходит object/family-disjoint holdout, либо zero-shot claim явно отклонён. |
| R5 | `BLOCKED_BY_R3` | M | Frozen automatic validator показывает bounded grouped risk и useful selective coverage без live human gate. |
| R6 | `BLOCKED_BY_R5` | M | Frozen generator/cooker/validator один раз открывают admission shadow и публикуют tri-state decision. |
| R7 | `BLOCKED_BY_R6` | L–XL | Есть exact admitted neural-cooked domains для glass, wood и metal либо явно зафиксированы недоступные families. |
| R8 | `POST_V1 / BLOCKED_BY_CONSUMER` | XL | Один visible prop использует production contact/content/mixer path с exact clip fallback. |

Размеры относительные; это порядок зависимостей, а не календарное обещание.

## R0 — Real data boundary

Цель — подготовить данные так, чтобы модель не обучалась на неверной
интерпретации сигнала или скрытой leakage.

Deliverables:

- schema V2 с явным `recorded_impact_waveform` или
  `force_deconvolved_transfer_response`;
- независимые optional claims для impact point и outward normal;
- verified REALIMPACT multi-listener slice с общим scale, сохраняющим
  относительную амплитуду между микрофонами;
- роли `train`, `development`, `calibration`, `method_holdout` и
  `admission_shadow` с object/source/recording/mutation-parent disjointness;
- capability report, где каждая отсутствующая axis указана явно;
- sealed rows представлены commitments, а не доступным audio payload.

Exit criteria:

- два запуска дают byte-identical manifests, reports и rendered references;
- stale lineage, non-finite samples, duplicate parents и cross-role leakage
  отвергаются до создания output;
- ни один transfer row не попадает в waveform baseline;
- выбран exact permitted internet slice для R1/R2;
- модель ещё не обучается, admission shadow не открывается.

Evidence: [R0–R1 real boundary](../development/physical-sound-neural-real-boundary-r0-r1-2026-08-30.md).
R0 закрыт: 15-row REALIMPACT slice и 23-row five-role projection повторены
byte-identical; все missing axes сохранены, sealed roles не materialized.

## R1 — Honest baselines and metric contract

Нужны две разные baseline surfaces:

1. transfer-domain baseline — nearest-neighbour/interpolation либо другой
   frozen classical field над impact/listener coordinates;
2. waveform-domain baseline — существующий Q30 modal/DCT или PCM-only path для
   совместимых recorded impacts и synthetic controls.

Одинаковые preprocessing и splits используются всеми будущими candidates.
Метрики замораживаются до обучения:

- multi-resolution log-spectrum и spectral-envelope error;
- modal frequency/damping error там, где extractor допустим;
- time-decay/envelope и residual autocorrelation;
- spatial relative-level and held-out-listener error;
- energy-scaling, silence, rest, separation, finiteness и exact-repeat gates;
- независимые learned-representation diagnostics только как часть ensemble;
- per-object/impact/listener distributions, а не только pooled mean;
- cooker size, offline latency и deterministic reference cost.

Exit criteria:

- каждый development row имеет `Supported` или точный
  `FallbackOutOfDomain` reason;
- baseline и feature reports повторяются byte-identical;
- metric direction, aggregation, primary endpoints и stop thresholds
  preregistered до R2;
- admission shadow остаётся sealed.

Evidence: [R0–R1 real boundary](../development/physical-sound-neural-real-boundary-r0-r1-2026-08-30.md).
R1 закрыт профилем `transfer-listener-field-r1-v1`: восемь context и семь
query listeners имеют повторяемые nearest/linear predictions. Linear control
достигает `9.1745 dB` mean gain-matched spectrum RMSE и `2.5373 dB` mean
absolute level error; эти числа не являются quality pass, а задают порог R2.

## R2 — Fixed-impact listener-field pilot

R2 использует один объект и один impact point с несколькими listener positions.
Он проверяет только пространственное поле и не притворяется полной моделью
удара. Milestone закрывается только моделью, которая обходит честную
интерполяцию; воспроизводимое обучение само по себе не является успехом.

Experiment ladder:

1. frozen nearest/linear controls;
2. direct time-domain rank-4/rank-7 listener latent — rejected;
3. coordinate-derived propagation-delay-aligned latent — rejected;
4. grouped dense complex/time-frequency separable SIREN — rejected after
   reproducible two-candidate training and one-shot query evaluation;
5. context trainability and objective gate — next;
6. frozen low-rank complex basis plus neural spatial coefficient field — only
   after the trainability gate passes.

Exit criteria:

- held-out listeners улучшаются относительно каждого frozen control по всем
  preregistered primary aggregates;
- relative amplitude, decay, finiteness и exact cook не регрессируют;
- ablation показывает, какой компонент даёт улучшение;
- training повторяется на фиксированных seeds в объявленном tolerance;
- failure публикуется как `REJECT_LISTENER_FIELD` или `DATA_INSUFFICIENT`, а не
  запускает свободный hyperparameter search.

Evidence: [V1 result](../development/physical-sound-listener-field-r2-v1-result-2026-08-30.md)
and [phase-aligned result and failure research](../development/physical-sound-listener-field-r2-phase-research-2026-08-30.md).
V1 and its separately preregistered propagation-delay-aligned successor both
repeat deterministically and return `RejectListenerField`. Phase alignment
makes rank 4 better than both controls on four of five endpoints, but it still
fails the frozen P95 spectrum endpoint. A query-informed subspace diagnostic
also fails the level/spectrum aggregates, so the opened time-domain latent
family is retired rather than tuned.

R2B is now complete. The [dense complex-field preflight](../development/physical-sound-r2b-dense-complex-field-preflight-2026-08-30.md)
projects the full 600-position fixed-impact REALIMPACT semicylinder, freezes
complete-angle-plane groups with `420 context / 180 query`, and repeats both
acquisition and 288 MB preflight trees byte-identically. Context-only
normalization excludes all query rows; the complex STFT inverse reaches
`-153.348 dB` worst NRMSE and at most one PCM16 LSB. Three controls are
measured before optimization, while method holdout and admission shadow remain
sealed. This closes data/representation readiness only.

R2C is now complete and rejected. The [dense complex-field result and bounded
failure research](../development/physical-sound-listener-field-r2c-result-2026-08-30.md)
records byte-identical repetitions for the data-only and `0.0001` Helmholtz
candidates. They produce about `53 dB` mean level error and `27 dB` mean
spectrum error, passing only the near-zero waveform endpoint. Helmholtz is not
the primary cause: both candidates collapse alike.

The context diagnostic changes the next hypothesis. Rank 96 can retain
`99.6396%` of context energy with Frobenius NRMSE `0.0600`, but the trained
data-only full-context objective is `1.0498x` the zero predictor and every
logged step reaches gradient clipping. The joint separable SIREN has therefore
failed before spatial generalization. Width, rank, step, seed and physics-loss
grids on the opened query are forbidden.

### R2D — Context trainability and objective gate

R2D reads only context data and performs no grouped-query candidate evaluation.
It freezes an energy-preserving objective, sampling policy and optimizer
diagnostics, then proves them in increasing order:

1. exact identity/cooker control;
2. one-row micro-overfit;
3. small spatial-block micro-overfit;
4. full-context fit against zero, global-mean and context-only rank oracles;
5. deterministic repeat with clipping/gradient and emitted-PCM metrics.

The profile must explicitly measure absolute RMS level, multi-resolution
spectrum, complex reconstruction, waveform NRMSE, active-bin coverage and
gradient clipping. A candidate cannot pass merely because a sampled loss falls.
All numeric thresholds are frozen from context controls before optimization;
query audio, method holdout and admission shadow reads remain zero.

Exit criteria:

- one-row and small-block controls reconstruct through the real inverse/PCM
  cooker within their preregistered bounds;
- the full-context result strictly improves both zero and global-mean controls
  and approaches the declared context-only low-rank oracle envelope;
- signal level does not collapse and no non-finite output occurs;
- two runs reproduce under the declared deterministic/tolerance policy;
- failure returns `REJECT_TRAINING_SUBSTRATE`, without opening query audio.

[R2D V1](../development/physical-sound-listener-field-r2d-trainability-v1-result-2026-08-30.md)
is complete and reproducible. The rank-96 basis retains `99.6396%` energy and
is orthonormal within `1.67e-6`; one-row fit and every coefficient, cooker,
oracle-proximity, clipping and trivial-control gate pass. The eight-row and
full-context tasks miss only the unchanged mean absolute log-energy limit:
`0.007785` and `0.005062` versus `0.005`. V1 remains rejected. The next
revision may change only the fixed learning rate to a preregistered decay
schedule; it may not relax thresholds, add query reads or alter the basis/loss.

### R2E — Low-rank neural spatial coefficient field

R2E begins only after R2D passes. It computes one frozen complex basis from
context rows only, then learns `listener coordinates -> complex basis
coefficients`. The time/frequency basis is not learned jointly with the
coordinate field in this revision. Non-neural coefficient interpolation and
the three original waveform controls remain explicit baselines.

One data-only candidate is trained and repeated before one frozen evaluation
on all 180 grouped queries. Physics regularization is deferred until this
candidate preserves context energy and demonstrates a held-listener advantage;
it cannot rescue a failed trainability substrate.

Exit criteria:

- all R2D trainability gates remain green under the final R2E path;
- query audio affects only the single frozen evaluation;
- the candidate is strictly better than every frozen control on all unchanged
  five primary aggregates;
- cooked outputs and reports repeat under the frozen policy;
- otherwise return `REJECT_LOW_RANK_COEFFICIENT_FIELD` or
  `DATA_INSUFFICIENT`, preserve the counterexample and stop R2.

Only a grouped held-listener result that beats all three frozen controls on all unchanged
primary aggregates can close R2. `DATA_INSUFFICIENT` and
`REJECT_COMPLEX_FIELD_REPRESENTATION` remain valid outcomes.

## R3 — Object-specific impact/listener few-shot model

Entry condition: R2 подтверждает representation, а опубликованный corpus даёт
несколько impact и listener conditions одного объекта. Если impact axis в
источнике отсутствует, milestone остаётся blocked, а metadata не выдумывается.

Model inputs:

- exact object/geometry revision;
- available material/support evidence;
- impact point и имеющийся excitation descriptor;
- listener coordinate/condition.

Model outputs:

- global modal frequencies and damping;
- impact/listener-conditioned gains;
- compact coloured residual;
- uncertainty, coverage distance и OOD reason.

Exit criteria:

- unseen impact positions и listeners лучше R1 baseline;
- bounded excitation scaling и negative controls проходят;
- prediction cooks в canonical bounded coefficients;
- одинаковый cooked record создаёт byte-identical 48 kHz PCM;
- method holdout открывается один раз только после freeze candidate;
- результат — `GO_EXACT_OBJECT`, `REJECT_REPRESENTATION` или
  `DATA_INSUFFICIENT`.

## R4 — Shared geometry-conditioned transfer

Entry condition: R3 доказал exact-object representation. Shared model получает
geometry encoder и проверяется на object- и family-disjoint method holdout.

Обязательные ablations:

- real-only training;
- synthetic FEM/BEM pretraining plus real adaptation;
- object-specific few-shot adaptation;
- shared zero-shot prediction;
- direct waveform model как report-only perceptual upper bound, если его можно
  воспроизвести без нарушения split policy.

Exit criteria:

- `GO_SHARED` выдаётся только при per-object improvement и calibrated OOD;
- pooled mean не скрывает провал отдельных объектов;
- source-project или near-duplicate leakage отсутствует;
- провал shared model оставляет допустимым `GO_EXACT_OBJECT`, но закрывает
  zero-shot/material-wide claim.

## R5 — Independent Validator Release V1

Validator — frozen ensemble, а не одна judge-network:

| Слой | Роль |
| --- | --- |
| Hard/causal gates | Finiteness, bounds, exact repeat, silence/rest/separation, energy relations |
| Acoustic specialists | Modes, decay, envelope, spectral evolution, spatial field, transient/residual |
| Learned representations | Real-corpus similarity и artifact evidence; не самостоятельный `Pass` |
| OOD/selective controller | Принимает только calibrated covered region |
| Grouped risk report | Confidence-bounded false-pass risk и useful coverage по независимым parent groups |

Exit criteria:

- calibration выбирает policy без holdout/shadow;
- frozen mutations, positives, negatives, unavailable-model и reward-hack
  controls имеют declared outcomes;
- report повторяется byte-identical;
- threshold нельзя менять после holdout/shadow;
- если useful coverage несовместима с bounded risk, release честно остаётся
  fallback-only.

## R6 — Neural cooker and one-shot admission

Frozen model, cooker и Validator Release встречаются на untouched admission
shadow один раз.

Cooker обязан проверить finiteness, counts, coordinate envelope, canonical mode
order, duplicate/unstable modes, coefficient bounds, quantization и fallback.
Model failure, missing artifact или OOD не являются audio fault: выбирается
authored clip.

Exit criteria:

- run manifest связывает corpus/model/checkpoint/code/environment/seed/cooker/
  validator hashes;
- validator публикует immutable `Pass`, `Reject` или
  `FallbackOutOfDomain`;
- accepted coefficients и reference PCM повторяются без human approval;
- та же revision не донастраивается по открытому shadow.

## R7 — Neural-cooked formula base V1

База хранит не общие коэффициенты материала, а independently admitted exact
domains:

1. thin glass vessel;
2. dry hardwood block;
3. thin metal vessel/shell.

Каждая запись содержит geometry/support/excitation/listener envelope,
model/checkpoint/cooker/validator revisions, cooked coefficients, risk,
coverage, cost, evidence hashes и authored fallback. Новая geometry, fixture,
energy band или listener envelope создаёт новую admission revision.

V1 считается готовой, если для каждой family либо существует один exact
admitted domain, либо зафиксировано воспроизводимое решение, почему family
остаётся fallback-only. Широкий claim `glass`, `wood` или `metal` не выводится
из одного объекта.

## R8 — Production rigid-impact vertical

Runtime work начинается только после отдельного slot в основном roadmap и
consumer-driven ADR. Recommended consumer — один visible interactable prop с
несколькими impact positions/energies.

Required product work:

1. закрыть engine-owned committed contact projection вместо raw PhysX callback;
2. определить минимальные cooked PresentationOnly content shapes;
3. импортировать один admitted record без datasets/weights/validator runtime;
4. подключить bounded extraction, deterministic voice и текущий mixer;
5. проверить enabled/disabled/OOD/fault/voice-limit clip fallbacks;
6. измерить whole-mixer p95/p99, memory, queue и voice bounds;
7. пройти candidate `AUDIO-PHYS-SOURCE-P1`, `AUDIO-PHYS-CONTENT-P1` и
   `AUDIO-PHYS-PCM-P1` checks.

Enabled и disabled paths должны сохранять одинаковые gameplay, physics,
ledger, persistence и `AcousticFactV1` roots.

## Текущие commit boundaries

1. `real neural transfer slice` — `COMPLETE`; schema V2, verified REALIMPACT
   slice, five-role projection и R0 evidence повторены;
2. `transfer-domain classical baseline` — `COMPLETE`; compatible controls,
   preprocessing, metrics и real development report заморожены;
3. `listener-field experiment v1` — `REJECTED`; two-run training, MLflow
   lineage и Rust-metric evaluation повторены, candidate не выбран;
4. `phase-aligned listener successor` — `REJECTED`; training/evaluation and
   failure diagnostic repeat, while the frozen conjunctive rule selects no
   candidate;
5. `dense complex-field data preflight` — `COMPLETE`; acquisition and full
   representation/control trees repeat byte-identically, with query-isolated
   normalization and zero optimizer steps;
6. `dense complex-field physics ablation` — `COMPLETE / REJECTED`; both frozen
   candidates and their checkpoints repeat, one-shot evaluation selects none,
   and the context diagnostic attributes the primary failure to loss/sampling/
   clipped optimization rather than Helmholtz or rank-96 capacity;
7. `context trainability gate V1` — `COMPLETE / REJECTED`; all gates except
   small-block/full-context log energy pass byte-identically, with zero query
   reads;
8. `context trainability gate V2` — `NEXT`; retain basis/objective/tasks/gates
   and change only fixed AdamW learning rate to deterministic decay;
9. `low-rank coefficient field` — `BLOCKED_BY_8`; freeze the successful
   context protocol, train one coordinate-to-coefficient model, repeat it, then
   evaluate once against all unchanged controls and endpoints.

После каждого boundary обновляются exact evidence, task state и этот roadmap.
Успешный commit без измеренного exit criterion не меняет milestone status.

## Stop/go policy

| Наблюдение | Решение |
| --- | --- |
| Transfer и recorded-waveform semantics нельзя согласовать | Не смешивать losses; сузить task или добавить явную excitation model |
| Context fit не обходит zero/global mean или теряет signal energy | `REJECT_TRAINING_SUBSTRATE`; query не открывать |
| R2E не превосходит classical interpolation | `REJECT_LISTENER_FIELD`; сохранить counterexample и остановить этот field, не запускать tuning-grid |
| R3 проходит exact object, R4 падает object-disjoint | `GO_EXACT_OBJECT`; zero-shot/shared claim закрыть |
| Direct waveform звучит лучше, но не проходит causal/exact cook | Оставить upper bound или authored asset source |
| Internet data не содержит нужную axis | `DATA_INSUFFICIENT`; искать другой published source, не local capture |
| Learned metric расходится с hard/acoustic specialists | Fallback; одна model score не перевешивает disagreement |
| Frozen candidate падает на shadow | `Reject`; сохранить counterexample и открыть новую revision с одной новой hypothesis |
| Ни neural, ни classical path не дают bounded quality/cost | Остановить domain и использовать authored clips |
| Нет concrete consumer или roadmap slot | Не создавать public/runtime contract |

## Definition of done

- **Training substrate:** R2D автоматически доказывает, что objective,
  optimizer и cooker сохраняют сигнал до любой query оценки.
- **Research model:** R2E/R3 воспроизводимо поддерживает listener/exact-object
  claim или честно отклоняет representation.
- **Automatic validation:** R5 принимает решения без per-sound human queue и
  показывает confidence-bounded grouped risk.
- **Closed research loop:** R6 один раз встречает frozen generator и validator
  на shadow и публикует immutable tri-state decision.
- **Formula base:** R7 хранит bounded cooked domains, negative knowledge и
  fallbacks без runtime neural inference.
- **Product value:** R8 даёт один visible prop, физически реагирующий на место и
  силу удара, при полном сохранении clip fallback и authoritative roots.

Rolling, scraping, fracture, footsteps, cloth, liquids, fire, voice и
biological sound не входят в этот roadmap. Для каждого потребуется отдельный
source model, corpus и validator domain после успешного impact vertical.
