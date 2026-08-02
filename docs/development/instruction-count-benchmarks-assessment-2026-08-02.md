# Instruction-count бенчмарки (iai-callgrind / gungraun) — оценка применимости

Дата: 2026-08-02. Статус: assessment note, не gate и не roadmap commitment.
Пункт T2 из [плана применения оптимизаций](../plans/2026-08-02-optimization-application-plan.md)
(Этап 0, «оценить, не внедрять как gate»).

## Что это

`iai-callgrind` и его преемник `gungraun` прогоняют `#[bench]`-функции под
valgrind/callgrind и возвращают **instruction counts** (и по желанию
cache simulation, DHAT, massif) вместо wall time. Для одного и того же
бинаря и входа число исполненных инструкций детерминировано — метрика
практически не шумит между прогонами.

## Зачем это нам

Наш decision protocol (goal prompt 2026-08-01) опирается на paired
same-hour A/B wall time с noise band <2%. Практика серии 08-01 показала
уязвимость: нагруженный хост даёт uniform shift незатронутых метрик
(±35% на driver-prepare), а allocator-кандидаты 6/7 ловили FAIL на
+4.3–4.8% — сигнал того же порядка, что и шум нагруженной системы.
Instruction counts закрывают именно эту дыру: регрессия «+N инструкций
в горячем encode» видна детерминированно, без погоды на машине.

Готовые драйверы у нас уже есть: детерминированные headless/replay
сценарии с фиксированными входами (neutral fixture, recorded manifests) —
идеальные тела для `#[bench]` (replay loop, ledger encode/commit,
checkpoint materialization).

## Ограничения

1. **Платформа.** callgrind — это valgrind: Linux (частично macOS), не
   Windows. На THOTH (ref-win-thoth-v1, Windows shipping target) нативно
   не запускается; WSL2 возможен, но это уже не измерение shipping target.
   Поэтому роль — **вспомогательный сигнал на Linux-ветке** (натурально
   прилегает к native-gate работам LNX-005/LNX-006), никогда не замена
   THOTH. ADR-036 wall-time authority не меняется.
2. **Слепые зоны метрики.** Instruction count не видит стоимость cache
   misses, branch misprediction, syscall и I/O — а наши известные
   hotspot'ы включают checkpoint encode/write с реальным I/O и allocator
   traffic. Cache-simulation режим частично компенсирует, но это модель,
   не железо. Улучшение «−15% инструкций» не гарантирует −15% wall time.
3. **Не authoritative.** Только REPORT_ONLY tooling сигнал; никакой
   связи с authoritative результатом (SPEC-23).

## Вывод и условия применения

Принять позже как REPORT_ONLY dev-signal на Linux runner, когда
native-gate (LNX-006) даст постоянный Linux execution environment.
Первые кандидаты на harness: ledger/identity-index encode-commit,
replay loop (post-C4), checkpoint materialization — по одному bench на
hotspot, с порогом «instruction regression >1% = повод для wall-time
A/B на THOTH», не наоборот. Оценка усилий: ~1 день на bench target +
извлечение сценария из существующих fixtures. До появления Linux runner
первичным остаются paired same-hour A/B и ten-run calibration (D5);
Windows-native замены с comparable noise immunity нет (ETW/xperf —
тоже wall time).

Заметка не открывает работу: внедрение — отдельным изменением после
LNX-006 или явного решения о WSL-based dev loop.
