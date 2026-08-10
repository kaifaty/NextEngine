# SPEC-33: Behavior-policy training, evaluation and deployment lifecycle

| Поле | Значение |
|---|---|
| ID | SPEC-33 |
| Статус | Proposed |
| Lifecycle | Optional consumer-driven R8 quality track |
| Версия | 0.3 |
| Последняя проверка | 2026-08-09 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-06](06-ai-agents-perception-and-memory.md), [SPEC-11](11-security-licensing-and-governance.md), [SPEC-12](12-vertical-slice-conformance.md), [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-24](24-content-catalog-bundle-and-neutral-asset-schemas.md), [SPEC-32](32-npc-cognition-intention-lifecycle-and-deterministic-behavior-inference.md), [SPEC-34](34-model-training-environments-trajectories-and-consolidation-lifecycle.md), [ADR-016](adr/016-compositional-gameplay-budgets.md), [ADR-030](adr/030-product-first-development-and-lightweight-validation.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-050](adr/050-hierarchical-npc-cognition-and-learned-behavior-policy-boundary.md), [ADR-053](adr/053-engine-native-model-training-and-immutable-artifact-boundary.md), [ADR-054](adr/054-bounded-strategic-adaptation-and-two-tier-sleep.md), [ADR-057](adr/057-hierarchical-learnable-motor-system-and-policy-family-architecture.md) |
| Заменяет | SPEC-33 0.1; first-party reference profiles, CTDE/self-play provenance and offline child-bundle retention |

## Статус и scope

SPEC-33 описывает proposed runtime-algorithm-neutral lifecycle для offline
training, evaluation, export, promotion и deployment двух behavior-policy
bundles. Документ фиксирует first-party reference training profiles and
comparators, но они не становятся public model enums или gameplay contracts.
Trainer, simulator и inference runtime остаются replaceable adapters; этот
changeset не добавляет code/contracts.

До optional R8 production consumer все checks здесь имеют
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
- Model output remains untrusted at runtime and always has the deterministic
  Utility + bounded GOAP/tactical fallback from ADR-056 and SPEC-32.
- Strategic and Tactical bundles may be promoted independently when their
  consumers and checks are independent. Joint co-evaluation is required only
  for a shipped profile that activates both roles together.

## Two training lanes

### Strategic lane

Reference training sequence is authored Utility + bounded GOAP teacher demonstrations →
behavior cloning → bounded recurrent RL. `HopeInspiredStrategicV1` is evaluated
as a non-public reference profile with fully externalized multi-timescale
state under ADR-054. A GRU using the same observation/candidate/state/resource
envelope is the mandatory learned comparator; deterministic Utility + bounded GOAP is the
mandatory gameplay baseline/fallback.

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

The reference actor uses permutation-stable set encoders for entities/items,
cross-attention between actor/context and canonical candidates, a bounded GRU
state and one masked scalar score for every composite candidate. These are a
first-party trainer/export profile, not public tensor or architecture types.

Reference sequence is behavior cloning → recurrent PPO/IPPO → gated
MAPPO/CTDE plus opponent-pool self-play. IPPO must establish the decentralized
actor baseline before MAPPO is accepted. A centralized/privileged critic is
training-only: actor inputs, actor recurrent state, normalization statistics
and export graph contain only engine-visible SPEC-32 semantic facts. Critic
fact leakage is a hard failure.

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

There is no latent learned communication channel in v1. Multi-agent
coordination uses only observable engine facts and closed help/yield/speech-act
candidates. Opponent snapshots, pool selection, curriculum revision, seed map,
match identity and self-play lineage are required run provenance.

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

SPEC-34 owns the common reset/step/trajectory/reward/dataset/run/export data
plane. This section specializes that contract for behavior lanes; a private
trainer-specific Gym/PyTorch record cannot replace the engine-owned manifest.

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

Vectorized environments use stable episode/slot ordering and per-slot named
RNG streams. Completion order cannot choose trajectory order. Domain
`terminated` and declared-limit `truncated` remain distinct in every record.

## Algorithm-neutral lane interface

The reference profiles above use imitation/recurrent PPO and optionally gated
MAPPO/self-play. PPO, offline RL, distillation and other algorithms MAY replace
them when the same public boundary and evaluation gates pass. Normative trainer
boundary is:

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
| Fallback | exact deterministic Utility + bounded GOAP/tactical fallback profile ID/hash; model cannot choose or rewrite it |

Shared foundation ancestry is recorded as provenance/compatibility. Strategic
and tactical manifests remain separate and may have different model formats,
state widths, cadence and resource envelope. Per-NPC weights, mutable adapter
files, hidden tokenizer/session, credentials, optimizer, checkpoint or dataset
are forbidden.

For MAPPO/self-play candidates, training provenance additionally binds actor
semantic-input schema, privileged critic schema, an explicit no-leak audit
result, opponent snapshot hashes, pool/matchmaking policy, curriculum revision,
match seed identities and self-play lineage root.

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
GPU evaluator capability for the optional learned route, plus CPU/planner fallback. Worker
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
- composite item/target/mode candidate coverage and pruning loss, including
  mandatory `HoldSafe` and retained teacher-action coverage;
- threat response and ally/support use;
- fight/flee/yield/help-call scenario outcomes;
- constraint/safety violation count (must remain zero at validator boundary);
- yielding lifecycle including ignored surrender and renewed hostility;
- emergency resolution and strategic handoff;
- Active-only cadence and multi-agent stability.

Every statistical quality claim uses a pre-registered SPEC-34 manifest with a
complete multi-seed set, held-out scenarios, deployment-matching evaluation
mode, episode/sample budget, confidence/effect-size method and practical
threshold. A single/best seed, peak checkpoint or training return cannot
promote a bundle.

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

### Optional integrated R8 suite

`BEHAVIOR-R8-P1` exact scenario pair must run when a profile activates both roles:

1. both learned roles using exact promoted bundles and compatible GPU evaluator;
2. deterministic Utility + bounded GOAP/tactical fallbacks with learned route unavailable at
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

### Offline consolidation and child bundles

Offline Sleep/Dreaming consumes one immutable parent bundle and immutable
trajectory/dataset closure and produces a new immutable child candidate. Its
SPEC-34 consolidation manifest binds parent, corpus, seed/config/tool,
retention/new-task suites and child export. It never mutates active world,
project, save, parent weights or runtime policy state.

The child must pass catastrophic-forgetting retention and the complete joint
Strategic/Tactical suite before publication. Failure retains the parent and
project lock exactly. V1 activates a successful child only through an explicit
project-lock change and new session; no live bundle hot-swap exists.

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
TrainingCandidate → EvaluatedCandidate → OptionalIntegrated → ShippedRevision
                                     ↘ Rejected
```

Promotion to `OptionalIntegrated` requires applicable proposed checks and exact
joint pair identity only when both roles activate together. `ShippedRevision`
additionally requires normal package/license and
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
| Declared role missing or not actually applied in its optional consumer | That role remains unpromoted; other independently evaluated role and deterministic fallback are unchanged. |
| Required Windows/Linux GPU lane not run | `NOT_RUN`; optional learned route remains unpromoted and R4/v1 are unaffected. |
| Runtime training request | Deny capability; weights and optimizer remain immutable/absent. |

## Proposed ProductCheck

All are `NOT_RUN(NO_PRODUCTION_CONSUMER)` for this docs-only proposal.

| ID | Scenario | Required result / fallback |
|---|---|---|
| `BEHAVIOR-TRAIN-P1` | Recreate the affected lane environment from exact SPEC-34 manifests; run Strategic Utility+GOAP→BC→RL Hope-vs-GRU or Tactical set/cross-attention+GRU BC→PPO/IPPO→gated MAPPO/self-play; export bundle and run candidate/state/applied-decision parity plus held-out multi-seed lane suite | Exact runtime/training canonical decisions and state roots on declared corpus; complete seed/opponent/curriculum/provenance closure; no critic leakage/latent channel. Failed role remains unpromoted; deterministic fallback remains. |
| `BEHAVIOR-SCHEMA-P1` | SPEC-32 schema/parity corpus replayed through trainer and runtime adapters | Candidate ordering/masks/quantization/state conversions byte-exact; malformed inputs rejected identically. |
| `BEHAVIOR-DETERMINISM-P1` | Exported pair on Windows/Linux GPU evaluators and worker/batch permutations | Exact applied strategic/tactical decisions, commits and roots; raw tolerance cannot hide changed canonical result. |
| `BEHAVIOR-FALLBACK-P1` | No compatible accelerator/model plus declared logical faults | Complete deterministic Strategic Agent loop runs without model/trainer/network and without behavior tick stall. |
| `BEHAVIOR-R8-P1` | Optional integrated learned pair and fallback over the same production scenario | When a profile declares both roles, both bundles are actually applied together; fallback completes the same gameplay outcomes through production owners. |

SPEC-32 owns deterministic `STRATEGIC-*` R4 checks; SPEC-34 owns common
data-plane, mirror, export, statistics, consolidation and data-governance
checks. Only checks applicable to the exact optional role/profile are promotion
prerequisites.

## Promotion boundary

SPEC-33 may become `Accepted` with an optional R8 production consumer when:

1. every activated role has an exact immutable behavior-policy bundle;
2. Windows/Linux evaluator parity and deterministic fallback pass for that role;
3. applicable schema, state, fault, training and SPEC-34 data-plane/export/
   statistical/governance checks pass;
4. joint co-evaluation passes when the shipped profile activates both roles;
5. minimal public schemas, project lock, save/replay, routing and roadmap are
   updated with that consumer under ADR-046.

SPEC-32 and ADR-056 remain independent Accepted/Proposed authorities for the
deterministic R4 path and never wait for this promotion.

An isolated trainer, benchmark, exported model, shadow inference or one passing
policy is not sufficient under ADR-046.
