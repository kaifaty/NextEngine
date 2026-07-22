# ADR-012: Deterministic command identity, ordering и replay

| Поле | Значение |
|---|---|
| ID | ADR-012 |
| Статус | Accepted |
| Версия | 1.0 |
| Владелец | Runtime Team + Persistence Team |
| Требуемые согласующие | Architecture Working Group, Runtime Team, Persistence Team, Security & Governance Team |
| Дата решения | 2026-07-23 |
| Последняя проверка | 2026-07-23 |
| Нормативные зависимости | [SPEC-02](../02-runtime-ecs-and-data.md), [SPEC-03](../03-assets-world-streaming-and-persistence.md), [SPEC-10](../10-gothic-importer-boundary.md) |
| Заменяет | [ADR-007](007-identities-persistence-and-replay.md) |
| Заменён | не заменён |

## История принятия

ADR принят атомарно в packet 1.5 и заменяет ADR-007. Он сохраняет stable-ID, fail-closed save/migration и replay-source semantics, устраняя неопределённость ordering, identity, encoding и hash composition. Принятие architecture contract не объявляет `RUNTIME-06`, `RUNTIME-07` или `CANON-01` пройденными.

## Контекст

Один ordered external command stream недостаточен, если разные producers могут назначить одинаковые sequence, выбирать priority, повторно войти в tick после physics или независимо сериализовать state. Save, replay, `game`, `headless` и `capture-worker` должны восстанавливать не приблизительно похожее выполнение, а один и тот же causal ledger и declared outcome.

## Public command contract

### IssuerPrincipal и CommandStreamId

`IssuerPrincipal` является closed tagged union:

| Tag | Variant | Canonical payload |
|---:|---|---|
| `0x01` | `Player(PlayerPrincipalId)` | 16 opaque bytes |
| `0x02` | `Agent(PersistentId)` | 16 opaque bytes |
| `0x03` | `Package(MechanicPackageId)` | `u32_le byte_length` + NFC UTF-8 bytes |
| `0x04` | `Plugin(PluginId)` | `u32_le byte_length` + NFC UTF-8 bytes |
| `0x05` | `InternalSystem(SystemId)` | `u32_le byte_length` + NFC UTF-8 bytes |

Scenario runner использует один из обычных principals и не получает privileged test issuer. `CommandStreamId` — отдельный opaque 128-bit identifier causal command ledger. Он создаётся composition root, записывается в save/replay и не выводится из wall clock, process ID или worker order.

`sequence: u64` строго возрастает для каждого `(CommandStreamId, IssuerPrincipal)`. Gaps разрешены и не переиспользуются. Последний committed sequence и соответствующий `command_id` образуют ledger, который входит в snapshot/save и replay input. Overflow закрывает stream стабильной ошибкой `COMMAND_SEQUENCE_EXHAUSTED`.

### Canonical command bytes и command_id

Canonical command bytes — versioned `CanonicalBinaryV1` encoding validated `WorldCommand` без transport framing. `command_id` равен первым 16 bytes SHA-256 следующего preimage:

```text
"nextengine.command-id.v1\0"
|| CommandStreamId[16]
|| u32_le(principal_bytes.len) || principal_bytes
|| u64_le(sequence)
|| u32_le(command_bytes.len) || command_bytes
```

Где `principal_bytes = issuer_tag || canonical_payload`. Truncation всегда берёт самые левые bytes digest. Любое изменение domain tag, length framing или truncation rule является schema/hash migration.

До сортировки validator выполняет admission:

- exact повтор `(stream, issuer, sequence, command_id, command_bytes)` дедуплицируется и даёт один result;
- одинаковый `(stream, issuer, sequence)` с различными `command_id` или bytes отклоняет всю conflicting equivalence class как `COMMAND_SEQUENCE_COLLISION`;
- sequence ниже или равный committed ledger с другим command получает `COMMAND_SEQUENCE_REUSE`;
- непредставимый principal, invalid NFC, неизвестная schema или hash mismatch отклоняются до mutation.

Arrival order, transport batching и worker scheduling не участвуют в результате.

### Validator-owned ordering

После admission commands сортируются лексикографически по:

```text
(target_tick,
 phase,
 priority_class,
 issuer_tag,
 issuer_id_bytes,
 sequence,
 command_id)
```

`phase` имеет только значения `Ingress = 0` и `Outcome = 1`. `priority_class` назначает validator из versioned command-kind registry после capability validation. Непроверенный issuer не передаёт priority в public input и не может повысить её косвенным alias command kind. Sort устойчив к arrival permutations; полное совпадение tuple означает exact duplicate, уже удалённый admission.

### Две commit-фазы одного tick

Authoritative tick сохраняет declared stages SPEC-02. `Ingress` проходит общий validator/transaction на обычном pre-simulation commit point. Physics, motor и dependent systems возвращают только immutable observations/proposals. На stage 9 создаётся один закрытый batch `Outcome` для того же `target_tick`:

- issuer MUST быть validator-authenticated `InternalSystem`;
- public transport, script, plugin, AI и scenario runner не могут создать `Outcome`;
- batch проходит тот же schema, capability, ordering и atomic transaction validator;
- результат может породить `DomainEvent`, но не новый same-tick command;
- command, предложенный во время `Outcome`, назначается не раньше следующего tick;
- повторный stage-9 entry даёт `OUTCOME_REENTRY_FORBIDDEN` и делает run non-conforming.

Тем самым post-physics gameplay outcomes коммитятся второй фазой того же tick без скрытой mutation и рекурсии.

## Runtime PersistentId

Runtime-created durable record получает ID только из causal command:

```text
PersistentId = Trunc128(SHA256(
  "nextengine.persistent-id.v1\0"
  || world_namespace[16]
  || causal_command_id[16]
  || u32_le(spawn_slot)
  || u32_le(record_kind_bytes.len)
  || record_kind_bytes
))
```

`spawn_slot` уникален внутри canonical spawn expansion, а `record_kind` — versioned NFC UTF-8 registry identifier. Retry exact causal spawn идемпотентно возвращает тот же record. Tombstone входит в authoritative state и запрещает reuse. Совпадение ID с иной provenance является fatal `PERSISTENT_ID_COLLISION`; implementation не выбирает новый random ID.

Explicit PersistentId разрешён только на validated cooker/import/load/migration boundary, где namespace, provenance и collision policy заданы соответствующей schema. Script, plugin, AI, gameplay system и scenario runner не могут передавать explicit runtime ID.

## Determinism classes

| Class | Обязательный контракт |
|---|---|
| Cooked content, public manifests, evidence manifests | Byte-exact на Windows x86_64 и Linux x86_64 для одной schema/input |
| Authoritative gameplay state | Exact state root на одной target triple, build hash и config hash |
| Cross-target replay | Exact accepted commands, `DomainEvent`, rejection codes и gameplay outcomes |
| Physics/motor numerics | Только declared per-field tolerance; outcome classification, contact ordering и gameplay events остаются exact |

Tolerance не применяется к IDs, integers, enums, ordering, commands, events, manifests или state roots. Cross-target numeric divergence за tolerance становится `NONDETERMINISTIC_RESULT`; retry-to-green запрещён.

## Canonical encoding и hashes

### JCS envelopes

Public manifests, policy manifests, evidence manifests и attestation payloads используют RFC 8785 JCS. Authoritative JCS:

- не содержит NaN, infinities или implementation-defined number formatting;
- нормализует semantic negative zero в `0` до serialization;
- кодирует integers вне exact JSON interoperable range, opaque IDs и hashes lowercase canonical strings согласно schema;
- допускает только Unicode scalar values; schema-designated identity strings MUST быть NFC до JCS;
- отвергает duplicate object keys до canonicalization.

### CanonicalBinaryV1

Numeric/state segments используют versioned binary envelope: magic `NECB`, version `u16_le = 1`, owner ID, schema ID и segment ID как length-prefixed NFC UTF-8, затем sorted field records. Каждый field record содержит stable `u32_le field_id`, `u8 type_tag`, `u64_le payload_length`, payload. Integers и IEEE-754 bit patterns имеют fixed width и little-endian encoding; booleans только `0`/`1`; strings NFC UTF-8; sequences сохраняют schema order; maps сортируются по full canonical encoded key bytes. Unknown required field/type и duplicate field/map key отклоняются.

Floats canonicalize `-0` в positive zero. NaN payload и infinities запрещены в authoritative state. Quantized/tolerant physics values MUST сначала преобразоваться declared schema rule; raw platform float не может незаметно попасть в exact outcome field.

### Paths, domains и state root

Canonical relative path использует `/`, не начинается с `/` или drive/UNC prefix, не содержит empty, `.`, `..` segments и состоит из NFC UTF-8. Case-fold collision внутри одного manifest отклоняется как `CANONICAL_PATH_COLLISION`; case-fold не переписывает сохранённое имя.

Все SHA-256 preimages имеют ASCII versioned domain tag с terminal NUL и explicit length framing. Raw evidence artifacts хэшируются как raw bytes с manifest-declared media type; `EvidenceBundleManifest` хэшируется как JCS bytes.

State root строится так:

```text
segment_hash = SHA256("nextengine.state-segment.v1\0" || u64_le(bytes.len) || bytes)
leaf_hash    = SHA256("nextengine.state-leaf.v1\0" || owner_id || schema_id || segment_id || segment_hash)
parent_hash  = SHA256("nextengine.state-node.v1\0" || left || right)
state_root   = SHA256("nextengine.state-root.v1\0" || u64_le(leaf_count) || merkle_root)
```

Owner/schema/segment identifiers length-prefixятся как в `CanonicalBinaryV1`; leaves сортируются по canonical tuple `(owner_id, schema_id, segment_id)`. Нечётный последний node переносится в следующий level как `SHA256("nextengine.state-carry.v1\0" || node)`. Empty tree использует SHA-256 domain `nextengine.state-empty.v1\0` как `merkle_root`.

## Gates

| Gate | Blocking contract | Required evidence |
|---|---|---|
| `RUNTIME-06` | Arrival permutations, exact duplicate, sequence collision/reuse, priority spoofing, Ingress/Outcome same-tick order и re-entry дают exact expected result | Versioned command corpus, accepted/rejected ledger, stage trace и state roots |
| `RUNTIME-07` | Spawn retry, multiple slots, tombstone и conflicting provenance дают specified ID или stable fatal diagnostic | Golden spawn vectors, save/reload ledger и collision corpus |
| `CANON-01` | Windows/Linux implementations совпадают на JCS, binary, path, Unicode, float, domain и Merkle vectors | Checked-in neutral vectors и raw-byte/hash report; protected data отсутствует |

Gate считается PASS только при 100% vectors. Runtime/hardware PASS не заявляется этим ADR; он определяет future acceptance contract.

## Последствия и синхронизация

Packet 1.5 одновременно обновил SPEC-02, SPEC-03, SPEC-10, affected requirements/failures, glossary и evidence links; ADR-007 получил `Superseded` и backlink. `game`, `headless` и `capture-worker` MUST использовать один registry, validator, ledger, persistence и replay path.
