# ADR-043: Codegen-proven non-reentrant count-bearing allocator callbacks

| Поле | Значение |
|---|---|
| ID | ADR-043 |
| Статус | Superseded |
| Версия | 1.0 |
| Дата решения | 2026-08-01 |
| Последняя проверка | 2026-08-01 |
| Нормативные зависимости | [SPEC-00](../00-product-contract.md), [SPEC-01](../01-system-architecture.md), [SPEC-09](../09-tooling-sdk-and-observability.md), [SPEC-12](../12-vertical-slice-conformance.md), [SPEC-23](../23-jobs-memory-resource-residency-and-io-backpressure.md), [ADR-030](030-product-first-development-and-lightweight-validation.md), [ADR-036](036-thoth-reference-performance-profile.md), [ADR-039](039-tooling-only-process-wide-system-global-allocator-measurement.md), [ADR-040](040-fixed-tls-sharded-global-allocator-measurement.md), [ADR-041](041-owner-thread-quiescent-global-allocator-measurement.md), [ADR-042](042-unobserved-deallocation-system-pass-through.md) |
| Заменяет | Узко заменяет обязательные per-call recursion-flag get/test/set/clear/reject clauses ADR-040 и ADR-041 только для count-bearing `alloc`/`alloc_zeroed`/`realloc` на target/build, прошедшем non-reentrancy admission ниже. Также явно заменяет отклонение варианта «Удалить только recursion flag» в ADR-042: после реализации direct `dealloc` pass-through новый immutable candidate-6 и exact callback-count evidence дают отдельную проверяемую гипотезу, а source+IR+ASM/backend proof устраняет ранее неограниченный recursion risk. Foreign admission/order ADR-040, owner cookie/counters ADR-041, direct `dealloc` ADR-042, tooling-only unsafe boundary ADR-039, exact report semantics, `3%`/`64 MiB` limits и fail-closed fallback не меняются. |
| Заменён | Полностью заменён [ADR-049](049-performance-evidence-without-allocator-instrumentation.md); allocator instrumentation удалена. |

## Контекст и immutable candidate-6

ADR-042 удалил всю measurement state machine из unobserved `dealloc`, сохранив
её только для трёх callbacks, которые способны изменить published gross
counter. Первый и единственный полный performance candidate этой реализации
сохранён как
`allocator-counter-check-candidate-6-20260801/allocator-counter-check-v1.json`
с SHA-256
`004DBE9238413E538C7EC84E6AF718123E5EE30EDB59935C84694AFF43A188BE` и дал:

- `System` median `1 653 928 600 ns`;
- inactive wrapper median `1 623 126 700 ns`, `-186 bp` (`-1.86%`), `PASS`;
- enabled wrapper median `1 733 485 600 ns`, `+481 bp` (`+4.81%`), `FAIL`;
- reserved state `786 472 B`, `PASS`;
- exact state/archive/identity/kernel roots не изменились.

Один diagnostic enabled run того же неизменного kernel наблюдал `15 253 608`
successful count-bearing callbacks: `9 051 775 alloc`, `0 alloc_zeroed` и
`6 201 833 realloc`, всего `1 695 357 014` requested bytes. Aggregate enabled
premium относительно System равен примерно `5.216 ns` на observed callback.
Неизменный `3%` budget допускает примерно `3.253 ns` на callback; разрыв —
примерно `1.963 ns` на callback.

Эти отношения являются только aggregate diagnostic: они включают owner/foreign
dispatch, TLS, admission, backend calls, checked counter stores и прочие
instructions. Они не измеряют стоимость recursion flag изолированно и не
предсказывают candidate-7 `PASS`. Candidate-6 immutable, не повторяется и не
отбрасывается как noise.

ADR-042 отклонил удаление одного recursion flag, потому что candidate-5 ещё
содержал лишнюю `dealloc` state machine, а отсутствие runtime guard не было
заменено доказательством non-reentrancy. После direct dealloc pass-through
remaining flag исполняется только на миллионах count-bearing callbacks. Новый
вариант допустим не из-за одной оценки стоимости, а только при более сильном
предварительном доказательстве: instrumentation call graph allocation-free, а
pinned `std::alloc::System` backend не может повторно войти в Rust global
allocator wrapper.

## Решение

### Неизменный observed domain

- Published values по-прежнему считают только successful `alloc`,
  `alloc_zeroed` и `realloc` calls, фактически дошедшие до process global
  wrapper в exact active scenario window.
- Owner callback сохраняет exact ADR-041 cookie и owner-only checked counters.
  Foreign callback сохраняет ADR-040 fixed-slot claim, odd admission RMW,
  identity postcheck, even publication и close handshake.
- Counter/window/sequence/slot overflow, TLS access/claim failure, wrong token,
  PID mismatch, abandoned window и close timeout продолжают fail closed и не
  публикуют partial values.
- `dealloc` остаётся безусловным exactly-once direct `System::dealloc` по
  ADR-042 и не читает state/TLS/slot/fault data.
- `ProcessAllocationCounterV1`, gross checked aggregates,
  `deallocation_semantics = "delegated-not-subtracted"`, process/PID/window
  scope и public performance schemas не меняются.

### Exact Windows non-reentrancy admission

Recursion-free active measurement допускается только для exact
`x86_64-pc-windows-msvc` build на pinned Rust `1.93.0`, где один bounded gate
до timing и до `Active` одновременно доказывает все условия ниже.

1. Source boundary остаётся std-only `tools/process-allocation-counter` с
   единственным production reverse dependency `xtask`. Production hook и
   minimal codegen probe устанавливают один и тот же
   `ProcessAllocationCounter::system()` type; source gate отклоняет иной global
   allocator, wrapper helper или consumer.
2. Call graph трёх outer count-bearing callbacks и всех reachable hot/cold
   instrumentation helpers не содержит allocation, formatting/logging, lock,
   process/thread API, panic/unwind, dynamic initialization или вызов, который
   может вернуться в Rust global allocator.
3. Release LLVM IR и assembly подтверждают отсутствие обращения к
   `in_callback`, recursion test/rejection branch и recursion-fault publication
   на owner и foreign paths. Остальные owner/foreign state, TLS, slot, atomic,
   postcheck, counter, fault и close operations обязаны точно соответствовать
   ADR-040/ADR-041.
4. Pinned Windows `System` path разрешается только в доказанном
   non-interposed shape: `alloc`/`alloc_zeroed` доходят до native process-heap
   `HeapAlloc`, `realloc` — до `HeapReAlloc`, без custom allocator/interposition,
   callback hook или обратного ребра в Rust wrapper. `alloc_zeroed` сохраняет
   zeroing semantics; pointer/null/data parity проверяется отдельно.
5. Gate сам задаёт fixed codegen command identity: exact `rustc -Vv`
   release/commit/host, locked graph, release profile, Windows target, emitted
   IR/assembly и bounded single-job build. Унаследованные `RUSTFLAGS`/
   `CARGO_ENCODED_RUSTFLAGS`, `RUSTC_WRAPPER`/`RUSTC_WORKSPACE_WRAPPER`,
   `CARGO_PROFILE_*` и linker/interposition overrides обязаны либо быть
   доказанно пустыми/отсутствующими, либо fail closed до build; произвольные
   effective flags не считаются частью принятого proof. Toolchain, target,
   fixed command, linker/backend или reachable call-graph change invalidates
   admission до нового source+IR+ASM audit.

Gate проверяет generated artifacts, а не имя source symbol или исходное
намерение. Если любой symbol неоднозначен, artifact отсутствует/слишком велик,
backend может быть interposed/reentrant либо proof неполон, `begin()` не
публикует `Active`: counter на этом target/build возвращает stable unavailable
diagnostic, а содержащий run — `NOT_RUN` без allocator values.

Native Linux `x86_64-unknown-linux-gnu` не наследует Windows proof и этим ADR
не admitted. `LNX-006` может собрать отдельное
`std::alloc::System`/libc/linker/interposition evidence только как основание
для нового Accepted target-extension ADR; до его принятия Linux allocator
metric остаётся `NOT_RUN` независимо от Windows candidate-7.

### Count-bearing callback без dynamic recursion guard

На admitted build три count-bearing callbacks не выполняют per-call
`in_callback` TLS get/test/set/clear и не имеют dynamic recursion rejection.

Owner success path после exact Active load и cookie selection:

1. вызывает ровно один matching `System` method;
2. при non-null result checked-обновляет один owner count и bytes;
3. возвращает исходный allocator result без recursion epilogue.

Foreign path сохраняет прежний порядок:

1. exact Active load и const-TLS slot selection/claim;
2. odd `SeqCst` admission RMW и exact identity postcheck;
3. ровно один matching `System` call и checked slot counter update при success;
4. even `Release` publication для close handshake.

Удаление flag не разрешает убирать owner cookie, foreign TLS selector, slot
capacity, admission/postcheck, checked arithmetic, sticky faults или close
coherence. Stable recursion fault больше не является per-call recovery
mechanism на admitted build: reentrancy исключена до Active. Synthetic custom,
hooked или reentrant backend обязан провалить admission, а не проверять
поведение уже активного recursion path.

### Codegen evidence protocol

Windows audit строит locked release IR/assembly с одним job и без унаследованных
Cargo/make jobserver descriptors. Он отдельно собирает allocator library и
minimal std-only `allocator_counter_codegen_probe` внутри allocator package;
полный `xtask` dependency graph не является частью backend-shape proof.
Production `xtask` hook остаётся отдельно source-gated к тому же allocator
type.

Protocol-valid actual Windows IR/ASM audit после этой изоляции завершился
`PASS` за `1.63 s` для текущей pre-change реализации ADR-042. Он валидирует
bounded probe/build protocol и существующие dealloc/owner/foreign shapes, но не
является будущим ADR-043 recursion-free codegen proof. Более ранняя попытка
строить full `xtask` была внешне остановлена после `10 min`; это
invalid/`NOT_RUN` attempt, не codegen `FAIL`, и её scratch сохраняется только
для диагностики. Timeout нельзя повторно интерпретировать как evidence или
скрывать новым run.

### Единственный candidate-7

После implementation выполняется ровно один полный candidate-7 на том же
неизменном rotated `2 warmup + 15 retained` release kernel. Он начинается
только после focused exactness/fault/race/capacity tests, source/boundary gate,
actual pinned Windows IR/ASM/backend admission, pointer/null/data parity и
authoritative root parity.

Candidate-7 сохраняется независимо от verdict и не повторяется как
retry-to-green. Для `PASS` одновременно обязательны:

- inactive и enabled medians независимо `<=3%` относительно System;
- reserved instrumentation state `<=64 MiB`;
- exact count/byte breakdown и checked aggregates;
- unchanged pointer/null/data behavior и exact authoritative roots;
- один и тот же pinned source/toolchain/target/backend identity во всех proof и
  timing artifacts.

Любой failure оставляет allocator metric disabled, performance hard scenarios
`NOT_RUN`, B-12 `OPEN` и обычный shipping allocator неизменным. Новый timing
candidate после candidate-7 требует нового material implementation hypothesis,
а semantic изменение — нового Accepted ADR.

## Memory-model invariants

1. Non-reentrancy является admission fact exact target/build, а не
   предположением внутри callback. Build без admission не открывает window.
2. Owner cookie/counter reset остаётся sequenced-before Active, owner writes —
   same-thread finish reads; removal recursion TLS не меняет этот order.
3. Foreign odd admission, identity postcheck, Closing CAS, even publication и
   close handshake сохраняют единый ADR-040 SC order.
4. Проверенные backend calls не имеют обратного ребра к wrapper; поэтому один
   outer callback не может одновременно стать nested owner/foreign callback.
5. Overflow/TLS/slot/window/PID/close fault предотвращает snapshot publication
   как раньше. Отсутствие dynamic recursion fault не превращает partial state в
   valid report.
6. Direct `dealloc` остаётся вне measurement memory/order по ADR-042.

## Product impact

Решение проверяет узкую performance-гипотезу для optional tooling counter и не
меняет gameplay, persistence, replay, scheduling, public contracts или
shipping allocator. При доказанном Windows backend оно убирает работу с
каждого count-bearing callback; реальный выигрыш определяет только candidate-7.
Любая недоказанная platform продолжает выполнять игру с обычным `System`, но
без allocator metric.

## Relevant product checks

| Check | Scenario | Expected | Fallback |
|---|---|---|---|
| `fast` | known-count owner+foreign operations, rotating windows/owners, shared logical capacity/exhaustion, close races, stale window, TLS/slot/counter/window/PID/token faults, synthetic reentrant backend admission | Exact values либо один из остальных существующих stable faults; reentrant/unproved backend rejected до Active; no partial snapshot | Counter unavailable |
| Source/boundary audit | allocator crate, minimal probe и production xtask hook | Один std-only allocator type, xtask-only production consumer, no allocation/log/lock/panic/reentrant edge in callback call graph | Counter unavailable |
| Boundary/codegen audit | pinned Rust 1.93.0 Windows release IR/ASM/backend | No `in_callback` access/branch/fault; owner/foreign ADR-041/040 shapes intact; direct non-interposed HeapAlloc/HeapReAlloc; ADR-042 dealloc unchanged | Target `NOT_RUN` |
| `allocator-counter-check` | unchanged rotated 2+15 release kernel, единственный candidate-7 | Inactive/enabled independently `<=3%`, state `<=64 MiB`, exact roots/counts and pointer/null/data parity | Metric disabled; no retry-to-green |
| `performance` smoke/worker/long | admitted prepared in-process scenario window | Nonzero exact process/PID/window counter with explicit deallocation semantics | Stable unavailable diagnostic and `NOT_RUN` |
| `play` / `persistence-replay` | counter/profiler off/on | Exact command/event/save/replay/state/ledger roots | Reject divergent run |

## Рассмотренные варианты

- **Принять candidate-6 `+4.81%`.** Отклонено: неизменно нарушает `3%` gate.
- **Удалить recursion flag только по timing estimate.** Отклонено: aggregate
  `5.216 ns/callback` не изолирует flag и не доказывает non-reentrancy. Решение
  требует independent source+IR+ASM/backend admission.
- **Считать `System` non-reentrant на всех targets.** Отклонено: Linux libc,
  linker interposition, другой compiler/backend shape и custom hooks не
  наследуют Windows proof.
- **Сохранить flag только у foreign callbacks.** Отклонено: admitted backend
  исключает recursion до owner/foreign selection, а foreign admission уже
  защищает close completeness, не callback recursion.
- **Ослабить exactness, поднять budget или заменить shipping allocator.**
  Отклонено ADR-036/ADR-039 и retained evidence policy.

## Последствия

- Следующая code subphase удаляет только count-bearing `in_callback`
  machinery и recursion-specific validators/tests, добавляя target/build
  non-reentrancy admission. Owner/foreign count protocol не переписывается.
- Codegen gate продолжает использовать isolated minimal probe и отдельно
  source-bind production xtask hook; external timeout остаётся `NOT_RUN`.
- Candidate-7 является единственным новым full overhead verdict после
  implementation.
- До candidate-7 `PASS` allocator fields unavailable, hard timing calibration
  не начинается и B-12 остаётся `OPEN`.
- `LNX-006` собирает independent Linux backend non-reentrancy/codegen/overhead
  evidence для отдельного Accepted target-extension ADR; сам ADR-043 Linux не
  admits, а Windows result его не подменяет.
- Shipping dependency graph, allocator policy, unsafe boundary и
  `ProcessAllocationCounterV1` не меняются.

## Implementation record 2026-08-01

Code subphase завершена: per-call `in_callback` get/test/set/clear/reject
удалён из трёх count-bearing callbacks, recursion TLS field и
recursion-specific validators/tests убраны, owner cookie/counters, foreign
admission/postcheck/close handshake, direct `dealloc` и stable diagnostics не
изменились. Source/boundary gate отклоняет recursion machinery; codegen
admission fail closed при non-empty inherited compiler/wrapper/profile/linker
overrides. Focused exactness/fault/race/capacity tests, boundary-scan,
host-check и actual pinned Windows release IR/ASM/backend audit (`PASS`,
`1.41 s`) проходят.

Единственный candidate-7 сохранён как
`allocator-counter-check-candidate-7-20260801/allocator-counter-check-v1.json`
с SHA-256
`658D834BFEF9707655115759EE50576B62FF88FBE10B6D6790B61474A5A14ABC`:
System median `1 359 449 600 ns`, inactive `1 352 539 300 ns` (`-50 bp`,
`PASS`), enabled `1 417 907 600 ns` (`+430 bp`, `FAIL`), reserved state
`786 472 B` (`PASS`), authoritative roots unchanged во всех `51` run.
Вердикт `FAIL` по неизменному `3%` budget: allocator metric остаётся
disabled, hard scenarios `NOT_RUN`, B-12 `OPEN`. Candidate immutable и не
повторяется; новый timing candidate требует новой material implementation
hypothesis, а semantic изменение — нового Accepted ADR.
