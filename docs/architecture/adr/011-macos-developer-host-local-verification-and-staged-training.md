# ADR-011: macOS developer host and staged training smoke

| Поле | Значение |
|---|---|
| ID | ADR-011 |
| Статус | Accepted |
| Версия | 1.1 |
| Дата решения | 2026-07-22 |
| Последняя проверка | 2026-07-25 |
| Нормативные зависимости | [ADR-001](001-product-repository-license-and-platforms.md), [SPEC-09](../09-tooling-sdk-and-observability.md), [SPEC-14](../14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [ADR-030](030-product-first-development-and-lightweight-validation.md), [ADR-066](066-contact-centric-physical-skill-and-morphology-conditioned-motor-architecture.md) |
| Заменяет | отсутствует; уточняет platform scope ADR-001 без добавления shipping target |
| Заменён | частично [ADR-030](030-product-first-development-and-lightweight-validation.md) |

## Частичное supersession ADR-030

[ADR-030](030-product-first-development-and-lightweight-validation.md)
заменяет прежний формальный lifecycle проверок. Apple Silicon macOS остаётся
developer-host tier, Windows/Linux остаются v1 shipping targets, а MPS/CPU
smoke используется как ограниченная проверка train/export toolchain. Learned
policy допускается в игровой путь только после релевантных compatibility,
runtime и safety checks и всегда имеет deterministic fallback.

## Контекст

Bootstrap Next Engine выполняется на Apple Silicon Mac без remote repository и
CI orchestration. ADR-001 не включает macOS в v1 shipping platforms, но
разрешает portable developer tools и engine crates на этом host. Rust
поддерживает `aarch64-apple-darwin`, а PyTorch предоставляет MPS backend.
Полный Isaac Lab/Isaac Sim stack ориентирован на Windows/Linux и NVIDIA GPU,
поэтому Mac не является source of truth для production physical-policy quality.

## Решение

- `macOS aarch64` является `DeveloperHostTier`, а не shipping target.
- Windows x86_64 и Linux x86_64 остаются единственными v1 shipping targets;
  macOS package не входит в v1 scope.
- `cargo run -p xtask -- host-check` является основным широким локальным check
  перед handoff. Будущая CI может запускать те же project-owned команды, но не
  создаёт альтернативную semantics.
- Mac собирает portable contracts, runtime, headless tooling и `xtask`;
  Apple/vendor types не выходят в public contracts.
- Mac training ограничивается deterministic generated 2-DoF smoke, bounded
  MPS/CPU optimization, ONNX export и parity. Он проверяет toolchain, но не
  заменяет runtime/training physics correspondence или проверку creature skill.
- PyTorch MPS остаётся `Proposed` development-only backend. CPU является
  обязательным fallback для smoke; выбранное устройство записывается в run
  metadata.
- Production training требует подходящего Linux/NVIDIA environment. Пока его
  нет, игровой runtime использует procedural controller или другой
  deterministic in-process fallback.
- Isaac Lab остаётся `Proposed`, не устанавливается на Mac и оценивается только
  на совместимом Linux/NVIDIA host.
- Local datasets, checkpoints, training runs, imported installations и media
  находятся вне Git index.

## Рассмотренные варианты

- **macOS как третий v1 shipping target** — `Rejected`: потребовал бы renderer,
  platform backend и отдельную package matrix до появления продуктовой
  необходимости.
- **Isaac Lab на Apple Silicon как baseline** — `Rejected`: upstream stack не
  объявляет macOS/NVIDIA-free configuration поддерживаемой средой.
- **Не проверять ML toolchain до появления RTX** — `Rejected`: bounded Mac
  smoke раньше обнаруживает ONNX, export и schema ошибки.
- **Cloud/CI training как обязательный путь** — `Rejected`: offline bootstrap
  не должен зависеть от внешнего service.

## Product checks

| Сценарий | Ожидаемый результат | Fallback |
|---|---|---|
| Clean `aarch64-apple-darwin` checkout запускает `cargo run -p xtask -- host-check` | Format, lint, tests и boundary scan проходят; public contracts не содержат Apple, vendor или importer types; сеть и CI не требуются | Исправить portable boundary; продолжить разработку на поддерживаемом developer host |
| `uv run --project lab python -m next_lab smoke --device auto` выполняет 4 fixed seeds, 8 environments × 256 steps и 1 000 parity observations | Нет NaN/Inf; max absolute PyTorch/ONNX action error не превышает `1e-5`; run завершается не более чем за 10 минут на MPS либо declared CPU path | Повторить на CPU; при export/parity error использовать procedural runtime policy и исправить toolchain |
| `python -m next_lab doctor --profile local-rtx` проверяет production-training workstation | Linux x86_64, RAM не менее 32 GiB, NVIDIA VRAM не менее 16 GiB, pinned driver/toolchain и offline cache/provenance/license checks | Не запускать production training; использовать immutable previously validated policy либо deterministic procedural controller |

## Последствия

- Mac позволяет развивать contracts, runtime, tooling, scenarios, procedural
  controllers и bounded model prototypes.
- Отсутствие NVIDIA hardware не блокирует unrelated engine work.
- Local commands остаются переносимой execution surface для developer host.
- Добавление macOS shipping package, изменение local-first policy или допуск
  production learned policy без runtime correspondence требует нового ADR.
