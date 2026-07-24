# ADR-024: Requirement, gate, evidence and profile closure

| Поле | Значение |
|---|---|
| ID | ADR-024 |
| Статус | Accepted |
| Версия | 1.0 |
| Владелец | Verification & Evidence Team |
| Требуемые согласующие | Repository Owner, Architecture Working Group, Release Engineering, Security & Governance |
| Дата решения | 2026-07-24 |
| Последняя проверка | 2026-07-24 |
| Нормативные зависимости | [ADR-023](023-human-review-decision-v2-and-offline-attestation.md) |
| Заменяет | отсутствует |
| Заменён | не заменён |

## История принятия

ADR подготовлен как часть packet 1.6. До admission exact packet root его статус в candidate tree не является человеческим approval и не создаёт implementation, `vertical-v1` или release claim.

## Контекст

Список requirements сам по себе не доказывает проверяемость архитектуры. До этого решения часть gate IDs была записана сокращениями (`A/B`, `A…B`), некоторые обязательные gates не имели обратной строки requirement, Proposed technology gate мог выглядеть обязательным для Accepted baseline, а `PERF-01` не был самостоятельным child gate `VS-12`. Это позволяло удалить gate, evidence либо profile mapping без локально обнаружимого разрыва.

Нужен один machine-readable graph, в котором Accepted requirement и failure path замыкаются на владельца, полные gate IDs, evidence и `VS-*` либо named profile, а каждый применимый gate замыкается обратно. Graph описывает architecture admission. Runtime gate results появятся только после реализации.

## Решение

### `RequirementGraphV1`

`RequirementGraphV1` является canonical projection нормативных Markdown-источников и содержит:

```text
RequirementGraphV1 {
  schema_version,
  architecture_version,
  packet_version,
  architecture_root_sha256,
  document_entries[],
  technology_entries[],
  reserved_requirement_ids[],
  reserved_failure_ids[],
  requirements[],
  failure_paths[],
  gate_descriptors[],
  vertical_slice_descriptors[],
  profile_descriptors[],
  edges[],
  graph_sha256
}
```

- `document_entries` фиксирует ID, status, version и SHA-256 каждого индексированного документа;
- `technology_entries` использует стабильный `TECH-NNN`, status, gate mapping и fallback;
- reservations имеют exact ID, `Allocation status`, owning document и reason; reserved ID никогда не переиспользуется и не считается Accepted requirement/failure;
- `edges` типизированы как `owned_by`, `verified_by`, `produces_evidence`, `aggregated_by`, `depends_on` либо `classified_as`;
- массивы сортируются по canonical ID bytes, повторяющиеся ID либо неоднозначный owner invalidates graph;
- `graph_sha256` вычисляется над canonical binary representation без собственного поля hash с domain `nextengine.requirement-graph.v1`.

Generated graph является evidence index, но не заменяет owning SPEC/ADR. При расхождении Markdown остаётся источником решения, а generation завершается `REQUIREMENT_GRAPH_SOURCE_MISMATCH`.

### `GateDescriptorV1`

Каждый gate, на который ссылается Accepted requirement, failure path, VS или profile, имеет ровно один descriptor:

```text
GateDescriptorV1 {
  gate_id,
  descriptor_source_id,
  classification,
  subject_id,
  owner,
  applicability,
  command_or_scenario,
  threshold,
  required_evidence[],
  fallback,
  requirement_ids[],
  failure_ids[],
  parent_gate_ids[],
  child_gate_ids[],
  vs_or_profile_ids[],
  result_policy
}
```

`descriptor_source_id` указывает ровно один Accepted SPEC/ADR, содержащий canonical scenario/command, threshold, evidence, fallback и semantic subject gate. Другие документы могут ссылаться на `gate_id`, агрегировать его в VS/profile либо перечислять requirement-specific evidence, но MUST NOT создавать второй semantic descriptor. Повторное descriptor-source определение допустимо только если его normalized fields byte-for-byte identical; несовместимый owner, subject, scenario, threshold, evidence, fallback или result policy даёт `GATE_DESCRIPTOR_CONFLICT`.

`classification` имеет closed set:

- `AcceptedBaseline` — обязательная engine-owned capability либо conformance check;
- `CandidateOnly` — выбор конкретной Proposed технологии, не закрывающий Accepted baseline;
- `Rejected` — историческая запись, которая не участвует в admission;
- `ImplementationChoice` — выбранная реализация за Accepted facade; descriptor применим только когда exact subject выбран composition lock.

`result_policy` имеет `Blocking`, `Informational` либо `CandidateSelection`. Accepted requirement/failure использует только применимый `AcceptedBaseline` или `ImplementationChoice`; `CandidateOnly` не может быть единственным gate Accepted строки. Выбор candidate добавляет его descriptor и evidence к baseline, но не удаляет baseline/fallback.

В любой ID-bearing ячейке разрешены только полные отдельные IDs. Символ `/`, многоточие `…`, диапазон, неявное повторение prefix и фраза `all gates` запрещены. Например, `SCRIPT-P1/SCRIPT-P2` и `SCRIPT-P1…SCRIPT-P5` invalid; требуется `SCRIPT-P1, SCRIPT-P2, SCRIPT-P3, SCRIPT-P4, SCRIPT-P5`.

### Двусторонняя closure

Для каждого Accepted `REQ-NNN` и `FAIL-NNN` validator MUST доказать:

1. ровно один primary owner;
2. хотя бы один полный gate ID;
3. конкретный required evidence type;
4. хотя бы один `VS-NN` либо declared profile в `VS / profile closure`;
5. declared fallback либо explicit fail-closed/blocking outcome;
6. owning Accepted SPEC/ADR;
7. обратное присутствие каждого gate в `GateDescriptorV1` и каждого gate descriptor — хотя бы в Accepted requirement/failure, VS/profile или явной CandidateOnly classification.

Отсутствующая capability возвращает `AwaitingCapability`, а не `PASS`. Unknown, orphan, shorthand, Proposed-only, status-incompatible либо ownerless gate invalidates architecture candidate. Proposed и Deferred Proposed rows проверяются на уникальность/reservation consistency, но исключаются из Accepted completeness.

### Candidate-only gates

Эти gates навсегда отделены от Accepted baseline до отдельного technology-promotion ADR:

| Gate ID | Classification | Subject | Rationale |
|---|---|---|---|
| ECS-P1 | CandidateOnly | TECH-002 Bevy ECS/app crates | Проверяет только candidate implementation; engine-owned deterministic scheduler facade и fallback задают baseline. |
| RENDER-ASH-P1 | CandidateOnly | TECH-004 ash | Проверяет exact binding adapter; Accepted `RENDER-P1` остаётся implementation-neutral Vulkan/`RenderDevice` baseline. |
| SHADER-SLANG-P1 | CandidateOnly | TECH-005 Slang | Проверяет exact compiler adapter; Accepted `SHADER-P1` остаётся compiler-neutral `ShaderInterface` baseline. |
| PLATFORM-P1 | CandidateOnly | TECH-006 SDL 3 | Проверяет только SDL adapter; public platform/session contracts и native-adapter fallback не зависят от SDL. |
| MCP-P1 | CandidateOnly | TECH-030 Model Context Protocol | Проверяет optional local projection; полный CLI/JSON workflow остаётся обязательным baseline. |

`ECS-P1`, `RENDER-ASH-P1`, `SHADER-SLANG-P1`, `PLATFORM-P1` и `MCP-P1` не могут закрывать Accepted requirement самостоятельно и не входят в обязательный `vertical-v1` aggregate, пока отдельный ADR не переведёт соответствующий subject в Accepted либо ImplementationChoice.

### Canonical gate-source reconciliation

Packet 1.6 закрепляет один semantic descriptor source для ранее orphaned или конфликтующих IDs. Эта таблица является source mapping, а не повторным descriptor:

| Catalog reference | Canonical descriptor source | Classification / closure role |
|---|---|---|
| ARCH-02 | SPEC-01 | `AcceptedBaseline`; exact game/null, headless and capture-worker authoritative parity; child of VS-11. |
| ARCH-03 | SPEC-01 | `AcceptedBaseline`; exact `ai-host` absence/timeout/bad-version/kill/restart isolation; child of VS-04. |
| ARCH-08 | SPEC-01 | `AcceptedBaseline`; independent Next Engine composition and dependency boundary; child of VS-10. |
| COMMAND-ID-P1 | SPEC-21 | `AcceptedBaseline`; V2 body/envelope identity and claim validation; child of VS-11. |
| CAUSAL-ID-P1 | SPEC-21 | `AcceptedBaseline`; persisted world/principal/stream/event/object identity; child of VS-02 and VS-11. |
| COMMAND-LEDGER-P1 | SPEC-21 | `AcceptedBaseline`; high-watermark, pending, receipt-window and restart semantics; child of VS-02 and VS-11. |
| CLOCK-P1 | SPEC-21 | `AcceptedBaseline`; exact current/next virtual-tick assignment; child of VS-11. |
| RNG-P1 | SPEC-21 | `AcceptedBaseline`; named ChaCha12 streams and persisted state; child of VS-06 and VS-11. |
| SCHEDULE-P1 | SPEC-21 | `AcceptedBaseline`; stable DAG, ordered work and deterministic merge; child of VS-11. |
| NUMERIC-P1 | SPEC-21 | `AcceptedBaseline`; checked arithmetic and canonical physical projection; child of VS-05 and VS-11. |
| COMMAND-V1-INVENTORY-P1 | SPEC-21 | `AcceptedBaseline`; pre-conformance V1 artifact inventory/migration admission; child of VS-12. |
| STREAM-01 | SPEC-03 | `AcceptedBaseline`; deterministic streaming lifecycle and atomic publication; child of VS-01. |
| PACKAGE-01 | SPEC-04 | `AcceptedBaseline`; clean Windows/Linux package closure; child of VS-08. |
| RENDER-P1 | SPEC-04 | `AcceptedBaseline`; implementation-neutral Vulkan/`RenderDevice` conformance; child of VS-07. |
| SHADER-P1 | SPEC-04 | `AcceptedBaseline`; compiler-neutral `ShaderInterface` conformance; child of VS-07. |
| WORLD-01 | SPEC-08 | `AcceptedBaseline`; calendar/weather/reservation save/load/replay; child of VS-02. |
| WORLD-02 | SPEC-08 | `AcceptedBaseline`; navigation/world-service/physical owner separation; child of VS-05. |
| OBS-01 | SPEC-09 | `AcceptedBaseline`; profiling/telemetry determinism and overhead; child of VS-15. |
| OBS-02 | SPEC-09 | `AcceptedBaseline`; bounded crash-capsule and atomic-write behavior; child of VS-15. |
| OBS-03 | SPEC-09 | `AcceptedBaseline`; structured causal diagnostic/trace semantics; child of VS-15. |
| OBS-04 | SPEC-09 | `AcceptedBaseline`; metrics/exporter fault isolation; child of VS-15. |
| PRIVACY-01 | SPEC-09 | `AcceptedBaseline`; redaction/prohibited-root publication scan; child of VS-09 and VS-12. |
| PRIVACY-02 | SPEC-15 | `AcceptedBaseline`; split-fixture cleanup and protected-data absence before evidence publication; child of VS-09, VS-12 and VS-15. |
| REVIEW-02 | SPEC-15 | `AcceptedBaseline`; V2 offline attestation/trust verification and V1 historical-only denial; child of VS-12 and VS-15. |
| HOST-MAC-01 | ADR-011 | `AcceptedBaseline`; portable macOS developer-host verification without a shipping claim; bootstrap profile closure. |
| TRAIN-MAC-P0 | ADR-011 | `AcceptedBaseline`; bounded Mac MPS/CPU train/export smoke without certification authority; child of VS-14. |
| TRAIN-RTX-01 | ADR-011 | `AcceptedBaseline`; local Linux/NVIDIA production-training capability preflight; child of VS-14. |
| AI-04 | ADR-016 | `AcceptedBaseline`; integrated agent-planning budget input; child of VS-04 and VS-12. |
| NAV-P3 | ADR-016 | `AcceptedBaseline`; integrated navigation budget input; child of VS-05 and VS-12. |
| MECH-05 | ADR-016 | `AcceptedBaseline`; integrated mechanics budget input; child of VS-12 and VS-13. |
| TRACE-01 | ADR-024 | `AcceptedBaseline`; requirement/gate/evidence/profile graph closure; child of VS-12. |
| PERF-01 | ADR-016 | `AcceptedBaseline`; independent integrated budget; child of VS-12. |

`AI-04`, `NAV-P3` и `MECH-05` являются обязательными inputs `PERF-01`, но их individual PASS не заменяет integrated `PERF-01` evidence.

Canonical `TRACE-01` descriptor:

| Gate | Сценарий | Threshold | Evidence | Fallback |
|---|---|---|---|---|
| TRACE-01 | rebuild and validate `RequirementGraphV1` plus the canonical applicable `GateDescriptorV1` set | 100% Accepted owner→gate→evidence→VS/profile closure, reverse gate coverage and descriptor-source uniqueness; 0 unknown, orphan, shorthand, Proposed-only or conflicting descriptors | `RequirementGraphV1`, canonical gate-descriptor-set hash, validator report and negative corpus | block packet/vertical admission and repair graph; no waiver |

### Reservation ledger

Allocation ranges фиксируются до promotion:

- Deferred Proposed SPEC-16: `REQ-079`–`REQ-086`, `FAIL-025`–`FAIL-030`;
- Proposed foundation SPEC-17…SPEC-20: `REQ-087`–`REQ-102`, `FAIL-031`–`FAIL-038`;
- Accepted SPEC-21: `REQ-103`–`REQ-110`, `FAIL-039`–`FAIL-042`;
- integrated performance: `REQ-111`, `FAIL-043`;
- future SPEC-22…SPEC-30: `REQ-112`–`REQ-147`, `FAIL-044`–`FAIL-061`.

Только `REQ-103`–`REQ-111` и `FAIL-039`–`FAIL-043` добавляются в Accepted completeness packet 1.6. Остальные IDs остаются reserved и не могут заполнять gaps validator.

## Validation and diagnostics

Architecture review SHOULD проверять:

- Pending/Approved mismatch, wrong target, skipped transition и manifest/hash drift;
- Accepted dependency на Proposed/Superseded ADR;
- duplicate, reused, conditionally allocated либо missing reservation;
- unknown, orphan, shorthand, Proposed-only, status-incompatible либо multiply-defined/conflicting gate;
- missing owner, evidence, fallback либо VS/profile closure;
- отсутствие `PERF-01` в `REQ-111` или `VS-12` blocking children;
- tampered `RequirementGraphV1` document/gate/graph hashes;
- Accepted baseline gate, который требует vendor-specific adapter, либо CandidateOnly adapter gate, включённый в blocking children `VS-*`.

Automatic tool results do not create reviewer identity for implementation evidence or release decisions. Architecture-document edits themselves use the normal repository review workflow and require no separate promotion capability.

## Последствия

- 15 существующих `VS-01`…`VS-15` сохраняются; новые gates являются их children, а не новыми vertical slices.
- Добавление Accepted MUST/failure/gate требует синхронного owning-document, trace, evidence и profile update.
- Rust traits, ECS layout, platform backend, renderer graph и other private implementation остаются вне serialized graph.
- Packet admission означает только согласованную архитектуру. Runtime suites, `vertical-v1` и release readiness требуют отдельной реализации и evidence.
