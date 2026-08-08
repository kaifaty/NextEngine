# SPEC-33: Behavior-policy training, evaluation and deployment lifecycle

| Поле | Значение |
|---|---|
| ID | SPEC-33 |
| Статус | Proposed |
| Lifecycle | Consumer-driven R4 proposal |
| Версия | 0.1 |
| Последняя проверка | 2026-08-08 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-06](06-ai-agents-perception-and-memory.md), [SPEC-11](11-security-licensing-and-governance.md), [SPEC-12](12-vertical-slice-conformance.md), [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-24](24-content-catalog-bundle-and-neutral-asset-schemas.md), [SPEC-32](32-npc-cognition-intention-lifecycle-and-deterministic-behavior-inference.md), [ADR-009](adr/009-pretrained-foundation-policies-and-progressive-motor-skills.md), [ADR-016](adr/016-compositional-gameplay-budgets.md), [ADR-030](adr/030-product-first-development-and-lightweight-validation.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-050](adr/050-hierarchical-npc-cognition-and-learned-behavior-policy-boundary.md) |
| Заменяет | отсутствует |

## Статус и scope

SPEC-33 описывает proposed algorithm-neutral lifecycle для offline training,
evaluation, export, promotion и deployment двух behavior-policy bundles.
Документ не выбирает PPO, imitation learning, self-play, model architecture,
trainer, simulator или inference runtime и не добавляет code/contracts в этом
changeset.

До production R4 consumer все checks здесь имеют
`NOT_RUN(NO_PRODUCTION_CONSUMER)`, bundle schema не входит в current registry,
а model artifacts не являются shipped content. Runtime learning и изменение
weights запрещены независимо от статуса документа.

## Invariants

- Strategic и tactical policy обучаются двумя independent lanes и выпускаются
  двумя independently content-addressed bundles.
- Algorithms replaceable. Normative surface consists of environment inputs,
  observation/candidate/output/state schemas, artifacts, provenance and
  evaluation behavior.
- Training scenarios use production-shaped inputs and immutable documented
  probes. Direct ECS/RPG/world mutation or test-only success injection is
  forbidden.
- Runtime/training parity covers exact candidate ordering/masks, score
  quantization, recurrent state conversion and final applied decision, not
  only raw model tensor similarity.
- Shipped weights are immutable package artifacts. Datasets, checkpoints,
  optimizer state, replay buffers, training runs, captures and generated model
  outputs do not enter Git or normal engine distribution.
- Model output remains untrusted at runtime and always has a deterministic
  utility/HTN fallback.
- A single good bundle cannot promote the track: both roles must be integrated
  and co-evaluated in one production R4 vertical.

## Two training lanes

### Strategic lane

Minimum curriculum progresses through:

1. authored routine following and bounded deviation;
2. drive-aware choice with fatigue/rest/sleep;
3. safe sleep outside schedule using available-place/navigation facts;
4. social-contact and conversation initiation from relationship/memory views;
5. medium/long-horizon goal persistence, interruption and expiry;
6. emergency suspension signal, post-emergency revalidation and
   resume/cancel/replan;
7. bounded valid LLM goal suggestions mixed with authored candidates;
8. `Simulated`/`Active` cadence, deterministic deferral and save/replay state.

Strategic environment never asks the model to produce combat action, waypoint,
WorldCommand or free-form dialogue text.

### Tactical lane

Minimum curriculum progresses through:

1. current perception/threat/support interpretation with uncertainty;
2. canonical navigation-query/corridor candidate choice;
3. planner-visible affordance selection and capability masks;
4. fight/flee/help-call modes and bounded emergency override;
5. yield initiation, observation by other actors and possible non-compliance;
6. shared speech-act/conversation request;
7. emergency resolution and handoff to strategic revalidation;
8. `Active`-only cadence, multi-agent combat/social situations and fallback.

Tactical environment never treats a selected affordance, route or surrender as
successful until production Mechanics/World Services/Physics/RPG owners commit
the corresponding outcome.

### Joint curriculum and co-evaluation

After independent lane thresholds, a joint suite runs both frozen bundle
revisions without joint online weight mutation:

```text
routine
  → fatigue and out-of-schedule sleep proposal
  → wake / return to activity
  → perceived threat
  → tactical fight, flee, help-call or yield
  → player↔NPC or NPC↔NPC speech act / dialogue consequence
  → emergency resolution
  → strategic goal revalidation and resume or replan
```

Co-evaluation measures system behavior and conflict handling. It does not merge
models, recurrent state or optimizers. One lane may be retrained while the
other bundle stays frozen, but every promoted pair gets a new exact joint
evaluation identity.

## Training environment contract

`BehaviorTrainingEnvironmentManifestV1` proposal binds:

- environment/scenario ID, version and content hash;
- exact engine build, project lock, schema/content/mechanics/world hashes;
- behavior observation/candidate/output/state and quantization profiles;
- fixed simulation rates, named RNG descriptors/seeds and episode boundaries;
- initial production fixture/save plus ordered production `ScenarioAction`;
- immutable `ProbeSpec` and bounded assertion/reward fact mapping;
- subject/archetype/trait/skill/relationship distribution closure;
- declared faults, resource/tick/episode/output limits;
- trainer adapter protocol and environment correspondence profile;
- provenance/license classification of every external input.

Environment mutation occurs only through normalized input,
`AgentIntent`/`InvokeAbility`, validated `WorldCommand`, lifecycle action or
declared adapter fault. Reward/label builder reads committed `DomainEvent`,
receipts and immutable probes; it cannot write health, relationship, goal,
candidate mask or success state.

Environment reset activates an immutable initial state and resets declared
named RNG streams/state schemas. Wall-clock sleep, trainer worker identity,
GPU order and environment process arrival do not choose simulation outcome.
Parallel environments have stable episode IDs and deterministic input/result
merge independent of worker count.

Training may intentionally be stochastic through named training seeds and
algorithm state, but every run records those inputs. Runtime artifact is valid
only if exported inference has no stochastic op/RNG and passes exact applied
decision parity.

## Algorithm-neutral lane interface

PPO, imitation, offline RL, self-play, curriculum learning, distillation and
other algorithms MAY be used. Normative trainer boundary is:

```text
immutable environment manifest + production observations/candidate masks
  → replaceable trainer
  → candidate model/checkpoint outside repository
  → deterministic export
  → immutable candidate bundle
  → schema/parity/quality/fault/integrated evaluation
  → promoted or rejected bundle revision
```

Algorithm-specific hyperparameters, optimizer, replay buffer and distributed
worker state are provenance inputs, not runtime contracts. Changing algorithm
does not require an architecture change if exported bundle satisfies the same
manifest, schemas, runtime behavior and checks.

No trainer receives credentials, arbitrary source-tree write access, protected
game assets, mutable production save or privileged engine state. A remote
training service requires explicit external capability/security policy; its
provider/session types stay outside engine public contracts.

## `BehaviorPolicyBundleManifestV1`

Each role has one immutable manifest:

| Field group | Required content |
|---|---|
| Identity | schema version, bundle ID/version, role `Strategic` or `Tactical`, policy ID, manifest hash |
| Model | exact model `AssetId`/SHA-256/byte size/format/opset or equivalent neutral feature set; no runtime path |
| Schemas | exact observation, candidate, output, score-quantization, sampling and recurrent-state schema hashes |
| Compatibility | supported archetype IDs/revisions, trait schema/ranges, skill and relationship-view schema, policy role/cadence/tier compatibility |
| Inference | closed vendor-neutral evaluator capabilities, maximum batch/candidate/state widths and deterministic-operator profile |
| Resources | supported target triples, CPU features, RAM/VRAM/disk, warm-up, concurrency and per-boundary resource envelope |
| Sampling | named engine RNG descriptor/profile, integer selection profile and proof that model itself has no stochastic op/RNG |
| Training provenance | environment/build/project/config/tool/parent-model hashes, algorithm family label, seed/run references and source/license classification |
| Evaluation | immutable lane suite, held-out suite, fault/parity/joint suite IDs and exact result hashes |
| Deployment | required runtime adapter protocol, package/content dependencies, compatibility key and previous compatible revision policy |
| Fallback | exact deterministic utility/HTN fallback profile ID/hash; model cannot choose or rewrite it |

Shared foundation ancestry is recorded as provenance/compatibility. Strategic
and tactical manifests remain separate and may have different model formats,
state widths, cadence and resource envelope. Per-NPC weights, mutable adapter
files, hidden tokenizer/session, credentials, optimizer, checkpoint or dataset
are forbidden.

The manifest and every referenced immutable artifact must be bounded,
canonical, content-addressed and resolve from exact project package closure.
Hash, schema, compatibility, license/provenance or resource mismatch rejects
the bundle before activation and retains planner fallback.

## Export and runtime/training parity

Export is a deterministic, pinned transform for equal source checkpoint/config
on the declared toolchain, except that source checkpoint itself may arise from
stochastic training. Export records tool hashes, operator conversion, numeric
profile and final model SHA-256. Candidate model must reject or eliminate:

- stochastic/dropout/random operators in inference;
- dynamic candidate/state widths not admitted by schema;
- implicit dtype/layout/unit conversion;
- hidden recurrent/cache/session state;
- provider-dependent sampling, top-k or temperature;
- unsupported/non-finite behavior.

Parity corpus includes, for both roles:

1. exact canonical observation and candidate ordering/mask;
2. evaluator input tensor bytes and state input conversion;
3. raw finite output diagnostic with declared tolerance only when necessary;
4. exact quantized score vector and next fixed-point recurrent state;
5. exact named RNG pre/post state;
6. exact selected/applied candidate;
7. exact `BehaviorDecisionCommitV1`, intention state and resulting command/
   event roots.

Raw evaluator tolerance can diagnose backend numerical correspondence but
cannot excuse a changed quantized score, tie, selected candidate, next state or
authoritative root. A raw difference that survives canonical conversion is
`NONDETERMINISTIC_RESULT`.

Runtime/training parity runs on Windows x86_64 and Linux x86_64 with declared
GPU evaluator capability for learned R4 gate, plus CPU/planner fallback. Worker
counts, batch splits and completion permutations must preserve applied result.

## Evaluation suites

### Lane quality suites

Each lane defines training, validation and held-out scenario sets with no
fixture overlap by exact scenario/content/provenance hash. Thresholds are
versioned in suite manifests and must include distribution as well as worst
case safety/constraint assertions.

Strategic measurements include:

- routine consistency and valid deviation;
- fatigue/sleep/rest success without unsafe/invalid place fabrication;
- social-contact/dialogue initiation precision;
- goal completion, expiry and replan;
- emergency suspend/resume correctness;
- personality/trait differentiation under shared weights;
- cadence/starvation behavior for Simulated/Active NPCs.

Tactical measurements include:

- valid affordance/navigation candidate rate;
- threat response and ally/support use;
- fight/flee/yield/help-call scenario outcomes;
- constraint/safety violation count (must remain zero at validator boundary);
- yielding lifecycle including ignored surrender and renewed hostility;
- emergency resolution and strategic handoff;
- Active-only cadence and multi-agent stability.

Quality metrics cannot replace exact schema, determinism, state, fallback or
safety results. A high reward with direct mutation, hidden facts or invalid
candidate is failure.

### Fault and adversarial suites

Both roles cover:

- malformed/oversized/non-finite/reordered/missing/extra outputs;
- stale observation/fact/route/state/intention bindings;
- model or evaluator absence/incompatibility/logical failure;
- recurrent/state/hash/commit-chain tamper;
- candidate-mask edge cases including zero optional candidates plus mandatory
  safe fallback;
- LLM suggestion, speech and prosody lateness/malformed provenance;
- GPU OOM/device loss as declared logical activation/evaluator fault;
- wall watchdog miss, which marks run nonconforming and must not choose a
  different authoritative result;
- save/load/restart and worker/completion permutations.

### Integrated R4 suite

`BEHAVIOR-R4-P1` exact scenario pair must run:

1. both learned roles using exact promoted bundles and compatible GPU evaluator;
2. both deterministic utility/HTN fallbacks with learned route unavailable at
   preflight.

The player-visible mandatory loop, owner/mutation boundaries and authored
consequences must complete in both. Diversity/efficiency may differ, but
fallback cannot omit routine, fatigue response, threat, fight/flee/yield,
dialogue and resume/replan stages required by the scenario. Learned run must
prove both roles actually produced applied decisions; loading model bytes or
shadow inference does not count.

## Artifact lifecycle

### Local training artifacts

The following stay outside Git and distributed game packages:

- raw/licensed/protected datasets and captures;
- intermediate checkpoints and candidate weights;
- optimizer, scheduler, replay buffer and distributed worker state;
- tensorboard/MLflow-like tracking stores, logs and generated videos;
- machine-local caches, environment images and credentials.

Repository may contain small engine-owned/CC0 schemas, deterministic fixtures,
golden vectors, manifests and scripts required to reproduce validation, subject
to ordinary license/provenance checks. A dataset reference in provenance does
not grant redistribution rights.

### Candidate publication

Candidate publication is:

```text
stage exact model + manifest + notices
  → validate bounds/hash/schema/operator/runtime compatibility
  → reopen exact staged bytes
  → run required local parity/fault suites
  → atomically publish immutable candidate revision
```

Failure quarantines candidate and preserves previous artifact/fallback. A
candidate revision never silently overwrites same ID/hash binding.

### Project activation

Project lock names exact strategic and tactical bundle revisions plus their
fallback profiles only after promotion. Activation validates complete closure,
evaluator capability policy and fallback before world creation. If optional
learned capability is unavailable, activation deterministically selects the
declared planner profile and records a diagnostic; it does not download a
model, enable network or scan an ambient model directory.

Changing active bundle during a live world requires a future explicit route
transition/state reset contract. V1 proposal does not hot-swap behavior bundles
mid-session. Save bound to exact learned route fails closed on incompatible
bundle unless an explicit migration/fallback transition has been accepted.

## Promotion and rollback

Bundle status is an artifact property for an exact revision, not organizational
approval or certification. Suggested lifecycle:

```text
TrainingCandidate → EvaluatedCandidate → R4Integrated → ShippedRevision
                                     ↘ Rejected
```

Promotion to `R4Integrated` requires all proposed checks and exact joint pair
identity. `ShippedRevision` additionally requires normal package/license and
affected target checks. A newer candidate never inherits results from previous
bytes.

Runtime rollback is only to an exact previously declared compatible bundle or
to deterministic fallback according to project/save policy. It does not
rewrite a save in place, reuse incompatible recurrent state or claim replay
compatibility. Production failure after activation follows SPEC-32 logical
fallback rules; wall time remains non-authoritative.

## Security, provenance and licensing

- All model/environment/manifest/provider output is untrusted and bounded
  before allocation/use.
- Manifest records upstream immutable revision, conversion tools, code/weights/
  data licenses, redistribution scope and notices.
- Unknown/incompatible redistribution terms reject distribution of the
  affected artifact; they do not remove planner fallback.
- Training/remote credentials and provider sessions never enter manifest,
  save, replay, log or package.
- Personal dialogue/audio data is not a behavior-training dataset by default.
  Any collection needs separate explicit consent/privacy/retention policy.
- Model pack cannot request filesystem/network/process capability at runtime.
- Runtime never executes trainer code or mutates model bytes.

## Stable failure semantics

| Failure | Required outcome |
|---|---|
| Environment/project/schema/probe mismatch | Reject training/evaluation run before first mutable episode. |
| Direct mutation/test-only success path detected | Invalidate run and artifact; no promotion. |
| Missing provenance/license/hash | Reject candidate publication/distribution; retain fallback. |
| Export contains stochastic/unsupported op or hidden state | Reject candidate before runtime parity. |
| Runtime/training quantized score/state/applied-decision mismatch | `NONDETERMINISTIC_RESULT`; reject exact bundle/backend pair. |
| Held-out or integrated threshold failure | Keep candidate unpromoted; previous bundle/fallback unchanged. |
| One role missing or not actually applied in R4 vertical | Track remains Proposed; no partial promotion. |
| Required Windows/Linux GPU lane not run | `NOT_RUN`; learned R4 gate remains open, game remains playable through fallback. |
| Runtime training request | Deny capability; weights and optimizer remain immutable/absent. |

## Proposed ProductCheck

All are `NOT_RUN(NO_PRODUCTION_CONSUMER)` for this docs-only proposal.

| ID | Scenario | Required result / fallback |
|---|---|---|
| `BEHAVIOR-TRAIN-P1` | Recreate both lane environments from exact manifests, export candidate bundles, run candidate ordering/mask/quantization/state/applied-decision parity, held-out lane suites and joint co-evaluation | Exact runtime/training applied decisions and state roots on declared corpus; provenance/license closure complete; both roles pass held-out and integrated thresholds. Failed role rejects pair; planner fallback remains. |
| `BEHAVIOR-SCHEMA-P1` | SPEC-32 schema/parity corpus replayed through trainer and runtime adapters | Candidate ordering/masks/quantization/state conversions byte-exact; malformed inputs rejected identically. |
| `BEHAVIOR-DETERMINISM-P1` | Exported pair on Windows/Linux GPU evaluators and worker/batch permutations | Exact applied strategic/tactical decisions, commits and roots; raw tolerance cannot hide changed canonical result. |
| `BEHAVIOR-FALLBACK-P1` | No compatible accelerator/model plus declared logical faults | Complete utility/HTN R4 loop runs without model/trainer/network and without behavior tick stall. |
| `BEHAVIOR-R4-P1` | Integrated learned pair and fallback pair over the same production vertical | Both learned bundles are actually applied together; fallback completes same mandatory loop; all state changes use production owners. |

SPEC-32 owns `BEHAVIOR-STATE-P1`, `BEHAVIOR-100NPC-P1` and
`BEHAVIOR-COMMS-P1`; all eight checks are joint promotion prerequisites.

## Promotion boundary

SPEC-33, SPEC-32 and ADR-050 may become `Accepted` only together with:

1. exact strategic and tactical `BehaviorPolicyBundleManifestV1` artifacts;
2. one production R4 vertical that applies both learned roles;
3. passing Windows/Linux learned GPU parity and complete planner fallback;
4. passing schema, determinism, state, fallback, integrated, 100-NPC,
   communications and training checks;
5. synchronized minimal public schemas, content/project lock, save/replay and
   routing/roadmap updates required by that consumer.

An isolated trainer, benchmark, exported model, shadow inference or one passing
policy is not sufficient under ADR-046.
