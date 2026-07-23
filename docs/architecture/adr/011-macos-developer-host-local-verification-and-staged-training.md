# ADR-011: macOS developer host, local verification и staged training capability

| Поле | Значение |
|---|---|
| ID | ADR-011 |
| Статус | Accepted |
| Версия | 1.0.1 |
| Владелец | Repository Owner + Developer Experience + Physical Embodiment |
| Дата решения | 2026-07-22 |
| Последняя проверка evidence | 2026-07-24 |
| Нормативные зависимости | [ADR-001](001-product-repository-license-and-platforms.md), [ADR-009](009-pretrained-foundation-policies-and-progressive-motor-skills.md), [SPEC-09](../09-tooling-sdk-and-observability.md), [SPEC-14](../14-physical-archetypes-motor-skills-and-policy-lifecycle.md) |
| Заменяет | отсутствует; уточняет platform scope ADR-001 без добавления shipping target |
| Заменён | не заменён |

## Контекст

Bootstrap Next Engine выполняется на Apple Silicon Mac без remote repository и CI orchestration. ADR-001 откладывает macOS как v1 shipping platform, но не запрещает portable developer tools и engine crates на macOS. Rust поддерживает `aarch64-apple-darwin` как host target, а PyTorch предоставляет MPS backend. Полный Isaac Lab/Isaac Sim stack официально ориентирован на Windows/Linux и NVIDIA GPU, поэтому Mac не может быть source of truth для production physical-policy certification.

## Решение

- `macOS aarch64` принимается как `DeveloperHostTier`, а не shipping/runtime conformance target.
- Windows x86_64 и Linux x86_64 остаются единственными v1 shipping targets; `vertical-v1` сохраняет 15 gates и не получает macOS package criterion.
- Bootstrap admission MUST выполняться локальной project-owned командой `cargo run -p xtask -- host-check`. Отсутствие CI service не меняет gates; future CI SPEC-09 является projection тех же команд.
- Mac MUST собирать portable contracts/runtime/verification/headless/xtask workspace и не экспортировать Apple/vendor types в public contracts.
- Mac training ограничивается gate `TRAIN-MAC-P0`: deterministic generated 2-DoF smoke, bounded MPS/CPU optimization, ONNX export и parity. Он доказывает training toolchain, но MUST NOT закрывать runtime/training physics correspondence, creature skill или certification gates.
- PyTorch MPS является `Proposed` development-only backend. CPU является обязательным fallback для smoke; fallback фиксируется в RunManifest и не превращается в MPS PASS.
- `TRAIN-RTX-01` является capability gate для production training. Пока он не пройден на локальной Linux/NVIDIA машине, learned package MAY иметь только `PrototypeFallback`, а `PhysicalCertified` promotion имеет `AwaitingCapability`.
- Isaac Lab остаётся `Proposed`, не устанавливается на Mac и оценивается только после `TRAIN-RTX-01`.
- Local datasets, checkpoints, training runs, imported installations и media MUST находиться вне Git index.

## Рассмотренные варианты

- **macOS как третий v1 shipping target** — `Rejected`: потребовал бы renderer/platform backend и новую package/conformance matrix до vertical slice.
- **Isaac Lab на Apple Silicon как baseline** — `Rejected`: официальный full stack не объявляет macOS/NVIDIA-free configuration supported source of truth.
- **Не проверять ML toolchain до появления RTX** — `Rejected`: поздно обнаруживает ONNX/export/schema проблемы; bounded Mac smoke даёт ранний сигнал без ложной certification.
- **Cloud/CI training как обязательный путь** — `Rejected`: bootstrap является local-first и не зависит от внешнего service.

## Последствия

- Mac позволяет начать contracts, runtime, tooling, scenarios, procedural controllers и `PrototypeFallback` packages.
- Hardware capability и package certification становятся разными состояниями: отсутствие RTX не блокирует repository migration, но блокирует learned promotion.
- Local `xtask` и lab commands являются normative execution surface; будущая CI не создаёт альтернативную semantics.
- Любая попытка объявить Mac smoke доказательством PHYS-P5, TRAIN-P1, POLICY-01, SKILL-01 или CREATURE-01 rejected fail-closed.

## Gate для Proposed частей

| Gate | Владелец | Сценарий/команда | Threshold | Evidence | Fallback |
|---|---|---|---|---|---|
| `HOST-MAC-01` | Developer Experience | `cargo run -p xtask -- host-check` на clean `aarch64-apple-darwin` checkout | format/clippy/tests/docs/boundary PASS; 0 Apple/vendor/importer type в public contracts; no CI/remote dependency | host/toolchain manifest, command report, dependency/API scan | fix portable boundary; Mac developer-host claim blocked |
| `TRAIN-MAC-P0` | Physical Embodiment + ML Tooling | `uv run --project lab python -m next_lab smoke --device auto` | 4 fixed seeds; 8 envs ×256 steps; 1 000 parity observations; max abs PyTorch/ONNX action error ≤`1e-5`; 0 NaN/Inf; ≤10 min; MPS либо declared CPU fallback | RunManifest, lock/device/model/corpus hashes, parity/metrics report | CPU smoke; toolchain lane blocked при export/parity failure |
| `TRAIN-RTX-01` | Physical Embodiment + Security | `python -m next_lab doctor --profile local-rtx` на pinned local workstation | Linux x86_64 Ubuntu 22.04/24.04; RAM ≥32 GiB; NVIDIA VRAM ≥16 GiB; pinned driver/toolchain; offline cache/provenance/license checks PASS | hardware/driver/toolchain manifest, license/SBOM/capability report | remain `AwaitingCapability`; only PrototypeFallback |

## Supersession

Добавление macOS shipping package, изменение local-first policy или разрешение certification без RTX/backend correspondence требует нового ADR. Конкретный trainer/backend MAY меняться за теми же manifests и gates.
