# ADR-010: Artifact-first headless validation и human review

| Поле | Значение |
|---|---|
| ID | ADR-010 |
| Статус | Accepted |
| Версия | 1.0 |
| Владелец | Architecture Working Group + Verification & Evidence Team |
| Дата решения | 2026-07-22 |
| Последняя проверка | 2026-07-22 |
| Нормативные зависимости | [SPEC-15](../15-headless-testing-agent-validation-and-human-evidence.md), [SPEC-09](../09-tooling-sdk-and-observability.md), [SPEC-11](../11-security-licensing-and-governance.md), [ADR-008](008-mechanics-mod-package-and-agent-authoring-model.md) |
| Заменяет | отсутствует |

## Контекст

Next Engine должен разрабатываться coding agents на Linux-машинах без monitor/display server. Existing `headless`, replay, RunManifest и physical media gates доказывают отдельные части workflow, но не определяют общий scenario runner, automatic impact selection, displayless capture job и human approval semantics. Ручной запуск игры не масштабируется, не воспроизводится и не даёт agent-readable failure evidence.

## Решение

Принято:

- обязательная validation является artifact-first: человек не обязан запускать interactive runtime;
- CPU-only `headless` выполняет authoritative simulation и automatic oracles, а displayless Vulkan capture worker строит presentation evidence из exact replay;
- required suites вычисляет engine-owned `ImpactResolver`; LLM/author не может уменьшить resolved set;
- observable categories `visual`, `ui`, `camera`, `animation`, `physics`, `motor`, `audio` требуют human review exact changeset;
- automatic gates всегда предшествуют review и не могут быть overridden человеком;
- approval является immutable hash-bound `HumanReviewDecision` с SPEC-11 attestation; agent identity/credential не может подписать trusted review;
- raw frame/audio roots являются canonical evidence, MP4/GIF — review derivatives;
- CPU agent может завершить non-observable work без GPU; required observable changeset при отсутствии worker/encoder получает `AwaitingCapability`, не `PASS`;
- target feedback profiles: `agent-fast` ≤5 минут и ordinary `changeset` compute ≤30 минут без queue/human time; declared long certification/fuzz/training gates остаются blocking для своего promotion.

## Отклонённые варианты

- **Interactive runtime как обязательный gate** — `Rejected`: требует monitor/input, плохо автоматизируется и не фиксирует exact evidence.
- **Hidden X11/Wayland window или virtual desktop как baseline capture** — `Rejected`: добавляет ненужную WSI/display dependency; capture рендерит в offscreen images.
- **LLM/author выбирает достаточный test subset** — `Rejected`: создаёт self-review и silent omissions; автор может только расширить resolved plan.
- **Screenshot/image diff как gameplay oracle** — `Rejected`: presentation не является authoritative state; semantic/replay assertions проходят первыми.
- **Agent self-approval или automatic baseline promotion** — `Rejected`: qualitative approval и baseline promotion принадлежат human reviewer role.
- **Software renderer как shipping-GPU performance evidence** — `Rejected`: software MAY быть smoke fallback, но не закрывает reference GPU gate.
- **Постоянный cloud capture/review service** — `Rejected` как обязательная архитектура: portable manifests допускают local или CI orchestration.

## Последствия

- Все public features получают machine-readable scenarios и stable diagnostics.
- Renderer получает `OffscreenPresentationTarget`, но simulation contracts и Vulkan backend API не зависят от capture scheduler.
- Changeset admission становится двухступенчатым: automatic evidence, затем human review только when required.
- Large media хранится во внешнем artifact root, а Git/review record содержит manifests/hashes.
- Capture/encoder unavailable может отложить merge observable change, но не блокирует CPU feedback loop.

## Technology gate: FFmpeg CLI

FFmpeg CLI — `Proposed` external tool adapter, не runtime dependency и не public engine ABI.

| Поле | Значение |
|---|---|
| Owner | Verification & Evidence Team + Security & Governance |
| Сценарий | `next gate MEDIA-P1 --corpus canonical-review-media --targets windows-x86_64,linux-x86_64` |
| Threshold | exact pinned binary/config/SBOM; GIF/MP4 outputs decode successfully; frame count, dimensions, timestamp sequence и audio sample count match CapturePlan на 100%; normalized raw frame/audio roots remain exact; 0 outbound network attempts; all license/config fields classified |
| Evidence | encoder manifest, build/config/license/SBOM report, decoded stream report, frame/audio hashes, packet capture |
| Fallback | engine-owned canonical PNG/WAV set + GIF encoder; MP4-required review remains `AwaitingCapability` until another MediaEncoder adapter passes MEDIA-P1 |

## Supersession

Artifact-only acceptance, review authority, observable-category policy или automatic-gate precedence меняются только новым ADR. Encoder implementation и worker scheduler MAY меняться за same manifests/gates без изменения этого решения.
