# SPEC-12: Vertical-slice conformance

| Поле | Значение |
|---|---|
| ID | SPEC-12 |
| Статус | Proposed |
| Версия | 1.5 |
| Владелец | Release Engineering |
| Последняя проверка | 2026-07-22 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-04](04-rendering-and-platform.md), [SPEC-05](05-physics-animation-and-motor-control.md), [SPEC-06](06-ai-agents-perception-and-memory.md), [SPEC-07](07-rpg-scripting-and-plugins.md), [SPEC-08](08-audio-navigation-and-world-services.md), [SPEC-09](09-tooling-sdk-and-observability.md), [SPEC-10](10-gothic-importer-boundary.md), [SPEC-11](11-security-licensing-and-governance.md), [SPEC-13](13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [SPEC-16](16-ai-assisted-world-and-asset-generation.md), [ADR-011](adr/011-macos-developer-host-local-verification-and-staged-training.md), [ADR-012](adr/012-standalone-authority-and-acyclic-dependencies.md), [ADR-013](adr/013-canonical-world-command-ordering.md), [ADR-014](adr/014-artifact-first-review-baselines-and-attestation-v2.md), [ADR-015](adr/015-luau-wasm-package-trust-v2.md), [ADR-016](adr/016-rust-first-audited-ffi-boundary-v2.md), [ADR-017](adr/017-artifact-first-ai-content-generation.md) |
| Связанные документы | [TRACE-001](traceability.md), [EVIDENCE-001](evidence-register.md) |
| Заменяет | SPEC-12 v1.4 после human approval exact candidate hash |

## Назначение

Vertical slice — первый интеграционный release gate, а не showcase. Он доказывает, что независимый engine может reproducibly построить, расширить public mechanic/creature packages, переключить заранее обученные motor skills, запустить, сохранить, воспроизвести и диагностировать bounded RPG scenario при соблюдении backend/security/importer/mod boundaries. Разработка и automatic acceptance MUST быть доступны CPU-only Linux agent без монитора, а качественная оценка observable changes MUST выполняться по hash-bound evidence dossier без обязательного интерактивного запуска игры. Ни один отдельный gate не заменяет весь пакет.

`HOST-MAC-01` и `TRAIN-MAC-P0` являются bootstrap capability gates и не добавляют шестнадцатый vertical gate. Они разрешают portable разработку и раннюю проверку ML/export toolchain на Mac, но не закрывают Windows/Linux package, renderer, runtime/training correspondence или physical certification. Перед `PhysicalCertified` частью VS-14 MUST быть `TRAIN-RTX-01=PASS`; отсутствие capability даёт `AwaitingCapability`, а `PrototypeFallback` остаётся допустимым development result.

Live `IMAGEGEN-P1`, `IMG3D-P1` и `IMG3D-F1` являются technology-admission gates, но не mandatory external-service requirements vertical-v1. Vertical conformance проверяет core generation pipeline через deterministic offline fixture adapter и `GEN-01…05`; отсутствие subscription account, network или generation GPU не блокирует existing-content path.

## Source of truth, ownership и public boundary

Versioned `vertical-v1` TestScenarioManifest + root RunManifest являются source of truth для inputs, thresholds, выполненных gates и evidence hashes. Release Engineering владеет orchestration и итоговым pass/fail; subsystem owner владеет своим child gate, Verification & Evidence Team — impact/capture/evidence contracts, Security & Governance — release trust evidence. Public conformance boundary состоит из canonical scenario schema, future `next gate` CLI, RunManifest, EvidenceBundleManifest и HumanReviewDecision; CI runner/vendor formats не являются contract.

## Canonical scenario `vertical-v1`

Один versioned scenario manifest MUST задавать fixed content/import whitelist, seeds, tick rates, supported machine profiles, expected state transitions, deliberate fault injection points, camera/render settings и thresholds.

Сценарий:

1. пользователь явно указывает локальную тестовую installation; importer экспортирует whitelisted scene subset, одного character archetype, items и reviewed declarative behavior в NIM;
2. offline fixture adapter создаёт один generated prop и bounded world-plan variant через provider-neutral recipe/job/result manifests; common validator/cooker публикует scene bundle, второй cook доказывает cache reuse, runtime больше не обращается к installation или generation worker;
3. player входит в B0-rendered scene, перемещается, взаимодействует с item/object и physical avatar obstacle/slope;
4. generic NPC воспринимает player, вступает в dialogue, выдаёт/обновляет малый Quest и меняет одну relationship dimension через accepted commands;
5. optional `ai-host` сначала работает, затем timeout/kill/restart; та же quest path корректно продолжается offline fallback;
6. physical avatar проходит full/simplified/capsule transitions, perturbation и recovery с required media evidence;
7. Luau overrun и Wasm forbidden capability инъецируются и изолируются;
8. first-party ranged package выполняет aim/fire/reload, projectile/contact/effect и presentation-cue loop; first-party elemental package выполняет cast/interruption/projectile/area/status loop;
9. cold author workflow из public AuthoringContextBundle добавляет data-only ranged variant и spell, затем через offline generation fixture создаёт/нормализует object candidate и bounded semantic world placement; единый AgentChangeSet проходит provenance, asset/world audits, validate/test/simulate без private source/provider knowledge;
10. package dependency/patch conflict, missing migration, unsafe changeset и MCP failure инъецируются; CLI/JSON fallback остаётся полным;
11. public SDK создаёт neutral quadruped prototype и certification candidate; foundation stand/locomotion/recovery, bite/lunge expert, data-driven habits и человеческий axe transition `SkillProficiency 0 → 6000` проходят success/failure suites;
12. save создаётся до, во время и после policy switch; process закрывается, save с exact MechanicsLock/model/route hashes загружается, scenario завершается; replay повторяется headless;
13. CPU-only Linux agent без display/GPU вычисляет ChangeImpactManifest, воспроизводит seeded regression, получает first divergent tick и minimized replay, исправляет изменение и проходит `agent-fast`/`changeset` automatic gates;
14. portable CaptureJobManifest воспроизводит exact replay на displayless Vulkan worker; before/after/comparison/failure media и offline dossier проходят tamper/staleness tests, затем authorized human принимает или отклоняет exact observable changeset;
15. packages собираются/запускаются на clean Windows/Linux, затем SBOM, protected-data/runtime symbol scans и review packet завершают suite.

## Reference profiles

Exact hardware models фиксируются milestone manifest, но MUST удовлетворять:

- CPU profile: x86_64, 8 physical или performance-class cores, 16 GiB RAM;
- agent CPU-only profile: pinned Linux container, no GPU device, display sockets и display environment variables; reference `agent-fast` workflow выполняет 0 display connection attempts;
- GPU B0 profile: shipping cross-vendor Vulkan 1.3 discrete GPU, 6 GiB VRAM, RT/mesh features принудительно disabled;
- capture profile: tagged pinned Vulkan 1.3 worker без monitor/window/surface/swapchain/X11/Wayland; resolution/FPS/camera/audio задаёт CapturePlan;
- storage: SSD, cold и warm/cache runs разделены;
- Windows current supported release и pinned Linux distribution/glibc baseline;
- debug/validation profile для correctness и release profile для budgets.

Изменение reference profile требует versioned scenario manifest и comparison run; hardware нельзя менять только для скрытия regression.

## Обязательные gates

Все команды выполняются из clean checkout нового monorepo/importer repository согласно runbook milestone. До появления CI Release Engineering запускает те же команды локально; runner/vendor не входит в contract. Здесь команда является нормативным будущим CLI contract; данная specification task не создаёт binaries.

| Gate | Owner | Команда/сценарий | Pass/fail threshold | Evidence artifacts | Fallback / rollback |
|---|---|---|---|---|---|
| VS-01 Import/cook/cache/load | Importer + Asset Team | `next gate VS-01 --scenario vertical-v1 --platform $TARGET` | IMPORT-P1/P2/P7; first cook/load success; second cook ≥90% cache hits; runtime opens 0 source-install files | NIM/provenance, cook manifests/hashes, file-access trace, screenshot | no publish; fix mapping/cooker; neutral demo не засчитывает importer criterion |
| VS-02 Player loop + persistence | Runtime + RPG + Persistence | `next gate VS-02 --scenario vertical-v1 --fault-save-points all` | item/object interaction and movement outcomes exact; SAVE-01/02; loaded state hash equals pre-close committed hash; 100% power points retain valid generation | replay/save manifests, event/state hashes, fault matrix | retain previous save; criterion blocks slice |
| VS-03 Generic NPC/dialogue/quest | RPG Team | `next gate VS-03 --scenario vertical-v1 --npc generic-01` | no legacy type/schema; dialogue and one quest transition exact; relationship delta caused by accepted command; 100 repeats exact hashes | command/event trace, RPG inspector report | compiled generic rules/authored dialogue; block if incorrect |
| VS-04 Offline AI + restart | Agent Team | `next gate VS-04 --scenario vertical-v1 --ai-faults absent,timeout,kill,restart,bad-version` | all variants complete same mandatory quest outcome; no tick blocked by AI; 0 duplicate commands; fallback ≤1 tick after deadline signal | ai inspector timeline, fault report, replay hashes | circuit-break/disable ai-host and deterministic planner |
| VS-05 Physical avatar + LOD | Physical Embodiment Team | `next gate VS-05 --scenario vertical-v1 --suite physical --render-artifacts` | PHYS-P1…P8/MOTOR-P1; scenario success aggregate ≥95%, 0 safety violation; LOD discontinuities within SPEC-05 | full metrics; baseline/selected/failures/comparison GIF, MP4 as required, SHA-256 RunManifest | Jolt→Bullet same gates; heuristic motor; if no backend/policy passes, slice blocked |
| VS-06 Luau/Wasm isolation | RPG + Security | `next gate VS-06 --scenario vertical-v1 --inject extension-faults` | forbidden capability 100% denied; script stops per SCRIPT-P2; 0 partial command/host crash; scenario continues with optional extension disabled | audit log, fuel/instruction/memory report, replay | pin runtimes/disable optional plugin; required failure pre-world blocks |
| VS-07 Baseline renderer | Rendering Team | `next gate VS-07 --scenario vertical-v1 --tier B0 --disable rt,mesh-shader` | RENDER-P1/02; ≥60 FPS 1080p, p95 GPU ≤16.6 ms on reference GPU; image SSIM ≥0.98; 0 validation errors | RunManifest, GPU trace, validation logs, screenshots/diff | indexed/bounded descriptor B0 path; unsupported B0 GPU clean diagnostic |
| VS-08 Windows/Linux packages | Release Engineering | `next gate VS-08 --scenario vertical-v1 --targets windows-x86_64,linux-x86_64 --clean-vm` | clean install/launch/save/replay success both; no undeclared shared libraries; package/SBOM hashes present | VM logs, package manifests, SBOM, hashes | rollback package; platform remains unshipped and slice blocked |
| VS-09 Protected-data absence | Security + Importer | `next gate VS-09 --scenario vertical-v1 --scan repo,cache,build,package,artifacts` | 0 protected source/derived byte/signature findings; provenance records contain hashes/metadata only | signed scanner report, quarantine log if any | quarantine/remove generated artifacts; incident review; rerun clean |
| VS-10 No legacy runtime types | Core Architecture | `next gate VS-10 --scenario vertical-v1 --scan-api-link-schema` | 0 legacy parser/VM deps and 0 forbidden-symbol-set entries from SPEC-10 in runtime public/source schemas; permitted only importer repo/RFC history | source/API/schema scan, link map, SBOM | boundary refactor; block slice |
| VS-11 Replay/headless parity | Runtime + Release Engineering | `next gate VS-11 --scenario vertical-v1 --compare game,headless` | accepted command, DomainEvent sequence and final gameplay hash exact; physical tolerance only where explicitly registered, outcomes exact | replay diff/first divergence report | deterministic fix; no waiver for gameplay divergence |
| VS-12 Security/governance review | Security & Governance | `next gate VS-12 --scenario vertical-v1 --review-packet` | SEC-01…07, LIC-01, GOV-01, PERF-01, ORDER-01, BASELINE-01, ATTEST-01, TRUST-P1, FFI-01 и mandatory offline GEN-01…05 pass; legal importer approval exact release; VS-01…VS-11 and VS-13…VS-15 child manifests PASS; traceability completeness 100%; all required hashes verify; optional live IMAGEGEN/IMG3D gates do not substitute offline path | signed review record, SBOM, scans, traceability/performance/generation reports, required HumanReviewDecision/BaselinePromotionDecision records | do not declare conformance; disable live providers and retain offline/manual authoring; engine-only review MAY proceed without importer release but not vertical-v1 acceptance |
| VS-13 Mechanics/mod/agent/AI-content authoring | Gameplay Extensibility; collaborators: Asset, DevEx, Security | `next gate VS-13 --scenario vertical-v1 --suite mechanics-mod-agent-generation --adapters cli,mcp,fixture` | MECH-01…07, AI-06, MOD-01/02, AGENT-01/02, MCP-P1, TOOL-03 and GEN-01…05 pass; ranged/magic/reference-agent/generation fixture use 0 hidden APIs; candidate follows recipe→provenance→normalization→NAM→cook; NPC discovers affordances; MCP/live-generator failure completes via CLI/fixture/manual path | package/lock/affordance graphs, context bundle, generation recipe/job/result/provenance/normalization manifests, asset/world audits, changesets, replay/profile/security reports, MCP schema diff, physical/visual media when required | disable MCP/live adapter and use CLI/fixture/manual source; reject candidate/package; add missing public primitive via ADR; slice blocked until public core path passes |
| VS-14 Physical creature + progressive motor skills | Physical Embodiment + RPG + Agent + DevEx | `next gate VS-14 --scenario vertical-v1 --suite physical-creature-skills` | `TRAIN-RTX-01`, EMB-01, POLICY-01, POLICY-02, SKILL-01, CREATURE-01, BEHAVIOR-01, TRAIN-P1, AUTHOR-P1, AI-07 and TOOL-04 pass; save/load/replay exact before/during/after switch; mismatch, missing model, unsafe switch and unsupported skill rejected with declared fallback | hardware/toolchain capability, creature/package graphs, model/provenance/certification manifests, route/state/replay traces, holdout metrics, CLI/context/changeset reports, required GIF/MP4 + hashes | `AwaitingCapability` без local RTX; retain PrototypeFallback/previous route; quarantine candidate; engine-owned lab; slice blocked until same public gates pass |
| VS-15 Headless agent development + human evidence | Verification & Evidence + Runtime + Rendering + Security | `next gate VS-15 --scenario vertical-v1 --suite headless-agent-evidence --profiles agent-fast,changeset` | TEST-01, HEADLESS-01, IMPACT-01, DIAG-01, AGENT-03, CAPTURE-01, MEDIA-P1, EVIDENCE-01, REVIEW-01, ARCH-07, RUNTIME-05, RENDER-04, AUDIO-02, TOOL-05 and SEC-07 pass; seeded agent edit/failure/minimized replay reproduced; displayless worker creates synchronized media/offline dossier; approve/reject works; stale/tampered/agent-issued decisions rejected; missing GPU/encoder/reviewer yields `AwaitingCapability`; null/capture gameplay hashes exact | impact/scenario/run manifests, replay/minimized replay, diagnostics, display/API/socket traces, raw frame/audio roots, media/encoder hashes, evidence bundle/offline dossier, approval/tamper/redaction audits | CPU development results remain valid; portable capture job waits for capability; candidate/baseline/decision is never promoted partially; slice remains blocked |

## Integrated performance budgets

`PERF-01` является одним aggregate child gate canonical `vertical-v1`, а не ручной сводкой. Команда `next gate PERF-01 --scenario vertical-v1 --profile <reference-profile>` запускает gameplay CPU, mechanics, physics/motor, renderer, streaming, RAM/VRAM, bounded queue/drop counters, `agent-fast` и ordinary changeset measurements. Root RunManifest MUST ссылаться на один immutable performance manifest с child measurement hashes. Missing/stale profile или required measurement означает `FAIL`; недоступная shipping GPU/hardware capability означает `AwaitingCapability` и не заменяется другим host.

On reference profile in release build, default 30 Hz gameplay/120 Hz physics/60 Hz motor inference:

- gameplay fixed-tick CPU excluding physics/motor p95 ≤8 ms and p99 ≤12 ms;
- Mechanics Runtime subset p95 ≤2 ms and p99 ≤4 ms в том же gameplay budget;
- physics+motor 16 full avatars p95 ≤4 ms, p99 ≤6 ms;
- renderer B0 p95 GPU ≤16.6 ms at 1080p;
- streaming commit never blocks gameplay tick >2 ms; decompression/I/O async;
- resident memory ≤12 GiB RAM and ≤5.5 GiB VRAM for canonical scene;
- no unbounded queue; each dropped/deferred optional work item counted.
- reference `agent-fast` CPU workflow completes in ≤5 minutes;
- complete ordinary changeset compute completes in ≤30 minutes, excluding worker queue and human wait; physical certification, fuzz and training gates remain explicitly named blocking long gates outside this budget.

Budget failure blocks performance conformance, даже если функциональный outcome correct. Optimization не может менять state hash/ownership.

| Gate | Owner | Pass/fail threshold | Evidence artifacts | Fallback / rollback |
|---|---|---|---|---|
| PERF-01 | Release Engineering + subsystem budget owners | все перечисленные budgets PASS на exact canonical reference profiles; bounded queues имеют 0 uncounted drop; profiling off/on сохраняет authoritative hashes | aggregate performance manifest, child timing/memory/GPU/queue reports, profile/toolchain hashes | retain prior conforming build; optimize without semantic change; AwaitingCapability if declared shipping hardware unavailable |

## Failure-path coverage

Canonical suite MUST явно включать missing AI, incompatible AI protocol, invalid motor model, physical/body/policy compatibility mismatch, missing required model/state schema, unsafe/interrupted policy switch, unsupported skill без evaluated novice route, forged/incomplete certification, runtime-training request, missing RTX certification capability, MPS unavailable/unsupported with CPU smoke fallback, physics policy rejection, plugin capability denial, Luau/Wasm overrun, package dependency/patch/hook conflict, missing mechanic package/migration, stale/path-escaping AgentChangeSet, MCP unavailable/incompatible, corrupt save/checksum, missing bundle, unsupported B0 GPU, device loss, importer malformed input, protected-data positive fixture stored outside publish path, unknown/omitted impact, deterministic retry divergence, missing GPU/encoder/reviewer capability, capture crash/gameplay divergence/partial publish, missing/stale/tampered baseline/evidence/review, malicious/oversized/redaction-failed media or dossier, generation provider faults, forbidden generation input, invalid/oversized generated asset, normalization/hash drift, invalid world constraints, incomplete generated-artifact provenance и generation-worker partial publication. Каждый path имеет stable diagnostic и declared fallback; fallback itself проходит replay/ownership checks.

## Evidence packet и acceptance

Release Engineering создаёт root `RunManifest`, ссылающийся на все child runs/artifacts. Verification & Evidence Team создаёт EvidenceBundleManifest; для каждого observable changeset Security & Governance проверяет exact HumanReviewDecision. Gate считается `PASS` только при доступных hash-verified evidence, успешных mandatory automatic gates и требуемом human attestation, не по ручной отметке. Worst/failure physical episodes сохраняются рядом с successful. Missing compute artifact is `FAIL`; недоступная внешняя capture/review capability даёт `AwaitingCapability`, которое также не является `PASS`.

Architecture Working Group, subsystem owners, Security & Governance и Release Engineering MUST одобрить один immutable implementation evidence packet. `vertical-v1-conformant` version/tag создаётся только после 15/15 `PASS`; waiver обязательного automatic gate не допускается, а human review не может изменить его результат. Неуспех возвращает backend/package/implementation в engineering/research, но не переписывает Accepted architecture baseline: опровергнутый technology candidate меняется новым ADR и повторяет тот же conformance suite.
