# Матрица трассируемости требований

| Поле | Значение |
|---|---|
| ID | TRACE-001 |
| Статус | Proposed |
| Версия | 1.5 |
| Владелец | Release Engineering |
| Последняя проверка | 2026-07-22 |
| Нормативные зависимости | [SPEC-12](12-vertical-slice-conformance.md) |
| Связанные документы | все профильные SPEC/ADR и [EVIDENCE-001](evidence-register.md) |
| Заменяет | TRACE-001 v1.4 после human approval exact candidate hash |

Матрица является обязательным release index. `Evidence` на стадии specification означает требуемый тип artifact; после implementation root RunManifest MUST разрешать каждую запись в конкретный path+SHA-256. Одна строка имеет ровно один primary owner. Дополнительные участники записываются в том же owner cell только как `; collaborators: ...`; `+` и другие составные primary owners запрещены.

## Product и architecture requirements

| Requirement | Нормативное требование | Owner | RFC / ADR | Gate | Evidence |
|---|---|---|---|---|---|
| REQ-001 | Независимый specialized AI-first RPG engine, не перенос OpenGothic | Product Architecture | SPEC-00, ADR-001 | VS-10, VS-12 | API/source scan, review record |
| REQ-002 | Windows и Linux — v1 shipping targets; macOS game/package и consoles вне scope | Release Engineering | SPEC-00, SPEC-04, ADR-001/011 | VS-08 | clean-VM logs, package manifests |
| REQ-003 | Apache-2.0 engine, closed games/services allowed | Security & Governance | SPEC-11, ADR-001 | LIC-01, VS-12 | license policy, SBOM, review record |
| REQ-004 | Engine monorepo + отдельный importer repository/process; ignored nested working tree не нарушает boundary | Core Architecture | SPEC-01, SPEC-10, ADR-001/011 | IMPORT-P4/P8, VS-10 | link map, SBOM, parent/nested repository scan |
| REQ-005 | Single-player v1, replay/stable IDs mandatory | Runtime Team | SPEC-02, SPEC-03, ADR-007 | RUNTIME-01, REPLAY-01, VS-11 | replay/state hashes |
| REQ-006 | CLI tools/inspectors v1; full editor deferred | Developer Experience | SPEC-09, ADR-001 | TOOL-01/02 | CLI/compatibility reports |
| REQ-007 | Один authoritative owner для каждого mutable state | Core Architecture | SPEC-01 | ARCH-04, VS-12 | generated ownership report |
| REQ-008 | Vendor types не пересекают public boundaries | Core Architecture | SPEC-01…SPEC-08, ADR-002/003/004 | ARCH-01, RUNTIME-04, VS-10 | public API/schema scans |

## Runtime, data и persistence

| Requirement | Нормативное требование | Owner | RFC / ADR | Gate | Evidence |
|---|---|---|---|---|---|
| REQ-009 | RuntimeEntityId не сериализуется; PersistentId/AssetId nominal/stable | Runtime Team | SPEC-02, ADR-007 | RUNTIME-04, CONTRACT-01 | schema/API scan |
| REQ-010 | Gameplay меняется только validated WorldCommand | Runtime Team | SPEC-02, SPEC-07, ADR-005/006 | RUNTIME-02, AI-03, PLUGIN-P1 | audit/fuzz/command trace |
| REQ-011 | Fixed schedule и async commit points воспроизводимы | Runtime Team | SPEC-02 | RUNTIME-01/03 | state hashes/permutation report |
| REQ-012 | Common neutral asset contract для importer/tools/game/headless | Asset Team | SPEC-03, SPEC-10 | CONTRACT-01, IMPORT-P7, VS-01 | schema/link/cook manifests |
| REQ-013 | Immutable content-addressed bundles и deterministic cooking | Asset Team | SPEC-03 | ASSET-01/02, VS-01 | hashes/cache/validator report |
| REQ-014 | Versioned saves, migrations и fail-closed corruption | Persistence Team | SPEC-03, ADR-007 | SAVE-01/02, VS-02 | power/migration matrices |
| REQ-015 | Replay command stream и first-divergence diagnostics | Runtime Team | SPEC-02/03/09 | REPLAY-01, VS-11 | replay diff/RunManifest |

## Rendering, physics и world services

| Requirement | Нормативное требование | Owner | RFC / ADR | Gate | Evidence |
|---|---|---|---|---|---|
| REQ-016 | Vulkan 1.3 B0 работает без RT/mesh shaders | Rendering Team | SPEC-04, ADR-003 | RENDER-P1/02, VS-07 | validation/GPU traces/screenshots |
| REQ-017 | ash/Slang остаются Proposed и compiler-neutral fallback доказан | Rendering Team | SPEC-04, ADR-003 | SHADER-P1 | SPIR-V/reflection hashes, captures |
| REQ-018 | Device loss не делает GPU source of truth | Rendering Team | SPEC-04 | RENDER-03 | fault/replay report |
| REQ-019 | Physics владеет active avatar pose; renderer читает snapshot | Physical Embodiment | SPEC-05, ADR-004 | PHYS-P1/P3, VS-05 | state/replay/pose reports |
| REQ-020 | PhysX Proposed, replaceable через PhysicsBackend | Physical Embodiment | SPEC-05, ADR-004 | PHYS-P1…P7 | backend gate suite |
| REQ-021 | Motor ONNX parity и runtime/training correspondence | Physical Embodiment | SPEC-05 | MOTOR-P1, PHYS-P5 | model/corpus hashes, parity report |
| REQ-022 | Physical LOD transitions безопасны | Physical Embodiment | SPEC-05 | PHYS-P7, VS-05 | transition metrics/video |
| REQ-023 | Physical gates всегда имеют baseline/success/failure/comparison media | Physical Embodiment | SPEC-05, SPEC-09, ADR-004 | PHYS-P8, MANIFEST-01 | GIF/MP4 + SHA-256 manifest |
| REQ-024 | Navigation intent отделён от physical traversal | World Services | SPEC-08 | NAV-P2, VS-05 | route/physical outcome replay/video |
| REQ-025 | Recast/Steam Audio — gated adapters с fallback | World Services | SPEC-08 | NAV-P1/P3, AUDIO-P1/L1 | nav/audio/legal reports |

## AI, RPG и extensions

| Requirement | Нормативное требование | Owner | RFC / ADR | Gate | Evidence |
|---|---|---|---|---|---|
| REQ-026 | Offline-first: AI-host/сеть не нужны для correctness | Agent Team | SPEC-06, ADR-005 | AI-01/02, VS-04 | offline/fault replay report |
| REQ-027 | LLM не участвует в deterministic/physics/motor tick | Agent Team | SPEC-05/06, ADR-005 | AI-04, VS-04/05 | tick traces, process dependency scan |
| REQ-028 | AgentIntent валидируется до WorldCommand | Agent Team | SPEC-06 | AI-03, VS-03/04 | intent rejection/command audit |
| REQ-029 | Durable memory canonical и работает без embeddings; SQLite Proposed | Agent Team | SPEC-06 | MEMORY-P1, AI-05 | DB/rebuild/save reports |
| REQ-030 | Generic RPG model без legacy types | RPG Team | SPEC-07 | RPG-01, VS-03/10 | RPG inspector/schema scan |
| REQ-031 | Luau capability sandbox и budgets | RPG Team | SPEC-07, ADR-006 | SCRIPT-P1…P4, VS-06 | sandbox/budget/replay reports |
| REQ-032 | Wasm/WIT plugin caps/fuel/memory/version negotiation | RPG Framework; collaborators: Security | SPEC-07/11, ADR-006/015 | PLUGIN-P1…P5, TRUST-P1, VS-06 | audit/fuzz/compat report |

## Importer, security и conformance

| Requirement | Нормативное требование | Owner | RFC / ADR | Gate | Evidence |
|---|---|---|---|---|---|
| REQ-033 | Importer читает только user-provided local install и сохраняет provenance | Importer Team | SPEC-10 | IMPORT-P1/P2, VS-01 | provenance/mapping manifests |
| REQ-034 | Pilot import ограничен whitelist scene/character/items/behavior | Importer Team | SPEC-10 | IMPORT-P2, VS-01 | whitelist coverage report |
| REQ-035 | Gothic parsers/types отсутствуют после cooking/runtime | Core Architecture | SPEC-10, ADR-001 | IMPORT-P4, VS-10 | source/API/schema/link scans |
| REQ-036 | Protected data не попадает в repo/build/package/artifacts | Security Team | SPEC-10/11 | IMPORT-P5, VS-09 | signed scan report |
| REQ-037 | Public importer release проходит отдельный legal review | Security & Governance | SPEC-10/11 | IMPORT-P6, LEGAL-IMPORT-01, VS-12 | written legal record |
| REQ-038 | Threat model покрывает scripts/plugins/AI/import/models/bundles | Security Team | SPEC-11 | SEC-01…04 | scan/fuzz/audit packet |
| REQ-039 | Dependency allowlist, DCO/inbound=outbound, disclosure/SBOM | Security & Governance | SPEC-11 | LIC-01, GOV-01 | SBOM, DCO/review packet |
| REQ-040 | Vertical slice принимается только единым 15/15 packet | Release Engineering | SPEC-12 | VS-01…VS-15 | root RunManifest + signed review |

## Gameplay mechanics, mods и agent authoring

| Requirement | Нормативное требование | Owner | RFC / ADR | Gate | Evidence |
|---|---|---|---|---|---|
| REQ-041 | First-party и community mechanics используют один public package API без hidden path | Gameplay Extensibility | SPEC-13, ADR-008 | ARCH-05, MECH-01, VS-13 | public API/link/package graph scan |
| REQ-042 | Ability/Target/Cost/Cooldown/Effect/Status/Projectile/AreaField — universal mechanics contracts | Gameplay Extensibility | SPEC-13 | MECH-01/02, VS-13 | schema registry, ranged/magic replays |
| REQ-043 | Durable mechanic state engine-owned и меняется только validated reducer proposal/WorldCommand transaction | Gameplay Extensibility; collaborators: Runtime | SPEC-02/13, ADR-008 | MECH-02/04, RUNTIME-02 | state ownership/audit/replay reports |
| REQ-044 | Exact MechanicsLock, dependency DAG и explicit preconditioned patches; no implicit load-order override | Gameplay Extensibility | SPEC-13, ADR-008 | MECH-03, MOD-01, VS-13 | lock/graph/conflict/hash reports |
| REQ-045 | Package state/save/replay migrations и uninstall fail-closed | Persistence; collaborators: Gameplay Extensibility | SPEC-03/13 | MECH-04, VS-02/11/13 | save/migration/fault reports |
| REQ-046 | Data-first → Luau → Wasm tiers; native Rust не является community mod ABI | Gameplay Extensibility | SPEC-07/13, ADR-006/008 | MECH-01, MOD-02 | package/API/SBOM scan |
| REQ-047 | Agent получает complete bounded AuthoringContextBundle и reviewable atomic AgentChangeSet | Developer Experience | SPEC-09/13, ADR-008 | AGENT-01/02, VS-13 | context closure, changeset/fault reports |
| REQ-048 | MCP только optional local projection; CLI/JSON остаётся normative complete fallback | Developer Experience; collaborators: Security | SPEC-09/13, ADR-008 | MCP-P1, TOOL-03, SEC-05, VS-13 | schema parity, audit, packet capture |
| REQ-049 | First-party стрельба реализована ordinary package через physics/contact/effect pipeline | Gameplay Extensibility; collaborators: Physical Embodiment | SPEC-05/13 | MECH-01/06, VS-05/13 | package graph, contacts, replay/media |
| REQ-050 | First-party магия реализована ordinary package через ability/effect/status pipeline | Gameplay Extensibility | SPEC-13 | MECH-01/02/06, VS-13 | package graph, replay/scenario report |
| REQ-051 | Planner-visible package abilities self-describing и доступны NPC без custom AI integration | Agent Intelligence; collaborators: Gameplay Extensibility | SPEC-06/13 | AI-06, MECH-07, VS-13 | affordance registry, planner traces/replays |

## Physical archetypes и progressive motor skills

| Requirement | Нормативное требование | Owner | RFC / ADR | Gate | Evidence |
|---|---|---|---|---|---|
| REQ-052 | CreatureArchetypeManifest добавляет generic creature/physical/agent/mechanics package без package-specific native runtime type | Physical Embodiment | SPEC-01/14, ADR-009 | ARCH-06, EMB-01, CREATURE-01, VS-14 | public API/package graph/schema scan |
| REQ-053 | PrototypeFallback и PhysicalCertified используют один public contract; certification требует exact signed evidence | Physical Embodiment | SPEC-14, ADR-009 | EMB-01, CREATURE-01, SEC-06, VS-14 | bundle/certification/provenance manifests |
| REQ-054 | Proficiency, pose/contact, active route, habits и gameplay effects имеют разных однозначных owners | Core Architecture | SPEC-01/05/06/07/13/14 | ARCH-04, BEHAVIOR-01, VS-12/14 | ownership report, boundary traces |
| REQ-055 | Morphology foundation + conditioned/residual/exclusive expert composition не смешивает arbitrary outputs | Physical Embodiment | SPEC-05/14, ADR-009 | POLICY-01/02, VS-14 | route catalog, action/transition traces |
| REQ-056 | LearnSkill меняет RPG SkillProficiency/routes offline и никогда не изменяет neural weights | RPG Framework | SPEC-07/14, ADR-009 | SKILL-01, SEC-06, VS-14 | command/event/model-hash audit |
| REQ-057 | Policy switch deterministic, atomic, safe-point supervised и не меняет body topology | Physical Embodiment | SPEC-05/14, ADR-009 | POLICY-02, VS-14 | resolver matrix, route/pose/state traces |
| REQ-058 | Novice использует только intentionally trained/evaluated novice/generic/procedural route; expert OOD запрещён | Physical Embodiment | SPEC-14, ADR-009 | SKILL-01, POLICY-01, VS-14 | proficiency-band holdout metrics, route report |
| REQ-059 | Agent habits/tactics создают AgentIntent/InvokeAbility, а Motor Runtime исполняет только physical intent | Agent Intelligence | SPEC-06/14, ADR-009 | AI-07, BEHAVIOR-01, VS-14 | archetype schema, planner/intent/motor boundary traces |
| REQ-060 | Human/agent authoring использует public physical/policy/lab CLI; model promotion/provenance/certification требуют owner review | Developer Experience | SPEC-09/13/14 | TOOL-04, AUTHOR-P1, SEC-06, VS-14 | context closure, CLI transcript, AgentChangeSet/review |
| REQ-061 | Save/replay фиксируют exact archetype/model/route hashes, transition и versioned bounded PolicyState | Persistence Team | SPEC-03/14, ADR-007/009 | POLICY-02, SAVE-01/02, VS-11/14 | save/replay manifests, state/route oracle |
| REQ-062 | Isaac Lab остаётся Proposed replaceable training backend с gate и engine-owned headless-lab fallback | Physical Embodiment | SPEC-14, ADR-009, EVIDENCE-001 | TRAIN-P1, VS-14 | license/SBOM, export/parity/heldout reports |
| REQ-063 | Neutral quadruped и axe progression 0→6000 dogfood public package/SDK и проходят measured physical gates | Physical Embodiment | SPEC-12/14 | CREATURE-01, SKILL-01, AUTHOR-P1, VS-14 | package graphs, metrics/replays/media manifests |

## Headless agent validation и human evidence

| Requirement | Нормативное требование | Owner | RFC / ADR | Gate | Evidence |
|---|---|---|---|---|---|
| REQ-064 | Mandatory acceptance не требует интерактивного запуска, монитора или manual-runtime checklist; человек оценивает self-contained artifacts | Product Architecture | SPEC-00/15, ADR-010 | HEADLESS-01, VS-15 | clean-container trace, offline dossier |
| REQ-065 | Все subsystem/mechanic/physical scenarios являются specialization одного versioned TestScenarioManifest и общего runner | Verification & Evidence | SPEC-02/13/15 | TEST-01, VS-15 | schema registry, scenario/run manifests |
| REQ-066 | Scenario изменяет мир только production input/WorldCommand interfaces; probes read-only, mutable test backdoor запрещён | Runtime Team | SPEC-02/15 | ARCH-07, RUNTIME-05, TEST-01, VS-15 | API/schema scan, invalid-mutation corpus |
| REQ-067 | Каждый oracle объявляет exact/tolerance/distribution/invariant/presentation/qualitative semantics; retry divergence является nondeterminism failure | Verification & Evidence | SPEC-15 | TEST-01, DIAG-01, VS-15 | assertion registry, repeat/order report, divergent replay |
| REQ-068 | Engine-owned ImpactResolver генерирует required gates; AgentChangeSet не может уменьшить plan, unknown impact включает maximal suite и human review | Verification & Evidence | SPEC-13/15, ADR-010 | IMPACT-01, VS-15 | dependency/ownership graph, mutation-corpus impact manifests |
| REQ-069 | VerificationPolicyManifest определяет `agent-fast` ≤5 минут и ordinary changeset compute ≤30 минут; long certification/fuzz/training gates остаются named blockers | Developer Experience | SPEC-09/15 | AGENT-03, VS-15 | profile manifests, timing reports, long-gate plan |
| REQ-070 | Clean CPU-only Linux agent выполняет simulation/tests/diagnostics без GPU, display server, window calls или display environment | Runtime Team | SPEC-01/15, ADR-010 | HEADLESS-01, VS-15 | container manifest, display/socket/API trace, RunManifest |
| REQ-071 | Portable CaptureJobManifest воспроизводит exact replay на displayless Vulkan worker без window/surface/swapchain и не меняет gameplay outcome | Rendering Team | SPEC-01/04/15, ADR-010 | CAPTURE-01, RENDER-04, VS-15 | job/result manifests, API trace, null/capture gameplay hashes |
| REQ-072 | Raw frame/audio roots canonical; synchronized GIF/MP4 — review derivatives; FFmpeg Proposed и проходит MEDIA-P1 либо declared fallback | Verification & Evidence | SPEC-08/09/15, ADR-010, EVIDENCE-001 | AUDIO-02, MEDIA-P1, VS-15 | PNG/WAV roots, encoder/SBOM/decoder report, media hashes |
| REQ-073 | EvidenceBundleManifest — content-addressed atomic closure impact/results/replays/metrics/media, внешняя к Git и проверяемая offline | Verification & Evidence | SPEC-03/09/15 | EVIDENCE-01, MANIFEST-01, VS-15 | bundle closure/hash/quota/redaction reports |
| REQ-074 | Evidence baseline immutable; agent создаёт только candidate, а promotion требует отдельного hash-bound human rationale | Verification & Evidence | SPEC-15 | EVIDENCE-01, REVIEW-01, VS-15 | baseline candidate, promotion decision, invalidation audit |
| REQ-075 | Observable visual/UI/camera/animation/physics/motor/audio changes требуют exact authorized HumanReviewDecision; agent/self-signing и override automatic failure запрещены | Security & Governance | SPEC-11/13/15, ADR-010 | REVIEW-01, SEC-07, VS-12/15 | attestation/role audit, stale/tamper/automatic-failure corpus |
| REQ-076 | Machine-readable failure содержит stable code/owner/first divergent tick/causal diff и minimized replay; public CLI замыкает cold-agent edit-test-diagnose-fix loop | Developer Experience | SPEC-09/13/15 | DIAG-01, AGENT-03, TOOL-05, VS-15 | diagnostic envelopes, minimized replays, cold-agent transcript |

## macOS bootstrap и staged training capability

| Requirement | Нормативное требование | Owner | RFC / ADR | Gate | Evidence |
|---|---|---|---|---|---|
| REQ-077 | `aarch64-apple-darwin` является local DeveloperHostTier: portable workspace/docs/boundaries проходят project-owned host-check без remote/CI, но не дают macOS shipping claim | Developer Experience | SPEC-00/01/09/12, ADR-011 | HOST-MAC-01 | host/toolchain manifest, command report, public API/dependency scan |
| REQ-078 | Mac MPS/CPU smoke доказывает только train→ONNX→inference toolchain; `PhysicalCertified` требует отдельный `TRAIN-RTX-01` и full correspondence gates | Physical Embodiment | SPEC-09/12/14, ADR-011 | TRAIN-MAC-P0, TRAIN-RTX-01, TRAIN-P1, VS-14 | device/lock/model/corpus hashes, parity metrics, RTX capability/correspondence report |

## Packet 1.5 authority, trust и enforcement

| Requirement | Нормативное требование | Owner | RFC / ADR | Gate | Evidence |
|---|---|---|---|---|---|
| REQ-079 | Нормативный packet self-contained: local frozen external inputs, ациклический authority graph, Proposed candidate не выдаётся за Accepted | Architecture Working Group | SPEC-00, ADR-012 | DOCS-01, VS-12 | graph/index/provenance report, candidate hash |
| REQ-080 | Accepted commands хранят exact CanonicalCommandOrderKey; priority validator-owned, issuer sequence monotonic и replay-bound | Runtime Team | SPEC-02, ADR-013 | ORDER-01, RUNTIME-01/03, VS-11 | command/order registry/ledger/replay manifests |
| REQ-081 | Первый baseline использует attested Bootstrap, replacement — exact-current Replace и atomic compare-and-swap | Verification & Evidence; collaborators: Security & Governance | SPEC-09/15, ADR-014 | BASELINE-01, EVIDENCE-01, VS-15 | baseline candidate/index/decision/CAS audit |
| REQ-082 | Review/baseline attestations используют RFC 8785 JCS, RFC 8032 Ed25519, independent capabilities и current revocation trust | Security & Governance | SPEC-11/15, ADR-014 | ATTEST-01, SEC-07, REVIEW-01 | golden vectors, trust/revocation/role audit |
| REQ-083 | Все packages content-addressed; unsigned local требует consent/ceiling, official/required distributed package — trusted publisher signature | RPG Framework; collaborators: Security & Governance | SPEC-07/11/13, ADR-015 | TRUST-P1, PLUGIN-P1, VS-06/13 | package/trust/consent/capability reports |
| REQ-084 | Workspace запрещает unsafe; только Accepted-ADR-named FFI crate может войти в exact Cargo allowlist и пройти safety gates | Core Architecture; collaborators: Security & Governance | SPEC-01/02/11, ADR-016 | FFI-01, ARCH-01, VS-10 | Cargo/boundary/API/SAFETY/Miri/sanitizer reports |
| REQ-085 | Все integrated performance budgets закрывает один hash-bound PERF-01 на canonical vertical scenario/reference profiles | Release Engineering; collaborators: subsystem budget owners | SPEC-12 | PERF-01, VS-12 | aggregate/child performance manifests |
| REQ-086 | Public bootstrap требует Contributor Covenant, disclosure, roles/capabilities и signing policy; missing permanent contact остаётся AwaitingCapability | Security & Governance | SPEC-11 | GOV-01, VS-12 | governance document/status validation report |
| REQ-087 | Metadata/index/status/supersession/counts/owners/gate references и annex hash проверяются project-owned docs-check | Architecture Working Group; collaborators: Developer Experience | INDEX-001, ADR-012, TRACE-001 | DOCS-01, HOST-MAC-01 | positive/negative fixture report, docs graph summary |

## AI-assisted content generation

| Requirement | Нормативное требование | Owner | RFC / ADR | Gate | Evidence |
|---|---|---|---|---|---|
| REQ-088 | V1 authoring имеет provider-neutral CLI/JSON orchestration и offline fixture adapter; live provider не нужен для correctness | Asset & Persistence; collaborators: Developer Experience | SPEC-16, ADR-017 | GEN-01, GEN-05, VS-12 | recipe/job/result manifests, CLI/schema parity report |
| REQ-089 | Generation jobs idempotent, content-addressed и публикуют только immutable terminal result после complete validation | Asset & Persistence | SPEC-16, ADR-017 | GEN-01, GEN-03 | job ledger, request/result hashes, atomic-publication fault report |
| REQ-090 | Prompts, references и generated outputs остаются untrusted; consent, privacy, provenance, license, quotas и quarantine проверяются до upload/view/publish | Security & Governance; collaborators: Asset & Persistence | SPEC-11/16, ADR-017 | GEN-02, SEC-01/02, VS-12 | consent/network/redaction/license/quarantine reports |
| REQ-091 | Generated assets проходят deterministic normalization, profile validation и common NeutralAuthoringModel/cooker path до admission | Asset & Persistence | SPEC-03/16, ADR-017 | GEN-03, ASSET-01/02 | normalization/audit/cook manifests and canonical hashes |
| REQ-092 | Generated world plans используют stable typed IDs, bounded constraints и deterministic reachability/streaming/gameplay audits | Asset & Persistence; collaborators: World Services, RPG Framework | SPEC-08/13/16, ADR-017 | GEN-04, VS-12 | world-plan/NAM hashes, nav/collision/streaming/RPG audit |
| REQ-093 | OpenAI ImageGen adapter optional и ArtifactFixed; subscription/API capability, credentials и provider types не входят в public/runtime contract | Developer Experience; collaborators: Security & Governance | SPEC-16, ADR-017 | IMAGEGEN-P1, GEN-01/02 | capability/request/result/provenance manifests, API scan |
| REQ-094 | Pixal3D и TripoSR остаются Proposed out-of-process research adapters с exact code/model/SBOM/hardware manifests | Asset & Persistence; collaborators: Security & Governance | SPEC-16, ADR-017 | IMG3D-P1, IMG3D-F1, GEN-02/03 | worker/model/SBOM/hardware/normalization reports |
| REQ-095 | Observable generated content проходит ImpactResolver, production scenario/capture/evidence path и exact-hash human review; agent/provider не публикует самостоятельно | Verification & Evidence; collaborators: Developer Experience | SPEC-15/16, ADR-014/017 | GEN-05, IMPACT-01, REVIEW-01, VS-15 | impact/run/evidence/review records and admitted hash |

## Обязательные failure paths

| Failure requirement | Owner | Нормативный путь | Gate | Evidence |
|---|---|---|---|---|
| FAIL-001 Missing/crashed/incompatible AI | Agent Team | deterministic fallback, no duplicate commit | AI-02, VS-04 | fault timeline/replay |
| FAIL-002 Invalid/mismatched motor model | Physical Embodiment | reject before actuation, heuristic recovery | MOTOR-P1, SEC-04, VS-05 | model validator/parity report |
| FAIL-003 Plugin/script overrun or denied capability | RPG Framework; collaborators: Security & Governance | terminate callback/instance, discard proposals, continue/clean pre-world fail | SCRIPT-P2, PLUGIN-P1/P2, VS-06 | budget/audit/replay |
| FAIL-004 Corrupt/incompatible save | Persistence Team | fail-closed, preserve prior/original | SAVE-01/02, VS-02 | fault/migration matrix |
| FAIL-005 Unsupported GPU/device loss | Rendering Team | pre-world B0 diagnostic or bounded recovery/clean exit | RENDER-03, VS-07 | fault/log/replay |
| FAIL-006 Physics policy/backend rejection | Physical Embodiment | safe controller or same-suite backend fallback; slice blocked if none | MOTOR-P1, PHYS-P1…P8, VS-05 | rejection + fallback RunManifests |
| FAIL-007 Malformed/protected importer data | Importer Team; collaborators: Security & Governance | sandbox abort/quarantine/no publish | IMPORT-P3/P5, VS-09 | fuzz/scan/quarantine record |
| FAIL-008 Package dependency/patch/hook conflict | Gameplay Extensibility | deterministic pre-world reject; no guessed order/partial registry | MECH-03, MOD-01, VS-13 | resolver/conflict report |
| FAIL-009 Missing mechanic package/schema/migration | Persistence; collaborators: Gameplay Extensibility | fail-closed, preserve original save; pin old package/export/uninstall path | MECH-04, VS-02/13 | migration/fault/lock report |
| FAIL-010 Unsafe/stale agent change | Developer Experience; collaborators: Security & Governance | changeset reject before mutation; project/base hashes unchanged | AGENT-02, SEC-05, VS-13 | changeset audit/fault report |
| FAIL-011 MCP unavailable/incompatible/denied | Developer Experience | disable adapter; complete CLI/JSON authoring path remains | MCP-P1, TOOL-03, VS-13 | parity/fallback run report |
| FAIL-012 Physical/body/policy compatibility mismatch | Physical Embodiment | reject exact key mismatch before activation; previous/recovery route remains | EMB-01, POLICY-01, SEC-06, VS-14 | validator/rejection/route report |
| FAIL-013 Unsafe or interrupted policy transition | Physical Embodiment | defer at unsafe point or atomically roll back; no teleport/partial joint ownership | POLICY-02, VS-14 | transition/pose/action/state traces, failure media |
| FAIL-014 Missing required model or PolicyState schema | Persistence Team | fail before world mutation, preserve original save; optional downgrade only by explicit project policy | POLICY-01/02, SAVE-01/02, VS-14 | load/fault matrix, preserved hash, diagnostic |
| FAIL-015 Unsupported skill or absent evaluated novice route | RPG Framework | MotorCapabilityView=Unavailable; planner must not invoke ability; no expert OOD execution | SKILL-01, BEHAVIOR-01, VS-14 | route/planner/rejection traces |
| FAIL-016 Invalid provenance/certification or runtime training request | Security & Governance | quarantine/no promotion; deny weight mutation; retain PrototypeFallback/previous certified revision | EMB-01, AUTHOR-P1, SEC-06, VS-12/14 | provenance/certification audit, denial trace |
| FAIL-017 Unknown, incomplete or silently omitted change impact | Verification & Evidence | classify observable; run maximal affected suite and require human review; never accept agent-proposed reduction | IMPACT-01, VS-15 | impact mutation corpus, resolver reasons, maximal-suite manifest |
| FAIL-018 Deterministic retry/order divergence or flaky oracle | Verification & Evidence | mark nondeterminism `FAIL`; preserve first divergent tick and minimized replay; retry cannot turn run green | TEST-01, DIAG-01, VS-15 | repeat/order hashes, divergent/minimized replay, diagnostic |
| FAIL-019 Required GPU, encoder or authorized reviewer unavailable | Release Engineering | publish portable pending job and set `AwaitingCapability`; never emit `PASS` or synthetic approval | CAPTURE-01, MEDIA-P1, REVIEW-01, VS-15 | capability report, pending job/bundle, admission denial |
| FAIL-020 Capture crash, partial publication or null/capture gameplay mismatch | Rendering Team | reject/quarantine atomic output, preserve CPU replay/semantic evidence, rerun exact job after fix | CAPTURE-01, RENDER-04, VS-15 | worker fault/API trace, partial-artifact audit, gameplay hash diff |
| FAIL-021 Missing, stale or tampered baseline, evidence bundle or review decision | Security & Governance | fail verification/invalidate approval; rebuild exact bundle and obtain new human decision after automatic PASS | EVIDENCE-01, REVIEW-01, SEC-07, VS-12/15 | hash/closure/staleness/tamper corpus, invalidation log |
| FAIL-022 Malicious/oversized/redaction-failed media or dossier, HTML injection or reviewer-key exposure | Security & Governance | reject/quarantine evidence, revoke exposed credential, sanitize/rebuild and repeat review; no viewing/admission before verification | EVIDENCE-01, SEC-07, VS-12/15 | quota/redaction/HTML/media fuzz report, quarantine/revocation audit |
| FAIL-023 MPS unavailable or unsupported operation during Mac smoke | ML Tooling | emit stable capability diagnostic, rerun exact deterministic fixture on CPU, record fallback; no false MPS PASS | TRAIN-MAC-P0 | doctor report, device/fallback RunManifest, parity metrics |
| FAIL-024 RTX/full-training capability absent or failed at certification | Physical Embodiment | set `AwaitingCapability`, reject `PhysicalCertified` promotion, retain `PrototypeFallback`; Mac/ONNX smoke cannot waive gate | TRAIN-RTX-01, TRAIN-P1, VS-14 | capability/preflight report, denied promotion record, prototype manifest |
| FAIL-025 External/missing normative dependency, graph cycle or annex hash/provenance mismatch | Architecture Working Group | reject packet candidate; Accepted Packet 1.4 remains authority | DOCS-01, VS-12 | graph/hash/provenance diagnostic |
| FAIL-026 Forged priority, stale/duplicate issuer sequence, command ID conflict or order-registry mismatch | Runtime Team | reject before mutation with stable code; no duplicate commit | ORDER-01, RUNTIME-02/03, VS-11 | rejection/permutation/replay report |
| FAIL-027 Illegal baseline Bootstrap/Replace, stale previous hash, CAS race or wrong promotion capability | Verification & Evidence | reject promotion atomically; current BaselineIndex unchanged | BASELINE-01, EVIDENCE-01, VS-15 | decision/index hashes, CAS fault audit |
| FAIL-028 Non-canonical/tampered/unknown/revoked/wrong-role attestation | Security & Governance | reject/invalidate decision; require new authorized review | ATTEST-01, SEC-07, REVIEW-01 | vector/tamper/revocation audit |
| FAIL-029 Missing consent, unsigned official/required package, tamper, revoked publisher or trust elevation | RPG Framework | deny/quarantine before world mutation; optional disable or required load failure | TRUST-P1, PLUGIN-P1/P5, VS-06 | trust/consent/capability audit |
| FAIL-030 Unsafe/lint opt-out outside allowlist, FFI metadata drift or vendor/raw type in public contract | Core Architecture | boundary failure; use safe adapter/process boundary; no workspace-wide waiver | FFI-01, ARCH-01, VS-10 | source/Cargo/public-API safety report |
| FAIL-031 Missing/stale profile/measurement or exceeded integrated performance budget | Release Engineering | PERF-01 FAIL; retain prior build; unavailable required hardware is AwaitingCapability, not substitution | PERF-01, VS-12 | aggregate manifest, missing/exceeded child diagnostics |
| FAIL-032 Missing/inconsistent governance document/contact/status, packet metadata, owner or gate reference | Security & Governance | public release/packet promotion blocked; configure/fix and rerun exact local gates | GOV-01, DOCS-01, VS-12 | governance/docs-check validation report |
| FAIL-033 Generation provider absent, timeout, crash, quota or protocol/version mismatch | Developer Experience | classify terminal failure; no partial publish/project mutation; use offline fixture/manual/catalog path | GEN-01, IMAGEGEN-P1, IMG3D-P1 | job/fault/capability timeline and unchanged project hash |
| FAIL-034 Malicious, private, protected, unlicensed or unconsented generation input | Security & Governance | deny upload or quarantine before decode/view/publish; redact secrets and preserve incident evidence | GEN-02, SEC-01/02, VS-09 | network/redaction/license/quarantine audit |
| FAIL-035 Invalid, malformed, oversized or profile-incompatible generated asset | Asset & Persistence | reject before NAM/cook publication; retain prior valid asset and stable diagnostic | GEN-03, ASSET-01/02 | validator/normalization rejection manifest |
| FAIL-036 Nondeterministic normalization, cooker drift or canonical hash mismatch | Asset & Persistence | fail closed, preserve both exact inputs/results and first divergent stage; no retry-to-green | GEN-03, TEST-01, DIAG-01 | cross-platform hashes and divergence report |
| FAIL-037 Generated world violates bounds, reachability, stable IDs, streaming or gameplay constraints | Asset & Persistence | reject world plan before project admission; use previous valid/manual world | GEN-04, NAV-P1/P2 | world/nav/collision/streaming/RPG rejection audit |
| FAIL-038 Missing provenance, license classification, consent or redaction record for generated artifact | Security & Governance | quarantine and block publish/review until a complete new provenance step exists | GEN-02/05, LIC-01, REVIEW-01 | provenance/license/redaction closure report |
| FAIL-039 Generation worker sandbox escape, resource overrun, crash or partial publication | Security & Governance | terminate worker, quarantine outputs, atomically preserve project baseline and disable adapter | GEN-01/02, SEC-02 | sandbox/quota/process/publication fault report |
| FAIL-040 Required network, GPU, provider entitlement or authorized reviewer unavailable | Release Engineering | mark affected optional/live gate `AwaitingCapability`; offline authoring evidence remains valid but no synthetic PASS/admission | IMAGEGEN-P1, IMG3D-P1, GEN-05, REVIEW-01 | capability report, pending job/evidence bundle, denied admission |

## Completeness rule

`next gate VS-12` MUST verify that every `REQ-*` and `FAIL-*` row maps to at least one executed gate and hash-valid artifact in root RunManifest. Missing, skipped, waived or stale evidence yields `FAIL`; ручная пометка не закрывает строку.
