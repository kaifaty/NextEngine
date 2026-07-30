# ADR-032: Grounded capsule physics checkpoint version boundary

| Поле | Значение |
|---|---|
| ID | ADR-032 |
| Статус | Accepted |
| Версия | 1.2 |
| Дата решения | 2026-07-27 |
| Последняя проверка | 2026-07-30 |
| Нормативные зависимости | [SPEC-03](../03-assets-world-streaming-and-persistence.md), [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-22](../22-schema-registry-compatibility-and-migration.md), [SPEC-26](../26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [ADR-025](025-schema-content-and-migration-authority.md), [ADR-027](027-physics-motor-and-animation-layering.md) |
| Заменяет | отсутствует |
| Заменён | частично [ADR-034](034-player-targeting-replay-v5-and-mapping-provenance.md): обозначение `ReplayManifestV4` как current generated replay и соответствующее последствие о version boundary; правила physics snapshot V1/V2 и replay V2 остаются Accepted |

## Контекст

Первый private runtime prototype использовал
`nextengine.physics-canonical-snapshot` version `1` только для capsule pose и
activation. Он не содержал descriptor closure, velocity, contact continuity
или solver continuation и потому не является complete SPEC-26 checkpoint.
Изменять те же version-1 bytes запрещает ADR-025/SPEC-22.

`SaveManifestV2` уже является generic ordered segment envelope. Его wire shape
не зависит от конкретной physics schema version. Replay V2, напротив, не
содержит per-tick physics step input и closed contact batch, поэтому его
расширение не является same-version-compatible.

## Решение

- Active physical state использует
  `PhysicsCanonicalSnapshotV2` с exact descriptor/profile/catalog bindings,
  body velocity и contact/solver continuation.
- Self-contained durable physics segment использует новый
  `PhysicsWorldCheckpointV1`, содержащий immutable catalog и snapshot V2.
- Grounded-capsule решение первоначально вошло в `WorldCheckpointV3`.
  Последующий incompatible RPG aggregate schema cut повышает только composite
  activation envelope до `WorldCheckpointV4`; физический owner segment
  остаётся byte-exact `PhysicsWorldCheckpointV1`.
- `SaveManifestV2` сохраняется: required physics segment определяется exact
  owner/schema/segment/version descriptor внутри неизменного envelope.
- Исторически это решение назначило `ReplayManifestV4` current replay и
  связало для каждого tick exact `PhysicsStepInputV2`, closed contact batch и
  resulting compare point. Назначение V4 как current generated replay заменено
  ADR-034: новый persistence/replay path создаёт V5, а V4 остаётся отдельным
  legacy exact-only runner. `ReplayManifestV3` остаётся unsupported historical
  envelope с bootstrap RPG schema и отклоняется до nested decode.
- Physics snapshot V1 и replay V2 имеют compatibility class `Unsupported` для
  activation. Runtime не мигрирует, не дополняет и не reinterpret-ирует их.
- Loader/replay decoder распознаёт unsupported top-level или segment version до
  authoritative activation и сохраняет исходные bytes неизменными.

Решение не меняет physics/motor/animation authority ADR-027 и не выбирает
vendor backend.

## Последствия

- Dev/test generations с prototype physics V1 требуется пересоздать; они не
  являются fallback candidates.
- Save envelope version не повышается без wire-shape change.
- Contact history после restart является частью exact physics checkpoint, а
  replay divergence может быть локализован до step-input или contact stage.
- Любое дальнейшее несовместимое изменение snapshot V2 требует следующей
  schema version; same-version drift остаётся invalid. То же правило сохраняет
  byte-exact legacy V4, но version boundary current generated replay V5
  определяет ADR-034.

## Rollback

До release implementation может быть reverted вместе с disposable V2/V3
generations. Source save/replay bytes не переписываются и не downgrade-ятся.
