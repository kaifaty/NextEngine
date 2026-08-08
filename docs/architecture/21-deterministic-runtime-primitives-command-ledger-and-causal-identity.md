# SPEC-21: Deterministic runtime primitives, command ledger и causal identity

| Поле | Значение |
|---|---|
| ID | SPEC-21 |
| Статус | Accepted |
| Версия | 1.5 |
| Последняя проверка | 2026-08-09 |
| Нормативные зависимости | [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md) |
| Заменяет | SPEC-21 1.3 speculative narrative cross-context and pre-public compatibility clauses |

## Назначение, authority и граница

SPEC-21 закрывает единый deterministic compatibility contract для command identity/admission, causal IDs, virtual time, input cutoff, RNG, system/task scheduling, reductions и authoritative numerics. Профиль является production architecture, а не test harness: один и тот же путь MUST использоваться в `game`, deterministic `headless`, `tools` там, где они исполняют simulation, и displayless `capture-worker`.

Для authoritative simulation принят bit-exact класс на declared Windows
x86_64/Linux x86_64 profiles: два run эквивалентны тогда и только тогда, когда
на каждом stage-11 tick boundary совпадают canonical commands/rejections,
receipts, RNG states, events и authoritative state root. Presentation pixels,
optional model diagnostics и reconstructible caches в этот predicate не входят
и не могут менять перечисленные значения.

Runtime subsystem владеет validation/admission, tick assignment, command ledger, RNG streams, schedule и numeric execution. Persistence subsystem владеет atomic representation в save/replay. Профильные subsystem owners владеют payload schemas и значениями своих полей, но не могут создавать альтернативный command/RNG/schedule/numeric path.

Public contracts находятся в `crates/contracts` и используют только engine-owned nominal IDs, fixed-width values, canonical bytes и immutable manifests. ECS components/storage, raw pointers, task/future handles, wall-clock objects, OS input objects, physics/backend types и database connections запрещены в public schemas.

Документ детализирует runtime, persistence, physics и extension boundaries [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-05](05-physics-animation-and-motor-control.md), [SPEC-07](07-rpg-scripting-and-plugins.md) и [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md). Dependency metadata остаётся направленной от этих consumers к ADR-022/SPEC-21; перечисление здесь не создаёт обратного normative edge.

## Канонические базовые правила

- `Hash256` — ровно 32 bytes SHA-256; canonical text, если нужен, — 64 lowercase hexadecimal characters.
- `Id128` — ровно 16 opaque bytes; canonical text — 32 lowercase hexadecimal characters.
- Integer fields имеют указанную fixed width и little-endian encoding.
- Строки являются Unicode scalar sequences в NFC UTF-8 и length-prefixятся `u32_le`; invalid UTF-8/NFC и duplicate normalized identity отклоняются.
- `lp(x)` означает `u32_le(x.len) || x`; byte arrays длиннее `u32::MAX` непредставимы.
- Перечисления имеют фиксированный `u8` tag. Unknown tag fail-closed.
- Collections в schema order сохраняют порядок; canonical sets/maps сортируются по полным canonical encoded bytes ключа и запрещают duplicate key.
- Все engine-derived hash preimages, кроме явно заданного raw `body_hash = SHA256(body_bytes)`, начинаются versioned ASCII domain с terminal NUL и имеют explicit length framing для variable fields.
- В authoritative state запрещены NaN, infinity и implicit native layout. Semantic `-0` нормализуется в positive zero до canonical encoding.

`CanonicalBinaryV1` из current ADR-022 является binary envelope profile. ADR-012 имеет только historical supersession status и не является current authority. Для schemas ниже stable `field_id` равен указанному номеру; поле с меньшим ID кодируется раньше. Изменение ID/type/meaning требует новой schema version.

### RuntimeAdmissionLimitsV1

Untrusted decode/admission использует один hash-bound limits profile:

```text
RuntimeAdmissionLimitsV1 {
  1 schema_version: u16 = 1,
  2 max_namespaced_id_bytes: u32 = 255,
  3 max_identity_string_bytes: u32 = 4096,
  4 max_command_body_bytes: u32 = 262144,
  5 max_capability_claims: u32 = 64,
  6 max_preconditions: u32 = 128,
  7 max_commands_per_closed_batch: u32 = 4096,
  8 max_unique_collision_candidates: u32 = 64,
  9 max_input_payload_bytes: u32 = 65536,
  10 max_task_result_bytes: u32 = 4194304,
  11 max_pending_per_stream: u32 = 256,
  12 receipt_window_size: u32 = 4096,
  13 max_identity_occurrences: u64 = 67108864,
}
```

Defaults являются v1 profile. Project MAY выбрать меньший bound до world creation; увеличение требует отдельного named profile и повторения security/performance checks. Значения `receipt_window_size` и `max_pending_per_stream` для v1 runtime profile MUST оставаться 4096/256. Boundary violation возвращает stable owner-specific `*_RESOURCE_LIMIT` до allocation/ledger/gameplay mutation. Product checks cover `N-1/N/N+1`.

## Public command schemas

### Nominal scalar representations

| Nominal type | CanonicalBinary underlying type |
|---|---|
| `SchemaId`, `ProjectId`, `OwnerId`, `SystemId`, `ReducerId`, `NamespacedId`, `StableDiagnosticCode`, package/script/plugin/tool IDs | `Utf8Nfc` |
| `CommandId`, `WorldNamespace`, `CommandStreamId`, `DomainEventId`, `PersistentId`, `RngStreamId`, `PlayerPrincipalId` | `Id128` |
| `SimulationTick`, revision, sequence, ordinal и queue generation | `U64` |
| `PriorityClass` | `U16`; меньшее значение сортируется раньше |
| `Hash256` | `Hash256` |

Nominal types не взаимозаменяемы, хотя имеют одинаковые bytes. Utf8 IDs ограничены `RuntimeAdmissionLimitsV1`; known-ID validation выполняет hash-bound registry без переписывания bytes.

### IssuerPrincipalV2

`IssuerPrincipalV2` является `CanonicalBinaryV1` `TaggedUnion`. Его payload равен `variant_tag || nested_value_record`, где `Player/Agent` используют `Id128`, остальные variants — `Utf8Nfc`. `canonical_principal_bytes` для stream derivation равны полному nested value record `0x15 || u64_le(union_payload.len) || union_payload`; ordering tuple отдельно использует `issuer_tag` и raw canonical variant payload bytes:

| Tag | Variant | Canonical payload |
|---:|---|---|
| `0x01` | `Player(PlayerPrincipalId)` | 16 bytes |
| `0x02` | `Agent(PersistentId)` | 16 bytes |
| `0x03` | `Package(MechanicPackageId)` | namespaced NFC UTF-8 |
| `0x04` | `Script(ScriptPrincipalId)` | namespaced NFC UTF-8 |
| `0x05` | `Plugin(PluginId)` | namespaced NFC UTF-8 |
| `0x06` | `Tool(ToolPrincipalId)` | namespaced NFC UTF-8 |
| `0x07` | `InternalSystem(SystemId)` | namespaced NFC UTF-8 |

Principal MUST быть создан или разрешён authenticated `PrincipalRegistryV1`. Scenario runner использует `Tool` либо обычный package/script principal и не получает test-only tag. Session, connection, process и worker IDs не являются principal payload.

```text
PrincipalRegistryV1 {
  1 schema_version: u16 = 1,
  2 world_namespace: WorldNamespace,
  3 principals: CanonicalMap<IssuerPrincipalV2, PrincipalRecordV1>,
}

PrincipalRecordV1 {
  1 provenance_hash: Hash256,
  2 capability_subject_id: NamespacedId,
  3 status: PrincipalStatus,          // Active = 0, Disabled = 1, Retired = 2
}
```

Повторная регистрация exact principal/provenance идемпотентна. Тот же canonical principal с другой provenance получает `PRINCIPAL_ID_COLLISION` до capability grant либо command admission.

### CanonicalCommandBodyV2

```text
CapabilityRefV1 {
  1 capability_id: NamespacedId,
  2 scope_hash: Option<Hash256>,
}

CommandPreconditionV1 {
  1 precondition_kind: NamespacedId,
  2 owner_id: OwnerId,
  3 target_schema_id: SchemaId,
  4 target_key: CanonicalBinary,
  5 expected_revision: u64,
  6 constraint_hash: Option<Hash256>,
}

CommandKindRegistryEntryV1 {
  1 payload_schema_id: SchemaId,
  2 payload_schema_version: u32,
  3 command_kind_id: NamespacedId,
  4 priority_class: PriorityClass,
  5 validator_profile_hash: Hash256,
  6 required_capabilities: CanonicalSet<CapabilityRefV1>,
}

CommandKindRegistryV1 {
  1 schema_version: u16 = 1,
  2 entries: CanonicalMap<(SchemaId, u32), CommandKindRegistryEntryV1>,
}
```

Registry map key MUST совпадать с entry payload schema/version. `registry_bytes` использует envelope `nextengine.runtime/nextengine.command-kind-registry/v1`:

```text
command_kind_registry_hash = SHA256(
  "nextengine.command-kind-registry.v1\0"
  || u64_le(registry_bytes.len)
  || registry_bytes
)
```

Перечисленные поля являются полным hash-bound body:

```text
CanonicalCommandBodyV2 {
  1  body_schema_version: u16 = 2,
  2  payload_schema_id: SchemaId,
  3  payload_schema_version: u32,
  4  issuer: IssuerPrincipalV2,
  5  stream_id: CommandStreamId,
  6  sequence: u64,
  7  target_tick: SimulationTick,
  8  phase: CommandPhase,
  9  target: Option<PersistentId>,
  10 capability_claims: CanonicalSet<CapabilityRefV1>,
  11 preconditions: CanonicalSet<CommandPreconditionV1>,
  12 payload: CanonicalBinary,
}
```

`CommandPhase` имеет только `Ingress = 0x00` и `Outcome = 0x01`. `capability_claims` сортируются по canonical bytes. Preconditions сортируются по `(precondition_kind, owner_id, target_schema_id, target_key, expected_revision)` canonical bytes; два preconditions с одинаковым key и разным constraint отклоняются.

Body **MUST NOT** содержать:

- `command_id` либо `claimed_command_id`;
- transport/session metadata;
- sampled/received wall-clock timestamp;
- arrival, batch, process, task или worker ordinal;
- validator result;
- `priority_class`, поскольку его назначает versioned validator registry.

### WorldCommandEnvelopeV2

```text
WorldCommandEnvelopeV2 {
  1 envelope_schema_version: u16 = 2,
  2 claimed_command_id: Option<CommandId>,
  3 body: CanonicalCommandBodyV2,
  4 transport_metadata: Option<NonAuthoritativeTransportMetadataV1>,
}
```

Поле 4 не входит в body bytes, command identity, ledger ordering, save state либо replay input. Transport adapter MUST удалить неизвестную metadata до validator boundary. `claimed_command_id` является проверяемым claim, а не source of truth.

### Command identity

```text
body_bytes =
  CanonicalBinaryV1(
    owner_id = "nextengine.runtime",
    schema_id = "nextengine.canonical-command-body",
    segment_id = "v2",
    value = CanonicalCommandBodyV2
  )

body_hash =
  SHA256(body_bytes)

command_id =
  left128(
    SHA256(
      "nextengine.command-id.v2\0"
      || u64_le(body_bytes.len)
      || body_bytes
    )
  )
```

`left128` всегда означает первые 16 bytes digest в network-independent byte order. Validator MUST повторно построить `body_bytes`; предоставление caller-owned canonical bytes без schema decode/re-encode запрещено. Если `claimed_command_id` существует и не равен computed ID, validator возвращает `COMMAND_ID_MISMATCH` до ledger reservation и без gameplay mutation.

Каждая accepted, reserved или terminally rejected authenticated command запись сохраняет полный `body_hash`. Если один `CommandId` сопоставлен разным full hashes, это `COMMAND_ID_COLLISION`: альтернативный/random ID запрещён. Если разные canonical body bytes имеют одинаковый full hash и это обнаружено, результат `COMMAND_BODY_HASH_COLLISION` и instance fail-closed.

### Validator-owned ordering

После admission accepted commands сортируются лексикографически:

```text
(target_tick,
 phase,
 priority_class,
 issuer_tag,
 issuer_payload_bytes,
 sequence,
 command_id)
```

`priority_class` вычисляется из hash-bound `CommandKindRegistryV1` после schema/capability validation и сохраняется в reservation/receipt вместе с registry hash. Caller не задаёт priority прямо или через unvalidated alias.

## World и causal identity

### WorldIdentityManifestV1

```text
WorldIdentityManifestV1 {
  1 schema_version: u16 = 1,
  2 project_id: ProjectId,
  3 world_creation_nonce: [u8; 32],
  4 world_namespace: WorldNamespace,          // Id128
  5 identity_epoch: u32,                     // v1 MUST equal 0
  6 rng_root_seed: [u8; 32],
  7 runtime_determinism_profile_hash: Hash256,
}
```

Composition root получает nonce и RNG seed до первого authoritative tick из platform secure entropy или из exact neutral scenario fixture и атомарно сохраняет manifest. OS entropy после создания мира не участвует в authoritative simulation. Validator пересчитывает namespace:

```text
world_namespace =
  left128(
    SHA256(
      "nextengine.world-namespace.v1\0"
      || lp(canonical_project_id_bytes)
      || world_creation_nonce
    )
  )
```

`Save As`, autosave, replay и recovery сохраняют namespace. World fork/remap и изменение `identity_epoch` вне v1; implementation MUST отказать, а не незаметно создать новую identity.

Default local player:

```text
PlayerPrincipalId =
  left128(
    SHA256(
      "nextengine.player-principal.v1\0"
      || world_namespace
      || u32_le(player_slot)
    )
  )
```

Single-player v1 использует `player_slot = 0`; дополнительные local slots могут быть созданы manifest-ом до первого tick.

### CommandStreamRegistryV1

```text
CommandStreamRegistryV1 {
  1 schema_version: u16 = 1,
  2 world_namespace: WorldNamespace,
  3 entries: CanonicalMap<(IssuerPrincipalV2, u32, u32), CommandStreamId>,
  4 next_stream_slot: CanonicalMap<IssuerPrincipalV2, u32>,
}
```

Ключ entry равен `(principal, stream_slot, stream_epoch)`. ID:

```text
command_stream_id =
  left128(
    SHA256(
      "nextengine.command-stream.v1\0"
      || world_namespace
      || lp(canonical_principal_bytes)
      || u32_le(stream_slot)
      || u32_le(stream_epoch)
    )
  )
```

Stream навсегда bound к principal. Slot выделяется монотонно и сохраняется; overflow fail-closed. Новый logical stream получает `next_stream_slot` и epoch 0. Rotation сохраняет slot и выполняется только lifecycle transaction:

```text
RotateCommandStreamV1 {
  principal,
  stream_slot,
  expected_stream_id,
  expected_epoch,
  reason: CollisionRecovery | SequenceExhausted | AdministrativeClose,
}
```

Validator требует, чтобы old stream имел соответствующее terminal/locked state, затем атомарно помечает его `Closed`, checked-инкрементирует epoch и создаёт один новый `Open` ID. Epoch overflow даёт `COMMAND_STREAM_EPOCH_EXHAUSTED`; slot/epoch/ID не выбираются caller-ом через gameplay payload. Retry exact lifecycle command возвращает тот же rotated stream.

### DomainEventId

Event body кодируется собственной registered canonical schema:

```text
event_body_hash =
  SHA256(
    "nextengine.event-body.v1\0"
    || u64_le(event_body_bytes.len)
    || event_body_bytes
  )

event_id =
  left128(
    SHA256(
      "nextengine.event-id.v1\0"
      || causal_command_id
      || u32_le(event_slot)
      || lp(canonical_event_schema_id_bytes)
      || event_body_hash
    )
  )
```

`event_slot` начинается с нуля для canonical transaction expansion и назначается после deterministic delta/reducer merge. Один command не может публиковать два event slots с разными provenance. Exact retry возвращает прежний receipt и MUST NOT повторно публиковать event.

```text
DomainEventEnvelopeV2 {
  1 schema_version: u16 = 2,
  2 event_id: DomainEventId,
  3 simulation_tick: SimulationTick,
  4 phase: CommandPhase,
  5 causal_command_id: CommandId,
  6 event_slot: u32,
  7 event_schema_id: SchemaId,
  8 event_schema_version: u32,
  9 event_body_hash: Hash256,
  10 payload: CanonicalBinary,
}
```

Envelope validator повторно кодирует payload и проверяет `event_body_hash/event_id`. DomainEvent создаётся только из committed transaction; rejection существует как receipt/diagnostic, не как DomainEvent.

### Runtime-created PersistentId

```text
persistent_id =
  left128(
    SHA256(
      "nextengine.persistent-id.v2\0"
      || world_namespace
      || causal_command_id
      || u32_le(spawn_slot)
      || lp(canonical_record_kind_bytes)
    )
  )
```

`spawn_slot` начинается с нуля внутри canonical spawn expansion после deterministic merge. `record_kind` — versioned namespaced NFC registry ID. Provenance `(world_namespace, command_id, spawn_slot, record_kind, body_hash)` сохраняется с durable record/tombstone. Exact retry возвращает тот же ID; совпадение с иной provenance останавливает instance как `PERSISTENT_ID_COLLISION`, не выбирая другой ID.

Explicit `PersistentId` разрешён только validated cooker/import/load/migration boundary. Script, plugin, AI, mechanic, scenario action и runtime gameplay system не могут задавать его напрямую.

### CausalIdentityRegistryV1

```text
CausalIdentityBindingV1 {
  1 identity_kind: CausalIdentityKind,
  2 identity_bytes: Id128,
  3 provenance_hash: Hash256,
}

CausalIdentityRegistryV1 {
  1 schema_version: u16 = 1,
  2 world_namespace: WorldNamespace,
  3 bindings: CanonicalMap<(CausalIdentityKind, Id128), Hash256>,
}
```

Registry содержит `PlayerPrincipal`, `CommandStream`, `DomainEvent`, `PersistentRecord` и `RngStream` bindings. До публикации нового derived ID runtime вычисляет canonical provenance bytes, domain-separated provenance hash и выполняет compare-or-insert. Exact binding идемпотентен; тот же `(kind, id)` с иным hash даёт соответственно `PLAYER_PRINCIPAL_COLLISION`, `COMMAND_STREAM_ID_COLLISION`, `DOMAIN_EVENT_ID_COLLISION`, `PERSISTENT_ID_COLLISION` либо `RNG_STREAM_ID_COLLISION`. Collision не получает salt/random fallback.

```text
provenance_hash = SHA256(
  "nextengine.causal-provenance.v1\0"
  || u8(identity_kind)
  || u64_le(canonical_provenance_bytes.len)
  || canonical_provenance_bytes
)
```

Provenance bytes являются exact ordered hash inputs соответствующей derivation до outer domain/hash; для event дополнительно включают full event-body hash, для durable record — causal full command body hash.

`world_namespace` проверяется по `WorldIdentityManifestV1`. Два manifests с одним namespace и различными canonical `(project_id, nonce)` дают `WORLD_NAMESPACE_COLLISION`; exact `Save As` manifest считается тем же миром.

## Command ledger schemas

```text
CommandReservationV1 {
  1 schema_version: u16 = 1,
  2 stream_id: CommandStreamId,
  3 issuer: IssuerPrincipalV2,
  4 sequence: u64,
  5 command_id: CommandId,
  6 body_hash: Hash256,
  7 canonical_body_ref: Hash256,
  8 reserved_at_tick: SimulationTick,
  9 target_tick: SimulationTick,
  10 phase: CommandPhase,
  11 priority_class: PriorityClass,
  12 command_kind_registry_hash: Hash256,
}

CommandBodyArchiveManifestV1 {
  1 schema_version: u16 = 1,
  2 entry_count: u64,
  3 archive_root: Hash256,
}

CommandIdentityOccurrenceV1 {
  1 body_hash: Hash256,
  2 first_stream_id: CommandStreamId,
  3 first_sequence: u64,
}

CommandIdentityBindingV1 {
  1 command_id: CommandId,
  2 occurrences: Vec<CommandIdentityOccurrenceV1>,
  3 state: CommandIdentityBindingState,       // Unique = 0, Collision = 1
}

CommandIdentityIndexBodyV1 {
  1 schema_version: u16 = 1,
  2 bindings: CanonicalMap<CommandId, CommandIdentityBindingV1>,
  3 command_id_count: u64,
  4 occurrence_count: u64,
}

CommandIdentityIndexV1 {
  1 schema_version: u16 = 1,
  2 body: CommandIdentityIndexBodyV1,
  3 index_root: Hash256,
}

CommandCollisionCandidateV1 {
  1 command_id: CommandId,
  2 body_hash: Hash256,
  3 canonical_body_ref: Hash256,
}

CommandCollisionIncidentV1 {
  1 stream_id: CommandStreamId,
  2 issuer: IssuerPrincipalV2,
  3 sequence: u64,
  4 candidates: Vec<CommandCollisionCandidateV1>,
  5 candidates_root: Hash256,
  6 incident_digest: Hash256,
}

CommandReceiptV1 {
  1 schema_version: u16 = 1,
  2 finalization_ordinal: u64,
  3 subject: CommandReceiptSubjectV1,
  4 phase: CommandPhase,
  5 target_tick: SimulationTick,
  6 finalized_at_tick: SimulationTick,
  7 priority_class: PriorityClass,
  8 command_kind_registry_hash: Hash256,
  9 result: CommandFinalResultV1,
  10 diagnostic_digest: Option<Hash256>,
  11 event_ids: Vec<DomainEventId>,
  12 transaction_result_root: Hash256,
}

CommandReceiptSubjectV1 =
  0x01 Command {
    stream_id: CommandStreamId,
    issuer: IssuerPrincipalV2,
    sequence: u64,
    command_id: CommandId,
    body_hash: Hash256,
    canonical_body_ref: Hash256,
  }
  | 0x02 CollisionSet {
    stream_id: CommandStreamId,
    issuer: IssuerPrincipalV2,
    sequence: u64,
    candidates_root: Hash256,
    candidate_count: u32,
    candidates: Vec<CommandCollisionCandidateV1>,
  }

CommandFinalResultV1 =
  0x01 Committed
  | 0x02 Rejected { code: StableDiagnosticCode }
  | 0x03 Collision { code: StableDiagnosticCode }

CommandStreamLedgerV2 {
  1 schema_version: u16 = 2,
  2 stream_id: CommandStreamId,
  3 issuer: IssuerPrincipalV2,
  4 stream_slot: u32,
  5 stream_epoch: u32,
  6 state: CommandStreamStateV1,
  7 admission_high_watermark: Option<u64>,
  8 greatest_reserved_target_tick: Option<SimulationTick>,
  9 pending: CanonicalMap<u64, CommandReservationV1>,
  10 receipt_window: Vec<CommandReceiptV1>,
  11 finalized_receipt_count: u64,
  12 receipt_chain_root: Hash256,
  13 collision_incident: Option<CommandCollisionIncidentV1>,
}

CommandLedgerV2 {
  1 schema_version: u16 = 2,
  2 world_namespace: WorldNamespace,
  3 streams: CanonicalMap<CommandStreamId, CommandStreamLedgerV2>,
  4 command_kind_registry_hash: Hash256,
  5 runtime_determinism_profile_hash: Hash256,
  6 body_archive: CommandBodyArchiveManifestV1,
  7 identity_index: CommandIdentityIndexV1,
  8 causal_identity_registry: CausalIdentityRegistryV1,
}
```

`CommandStreamStateV1` имеет tags `Open = 0`, `CollisionLocked = 1`, `Exhausted = 2`, `Closed = 3`. `pending` сортируется по sequence. Не более 256 future reservations на stream допускаются default v1 profile; boundary 257 получает `COMMAND_PENDING_LIMIT`.

Body archive является logical immutable content-addressed map `body_hash → exact canonical body bytes`. `canonical_body_ref` равен `body_hash`; lookup MUST проверить entry key и повторно вычисленный full hash. Exact bytes сохраняются на lifetime world identity и не удаляются при receipt eviction. Physical packing/segmentation является reconstructible cache и не входит в manifest/root. Missing/corrupt entry — `COMMAND_BODY_ARCHIVE_CORRUPT`, load/admission fail-closed.

```text
body_archive_leaf = SHA256(
  "nextengine.command-body-archive-leaf.v1\0"
  || body_hash
  || u64_le(body_bytes.len)
  || body_bytes
)

body_archive_parent = SHA256(
  "nextengine.command-body-archive-node.v1\0"
  || left
  || right
)

index_body_bytes =
  CanonicalBinaryV1(
    owner_id = "nextengine.runtime",
    schema_id = "nextengine.command-identity-index-body",
    segment_id = "v1",
    value = CommandIdentityIndexBodyV1
  )

identity_index_root = SHA256(
  "nextengine.command-identity-index.v1\0"
  || u64_le(index_body_bytes.len)
  || index_body_bytes
)
```

Leaves сортируются по `body_hash`. Нечётный last node переносится как `SHA256("nextengine.command-body-archive-carry.v1\0" || node)`; empty map использует `SHA256("nextengine.command-body-archive-empty.v1\0")` как Merkle root. Final `archive_root = SHA256("nextengine.command-body-archive-root.v1\0" || u64_le(entry_count) || merkle_root)`. Это делает root независимым от physical packing. `index_root` находится только во внешнем `CommandIdentityIndexV1` и не входит в собственный preimage.

Перед reservation/terminal receipt runtime выполняет compare-or-insert в global logical `CommandIdentityIndexV1`. Binding occurrences сортируются по `(body_hash, first_stream_id, first_sequence)` и запрещают duplicate body hash. Первая occurrence создаёт `Unique`; existing `command_id` с новым `body_hash` атомарно добавляет occurrence, переводит binding в `Collision` и даёт `COMMAND_ID_COLLISION`; existing body archive key с иными bytes даёт `COMMAND_BODY_HASH_COLLISION`. Collision occurrence сохраняется вместе с archive body и incident, хотя command не исполняется. Index логически сохраняется на lifetime world identity; physical index/cache не меняет canonical logical map/root. `max_identity_occurrences` исчерпывается стабильным `COMMAND_IDENTITY_ARCHIVE_LIMIT` до command mutation.

Union всех `identity_index.body.bindings.values().occurrences.body_hash` MUST точно совпадать с key set body archive; каждый reservation/receipt/collision member ref обязан принадлежать этому set. `body.command_id_count == body.bindings.len`; `body.occurrence_count` равен сумме occurrence lengths; `Unique` имеет ровно одну occurrence, `Collision` — не меньше двух. Counts и roots пересчитываются при load. Missing, extra или mismatched closure — `COMMAND_BODY_ARCHIVE_CORRUPT`.

`CommandCollisionCandidateV1` list сортируется по `(body_hash, command_id)` и содержит все unique members equivalence class. Exact retry collision проверяет membership через list и body archive. Больше `max_unique_collision_candidates` в одном group отклоняет весь ещё не закрытый external batch как `COMMAND_COLLISION_SET_RESOURCE_LIMIT` до ledger mutation.

Для каждого collision receipt/incident `candidate_count == candidates.len`, list non-empty и recomputed `candidates_root` MUST совпасть. Нарушение — ledger corruption, не обычный command rejection.

```text
collision_incident_digest = SHA256(
  "nextengine.command-collision-incident.v1\0"
  || stream_id
  || u64_le(sequence)
  || candidates_root
)
```

`transaction_result_root` не является stage-11 state root и потому не создаёт hash cycle с receipt ledger:

```text
delta_root = SHA256(
  "nextengine.transaction-deltas.v1\0"
  || u64_le(canonical_delta_bytes.len)
  || canonical_delta_bytes
)

event_root = SHA256(
  "nextengine.transaction-events.v1\0"
  || u32_le(event_ids.len)
  || concat(event_ids)
)

transaction_result_root = SHA256(
  "nextengine.transaction-result.v1\0"
  || command_id_or_collision_root
  || u8(phase)
  || delta_root
  || event_root
)
```

Committed result использует sorted committed deltas и emitted event IDs. Rejected/collision result использует empty canonical delta/event collections. Global state root вычисляется отдельно на stage 11 над post-Outcome owner segments, включая finalized ledger, и не входит обратно в receipt.

Receipt window имеет фиксированную capacity 4096:

```text
receipt_window.len =
  min(finalized_receipt_count, 4096)
```

После 4096 finalizations published snapshot/save MUST содержать **ровно 4096** последних receipts по `finalization_ordinal`; capacity не конфигурируется. При добавлении receipt с ordinal `n` oldest ordinal вытесняется. Ordinal строго возрастает и не оборачивается; exhaustion закрывает stream.

`finalization_ordinal` является per-stream ordinal. Genesis stream имеет
`finalized_receipt_count = 0`, empty window, `chain_root_0` и следующий
ordinal `0`. Перед append runtime требует
`finalization_ordinal == finalized_receipt_count`; после append он выполняет
checked `finalized_receipt_count + 1`. Receipt с ordinal
`u64::MAX` не представим вместе с следующим count, поэтому последний
допустимый receipt имеет ordinal `u64::MAX - 1` и переводит stream в
`Exhausted` атомарно с установкой count в `u64::MAX`. Любая следующая
новая sequence возвращает `COMMAND_FINALIZATION_ORDINAL_EXHAUSTED` без
reservation, identity/archive insert, receipt или gameplay mutation.
Exact retry retained reservation/receipt/incident по-прежнему возвращает
исходный result, а закрытая sequence после eviction возвращает
`COMMAND_SEQUENCE_FINALIZED`; оба пути не требуют нового ordinal.
Persisted `Open` stream с count `u64::MAX`, ordinal gap/duplicate либо
window, не являющимся exact suffix `[count - len, count)`, является
`COMMAND_LEDGER_CORRUPT` и fail-closed. `chain_root_n` ниже означает root
после ровно `n` receipts; receipt ordinal равен `n - 1`.

Receipt chain сохраняет всю terminal историю, включая вытесненные receipts:

```text
receipt_bytes =
  CanonicalBinaryV1(
    owner_id = "nextengine.runtime",
    schema_id = "nextengine.command-receipt",
    segment_id = "v1",
    value = CommandReceiptV1
  )

receipt_digest =
  SHA256(
    "nextengine.command-receipt.v1\0"
    || u64_le(receipt_bytes.len)
    || receipt_bytes
  )

chain_root_0 =
  SHA256("nextengine.command-ledger-chain.genesis.v1\0")

chain_root_n =
  SHA256(
    "nextengine.command-ledger-chain.v1\0"
    || chain_root_(n - 1)
    || receipt_digest
  )
```

Collision-set candidates сортируются по `(body_hash, command_id)` после удаления exact duplicates:

```text
candidates_root =
  SHA256(
    "nextengine.command-collision-set.v1\0"
    || u32_le(candidate_count)
    || concat(body_hash || command_id)
  )
```

## Admission, retry и collision algorithm

Stage 2–4 каждого tick выполняет следующий алгоритм:

1. Decode envelope с bounds; authenticate principal; resolve stream binding; normalize schema, capabilities и preconditions. Malformed или unauthenticated data не меняет high-watermark.
2. Re-encode `CanonicalCommandBodyV2`, вычислить full body hash и command ID. Mismatched claim отклоняется без reservation.
3. Закрыть admission batch. Сгруппировать по `(stream_id, sequence)`, внутри удалить exact duplicates и отсортировать unique candidates по `(body_hash, command_id)`.
4. Проверить batch/body/candidate resource limits до ledger mutation. Сортировать groups по `(stream_id, sequence)`; для каждого stream обрабатывать sequence по возрастанию.
5. До новых archive/index inserts проверить persisted stream state и sequence. Exact retry существующей reservation/receipt/collision membership возвращает прежний result. Если sequence уже закрыта high-watermark и отсутствует в retained state, вернуть `COMMAND_SEQUENCE_FINALIZED`. Для не-`Open` stream любой оставшийся новый candidate получает state-specific rejection без archive/index insert, нового receipt или gameplay mutation.
6. Если `Open` stream и sequence находится в `pending` или retained receipt:
   - тот же `body_hash` и exact archived canonical body — exact retry;
   - для иного hash/body staged compare-or-insert сохраняет каждый bounded candidate в body archive/global identity index, затем создаёт `CommandCollisionIncidentV1`, не меняет исходную reservation/receipt и переводит внешний stream в `CollisionLocked`;
   - exact retry recorded incident возвращает тот же collision response;
   - для `InternalSystem` conflict является fatal invariant до gameplay commit.
7. Для каждого candidate ранее unseen `sequence > high_watermark` выполнить staged compare-or-insert exact bytes в body archive и `(command_id, body_hash)` в global identity index. Любая ID/full-hash collision отклоняет group до reservation, записывает collision incident, блокирует внешний stream или fatal-останавливает `InternalSystem`; staged inserts становятся durable только вместе с terminal receipt/reservation/incident.
8. Для unseen `sequence > high_watermark` все пропущенные sequence закрываются. High-watermark может только увеличиваться. Если sequence равна `u64::MAX`, stream атомарно становится `Exhausted` после reservation/terminal result и больше не принимает новую sequence; pending execution и exact retry остаются допустимы.
9. Если group содержит больше одного unique body:
   - ни один body не выбирается;
   - high-watermark продвигается до sequence;
   - создаётся один terminal collision receipt со всеми sorted members и archive refs;
   - внешний stream становится `CollisionLocked`, internal conflict fatal.
10. Для одного body проверить monotonic target tick, phase authority, future bound, validator-owned priority и domain preconditions:
    - deterministic rejection создаёт terminal receipt и закрывает sequence;
    - accepted future command создаёт reservation;
    - accepted command, due в текущей разрешённой phase, резервируется и передаётся atomic transaction.
11. Transaction либо полностью commit-ит owned deltas/events и receipt, либо создаёт stable rejection receipt без domain mutation. Rollback failure/owner invariant не создаёт ложный success receipt и останавливает instance с crash capsule.

`current_tick` для admission равен `simulation_tick` закрытого
`ClosedCommandAdmissionBatchV2`. Body с `target_tick < current_tick`
получает terminal `COMMAND_TARGET_TICK_EXPIRED`, продвигает high-watermark
и закрывает sequence receipt-ом без reservation или domain mutation.
Неявный перенос просроченной команды запрещён. Body с
`target_tick == current_tick` допустим только в совпадающей declared phase
и только в fixed close barrier этой phase. Envelope, linearized после
barrier, попадает в следующий declared batch; если сохранённый в body tick
к тому моменту стал прошлым, он получает тот же expired result. Внешний
principal не может запросить `Outcome`; runtime-generated command,
предложенный во время `Outcome`, получает `target_tick >= current_tick + 1`.

Для accepted commands одного stream `target_tick` MUST быть не меньше
`greatest_reserved_target_tick`; regression получает
`COMMAND_STREAM_TIME_REGRESSION`. Проверка past/current phase выполняется
раньше этой monotonic проверки. Default future bound — 120 gameplay ticks,
hard maximum profile — 3600; project может выбрать меньшее значение до
создания мира. Player input не выбирает `target_tick`: production input
mapper ставит tick из persisted `IngressAssignmentV1`.

Exact retry означает:

- pending retry возвращает ту же reservation;
- retained committed/rejected/collision member retry возвращает тот же receipt;
- ни один retry не создаёт новый `DomainEvent`, spawn, state delta или receipt ordinal;
- retry после eviction возвращает только `COMMAND_SEQUENCE_FINALIZED`, а не реконструированный приблизительный result.

## Ingress cutoff, clocks и async completion

Current packaged world streaming is the production async-completion consumer
of this fixed-stage rule. A revision-bound result is validated together with a
prepared Runtime tick and published infallibly at `WorldStreamingCommit`
between `IngressCommit` and `PhysicalStep`. The four-region/64-chunk reference
partition uses the same two existing ticks for every mandatory transition:
one publishes `Requested`, the next eligible tick publishes completion. I/O
duration pauses simulation advancement and never selects another simulation
tick. A stale/faulted completion publishes neither the Runtime nor World staged
generation. This consumer profile is governed by SPEC-03/SPEC-25/ADR-051 and
does not create a generic task scheduler or public completion framework here.

### TickRateProfileV1

```text
TickRateProfileV1 {
  1 schema_version: u16 = 1,
  2 gameplay_hz: u32,
  3 physics_substeps_per_gameplay_tick: u32,
  4 motor_period_physics_substeps: u32,
  5 first_gameplay_tick: u64 = 0,
}
```

Profile валидируется до world creation. `gameplay_hz` MUST быть одним из `{20, 30, 60}`; derived `physics_hz = gameplay_hz * physics_substeps_per_gameplay_tick` MUST быть одним из `{60, 120, 240}`; motor period ненулевой и делит physics cadence. Time хранится integer ticks/rational conversion; accumulated float seconds запрещены. Profile hash входит в world/save/replay. Изменение после первого tick требует explicit world migration.

### Current/next ingress

```text
InputSampleV1 {
  1 schema_version: u16 = 1,
  2 source_class: NamespacedId,
  3 source_id: Id128,
  4 source_sequence: u64,
  5 payload_schema_id: SchemaId,
  6 payload_schema_version: u32,
  7 payload: CanonicalBinary,
  8 sampled_wall_time: Option<NonAuthoritativeTimestamp>,
}

VirtualInputEventV1 {
  1 schema_version: u16 = 1,
  2 assigned_tick: SimulationTick,
  3 source_class: NamespacedId,
  4 source_id: Id128,
  5 source_sequence: u64,
  6 payload_schema_id: SchemaId,
  7 payload_schema_version: u32,
  8 payload: CanonicalBinary,
}

IngressAssignmentV1 {
  1 schema_version: u16 = 1,
  2 queue_generation: u64,
  3 assigned_tick: SimulationTick,
  4 source_class: NamespacedId,
  5 source_id: Id128,
  6 source_sequence: u64,
  7 payload_hash: Hash256,
}

IngressEquivalenceReceiptV1 {
  1 assigned_tick: SimulationTick,
  2 subject_kind: Input | Completion,
  3 subject_id: CanonicalBinary,
  4 candidate_hashes: CanonicalSet<Hash256>,
  5 result_code: StableDiagnosticCode,
}

ClosedIngressBatchBodyV1 {
  1 schema_version: u16 = 1,
  2 queue_generation: u64,
  3 assigned_tick: SimulationTick,
  4 input_samples: Vec<InputSampleV1>,
  5 completion_signals: Vec<CompletionSignalV1>,
  6 input_assignments: Vec<IngressAssignmentV1>,
  7 completion_assignments: Vec<CompletionAssignmentV1>,
  8 equivalence_receipts: Vec<IngressEquivalenceReceiptV1>,
}

ClosedIngressBatchV1 {
  1 schema_version: u16 = 1,
  2 body: ClosedIngressBatchBodyV1,
  3 batch_hash: Hash256,
}
```

Ingress adapter имеет две logical queues: `current(T)` и `next(T + 1)`. В начале stage 1 runtime выполняет один linearizable close/swap barrier:

- enqueue, linearized до barrier, принадлежит закрываемому current batch tick `T`;
- enqueue, linearized в barrier или после него, принадлежит next batch tick `T + 1`;
- wall-clock timestamp не решает сторону cutoff;
- renderer frame, OS callback thread и worker order не участвуют.

При создании мира `current(first_gameplay_tick)` имеет
`queue_generation = 0`; `next` имеет ещё не опубликованную generation `1`.
Каждый успешно закрытый `ClosedIngressBatchV1` использует generation
текущей queue, и все его input/completion assignments обязаны иметь ровно
то же значение. Atomic publish/close/swap выполняет ровно один checked
increment и делает `g + 1` новой current generation; save/restart сохраняет
`g` и не создаёт дополнительного increment. При `g = u64::MAX` новый
barrier не начинается: runtime возвращает
`INGRESS_QUEUE_GENERATION_EXHAUSTED`, не публикует batch, assignment,
command, event или partial tick и сохраняет предыдущий valid checkpoint.
Generation gap, duplicate generation либо mismatch batch/assignment при
load/replay является `INGRESS_ASSIGNMENT_CORRUPT`.

Input payload hash:

```text
input_payload_hash = SHA256(
  "nextengine.input-payload.v1\0"
  || lp(payload_schema_id)
  || u32_le(payload_schema_version)
  || u64_le(payload.len)
  || payload
)
```

Current batch сортируется по `(source_class, source_id_bytes, source_sequence, input_payload_hash)`. Exact duplicate удаляется. Один source key с разными payload hashes получает persisted `IngressEquivalenceReceiptV1(INPUT_SEQUENCE_COLLISION)`; ни один конфликтующий input не становится `VirtualInputEvent`.

`ClosedIngressBatchBodyV1` сохраняет все bounded decoded samples/signals, включая duplicates/conflicts, и assignments/receipts. Его vectors находятся в указанном canonical order. `body_bytes` кодируется envelope `nextengine.runtime/nextengine.closed-ingress-batch-body/v1` после замены wall timestamp полей на `None`; raw timestamp остаётся только отдельной diagnostic metadata.

```text
batch_hash = SHA256(
  "nextengine.closed-ingress-batch.v1\0"
  || u64_le(body_bytes.len)
  || body_bytes
)
```

Hash field находится только во внешнем `ClosedIngressBatchV1` и не входит в собственный preimage. Replay хранит closed batch и не пересчитывает assignment по timestamp.

### CompletionSignalV1

```text
TypedTaskOutcomeV1 =
  0x01 Success {
    payload: CanonicalBinary,
  }
  | 0x02 Failure {
    code: StableDiagnosticCode,
    details_hash: Option<Hash256>,
  }

CompletionSignalV1 {
  1 schema_version: u16 = 1,
  2 owner_id: OwnerId,
  3 request_id: Id128,
  4 input_hash: Hash256,
  5 result_schema_id: SchemaId,
  6 result_schema_version: u32,
  7 result_hash: Hash256,
  8 precondition_revision: u64,
  9 outcome: TypedTaskOutcomeV1,
}

CompletionAssignmentV1 {
  1 schema_version: u16 = 1,
  2 queue_generation: u64,
  3 assigned_tick: SimulationTick,
  4 owner_id: OwnerId,
  5 request_id: Id128,
  6 input_hash: Hash256,
  7 result_hash: Hash256,
}
```

Outcome кодируется `CanonicalBinaryV1` envelope `nextengine.runtime/nextengine.task-outcome/v1`:

```text
result_hash = SHA256(
  "nextengine.task-result.v1\0"
  || lp(result_schema_id)
  || u32_le(result_schema_version)
  || u64_le(outcome_bytes.len)
  || outcome_bytes
)
```

Validator re-encode-ит outcome и проверяет hash. Async result публикуется в тот же current/next cutoff и получает `CompletionAssignmentV1`. Commit candidates сортируются по `(owner_id, request_id, result_hash)`. Exact duplicate dedupлицируется; один request с разными result hashes получает persisted `IngressEquivalenceReceiptV1(TASK_RESULT_COLLISION)`. Changed precondition даёт `STALE_REVISION`. Task получает только owned immutable input и не удерживает mutable ECS/backend access через `await`.

### Ingress и Outcome после save/restart

`Ingress` commit происходит на stage 5; `Outcome` создаётся ровно одним закрытым `InternalSystem` batch на stage 9. Outcome использует тот же body/hash/admission/transaction/receipt path. Script, plugin, AI, tool, player и public transport не могут создать `Outcome`.

- Command, предложенный во время `Outcome`, назначается не раньше следующего gameplay tick.
- Второй stage-9 entry даёт `OUTCOME_REENTRY_FORBIDDEN` и invalidates run.
- Ledger changes, state deltas, events и stage marker одного phase commit атомарны.
- Published checkpoint появляется только после завершения обеих phases и state root stage 11.
- Crash до checkpoint восстанавливает последний полностью committed tick и replay-ит сохранённые assignments; prepared staging journal отбрасывается.
- Crash после atomic checkpoint начинает следующий tick; exact retry retained command возвращает receipt без republish events.
- Future `Ingress` reservations сохраняются. Published save MUST NOT содержать незавершённую `Outcome` reservation для current/past tick; такая запись даёт `LEDGER_OUTCOME_PARTIAL` и fail-closed recovery к предыдущей valid generation.

## Deterministic RNG

### Stream schemas и derivation

```text
RngStreamDescriptorV1 {
  1 schema_version: u16 = 1,
  2 owner_id: OwnerId,
  3 stream_name: NamespacedId,
  4 stable_subject_id: CanonicalIdBytes,
  5 instance_ordinal: u64,
}

RngStreamStateV1 {
  1 schema_version: u16 = 1,
  2 algorithm_id: NamespacedId = "nextengine.chacha12-ietf.v1",
  3 stream_id: RngStreamId,
  4 block_counter: u32,
  5 next_word_index: u8,
  6 exhausted: bool,
}

RngProfileV1 {
  1 schema_version: u16 = 1,
  2 algorithm_id: NamespacedId,
  3 derivation_profile_id: NamespacedId,
  4 stream_descriptors: CanonicalSet<RngStreamDescriptorV1>,
}
```

`stream_material`:

```text
rng_root_seed
|| lp(canonical_owner_id)
|| lp(canonical_stream_name)
|| lp(canonical_stable_subject_id)
|| u64_le(instance_ordinal)
```

Derivation:

```text
rng_stream_id =
  left128(
    SHA256(
      "nextengine.rng-stream.v1\0"
      || stream_material
    )
  )

key =
  SHA256(
    "nextengine.rng-key.chacha12.v1\0"
    || stream_material
  )

nonce =
  first96(
    SHA256(
      "nextengine.rng-nonce.chacha12.v1\0"
      || stream_material
    )
  )
```

Descriptor registry предотвращает implicit allocation. Один descriptor всегда обозначает один stream; независимая потребность получает отдельные stable `stream_name`/subject/ordinal. Descriptor canonical hash регистрируется как `RngStream` provenance в `CausalIdentityRegistryV1` до первого draw; conflicting binding даёт `RNG_STREAM_ID_COLLISION`. Global RNG, RNG на worker/thread, allocation по iteration order и чтение OS entropy во время simulation запрещены.

### ChaCha12IetfV1 exact profile

State состоит из 16 little-endian `u32` words:

```text
0..3   = 0x61707865, 0x3320646e, 0x79622d32, 0x6b206574
4..11  = eight key words
12     = block_counter
13..15 = three nonce words
```

Quarter round `QR(a,b,c,d)`:

```text
a = a +% b; d = rotl(d xor a, 16)
c = c +% d; b = rotl(b xor c, 12)
a = a +% b; d = rotl(d xor a, 8)
c = c +% d; b = rotl(b xor c, 7)
```

`+%` — wrapping `u32` addition. Один double round выполняет:

```text
QR(0,4,8,12); QR(1,5,9,13); QR(2,6,10,14); QR(3,7,11,15)
QR(0,5,10,15); QR(1,6,11,12); QR(2,7,8,13); QR(3,4,9,14)
```

ChaCha12 выполняет ровно 6 double rounds, затем word-wise wrapping-add initial state и выдаёт 64 bytes как 16 little-endian words. Первый block counter равен 0.

Initial state: `block_counter = 0`, `next_word_index = 0`, `exhausted = false`. `next_u32` возвращает текущий word, затем увеличивает index. После word 15 index становится 0 и counter увеличивается; после word 15 при `counter = u32::MAX` counter остаётся `u32::MAX`, index становится 0 и `exhausted = true`. Следующий draw возвращает `RNG_STREAM_EXHAUSTED` без state mutation. Единственный valid exhausted representation — `(u32::MAX, 0, true)`; non-exhausted state требует `next_word_index <= 15`. Иные persisted combinations corrupt.

Нормативные helpers:

- `next_u64`: первый `next_u32` — low word, второй — high word;
- `uniform_below_u32(n)`: `n = 0` invalid; `threshold = 2^32 mod n`; draws меньше threshold отбрасываются, результат `draw mod n`;
- `sample_q0_32`: raw `next_u32`, где значение интерпретируется как unsigned Q0.32 без float conversion;
- Fisher–Yates: для `i = len - 1` до `1` выбрать `j = uniform_below_u32(i + 1)` и swap.

Authoritative float distributions, transcendental functions и backend/library default distributions не являются API. RNG state сохраняется после каждого committed owner transaction; aborted transaction возвращает stream state к pre-transaction value.

## Stable schedule, queries, shards и task merge

### Public schemas

```text
AccessKeyV1 {
  owner_id: OwnerId,
  schema_id: SchemaId,
  field_id: u32,
}

AccessSetV1 {
  reads: CanonicalSet<AccessKeyV1>,
  writes: CanonicalSet<AccessKeyV1>,
}

QueryOrderV1 =
  0x01 PersistentId
  | 0x02 StableRecordKey { key_schema_id: SchemaId }

ReducerKindV1 =
  0x01 CheckedSum
  | 0x02 Minimum
  | 0x03 Maximum
  | 0x04 BitAnd
  | 0x05 BitOr
  | 0x06 BitXor
  | 0x07 StableOrderedFold

ShardPartitionRule =
  0x01 Sha256StableKeyFirstU64LeModulo

ShardRecordOrder =
  0x01 CanonicalStableRecordKey

DeltaMergeOrder =
  0x01 OwnerSchemaRecordFieldSystemShard

ReducerDescriptorV1 {
  1 schema_version: u16 = 1,
  2 reducer_id: ReducerId,
  3 target: AccessKeyV1,
  4 value_schema_id: SchemaId,
  5 kind: ReducerKindV1,
  6 ordered_fold_schema_hash: Option<Hash256>,
}

LogicalShardPlanV1 {
  1 schema_version: u16 = 1,
  2 shard_plan_id: NamespacedId,
  3 system_id: SystemId,
  4 logical_shard_count: u32,
  5 partition_rule: ShardPartitionRule = Sha256StableKeyFirstU64LeModulo,
  6 record_order: ShardRecordOrder = CanonicalStableRecordKey,
  7 merge_order: DeltaMergeOrder = OwnerSchemaRecordFieldSystemShard,
}

CommandBarrierSourceV1 =
  0x01 AuthenticatedExternalAndQueuedInternal
  | 0x02 InternalSystemOnly

CommandAdmissionBarrierV1 {
  1 schema_version: u16 = 1,
  2 phase: CommandPhase,
  3 stage_index: u8,
  4 batch_ordinal: u32,
  5 source: CommandBarrierSourceV1,
}

SystemDescriptorV1 {
  1 schema_version: u16 = 1,
  2 system_id: SystemId,
  3 owner_id: OwnerId,
  4 stage_id: RuntimeStageId,
  5 before: CanonicalSet<SystemId>,
  6 after: CanonicalSet<SystemId>,
  7 access: AccessSetV1,
  8 query_order: QueryOrderV1,
  9 shard_plan_id: NamespacedId,
  10 reducer_ids: CanonicalSet<ReducerId>,
}

ScheduleManifestV1 {
  1 schema_version: u16 = 1,
  2 stage_order: Vec<RuntimeStageId>,
  3 systems: CanonicalMap<SystemId, SystemDescriptorV1>,
  4 reducers: CanonicalMap<ReducerId, ReducerDescriptorV1>,
  5 shard_plans: CanonicalMap<NamespacedId, LogicalShardPlanV1>,
  6 command_admission_barriers: Vec<CommandAdmissionBarrierV1>,
}

DeterministicDeltaV1 {
  1 owner_id: OwnerId,
  2 schema_id: SchemaId,
  3 stable_record_key: CanonicalIdBytes,
  4 field_id: u32,
  5 system_id: SystemId,
  6 shard_index: u32,
  7 value: CanonicalBinary,
  8 reducer_id: Option<ReducerId>,
}
```

Stage order остаётся 12-stage order SPEC-02. Edges не могут идти назад через stage boundary. Внутри stage manifest строит DAG:

1. Validate all referenced systems/stages/access keys/reducers/barriers.
2. Если write/write либо read/write conflict не имеет directed path в одну сторону, вернуть `SCHEDULE_ACCESS_AMBIGUOUS`; hidden auto-order запрещён.
3. Cycle возвращает `SCHEDULE_CYCLE`.
4. Stable topological sort каждый раз выбирает lexicographically smallest canonical `SystemId` среди zero-indegree nodes.

System registration order не влияет на manifest hash или order.

`command_admission_barriers` сортируется по
`(stage_index, phase_tag, batch_ordinal)`. `stage_index` использует
one-based index в exact `stage_order`, должен быть в `1..=12`; ordinals
каждой `(tick, phase)` начинаются с `0` и не имеют gaps. V1 manifest
содержит ровно две записи: `Ingress`, stage 2, ordinal `0`,
`AuthenticatedExternalAndQueuedInternal`; и `Outcome`, stage 9, ordinal
`0`, `InternalSystemOnly`. Missing, extra или reordered barrier блокирует
world open как `SCHEDULE_COMMAND_BARRIER_MISMATCH`.

Authoritative query MUST объявить tag `0x01 PersistentId` либо `0x02 StableRecordKey` с registered key schema. ECS archetype/chunk/storage order, `RuntimeEntityId`, pointer address и hash-map iteration запрещены. Duplicate stable key — invariant violation до system execution.

Каждый system ссылается ровно на один `LogicalShardPlanV1` с matching `system_id`. Logical shard count больше нуля и фиксирован manifest-ом. Worker count не меняет shards:

```text
shard_index =
  decode_u64_le(first_8_bytes(SHA256(
    "nextengine.logical-shard.v1\0"
    || lp(stable_record_key)
  )))
  mod logical_shard_count
```

Каждый shard сортирует records по stable key. Dispatch MAY быть parallel; merge всегда сортирует deltas по:

```text
(owner_id, schema_id, stable_record_key, field_id, system_id, shard_index)
```

Две записи одного `(owner_id, schema_id, stable_record_key, field_id)` запрещены без единственного `ReducerDescriptorV1`, whose `target == (owner_id, schema_id, field_id)` совпадает с delta access key. Каждый target имеет не больше одного reducer; каждая contributing delta несёт тот же `reducer_id`, а canonical `value` обязан соответствовать `ReducerDescriptorV1.value_schema_id`. `None` допустим только при единственном writer. Разрешены:

- checked fixed-point/integer sum;
- integer/fixed-point `min` или `max`;
- integer bitwise `and`, `or`, `xor`;
- engine-owned stable ordered fold с non-None versioned `ordered_fold_schema_hash` и входами в полном merge order.

Другие kinds требуют `ordered_fold_schema_hash = None`. Float associative reduction, unordered fold и package-supplied native callback запрещены. Reducer overflow aborts transaction.

Task result merge происходит только на declared commit stage по:

```text
(commit_stage, owner_id, request_id, result_hash)
```

Exact duplicate удаляется. Один request с разными hashes отклоняет всю equivalence class. Worker panic/cancellation отбрасывает незакоммиченный batch; task не публикует частичный delta. Stale precondition отклоняется до merge.

Canonical `ScheduleManifestV1` hash и task-merge profile входят в `RuntimeDeterminismProfileV1`, build/save/replay. Schedule mismatch блокирует load/replay до execution.

## Authoritative numerics и physics quantization

### Numeric schemas

```text
FixedPointDescriptorV1 {
  1 descriptor_id: NamespacedId,
  2 storage_bits: u8,                 // 16, 32 или signed 64
  3 signed: bool,
  4 fractional_bits: u8,
  5 minimum_raw: i64,
  6 maximum_raw: i64,
  7 rounding: RoundingMode = NearestTiesToEven,
  8 overflow: OverflowMode = RejectTransaction,
}

AuthoritativeNumericProfileV1 {
  1 schema_version: u16 = 1,
  2 integer_overflow: OverflowMode = RejectTransaction,
  3 fixed_points: CanonicalMap<NamespacedId, FixedPointDescriptorV1>,
  4 authoritative_float_reduction: bool = false,
  5 fast_math_allowed: bool = false,
  6 physics_quantization_profile_hash: Hash256,
}

PhysicsQuantizationRuleV1 {
  1 field_id: NamespacedId,
  2 source_format: PhysicsSourceFormat,      // IEEE-754 binary32 или binary64
  3 source_unit: NamespacedId,
  4 destination_unit: NamespacedId,
  5 destination_fixed_point: NamespacedId,
  6 scale_numerator: i64,
  7 scale_denominator: u64,
  8 offset_raw: i64,
  9 minimum_raw: i64,
  10 maximum_raw: i64,
}

PhysicsQuantizationProfileV1 {
  1 schema_version: u16 = 1,
  2 profile_id: NamespacedId,
  3 rules: CanonicalMap<NamespacedId, PhysicsQuantizationRuleV1>,
}

QuantizedPhysicsProjectionV1 {
  1 schema_version: u16 = 1,
  2 physics_tick: u64,
  3 subject_id: PersistentId,
  4 body_slot: u32,
  5 profile_hash: Hash256,
  6 values: CanonicalMap<NamespacedId, i64>,
}
```

Для fixed point `fractional_bits < storage_bits` и declared raw bounds должны помещаться в storage. Unsigned v1 descriptor разрешает только 16/32-bit storage; 64-bit descriptor MUST быть signed, чтобы все bounds и checked intermediates имели одну exact representation. Add/sub/mul/div используют signed `i128` intermediate. Multiplication делит raw product на `2^fractional_bits`; division сначала сдвигает numerator на `fractional_bits`. Division by zero invalid.

Round-to-nearest-ties-to-even определяется без float:

1. вычислить quotient, truncated toward zero, и remainder;
2. сравнить `2 * abs(remainder)` с `abs(divisor)`;
3. меньше — оставить quotient;
4. больше — увеличить magnitude quotient на один;
5. равно — оставить even quotient, иначе увеличить magnitude на один.

Результат вне declared raw bounds, intermediate overflow или divide-by-zero aborts всю transaction как `NUMERIC_OVERFLOW`; wrapping/saturation не допускаются, если отдельная domain schema явно не кодирует bounded clamp до arithmetic operation.

Backend physics values остаются внутри Physical context. Source bits означают значение `x` в declared `source_unit`; `scale_numerator / scale_denominator` переводит одну source unit в `destination_unit`. Unit registry mismatch отклоняется до conversion. Для destination descriptor с `F = fractional_bits` exact destination raw rational равен:

```text
raw_exact =
  x
  * scale_numerator
  * 2^F
  / scale_denominator
  + offset_raw
```

Adapter выполняет:

1. decode IEEE-754 binary32/binary64 bits напрямую в exact signed rational `x`, без host float arithmetic;
2. normalize negative zero в exact zero;
3. reject NaN/infinity;
4. проверить nonzero denominator, unit IDs, destination descriptor и declared rule bounds;
5. вычислить `raw_exact` как rational с checked signed `i128` numerator/denominator; representability overflow отклоняется;
6. выполнить **одно** round-to-nearest-ties-to-even над всем `raw_exact`;
7. проверить descriptor и rule bounds;
8. emit полученный `i64 raw` в `QuantizedPhysicsProjectionV1`.

Map key каждой rule MUST равняться её `field_id`; duplicate target field запрещён. Projection `values` содержит ровно required fields active profile для subject/body role, sorted по canonical field ID; missing/extra field fail-closed.

```text
physics_projection_bytes =
  CanonicalBinaryV1(
    owner_id = "nextengine.physics",
    schema_id = "nextengine.quantized-physics-projection",
    segment_id = "v1",
    value = QuantizedPhysicsProjectionV1
  )

physics_projection_root =
  SHA256(
    "nextengine.physics-projection.v1\0"
    || u64_le(physics_projection_bytes.len)
    || physics_projection_bytes
  )
```

Gameplay classification, event emission, contact order, IDs и state root используют только quantized projection/declared exact thresholds. Tolerance MAY использоваться только для явно неавторитетной physics-quality metric; она не применяется к command/event/outcome comparison. Fast-math, implicit epsilon, unordered float sum и raw backend bits в public/save/replay contracts запрещены.

## RuntimeDeterminismProfileV1 и persistence

### Hash-bound component profiles

```text
IngressCutoffPolicy = 0x01 LinearizableCurrentNext
InputSortProfile = 0x01 SourceClassIdSequencePayloadHash
CompletionSortProfile = 0x01 OwnerRequestResultHash
EquivalenceCollisionPolicy = 0x01 RejectAllPersistReceipt
BodyHashAlgorithm = 0x01 Sha256RawCanonicalBytes
TruncationRule = 0x01 Left128
SequencePolicy = 0x01 StrictHighWatermarkGapsFinal
RetryPolicy = 0x01 PendingOrRetainedExactElseFinalized
CollisionPolicy = 0x01 RejectAllAndLockExternalFatalInternal
TaskSortProfile = 0x01 CommitStageOwnerRequestResultHash
DuplicatePolicy = 0x01 ExactDeduplicate
ConflictPolicy = 0x01 RejectEquivalenceClass
StalePolicy = 0x01 RejectBeforeMerge

IngressAssignmentProfileV1 {
  1 schema_version: u16 = 1,
  2 cutoff_policy: IngressCutoffPolicy = LinearizableCurrentNext,
  3 input_sort: InputSortProfile = SourceClassIdSequencePayloadHash,
  4 completion_sort: CompletionSortProfile = OwnerRequestResultHash,
  5 collision_policy: EquivalenceCollisionPolicy = RejectAllPersistReceipt,
  6 admission_limits_hash: Hash256,
}

CommandIdentityProfileV2 {
  1 schema_version: u16 = 2,
  2 canonical_binary_version: u16 = 1,
  3 canonical_body_schema_hash: Hash256,
  4 body_hash_algorithm: BodyHashAlgorithm = Sha256RawCanonicalBytes,
  5 command_id_domain: Utf8Nfc = "nextengine.command-id.v2",
  6 command_id_truncation: TruncationRule = Left128,
}

CommandLedgerProfileV2 {
  1 schema_version: u16 = 2,
  2 receipt_window_size: u32 = 4096,
  3 maximum_pending_per_stream: u32 = 256,
  4 sequence_policy: SequencePolicy = StrictHighWatermarkGapsFinal,
  5 retry_policy: RetryPolicy = PendingOrRetainedExactElseFinalized,
  6 collision_policy: CollisionPolicy = RejectAllAndLockExternalFatalInternal,
  7 receipt_chain_profile_id: NamespacedId = "nextengine.command-ledger-chain.v1",
  8 body_archive_profile_id: NamespacedId = "nextengine.command-body-archive.v1",
  9 admission_limits_hash: Hash256,
}

TaskMergeProfileV1 {
  1 schema_version: u16 = 1,
  2 sort_profile: TaskSortProfile = CommitStageOwnerRequestResultHash,
  3 duplicate_policy: DuplicatePolicy = ExactDeduplicate,
  4 conflict_policy: ConflictPolicy = RejectEquivalenceClass,
  5 stale_policy: StalePolicy = RejectBeforeMerge,
  6 maximum_result_bytes: u32,
}

RuntimeDeterminismProfileV1 {
  1 schema_version: u16 = 1,
  2 tick_rate_profile_hash: Hash256,
  3 ingress_assignment_profile_hash: Hash256,
  4 command_identity_profile_hash: Hash256,
  5 command_ledger_profile_hash: Hash256,
  6 rng_profile_hash: Hash256,
  7 schedule_manifest_hash: Hash256,
  8 task_merge_profile_hash: Hash256,
  9 numeric_profile_hash: Hash256,
  10 physics_quantization_profile_hash: Hash256,
  11 command_kind_registry_hash: Hash256,
  12 admission_limits_profile_hash: Hash256,
  13 maximum_future_command_ticks: u32,
}
```

Exact profile envelope IDs:

| Type | owner_id | schema_id | segment_id |
|---|---|---|---|
| `TickRateProfileV1` | `nextengine.runtime` | `nextengine.tick-rate-profile` | `v1` |
| `IngressAssignmentProfileV1` | `nextengine.runtime` | `nextengine.ingress-assignment-profile` | `v1` |
| `CommandIdentityProfileV2` | `nextengine.runtime` | `nextengine.command-identity-profile` | `v2` |
| `CommandLedgerProfileV2` | `nextengine.runtime` | `nextengine.command-ledger-profile` | `v2` |
| `RuntimeAdmissionLimitsV1` | `nextengine.runtime` | `nextengine.runtime-admission-limits` | `v1` |
| `RngProfileV1` | `nextengine.runtime` | `nextengine.rng-profile` | `v1` |
| `ScheduleManifestV1` | `nextengine.runtime` | `nextengine.schedule-manifest` | `v1` |
| `TaskMergeProfileV1` | `nextengine.runtime` | `nextengine.task-merge-profile` | `v1` |
| `AuthoritativeNumericProfileV1` | `nextengine.runtime` | `nextengine.authoritative-numeric-profile` | `v1` |
| `PhysicsQuantizationProfileV1` | `nextengine.physics` | `nextengine.physics-quantization-profile` | `v1` |
| `RuntimeDeterminismProfileV1` | `nextengine.runtime` | `nextengine.runtime-determinism-profile` | `v1` |

Exact profile hash:

```text
profile_hash = SHA256(
  "nextengine.runtime-profile.v1\0"
  || lp(canonical_schema_id)
  || u64_le(profile_bytes.len)
  || profile_bytes
)
```

`profile_bytes` — полностью canonical `CanonicalBinaryV1` envelope, а schema ID обязан совпасть с envelope. Loader re-encode-ит profile и проверяет hash; неизвестный enum/tag/profile fail-closed. `ScheduleManifestV1` и `PhysicsQuantizationProfileV1` используют ту же formula со своими schema IDs.

Canonical profile hash входит в build metadata, `WorldIdentityManifestV1`, `SaveManifest`, `ReplayManifest`, root `RunManifest` и capture job. `game`, `headless` и `capture-worker` MUST отвергнуть несовместимый required profile до partial world mutation.

Published save MUST атомарно содержать:

- `WorldIdentityManifestV1`;
- `PrincipalRegistryV1` и `CausalIdentityRegistryV1`;
- `CommandStreamRegistryV1`;
- `CommandLedgerV2`, включая body archive/index, каждый полный stream ledger, pending, exact 4096 receipt rule, chain count/root и state;
- tick/cutoff generation и future ingress assignments;
- все closed ingress batches, ещё необходимые для recovery;
- все RNG stream states;
- schedule/task/numeric/physics profile hashes;
- owner state segments и root после обеих commit phases.

```text
ClosedCommandAdmissionBatchBodyV2 {
  1 schema_version: u16 = 2,
  2 simulation_tick: SimulationTick,
  3 phase: CommandPhase,
  4 batch_ordinal: u32,
  5 envelopes: Vec<WorldCommandEnvelopeV2>,
}

ClosedCommandAdmissionBatchV2 {
  1 schema_version: u16 = 2,
  2 body: ClosedCommandAdmissionBatchBodyV2,
  3 batch_hash: Hash256,
}
```

Phase-local batch allocator создаётся при открытии каждого
`(simulation_tick, phase)` со следующим `batch_ordinal = 0`; новый tick
или другая phase создаёт отдельный allocator и не продолжает ordinal.
Каждый engine-declared close barrier потребляет current ordinal, даже для
empty batch, затем вычисляет следующий checked `u32 + 1`. Batch с ordinal
`u32::MAX` является последним представимым и переводит allocator в
`Exhausted`; ещё один close возвращает
`COMMAND_BATCH_ORDINAL_EXHAUSTED` до закрытия batch, ledger mutation или
domain execution. Gaps, duplicates, перестановка ordinal-ов либо
необъявленная boundary при replay дают `NONDETERMINISTIC_RESULT`.

V1 `ScheduleManifestV1` объявляет ровно один command-admission close для
`Ingress` и ровно один закрытый `InternalSystem` batch для `Outcome` на
gameplay tick; оба имеют ordinal `0`. Количество barriers не зависит от
того, пусты ли queues, и от transport/worker arrival. Изменение количества
barriers требует нового hash-bound schedule/runtime profile.

Replay хранит initial checkpoint, все `ClosedIngressBatchV1` и все bounded/authenticated/decodable external `WorldCommandEnvelopeV2` в batch body, включая exact duplicates, conflicting bodies, invalid claim, deterministic rejection и finalized retry. Envelope transport metadata заменяется `None`; candidates сортируются по `(stream_id, sequence, body_hash, claimed_command_id)` с сохранением batch boundary.

`body_bytes` кодируется envelope `nextengine.runtime/nextengine.closed-command-admission-batch-body/v2`. Внешний hash:

```text
batch_hash = SHA256(
  "nextengine.closed-command-admission-batch.v2\0"
  || u64_le(body_bytes.len)
  || body_bytes
)
```

Hash field находится только во внешнем `ClosedCommandAdmissionBatchV2` и не входит в собственный preimage.

Malformed/unauthenticated transport bytes не являются gameplay input; отдельно MAY храниться только raw hash/diagnostic. Runtime-generated Outcome повторно выводится production systems; expected receipts/events MAY быть oracle, но execution повторно проходит production validator/ledger/RNG/schedule path. First mismatch command bytes/hash/rejection/receipt, assignment, RNG state, schedule delta, projection, event или state root немедленно даёт `NONDETERMINISTIC_RESULT` с first divergent tick/stage/owner; repeating identical inputs cannot change that result.

Corrupt/incompatible ledger, invalid RNG state, unknown schedule/numeric profile или partial Outcome checkpoint fail-closed. Loader работает с копией, сохраняет предыдущую valid save generation и не публикует partial state.

## Technical requirements

| ID | Technical requirement |
|---|---|
| REQ-103 | `CanonicalCommandBodyV2` MUST быть несамоссылочным, иметь exact canonical encoding/full body hash/computed ID и проверять envelope claim до mutation. |
| REQ-104 | World, principal, stream, event и runtime-created durable identity MUST иметь persisted causal derivation, provenance и fail-closed collision policy. |
| REQ-105 | Каждый command stream MUST иметь persisted high-watermark, pending reservations, fixed 4096 receipt window, chain root и exact retry/collision/finalized/exhaustion/restart semantics. |
| REQ-106 | `game`, `headless`, `capture-worker` и replay MUST разделять admission, priority/order, two-phase Ingress/Outcome transaction и receipt semantics. |
| REQ-107 | Input и async completion MUST назначаться current/next virtual tick exact cutoff-ом, canonical source ordering и recorded assignment без wall-clock authority. |
| REQ-108 | Authoritative randomness MUST использовать versioned named ChaCha12 streams с exact derivation, operations, state persistence и exhaustion behavior. |
| REQ-109 | System/task execution MUST использовать stable DAG, ordered queries, worker-independent logical shards, deterministic merge и declared exact reducers. |
| REQ-110 | Authoritative arithmetic и physics boundary MUST использовать hash-bound checked fixed-point/quantization/overflow profile, сохранённый в compatibility metadata. |

## Failure paths

| ID | Trigger | Required result |
|---|---|---|
| FAIL-039 | Claimed-ID mismatch, command/body collision, sequence conflict/reuse/finalized/time regression/exhaustion | Return a stable diagnostic, publish no partial gameplay mutation, preserve the prior exact result, isolate an external stream where specified and treat internal collision as fatal. |
| FAIL-040 | World/stream/event/`PersistentId` collision, invalid causal provenance or corrupt ledger/save | Fail closed before partial load/commit, retain the previous valid save and stop the instance on internal identity collision. |
| FAIL-041 | Invalid/late/conflicting input, stale/conflicting completion, invalid RNG state/request or RNG exhaustion | Produce the exact reject/defer result with no unauthorized state change or partial task/RNG commit. |
| FAIL-042 | Schedule cycle/access ambiguity/duplicate stable key/unordered authority, reducer or numeric overflow, nonfinite physics value or repeat divergence | Reject before world activation or abort the complete transaction; divergence returns `NONDETERMINISTIC_RESULT`. |

## Product checks

| ID | Scenario / command | Expected behavior | Fallback |
|---|---|---|---|
| COMMAND-ID-P1 | `next check command-id --bounds n-1,n,n+1 --targets windows-x86_64,linux-x86_64` | Body/envelope, Unicode/schema, claim and collision vectors produce exact canonical bytes, hashes, IDs and codes; invalid vectors mutate no gameplay state. | Reject the schema/hash implementation. |
| CAUSAL-ID-P1 | `next check causal-id --cases world,principal,stream,event,spawn,retry,tombstone,save` | Golden IDs are exact and every provenance collision fails before alternate ID or mutation. | Retain the prior identity state and reject the operation. |
| COMMAND-LEDGER-P1 | `next check command-ledger --receipts 0,1,4095,4096,4097 --pending 255,256,257 --permutations 10000` | High-watermark, pending set, receipt window/chain and event/state roots remain exact across arrival and restart permutations. | Retain the previous checkpoint and reject the incompatible ledger. |
| CLOCK-P1 | `next check ingress-cutoff --bounds n-1,n,n+1 --permutations all` | Before/at/after cutoff, duplicate, collision, late and completion vectors produce exact current/next assignments; worker and wall-time variation changes no outcome. | Serialize the close barrier through the same contract. |
| RNG-P1 | `next check rng --draws 10000 --targets windows-x86_64,linux-x86_64` | Standard blocks, helpers, rollback, continuation and exhaustion produce exact bytes/state; invalid calls mutate no state. | Reject the incompatible RNG implementation. |
| SCHEDULE-P1 | `next check schedule --permutations 1000 --workers 1,2,8,16` | Schedule, shard, delta, event and state roots remain exact; access ambiguity and cycles reject. | Serialize execution behind the same manifest. |
| NUMERIC-P1 | `next check numerics --targets windows-x86_64,linux-x86_64` | Checked integer/fixed-point/IEEE conversion and physics thresholds produce exact results; every invalid value aborts atomically. | Reject the numeric/backend adapter. |
| WORLD-STREAMING-P1 | `play`, `persistence-replay`; focused World/Runtime tests | The paired Runtime/World publication keeps `IngressCommit → WorldStreamingCommit → PhysicalStep`, rejects stale/faulted completion atomically and reproduces the exact root after restart from `Requested` across the canonical 64-chunk route. | Retain the declared `Requested` root and previous active generation; rebuild the immutable request. |

Windows and Linux must both satisfy cross-target checks; a result from another target does not substitute for a missing target run.

## Current-only formats and technology neutrality

Pre-public alpha command, replay and durable-identity artifacts have no
compatibility reader or migration obligation. A non-current version rejects
with a stable typed `UNSUPPORTED_*` diagnostic before mutation and preserves
the source bytes. A future migration requires a real publicly supported
predecessor and a separate Accepted decision.

Выбор ECS scheduler, ChaCha implementation crate, task runtime, fixed-point helper или physics backend не является public technology decision. Replaceable implementation допускается только за exact schemas and algorithms этого документа.

## Future narrative work

Future narrative/divine work remains Proposed under SPEC-31/ADR-046. The
current Runtime defines no narrative decision-boundary API, generated graph
admission, divine judgment batch or generic cross-context transaction wrapper.
