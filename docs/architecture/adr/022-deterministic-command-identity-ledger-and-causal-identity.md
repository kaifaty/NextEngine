# ADR-022: Deterministic command identity, ledger и causal identity

| Поле | Значение |
|---|---|
| ID | ADR-022 |
| Статус | Accepted |
| Версия | 1.0 |
| Владелец | Runtime Team |
| Требуемые согласующие | Architecture Working Group, Runtime Team, Persistence Team, Security & Governance Team, Verification & Evidence Team |
| Дата решения | 2026-07-24 |
| Последняя проверка | 2026-07-24 |
| Нормативные зависимости | отсутствуют |
| Заменяет | [ADR-012](012-deterministic-command-identity-and-replay.md) |
| Заменён | не заменён |

## История принятия

ADR принят в architecture packet 1.6 и заменяет ADR-012. Он сохраняет engine-owned command boundary, две commit-фазы tick, `CanonicalBinaryV1`, domain-separated hashes, fail-closed collision policy и exact replay, но устраняет самоссылочную неоднозначность `command_id`, недостаточный ledger последней команды и незафиксированные clock/RNG/schedule/numeric primitives. Принятие решения не объявляет runtime implementation, gates или `vertical-v1` conformance пройденными.

## Контекст

ADR-012 определял `command_id` через canonical validated `WorldCommand`, одновременно требуя поле `command_id` в envelope. Без отдельного canonical body implementation может либо включить ID в собственный preimage, либо исключить его незафиксированным способом. Ledger только последнего committed `(sequence, command_id)` не может одновременно доказать точный повтор более старой команды, отличить конфликт после save/restart и закрыть gaps без arrival-dependent выбора.

Даже исправленный command hash не обеспечивает воспроизводимость, если wall-clock cutoff, RNG implementation, ECS iteration order, worker count, async completion order, float reduction или physics projection могут менять authoritative result. Эти primitives являются одной runtime compatibility boundary и должны версионироваться вместе с command ledger.

## Решение

### Source of truth и public boundary

Runtime Team владеет command admission, virtual tick assignment, deterministic RNG, system schedule, task-result merge, numeric profile и causal identity derivation. Persistence Team владеет их atomic save/restart representation. Это ADR является authority решения; companion [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md) зависит от ADR-022 в одном направлении и детализирует engine-owned schemas/algorithms. ECS/backend/task/OS objects в публичную границу не входят.

`game`, deterministic `headless` и displayless `capture-worker` MUST использовать одни и те же schemas, registry hashes, validator, ledger, RNG implementation, schedule manifest, numeric profile, save recovery и replay path. Compile-time features, worker count и renderer presence MUST NOT менять domain semantics.

### Несамоссылочная command identity

`WorldCommandEnvelopeV2` содержит optional `claimed_command_id`, transport metadata и `CanonicalCommandBodyV2`. Только body является authoritative command input. Body MUST содержать schema/version, issuer, bound stream, sequence, target tick, phase, optional target, capability claims, preconditions и deterministic payload и MUST NOT содержать:

- `command_id` или `claimed_command_id`;
- transport/session framing;
- arrival, batch, process или worker ordinal;
- wall-clock timestamp;
- validator result или validator-owned `priority_class`.

После schema/capability normalization runtime кодирует body ровно одним `CanonicalBinaryV1` profile. Hash contract:

```text
body_hash = SHA256(body_bytes)

command_id = left128(SHA256(
  "nextengine.command-id.v2\0"
  || u64_le(body_bytes.len)
  || body_bytes
))
```

Переданный claim MUST совпасть с вычисленным ID; mismatch отклоняется до reservation или gameplay mutation.

Полный `body_hash` сохраняется в каждой reservation/receipt. Усечённая ID collision с разными full hashes не получает альтернативный ID: внешний stream блокируется, а collision из authenticated `InternalSystem` останавливает instance до следующего commit. Изменение body schema, encoding, domain или truncation является explicit migration.

### Canonical encoding, paths и state root

ADR-022 сохраняет encoding contract как current authority, не полагаясь на Superseded ADR.

Public manifests, policy manifests, evidence manifests и attestation payloads используют RFC 8785 JCS. Authoritative JCS:

- не содержит NaN, infinities или implementation-defined number formatting;
- нормализует semantic negative zero в `0` до serialization;
- кодирует integers вне exact JSON interoperable range, opaque IDs и hashes lowercase canonical strings согласно schema;
- допускает только Unicode scalar values; schema-designated identity strings MUST быть NFC до JCS;
- отвергает duplicate object keys до canonicalization.

`CanonicalBinaryV1` использует envelope:

```text
"NECB"
|| u16_le(version = 1)
|| u32_le(owner_id_utf8.len)   || owner_id_utf8
|| u32_le(schema_id_utf8.len)  || schema_id_utf8
|| u32_le(segment_id_utf8.len) || segment_id_utf8
|| u32_le(field_count)
|| field_record[0..field_count]
```

IDs являются NFC UTF-8. Каждый field record содержит stable `u32_le field_id`, `u8 type_tag`, `u64_le payload_length`, payload. Records строго sorted по field ID; decoder MUST потребить ровно `field_count` records и весь envelope, trailing bytes запрещены.

Type-tag registry:

| Tag | Type | Payload |
|---:|---|---|
| `0x00` | `Unit` | empty |
| `0x01` | `U8` | 1 byte |
| `0x02` | `U16` | 2 bytes little-endian |
| `0x03` | `U32` | 4 bytes little-endian |
| `0x04` | `U64` | 8 bytes little-endian |
| `0x05` | `I8` | 1-byte two's complement |
| `0x06` | `I16` | 2-byte little-endian two's complement |
| `0x07` | `I32` | 4-byte little-endian two's complement |
| `0x08` | `I64` | 8-byte little-endian two's complement |
| `0x09` | `F32Bits` | 4 canonical IEEE-754 bytes little-endian |
| `0x0a` | `F64Bits` | 8 canonical IEEE-754 bytes little-endian |
| `0x0b` | `Bool` | exactly `0x00` or `0x01` |
| `0x0c` | `Bytes` | raw bytes |
| `0x0d` | `Utf8Nfc` | NFC UTF-8 bytes |
| `0x0e` | `Id128` | 16 bytes |
| `0x0f` | `Hash256` | 32 bytes |
| `0x10` | `Struct` | compound encoding below |
| `0x11` | `Sequence` | compound encoding below |
| `0x12` | `Map` | compound encoding below |
| `0x13` | `Set` | compound encoding below |
| `0x14` | `Option` | compound encoding below |
| `0x15` | `TaggedUnion` | compound encoding below |

Nested value record равен `type_tag || u64_le(payload_length) || payload`. Compound payloads:

- `Struct`: `u32_le(field_count)` и field records, строго sorted по `field_id`;
- `Sequence`: `u32_le(item_count)` и nested value records в schema order;
- `Map`: `u32_le(entry_count)` и пары nested key/value records, sorted по полным encoded key records;
- `Set`: `u32_le(item_count)` и nested value records, sorted по полным encoded records;
- `Option`: `0x00` для absent либо `0x01 || nested_value_record` для present;
- `TaggedUnion`: stable `u8 variant_tag || nested_value_record`; payload-less variant использует nested `Unit`;
- tuple кодируется как `Struct` с field IDs `1..N`;
- enum без payload кодируется declared fixed-width unsigned integer; nominal ID использует underlying `Id128` либо `Utf8Nfc`, заданный schema.

Duplicate map/set encoded key, duplicate struct field и noncanonical order отклоняются. В hash-bound command/receipt schemas все declared fields присутствуют, включая `Option(None)`; defaults не опускаются, unknown fields запрещены. Integers имеют fixed width; strings — NFC UTF-8. Sequences сохраняют schema order. Float `-0` нормализуется в positive zero; NaN и infinities запрещены в authoritative state.

Для `CanonicalCommandBodyV2` envelope IDs равны `owner_id = "nextengine.runtime"`, `schema_id = "nextengine.canonical-command-body"`, `segment_id = "v2"`. Для `CommandReceiptV1` они равны `owner_id = "nextengine.runtime"`, `schema_id = "nextengine.command-receipt"`, `segment_id = "v1"`. Остальные public binary schemas получают exact IDs из hash-bound schema registry; registry hash входит в compatibility metadata.

Canonical relative path использует `/`, не начинается с `/` или drive/UNC prefix, не содержит empty, `.` или `..` segments и состоит из NFC UTF-8. Case-fold collision внутри одного manifest отклоняется как `CANONICAL_PATH_COLLISION`; case-fold не переписывает сохранённое имя.

State root:

```text
segment_hash = SHA256("nextengine.state-segment.v1\0" || u64_le(bytes.len) || bytes)
leaf_hash    = SHA256("nextengine.state-leaf.v1\0" || lp(owner_id) || lp(schema_id) || lp(segment_id) || segment_hash)
parent_hash  = SHA256("nextengine.state-node.v1\0" || left || right)
state_root   = SHA256("nextengine.state-root.v1\0" || u64_le(leaf_count) || merkle_root)
```

Leaves сортируются по canonical tuple `(owner_id, schema_id, segment_id)`. Нечётный последний node переносится как `SHA256("nextengine.state-carry.v1\0" || node)`. Empty tree использует `SHA256("nextengine.state-empty.v1\0")` как `merkle_root`.

### Causal identities

Каждый мир имеет persisted `WorldIdentityManifestV1` с project identity, creation nonce, `world_namespace`, identity epoch и RNG root seed. Nonce создаётся composition root до первого authoritative tick из platform secure entropy либо задаётся exact scenario fixture; OS entropy после создания мира не участвует в authoritative execution. `Save As` сохраняет namespace. Fork/remap namespace не входит в v1.

`IssuerPrincipalV2` является closed union `Player`, `Agent`, `Package`, `Script`, `Plugin`, `Tool`, `InternalSystem`. Principal поступает только из authenticated engine registry. `CommandStreamId` детерминированно выводится из world namespace, canonical principal, stream slot и stream epoch; wall clock, process/session/task/worker IDs запрещены. Stream навсегда bound к одному principal.

`DomainEventId` выводится из causal command, canonical event slot, schema ID и full event-body hash. Runtime-created `PersistentId` выводится из world namespace, causal command, canonical spawn slot и versioned record kind. Retry возвращает тот же ID; collision с другой provenance является fatal. Explicit runtime ID недоступен script/plugin/AI/scenario input.

### Persisted command ledger

Для каждого stream authoritative `CommandStreamLedgerV2` хранит:

- monotonic admission high-watermark;
- stream state `Open`, `CollisionLocked`, `Exhausted` или `Closed`;
- ordered pending reservations;
- ровно последние `min(finalized_receipt_count, 4096)` receipts, а после 4096 finalizations — **ровно 4096** receipts;
- total finalized count и rolling receipt-chain root.

Размер receipt window равен 4096 и не является project setting. High-watermark закрывает все меньшие gaps без reuse. Exact retry pending или retained finalized command возвращает прежний result без новых events или mutation. Sequence не выше high-watermark принимается только как exact retry найденной reservation/receipt; если запись уже вытеснена из окна, результат стабильно `COMMAND_SEQUENCE_FINALIZED`.

Per-world `CommandLedgerV2` дополнительно хранит persisted global `CommandId → sorted body-hash occurrences` bindings и immutable content-addressed archive exact canonical body bytes. Collision binding сохраняет обе/all bounded occurrences, а не заменяет исходную. Bindings/body bytes живут весь lifetime world identity независимо от receipt eviction. Они позволяют обнаруживать truncated-ID/full-hash collision между streams и проверять exact retry/collision membership после save/restart; resource exhaustion отклоняет новый admission до mutation.

Conflicting bodies одного ранее unseen `(stream, sequence)` в одном закрытом batch образуют canonical collision set со всеми bounded sorted candidate members/body refs: ни один кандидат не выбирается, создаётся один terminal collision receipt, stream блокируется. Conflict с существующей reservation/receipt не меняет прежний result, сохраняет collision incident и блокирует stream. `u64` overflow не оборачивается и переводит stream в `Exhausted`.

Published save содержит high-watermark, reservations, receipt window, chain count/root и stream state. Save/restart не может повторно публиковать events: retained receipt возвращается как exact retry. Частично committed tick или текущий `Outcome` не публикуется; recovery возвращается к последнему atomic tick checkpoint и воспроизводит закрытые inputs. Persisted current/past `Outcome` reservation вне atomic checkpoint считается corruption и fail-closed.

### Deterministic runtime primitives

- Input ingress использует атомарный current/next queue cutoff. Элемент, linearized до close barrier stage 1, принадлежит current tick; элемент после barrier — next tick. Wall-clock timestamp остаётся metadata.
- RNG profile является engine-owned exact `ChaCha12IetfV1`; stream key/nonce выводятся из persisted root seed и canonical stream identity. Неявный global/worker RNG запрещён.
- System execution использует versioned stable DAG, declared access sets, stable topological tie-break, ordered authoritative queries, worker-independent logical shards и deterministic delta/reducer merge.
- Async/task completion становится authoritative только как immutable staged result с request/input/result hashes, precondition revision и canonical merge key. Mutable ECS access через `await` запрещён.
- Authoritative numerics используют checked integer/fixed-point rules. Raw backend physics float проходит declared exact quantization до gameplay/event/state boundary. NaN, infinity, unchecked overflow, authoritative float reduction и implicit epsilon запрещены.

Hashes clock/RNG/schedule/numeric profiles входят в `RuntimeDeterminismProfileV1`, build metadata, save и replay compatibility. Любое расхождение exact result — `NONDETERMINISTIC_RESULT`, не flaky retry.

## Рассмотренные варианты

### Оставить ADR-012 и уточнить implementation comment

Отклонено: это молча меняло бы Accepted hash preimage и не закрывало persisted retry window, cutoff, RNG, schedule и numeric semantics.

### Использовать UUID/random IDs для commands и runtime objects

Отклонено: random allocation делает retry/replay зависимым от hidden mutable state и не устраняет collision policy. Entropy допускается только при создании persisted world namespace.

### Сохранять неограниченный receipt map

Отклонено как обязательный v1 contract из-за неограниченного authoritative save growth. Append-only replay/evidence MAY хранить полную историю, но runtime retry contract использует fixed 4096 window, high-watermark и chain root.

### Считать arrival order или worker index tie-break

Отклонено: результат менялся бы от load, OS scheduler и worker count.

### Использовать platform RNG, native float reducers или backend physics values напрямую

Отклонено: они не задают byte-exact engine-owned compatibility contract. Backend остаётся заменяемым только за quantized projection boundary.

## Последствия

- `CommandId` v1 и causal `PersistentId` v1 не объявляются совместимыми с v2 автоматически. До implementation admission MUST пройти inventory старых save/replay/fixture artifacts; ненулевой inventory требует отдельного migration plan.
- Save/replay schemas MUST включать world identity, full command ledger state, input assignments, RNG states и runtime profile hashes.
- Command/event/spawn golden vectors становятся portable public fixtures без protected data.
- Runtime memory получает bounded receipt overhead на stream. Exact canonical body archive и global `CommandId → body-hash occurrences` binding index растут append-only как logical content-addressed maps до hash-bound resource limit; published save связывает их segmentation-independent roots и required object closure. Physical packing является reconstructible cache. Storage exhaustion fail-closed до command mutation. Полный result/replay log остаётся отдельным artifact.
- Выбор конкретной Rust-библиотеки ChaCha, scheduler или fixed-point arithmetic остаётся implementation detail и не становится public dependency.
- Никакой technology row не принимается этим ADR.

## Gates

| Gate | Owner | Threshold | Evidence | Fallback | VS / profile closure |
|---|---|---|---|---|---|
| `COMMAND-ID-P1` | Runtime Team | 100% Body V2 canonical bytes/full hashes/IDs/codes exact on Windows x86_64 and Linux x86_64; invalid claims/collisions mutate 0 gameplay state | Cross-platform golden bytes/hashes, negative corpus, validator/ledger report | Blocking schema/hash fix | VS-11; shipping exact-command profile |
| `CAUSAL-ID-P1` | Runtime Team | 100% world/principal/stream/event/spawn IDs exact; every injected provenance collision fails before alternate ID/mutation | Identity manifests, vectors, collision and save/reload corpus | Blocking identity/migration fix | VS-02, VS-11; persisted identity profile |
| `COMMAND-LEDGER-P1` | Runtime Team | Exact high-watermark/window/chain/results for all 0/1/4095/4096/4097 and 255/256/257 boundaries plus 10 000 arrival/restart permutations | Ledger snapshots, receipt chains, stage/recovery/state-root report | Blocking ledger/recovery fix | VS-02, VS-04, VS-11; save/replay profile |
| `CLOCK-P1` | Runtime Team | 100% current/next assignment exact across cutoff/source/completion permutations; wall time/worker variation changes 0 outcomes | Assignment vectors, stage traces, replay | Serialize close barrier behind same contract | VS-04, VS-11; virtual-time profile |
| `RNG-P1` | Runtime Team | 100% raw blocks and at least 10 000 draws/stream exact across Windows/Linux and save/reload; invalid/exhausted requests mutate 0 state | Key/nonce/block/draw vectors and state snapshots | Blocking engine-owned RNG fix | VS-02, VS-06, VS-11; deterministic RNG profile |
| `SCHEDULE-P1` | Runtime Team | Exact schedule/shard/delta/event/state roots for at least 1 000 registration/completion permutations and worker counts 1/2/8/16; 100% ambiguous/cyclic graphs rejected | Schedule manifests, DAG/stage/shard/task/delta traces | Serialize execution behind same manifest | VS-11, VS-15; deterministic schedule profile |
| `NUMERIC-P1` | Runtime Team | 100% checked integer/fixed-point/IEEE quantization vectors and projection roots exact; all overflow/nonfinite cases abort atomically | Boundary/rounding/overflow/nonfinite/projection corpus | Blocking numeric/adapter fix | VS-11, VS-14; authoritative numeric/physics profile |
| `COMMAND-V1-INVENTORY-P1` | Persistence Team | 0 admitted v1 artifacts or one separately Accepted migration covering 100% discovered hashes | Inventory manifest, hashes and migration decision/fixtures | Keep implementation admission blocked | VS-02, VS-12; migration profile |

Каждый vector gate требует 100% exact match. ADR не создаёт gate PASS; он задаёт acceptance contract.

## Gate для Proposed частей

не применяется. Все решения engine-owned; replaceable implementations не принимаются этим ADR.

## Supersession

ADR-022 заменяет ADR-012 целиком. ADR-012 сохраняется неизменным по смыслу со статусом `Superseded` и backlink. Зависимые SPEC, glossary, traceability, evidence register и packet index синхронизируются в одном architecture changeset до promotion.
