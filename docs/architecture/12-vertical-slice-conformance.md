# SPEC-12: Vertical-slice conformance

| Поле | Значение |
|---|---|
| ID | SPEC-12 |
| Статус | Accepted |
| Версия | 1.5 |
| Владелец | Repository Owner |
| Последняя проверка | 2026-07-23 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md)…[SPEC-11](11-security-licensing-and-governance.md), [SPEC-13](13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [ADR-011](adr/011-macos-developer-host-local-verification-and-staged-training.md), [ADR-015](adr/015-evidence-trust-fixture-separation-and-attestation.md), [ADR-016](adr/016-compositional-gameplay-budgets.md), [TRACE-001](traceability.md) |
| Заменяет | отсутствует |

## Назначение

Vertical slice — первый интеграционный release gate, а не showcase. Он доказывает, что независимый engine может reproducibly построить, расширить public mechanic/creature packages, переключить заранее обученные motor skills, запустить, сохранить, воспроизвести и диагностировать bounded RPG scenario при соблюдении backend/security/importer/mod boundaries. Разработка и automatic acceptance MUST быть доступны CPU-only Linux agent без монитора, а качественная оценка observable changes MUST выполняться по hash-bound evidence dossier без обязательного интерактивного запуска игры. Ни один отдельный gate не заменяет весь пакет.

`HOST-MAC-01` и `TRAIN-MAC-P0` являются bootstrap capability gates и не добавляют шестнадцатый vertical gate. Они разрешают portable разработку и раннюю проверку ML/export toolchain на Mac, но не закрывают Windows/Linux package, renderer, runtime/training correspondence или physical certification. Перед `PhysicalCertified` частью VS-14 MUST быть `TRAIN-RTX-01=PASS`; отсутствие capability даёт `AwaitingCapability`, а `PrototypeFallback` остаётся допустимым development result.

## Source of truth, ownership и public boundary

Versioned `vertical-v1` TestScenarioManifest + root RunManifest являются source of truth для inputs, thresholds, выполненных gates и evidence hashes. Release Engineering владеет orchestration и итоговым pass/fail; subsystem owner владеет своим child gate, Verification & Evidence Team — impact/capture/evidence contracts, Security & Governance — release trust evidence. Public conformance boundary состоит из canonical scenario schema, future `next gate` CLI, RunManifest, EvidenceBundleManifest и `HumanReviewDecisionV1`/`AttestationEnvelopeV1`; CI runner/vendor formats не являются contract.

## Canonical scenario `vertical-v1`

Один versioned scenario manifest MUST задавать fixed content/import whitelist, seeds, tick rates, supported machine profiles, expected state transitions, deliberate fault injection points, camera/render settings и thresholds.

Сценарий:

1. пользователь явно указывает локальную тестовую installation; importer экспортирует whitelisted scene subset, одного character archetype, items и reviewed declarative behavior в NIM;
2. common validator/cooker публикует scene bundle; второй cook доказывает cache reuse; runtime больше не обращается к installation;
3. player входит в B0-rendered scene, перемещается, взаимодействует с item/object и physical avatar obstacle/slope;
4. generic NPC воспринимает player, вступает в dialogue, выдаёт/обновляет малый Quest и меняет одну relationship dimension через accepted commands;
5. optional `ai-host` сначала работает, затем timeout/kill/restart; та же quest path корректно продолжается offline fallback;
6. physical avatar проходит full/simplified/capsule transitions, perturbation и recovery с required media evidence;
7. Luau overrun и Wasm forbidden capability инъецируются и изолируются;
8. first-party ranged package выполняет aim/fire/reload, projectile/contact/effect и presentation-cue loop; first-party elemental package выполняет cast/interruption/projectile/area/status loop;
9. cold author workflow из public AuthoringContextBundle добавляет data-only ranged variant и spell, создаёт AgentChangeSet и проходит validate/test/simulate без private source knowledge;
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
| VS-01 Import/cook/cache/load | Importer + Asset Team | `next gate VS-01 --scenario vertical-v1-import-smoke --platform $TARGET` | IMPORT-P1/P2/P7; first cook/load success; second cook ≥90% cache hits; runtime opens 0 source-install files; imported/NIM/cooked bytes and media never enter publishable evidence | sanitized provenance/validation/load projection, cook hashes, source-access/cleanup trace; no screenshot | no publish; fix mapping/cooker; neutral demo не засчитывает importer criterion |
| VS-02 Player loop + persistence | Runtime + RPG + Persistence | `next gate VS-02 --scenario vertical-v1 --fault-save-points all` | item/object interaction and movement outcomes exact; SAVE-01/02; loaded state hash equals pre-close committed hash; 100% power points retain valid generation | replay/save manifests, event/state hashes, fault matrix | retain previous save; criterion blocks slice |
| VS-03 Generic NPC/dialogue/quest | RPG Team | `next gate VS-03 --scenario vertical-v1 --npc generic-01` | no legacy type/schema; dialogue and one quest transition exact; relationship delta caused by accepted command; 100 repeats exact hashes | command/event trace, RPG inspector report | compiled generic rules/authored dialogue; block if incorrect |
| VS-04 Offline AI + restart | Agent Team | `next gate VS-04 --scenario vertical-v1 --ai-faults absent,timeout,kill,restart,bad-version` | all variants complete same mandatory quest outcome; no tick blocked by AI; 0 duplicate commands; fallback ≤1 tick after deadline signal | ai inspector timeline, fault report, replay hashes | circuit-break/disable ai-host and deterministic planner |
| VS-05 Physical avatar + LOD | Physical Embodiment Team | `next gate VS-05 --scenario vertical-v1 --suite physical --render-artifacts` | PHYS-P1…P8/MOTOR-P1; scenario success aggregate ≥95%, 0 safety violation; LOD discontinuities within SPEC-05 | full metrics; baseline/selected/failures/comparison GIF, MP4 as required, SHA-256 RunManifest | Jolt→Bullet same gates; heuristic motor; if no backend/policy passes, slice blocked |
| VS-06 Luau/Wasm isolation | RPG + Security | `next gate VS-06 --scenario vertical-v1-neutral --inject extension-faults` | SCRIPT-P1/P2/P5, PLUGIN-P1/P2 and MOD-P2 pass; forbidden capability 100% denied; authoritative counter outcomes exact across load permutations; every watchdog trip NonConforming; 0 partial command/host crash | audit log, PackageTrustManifest/consent matrix, fuel/instruction/allocation report, watchdog diagnostics, replay | pin runtimes/disable optional plugin; required failure pre-world blocks |
| VS-07 Baseline renderer | Rendering Team | `next gate VS-07 --scenario vertical-v1 --tier B0 --disable rt,mesh-shader` | RENDER-P1/02; ≥60 FPS 1080p, p95 GPU ≤16.6 ms on reference GPU; image SSIM ≥0.98; 0 validation errors | RunManifest, GPU trace, validation logs, screenshots/diff | indexed/bounded descriptor B0 path; unsupported B0 GPU clean diagnostic |
| VS-08 Windows/Linux packages | Release Engineering | `next gate VS-08 --scenario vertical-v1 --targets windows-x86_64,linux-x86_64 --clean-vm` | clean install/launch/save/replay success both; no undeclared shared libraries; package/SBOM hashes present | VM logs, package manifests, SBOM, hashes | rollback package; platform remains unshipped and slice blocked |
| VS-09 Protected-data absence | Security + Importer | `next gate VS-09 --scenario vertical-v1-neutral --scan repo,cache,build,package,artifacts,evidence,media,logs` | PRIVACY-02; 0 protected source/derived byte/signature findings; import-smoke cleanup complete; neutral bundle has no imported dependency; mixed fixture rejected | signed scanner report, fixture provenance, source-access/cleanup/quarantine audit | quarantine/remove generated artifacts; incident review; rerun clean |
| VS-10 No legacy runtime types | Core Architecture | `next gate VS-10 --scenario vertical-v1 --scan-api-link-schema` | 0 legacy parser/VM deps and 0 forbidden-symbol-set entries from SPEC-10 in runtime public/source schemas; permitted only importer repo/RFC history | source/API/schema scan, link map, SBOM | boundary refactor; block slice |
| VS-11 Replay/headless parity | Runtime + Release Engineering | `next gate VS-11 --scenario vertical-v1-neutral --compare game,headless` | RUNTIME-06/07, CANON-01 and REPLAY-01; accepted command ledger/rejections/DomainEvent/outcomes exact; state root exact on same target; physical tolerance only where explicitly registered | command/spawn/canonical vectors, replay ledger, state-root/first-divergence report | deterministic fix; no waiver for gameplay divergence |
| VS-12 Security/governance review | Security & Governance | `next gate VS-12 --scenario vertical-v1 --review-packet` | SEC-01…07, REVIEW-02, PRIVACY-02, LIC-01, GOV-01 pass; legal importer approval exact release; VS-01…VS-11 and VS-13…VS-15 child manifests PASS; traceability completeness 100%; all hashes/trust/revocation state verify offline | signed review record/envelope/trust manifests, SBOM, scans, traceability report, required HumanReviewDecisionV1 records | do not declare conformance; engine-only review MAY proceed without importer release but not vertical-v1 acceptance |
| VS-13 Mechanics/mod/agent authoring | Gameplay Extensibility + DevEx + Security | `next gate VS-13 --scenario vertical-v1 --suite mechanics-mod-agent --adapters cli,mcp` | MECH-01…07, AI-06, MOD-01/02, AGENT-01/02, MCP-P1 and TOOL-03 pass; ranged/magic/reference-agent packages use 0 hidden APIs; NPC discovers affordances; MCP failure completes via CLI | package/lock/affordance graphs, context bundle, changesets, replay/profile/security reports, MCP schema diff, physical media when required | disable MCP and use CLI; reject/pin package; add missing public primitive via ADR; slice blocked until public path passes |
| VS-14 Physical creature + progressive motor skills | Physical Embodiment + RPG + Agent + DevEx | `next gate VS-14 --scenario vertical-v1 --suite physical-creature-skills` | `TRAIN-RTX-01`, EMB-01, POLICY-01, POLICY-02, SKILL-01, CREATURE-01, BEHAVIOR-01, TRAIN-P1, AUTHOR-P1, AI-07 and TOOL-04 pass; save/load/replay exact before/during/after switch; mismatch, missing model, unsafe switch and unsupported skill rejected with declared fallback | hardware/toolchain capability, creature/package graphs, model/provenance/certification manifests, route/state/replay traces, holdout metrics, CLI/context/changeset reports, required GIF/MP4 + hashes | `AwaitingCapability` без local RTX; retain PrototypeFallback/previous route; quarantine candidate; engine-owned lab; slice blocked until same public gates pass |
| VS-15 Headless agent development + human evidence | Verification & Evidence + Runtime + Rendering + Security | `next gate VS-15 --scenario vertical-v1-neutral --suite headless-agent-evidence --profiles agent-fast,changeset` | TEST-01, HEADLESS-01, IMPACT-01, DIAG-01, AGENT-03, CAPTURE-01, MEDIA-P1, EVIDENCE-01, REVIEW-01/02, PRIVACY-02, ARCH-07, RUNTIME-05, RENDER-04, AUDIO-02, TOOL-05 and SEC-07 pass; seeded edit/failure/minimized replay reproduced; displayless worker creates neutral synchronized media/offline dossier; stale/tampered/revoked/agent decisions rejected; missing capability is `AwaitingCapability`; null/capture gameplay hashes exact | impact/scenario/run manifests, replay/minimized replay, diagnostics, display/API/socket traces, fixture provenance/cleanup, raw frame/audio roots, media/encoder hashes, evidence bundle/offline dossier, payload/envelope/trust/tamper audits | CPU development results remain valid; portable capture job waits for capability; candidate/baseline/decision is never promoted partially; slice remains blocked |

## Integrated performance budgets

On reference profile in release build, default 30 Hz gameplay/120 Hz physics/60 Hz motor inference:

- ADR-016 `GameplayBudgetMatrix` имеет mutually-exclusive owner rows, total p95 ≤8 000 us / p99 ≤12 000 us и reserved headroom ≥250/500 us;
- Mechanics Runtime subset p95 ≤2 000 us / p99 ≤4 000 us, agent-planning and navigation rows each p95 ≤1 250 us / p99 ≤1 500 us в том же integrated run;
- physics+motor 16 full avatars p95 ≤4 ms, p99 ≤6 ms;
- renderer B0 p95 GPU ≤16.6 ms at 1080p;
- streaming commit never blocks gameplay tick >2 ms; decompression/I/O async;
- resident memory ≤12 GiB RAM and ≤5.5 GiB VRAM for canonical scene;
- no unowned span, starvation or dropped due work; every bounded deterministic defer recorded.
- reference `agent-fast` CPU workflow completes in ≤5 minutes;
- complete ordinary changeset compute completes in ≤30 minutes, excluding worker queue and human wait; physical certification, fuzz and training gates remain explicitly named blocking long gates outside this budget.

`PERF-01` использует exact ADR-016 100-NPC membership/cadence phases, 1 000 warm-up + 10 000 measured ticks и nearest-rank percentiles. Budget failure blocks performance conformance, даже если функциональный outcome correct. Subsystem-only PASS не закрывает integrated failure; optimization не может менять state hash/ownership.

## Failure-path coverage

Canonical suite MUST явно включать missing AI, incompatible AI protocol, invalid motor model, physical/body/policy compatibility mismatch, missing required model/state schema, unsafe/interrupted policy switch, unsupported skill без evaluated novice route, forged/incomplete certification, runtime-training request, missing RTX certification capability, MPS unavailable/unsupported with CPU smoke fallback, physics policy rejection, plugin capability denial, Luau/Wasm overrun, package dependency/patch/hook conflict, missing mechanic package/migration, stale/path-escaping AgentChangeSet, MCP unavailable/incompatible, corrupt save/checksum, missing bundle, unsupported B0 GPU, device loss, importer malformed input, protected-data positive fixture stored outside publish path, unknown/omitted impact, deterministic retry divergence, missing GPU/encoder/reviewer capability, capture crash/gameplay divergence/partial publish, missing/stale/tampered baseline/evidence/review и malicious/oversized/redaction-failed media or dossier. Каждый path имеет stable diagnostic и declared fallback; fallback itself проходит replay/ownership checks.

## Evidence packet и acceptance

Release Engineering создаёт root `RunManifest`, ссылающийся на все child runs/artifacts. Verification & Evidence Team создаёт EvidenceBundleManifest; для каждого observable changeset Security & Governance проверяет exact HumanReviewDecision. Gate считается `PASS` только при доступных hash-verified evidence, успешных mandatory automatic gates и требуемом human attestation, не по ручной отметке. Worst/failure physical episodes сохраняются рядом с successful. Missing compute artifact is `FAIL`; недоступная внешняя capture/review capability даёт `AwaitingCapability`, которое также не является `PASS`.

Architecture Working Group, subsystem owners, Security & Governance и Release Engineering MUST одобрить один immutable implementation evidence packet. `vertical-v1-conformant` version/tag создаётся только после 15/15 `PASS`; waiver обязательного automatic gate не допускается, а human review не может изменить его результат. Неуспех возвращает backend/package/implementation в engineering/research, но не переписывает Accepted architecture baseline: опровергнутый technology candidate меняется новым ADR и повторяет тот же conformance suite.
