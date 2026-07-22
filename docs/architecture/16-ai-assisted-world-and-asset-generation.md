# SPEC-16: AI-assisted world and asset generation

| Поле | Значение |
|---|---|
| ID | SPEC-16 |
| Статус | Proposed |
| Версия | 1.0 |
| Владелец | Asset & Persistence Team |
| Последняя проверка | 2026-07-22 |
| Нормативные зависимости | [SPEC-01](01-system-architecture.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-09](09-tooling-sdk-and-observability.md), [SPEC-11](11-security-licensing-and-governance.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [SPEC-13](13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [ADR-017](adr/017-artifact-first-ai-content-generation.md) |
| Связанные документы | [SPEC-05](05-physics-animation-and-motor-control.md), [SPEC-08](08-audio-navigation-and-world-services.md), [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [EVIDENCE-001](evidence-register.md) |
| Заменяет | отсутствует |

## Назначение и scope

Подсистема задаёт AI-first authoring pipeline для концептов мира, landscape references, textures/materials, props, weapons, armor, architecture modules, characters и monsters. Она превращает human/agent brief и generated candidates в валидируемые engine-owned records, но не делает generative model source of truth.

Обязательная v1 часть — provider-neutral CLI/JSON orchestration, offline fixture adapter, generated-input quarantine, deterministic normalization, world/asset audits, common `NeutralAuthoringModel`/cooker integration и evidence workflow. Live OpenAI ImageGen, Pixal3D и другие adapters являются optional `Proposed` capabilities. Full scene editor, runtime generation, runtime training, обязательный cloud account и полностью автоматическое qualitative approval не входят в scope.

## Invariants

- Generated image, mesh, texture, rig, material или world layout является untrusted candidate до полного automatic gate closure.
- Ни prompt, provider response, model checkpoint, DCC scene, GLB, image или geometry cache не является runtime source of truth.
- Единственный publish path: candidate → validation/normalization → `NeutralAuthoringModel` → deterministic cooker → immutable bundle/`WorldChunk`.
- OpenAI ImageGen используется first-party pipeline для 2D generation/editing, но provider/model/account types остаются private adapter metadata.
- Image-to-3D output MUST NOT считаться game-ready по факту GLB export; topology, units, UV/PBR, LOD, collision, sockets, rig/skin и gameplay affordances проверяются отдельно.
- Landscape/world concept image MUST NOT определять authoritative terrain, navigation или chunk boundaries. Semantic constraints и procedural recipe имеют приоритет.
- AI не создаёт PersistentId произвольным текстом: allocator выдаёт IDs детерминированно из project namespace + recipe path + stable local key и проверяет collisions.
- Generative network/GPU failure не блокирует существующий content build, game/headless correctness или manual authoring.
- Все generated observable changes проходят `ImpactResolver`, automatic scenarios/capture и authorized human review SPEC-15.
- Raw generated/model outputs, datasets/checkpoints, caches и evidence хранятся вне source repository; manifests ссылаются на exact SHA-256.

## Ownership и source of truth

| Состояние | Единственный owner/source of truth | Не является source |
|---|---|---|
| Generation intent и constraints | versioned `GenerationRecipeManifest` в project source | conversational prompt/history alone |
| Approved source candidate bytes | external content-addressed artifact store + `GenerationResultManifest` hash | provider URL/session/gallery thumbnail |
| Generation execution facts | immutable `GenerationJobManifest` + `GenerationResultManifest` | worker logs without manifest |
| Provenance/license classification | `GeneratedAssetProvenanceManifest` reviewed under SPEC-11 | model README или provider marketing claim alone |
| Normalized authoring output | `AssetNormalizationManifest` → `NeutralAuthoringModel` hash | source GLB/image/DCC scene |
| World semantics | typed world profile inside recipe + validated neutral world records | landscape concept image |
| Published runtime content | SPEC-03 `ContentManifest` + immutable bundles/`WorldChunk` | generated candidate or normalization staging |
| Qualitative acceptance | SPEC-15 `HumanReviewDecision` exact candidate/evidence hashes | agent score, aesthetic model or prompt text |

Asset & Persistence Team владеет schemas, candidate state machine, normalization, world synthesis и cooker handoff. Developer Experience владеет CLI/JSON projection. Security & Governance владеет provenance/license/privacy/network policy. Verification & Evidence владеет impact/capture/evidence/review orchestration. RPG, World Services, Physical Embodiment и Presentation owners задают свои semantic/technical budgets.

## Public contracts

### GenerationRecipeManifest

Manifest MUST содержать:

- schema/version, recipe ID/revision, project/base hashes и owner;
- profile: `concept`, `world`, `object`, `weapon`, `armor`, `architecture`, `character`, `creature`, `material` или versioned extension;
- human objective, machine-readable constraints и forbidden content;
- source/reference artifact hashes и provenance references;
- target units, dimensions, orientation, silhouette/material/style invariants;
- required output views/channels/formats и candidate count bounds;
- geometry/texture/material/rig/collision/LOD/streaming budgets where applicable;
- declared adapters/capabilities, network consent, deadline, byte/output/cost ceilings;
- reproducibility class, named seeds where supported и canonical procedural parameters;
- required validation suites, observable categories и review policy;
- prompt body либо encrypted/private prompt artifact hash + safe redacted summary.

Free-form text MAY уточнять creative intent, но MUST NOT заменять typed scale, topology, traversal, safety, license или budget constraints.

### GenerationJobManifest

Job является immutable portable request к одному adapter и содержит recipe/base/input hashes, adapter ID + contract version, requested operation, explicit input/output roots, granted capabilities, network endpoint class, resource/deadline/output quotas, idempotency key и atomic publication rule. Secret/account token передаётся через environment-owned credential channel и никогда не сериализуется в manifest/log/evidence.

### GenerationResultManifest

Result содержит terminal status `Succeeded`, `Rejected`, `Failed` или `AwaitingCapability`; exact job/recipe/input hashes; actual adapter/tool/model/checkpoint/surface metadata when reported; output artifact hashes/types/bytes/dimensions; timing/cost metadata when available; warnings/diagnostics; redaction/protected-data scan; provider terms/license snapshot references; reproducibility class и atomic publication root.

Provider не сообщил model revision — result MAY оставаться `ArtifactFixed`, но MUST явно записать `model_revision=unavailable`; он не может заявлять recipe-reproducibility.

### GeneratedAssetProvenanceManifest

Provenance MUST замыкать human/agent author, source/reference ownership/license, generator/provider/tool/model identity насколько доступно, prompt/reference hashes, transformation chain, output usage/redistribution classification, content policy scan, reviewer-required flags и every normalization/cook derivative. Generated output само по себе не доказывает rights на source или training data.

### AssetNormalizationManifest

Manifest фиксирует ordered deterministic transforms, exact tool/version/config hashes, input/output hashes, canonical units/orientation, bounds, mesh/material/UV/rig/collision/LOD statistics, rejected/fixed conditions, target profile budgets, resulting `NeutralAuthoringModel` hash и required semantic records. External DCC/tool types не сохраняются после boundary.

## Candidate lifecycle

```text
DraftRecipe
  → ValidatedRecipe
  → QueuedJob
  → RunningExternal
  → StagedResult
  → Quarantined
  → ValidatedCandidate
  → NormalizedCandidate
  → ReviewedCandidate
  → ProposedChangeSet
  → CookedPublished
```

`Rejected`, `Failed` и `AwaitingCapability` terminal для exact job revision. Retry создаёт новый job ID и result; он не меняет прошлый result. `Quarantined` candidate не доступен DCC preview, agent context или human dossier до schema/hash/size/redaction/protected-data checks. Atomic publication exposes result/normalization manifest last.

## Reproducibility classes

| Class | Требование | Разрешённое использование |
|---|---|---|
| `RecipeDeterministic` | equal canonical recipe/inputs/toolchain дают byte-identical neutral output на declared Win/Linux targets | procedural terrain/world/layout, deterministic transforms, official recook |
| `ArtifactFixed` | inference MAY различаться; exact selected input/output bytes и metadata immutable/hash-addressed; downstream deterministic | OpenAI ImageGen, Pixal3D и иные generative candidates |
| `EphemeralPreview` | output может быть transient/unversioned и не имеет complete provenance | brainstorming only; publish/cook/AgentChangeSet запрещены |

`ArtifactFixed` не обещает повторную генерацию того же изображения/меша. Оно обещает, что exact approved bytes, provenance и deterministic downstream result восстановимы как факт.

## OpenAI ImageGen workflow

First-party 2D pipeline использует OpenAI ImageGen adapter для:

- world/landscape/interior keyframes и mood references;
- object identity anchors и controlled edits;
- front/back/left/right/three-quarter reference sets;
- isolated concepts weapons, armor, architecture modules, characters и creatures;
- tileable material/albedo references, ornaments, decals и wear variants.

Adapter MUST использовать generation/edit operations через доступную subscription surface только после explicit user/project network consent. Multi-turn refinement сохраняет parent artifact hash и invariant set. Public recipe не фиксирует private account/session; actual surface/model metadata записывается только если provider её сообщает.

Технический 3D reference profile SHOULD требовать: один complete isolated subject, unclipped silhouette, neutral background, even diffuse lighting, minimal cast shadow, declared camera/view, no text/logo/watermark/extra props. Character/creature reference требует neutral A/T-like pose, separated limbs и visible extremities. Weapon/armor reference не содержит hand/body unless profile explicitly uses a mannequin. Each view MUST сохранять identity, proportions, construction, materials и scale anchors; inconsistent view set rejected or reduced to an explicitly single-view job.

ImageGen texture output является visual/albedo candidate. Normal, height, roughness, metallic, opacity и AO channels MUST проходить deterministic derivation/validation; baked directional lighting и seams rejected according to material profile. AI-generated technical map не получает доверие без numeric/channel validation.

## Image-to-3D workflow

Pixal3D является first-party `Proposed` research adapter. Worker принимает exact image hash, выполняет pinned inference и публикует GLB + preview/result manifest. Output затем проходит:

1. container/parser bounds, finite-value и decompression checks;
2. canonical metres/right-handed orientation, pivot и scale normalization;
3. duplicate/degenerate/self-intersection/non-manifold/open-surface classification;
4. repair/remesh/retopology according to target profile, never silent arbitrary sculpt;
5. UV chart/overlap/density/bounds validation;
6. PBR channel/colorspace/range/resolution validation and texture compression staging;
7. deterministic LOD generation with declared geometric/silhouette error budgets;
8. collision decomposition/hulls with volume/contact/error budgets;
9. sockets, interaction volumes, grip points, equip slots and semantic tags;
10. character/creature skeleton, skin weights, animation retarget and physical-archetype compatibility where required;
11. `NeutralAuthoringModel` emission, common validation/cook and scenario/capture gates.

Automatic repair MUST preserve an audit mapping input primitive/material → output or report dropped/merged ranges. If correction changes recognizable shape, topology class, hard-surface construction or character anatomy beyond profile tolerance, candidate requires regeneration/manual source edit and new review; normalizer не становится creative co-author hidden from provenance.

## World generation workflow

World profile MUST описывать не только visual theme, но и:

- world/region hierarchy, bounds и canonical scale;
- mandatory/optional POI, entrance/exit и traversal graph;
- terrain constraints, water, biome/vegetation masks и forbidden volumes;
- road/path/spline network, slope/clearance/width classes и physical traversal envelopes;
- settlement/dungeon plots, modular grammar, interior/exterior portals и interaction slots;
- factions/ownership zones, spawn/encounter/resource budgets и RPG affordance requirements;
- streaming chunk targets, dependency limits, PersistentId namespaces и visibility/audio/nav metadata;
- required quests/dialogue/interactions/mechanic/physical scenario hooks;
- camera/capture anchors and human-review views.

World synthesis order is normative:

```text
brief + concept references
  → typed semantic world recipe
  → deterministic region/terrain/spline/plot generation
  → catalog retrieval and bounded generated-asset requests
  → constrained placement and chunk partition
  → collision/navigation/streaming/RPG audits
  → NeutralAuthoringModel
  → deterministic cooker/WorldChunks
  → headless traversal/gameplay scenarios
  → displayless capture and human review
```

Concept image MAY guide style, skyline, biome and landmark constraints but MUST NOT bypass typed topology. Full-scene image-to-3D MAY produce `EphemeralPreview` or background reference only. It cannot produce authoritative playable world chunk until reconstructed into typed records and all world gates pass.

## Catalog-first generation policy

Pipeline SHOULD query project/approved asset catalog before new generation. Catalog match considers semantic type, dimensions, style/material tags, required sockets/rig/collision/LOD, license/provenance and target budgets. Existing approved asset reuse does not invoke a network generator. Generated variant still receives a new recipe/result/provenance chain and AssetId revision; similarity search result is not implicit license permission.

## CLI/JSON contract

Normative commands:

| Команда | Contract |
|---|---|
| `next generate recipe new|describe|validate <profile>` | scaffold/introspect/validate closed recipe schema and constraints |
| `next generate plan <recipe> --out <job-root>` | resolve catalog hits, required jobs, capabilities, budgets and review impact without generation |
| `next generate run <job> --artifact-root <dir>` | execute one bounded adapter job and atomically publish result manifest |
| `next generate inspect <result-or-candidate>` | read-only provenance, tool/model, outputs, diagnostics, quotas and missing gates |
| `next generate resume <failed-or-awaiting-job>` | create new job revision with explicit reason; never mutate old result |
| `next asset normalize <candidate> --profile <profile> --out <dir>` | deterministic normalization → manifest + NeutralAuthoringModel candidate |
| `next asset audit <candidate-or-neutral>` | geometry/material/UV/LOD/collision/rig/semantic validation |
| `next world synth <recipe> --seed <seed> --out <dir>` | deterministic semantic/procedural synthesis and neutral world output |
| `next world audit <neutral-or-bundle>` | traversal/nav/collision/placement/streaming/RPG/ID budget checks |
| `next generate diff <base> <candidate>` | recipe/provenance/geometry/material/world/observable category diff |
| `next generate changeset <candidate> --project <p>` | produce reviewable AgentChangeSet referencing exact manifests/evidence |

Every command supports SPEC-09 `--help`, `--version`, `--format json`, explicit outputs, exit codes and non-interactive fixture mode. Optional MCP/GUI projections MUST produce identical schemas/results and cannot expose generic shell/network/file access or reviewer credentials.

## Security, privacy, licensing and budgets

- Network default-off. Recipe/job must name adapter endpoint class and receive explicit project/user grant; credential never enters manifest.
- Input references, prompts and outputs are scanned/classified before upload and again before human viewing/publication. Protected/imported game data and private user images cannot be uploaded without an explicit policy/legal path; first-party baseline denies them.
- Worker runs sandboxed with input read-only, dedicated staging output, no project write access, bounded CPU/GPU/RAM/disk/bytes/files/duration and no ambient network.
- Every code/model/checkpoint/service/data/output license and redistribution/usage term is classified separately under SPEC-11. MIT code does not classify third-party dependencies, checkpoints, training data or generated output automatically.
- Recipe declares maximum output count, dimensions, bytes, polygons, texture pixels, wall time and provider cost ceiling when cost is available. Overrun cancels and quarantines staging; it never silently lowers required quality or continues billing.
- Prompts and private references are default-redacted in normal logs/evidence. Full prompt MAY live in protected content-addressed artifact storage; public provenance retains hash and safe summary.
- Provider/tool safety filtering and human review complement but do not replace project protected-data, trademark, license and originality review.

## Impact, evidence and review

Every generated candidate adds `generated-content`, `provenance` and its target output categories to ImpactResolver. Images, textures, geometry, materials, rigs, animation, placement and world composition reach `visual`; collision reaches `physics`; rig/controller/physical archetype reaches `animation`, `physics` and possibly `motor`; navigation/world topology adds world traversal/gameplay suites.

Automatic evidence MUST include recipe/job/result/provenance/normalization manifests, source/candidate hashes, asset/world audits, common cook hashes, relevant scenarios/replays, capture artifacts, tool/model/SBOM/license reports and failure samples. Human dossier compares concept/source, turntable/reference views, normalized render, in-world scale/material/collision/LOD views and representative failures. Human review cannot approve unreachable POI, invalid collision, broken rig, license gap or failed cooker.

## Failure semantics

- OpenAI subscription/account/network/quota unavailable → live job `AwaitingCapability`; existing/manual/catalog authoring and offline fixture gates remain available.
- Provider timeout/error/content refusal → classified failed result, staging discarded; retry is a new job revision.
- Provider/model revision unavailable or drifted → `ArtifactFixed` only; prior approved bytes remain valid, new output requires new result/evidence/review.
- Input/output hash, MIME, dimensions, archive/container or quota mismatch → quarantine before decode/DCC/human viewing.
- Prompt/reference contains protected/private/unlicensed content outside granted policy → deny upload, quarantine local candidate and emit incident diagnostic.
- Multi-view identity/proportion/material inconsistency → reject view set, regenerate bounded view or explicitly downgrade to single-view reconstruction; no silent merge.
- Pixal3D/CUDA/dependency unavailable → `IMG3D-P1=AwaitingCapability`; use pinned TripoSR comparator/manual mesh or omit generated 3D.
- Worker crash/GPU OOM/disk full → no atomic result publish; prior valid candidate/manifests unchanged.
- Mesh has non-finite/out-of-bounds/invalid indices/resource bomb → reject before normalization allocation or DCC load.
- Repair/retopology exceeds declared shape/topology tolerance → reject candidate; no automatic creative mutation hidden as technical fix.
- Material channels invalid, baked lighting/seams exceed profile or texture budget fails → reject/regenerate/derive deterministically; no mislabeled PBR publish.
- Character/creature rig, skin, physical compatibility or safety fails → no playable character publish; visual static preview MAY remain separate candidate profile.
- World mandatory POI unreachable, portal/slope/clearance invalid, placement overlaps forbidden volume, IDs duplicate or chunk budgets exceed → world candidate rejected; human approval cannot waive automatic failure.
- Generated content lacks complete provenance/license/output-terms classification → no AgentChangeSet apply or cook publication.
- Capture/reviewer unavailable after automatic PASS → candidate remains `AwaitingCapability`, never baseline/promoted content.

## Verification gates

| Gate | Сценарий | Pass/fail threshold | Evidence | Fallback/rollback |
|---|---|---|---|---|
| GEN-01 | offline fixture adapters across every recipe profile, success/refusal/timeout/crash/quota/version faults | 100% jobs reach one terminal status; exact JSON schemas/hashes/idempotency; 0 vendor type in public contracts; 0 partial publish or project mutation | golden recipe/job/result manifests, adapter parity/schema/API scan, fault matrix | disable adapter; manual/catalog source + same validator/cooker |
| GEN-02 | malicious/protected/private/unlicensed/oversized prompt, image, GLB, texture and manifest corpus | 100% forbidden upload/decode/view/publish denied or quarantined; 0 secret in artifacts; 0 unconsented network; every accepted artifact license/provenance classified | packet capture, redaction/protected-data/license report, sandbox/quota audit | network adapters disabled; local/manual authoring only |
| GEN-03 | 200 valid/adversarial generated asset candidates × two Win/Linux normalization/cook runs | all accepted outputs meet declared profile bounds/topology/UV/PBR/LOD/collision/rig/semantic budgets; all invalid cases rejected; equal canonical inputs produce byte-identical neutral/platform-neutral bundles | normalization/audit manifests, geometry/material metrics, SBOM, NAM/bundle hashes | regenerate/manual repair as new provenance step; prior asset retained |
| GEN-04 | 100 bounded world recipes/seeds including valley, settlement, cave and invalid constraints | exact world-plan/NAM/platform-neutral bundle hashes across Win/Linux; 100% mandatory POI reachable and required interaction/portal/traversal assertions PASS; 0 duplicate IDs/forbidden overlaps; all invalid constraints rejected | recipe/seed hashes, world/nav/collision/streaming/RPG audit, scenarios/replays/captures | fix recipe/procedural generator; use previous valid/manual world |
| GEN-05 | cold agent uses only public context/CLI to plan fixture ImageGen-like job, normalize prop, place it in bounded world and propose observable changeset | complete recipe→result→normalization→NAM→cook→scenario→capture/evidence chain; ImpactResolver selects all categories; no private API/runtime UI; valid human decision admits exact hash only | AuthoringContextBundle, command transcript, manifests, AgentChangeSet/impact/run/evidence/review records | reviewed manual CLI workflow; candidate remains unadmitted |
| IMAGEGEN-P1 | live OpenAI ImageGen subscription adapter corpus defined ADR-017 | ADR-017 threshold; generation quality remains human-reviewed, not automatic gameplay oracle | capability/network/result/provenance manifests, samples and review decisions | `AwaitingCapability`; manual/imported approved reference images |
| IMG3D-P1 | live pinned Pixal3D corpus defined ADR-017 on declared Linux/NVIDIA worker | ADR-017 threshold plus GEN-02/03; no claim beyond exact hardware/checkpoint/corpus | model/tool/SBOM/hardware, normalization metrics, failure and media evidence | TripoSR/manual mesh; adapter unaccepted |
| IMG3D-F1 | pinned TripoSR low-resource comparator corpus defined ADR-017 | ADR-017 threshold plus GEN-02/03; measured comparator only | same manifest/normalization/license/timing evidence | manual mesh or generation unavailable |

Asset & Persistence Team владеет GEN-01/03/04, Security & Governance co-owns GEN-02 and technology license closure, Developer Experience + Verification & Evidence co-own GEN-05, а live adapter owners предоставляют IMAGEGEN/IMG3D evidence. Passing technology gate принимает exact adapter revision only; architecture остается provider-neutral.
