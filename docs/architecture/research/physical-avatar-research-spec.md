# Physical Avatar Research Specification

Status: Historical research context; not normative NextEngine architecture

Last updated: 2026-08-16

> Current authority is [SPEC-05](../05-physics-animation-and-motor-control.md),
> [SPEC-36](../36-functional-tissue-condition-and-injury.md),
> [SPEC-37](../37-character-embodiment-and-surface-deformation.md) and
> [ADR-075](../adr/075-product-grounded-functional-anatomy-and-character-embodiment.md).
> The OpenGothic/Bullet assumptions below are retained only as historical
> research provenance and cannot define NextEngine requirements, backends,
> phases, public types or support claims.

## 1. Purpose

This document defines a research program for physically simulated player and
NPC bodies in OpenGothic. The target is an active-ragdoll character whose
rendered skeleton is driven by physics and whose motor controller can execute
high-level commands such as standing, moving, striking, evading, crouching,
recovering, gripping, and releasing.

The document is a research specification, not an implementation commitment.
Its purpose is to make experiments reproducible, define evidence gates, and
prevent early prototypes from becoming an unbounded replacement of the current
NPC, animation, combat, and scripting systems.

This work is separate from `docs/npc-policy-model-implementation-plan.md`:

- The NPC policy model selects slow, high-level gameplay intents.
- The physical-avatar motor policy executes bounded body-level commands.
- An LLM may propose high-level goals but never controls joints, torques, hit
  resolution, or the physics step directly.

Dynamic severing, runtime cut geometry, detached body parts, disarming, and
damage-dependent crawling are specified separately in
`docs/physical-avatar-dynamic-dismemberment-spec.md`. That document is a
dependent extension of this research program. It may reuse evidence and
contracts established here, but it may not redefine the physical body,
render-pose authority, contact schema, limb-state mask, grip model, or
OpenGothic integration boundary.

## 2. Current Engine Baseline

OpenGothic already links Bullet and wraps it in `game/physics/`.

The current NPC physics representation is not an active ragdoll:

- `DynamicWorld::NpcBody` is a zero-mass `btRigidBody` with a capsule-like
  collision shape.
- NPC collision uses no contact response and manually validated translation.
- `MoveAlgo` derives normal movement from animation displacement.
- `Npc::tick` and gameplay states assume that animations exist and generate
  tags, movement, combat timing, sounds, and interaction state.

Therefore, a physical avatar cannot be introduced by only replacing the NPC
collision shape. It needs a separate articulated body, controller, render-pose
bridge, contact model, and compatibility boundary.

## 3. Research Goal

Determine whether OpenGothic can support physically authoritative humanoid and
monster avatars whose movement and combat are generated from current body
state, intent, contacts, inertia, and external forces instead of selecting and
playing fixed animation clips.

The first successful result is a standalone human-shaped avatar that can:

- remain upright on flat ground;
- recover from bounded external impulses;
- move toward a commanded planar velocity and heading;
- strike a dynamic target with a commanded limb, direction, and power;
- crouch, evade, fall, and attempt recovery;
- grip and release a weapon;
- run a trained policy in C++ with a bounded inference cost;
- reproduce the same evaluation scenario from a seed and recorded commands.

## 4. Non-Goals

The initial research does not aim to:

- replace all existing Gothic NPCs, animations, or combat logic;
- preserve quest, dialog, cutscene, mob-interaction, and save compatibility;
- support every Gothic skeleton or armor asset;
- generate final gore, severed-mesh, blood, or wound visuals;
- train from rendered pixels;
- use an LLM in the real-time motor loop;
- support arbitrary runtime mesh cutting;
- ship a production multiplayer or deterministic-networking solution;
- make physically exact biomechanical humans;
- train a single universal policy for humans and all monsters.

## 5. Design Principles

### 5.1 Physics Is Authoritative

The Bullet body owns the actual world-space pose. The render skeleton follows
the simulated bodies. A reference pose or neural target is an input to joint
motors, not a second authoritative pose that is blended back into physics.

### 5.2 Commands Describe Outcomes

Higher layers request outcomes such as `move`, `strike`, or `evade`. They do not
select animation frames or set global bone transforms.

### 5.3 Control Is Hierarchical

Different time scales remain separate:

- High-level AI or LLM: goals and social/gameplay intent, approximately
  `0.2-2 Hz`.
- Skill selector or trajectory planner: body action commands, approximately
  `5-20 Hz`.
- Neural motor policy: joint targets, approximately `30-60 Hz`.
- PD motors and Bullet simulation: fixed step, provisionally `120-240 Hz`.

The exact rates are research variables, not final decisions.

### 5.4 Game Rules Remain Explicit

Physics may provide contact point, relative velocity, impulse, and effective
mass. Engine code remains responsible for damage rules, friendly-fire rules,
invulnerability, weapon ownership, limb state, and action authorization.

### 5.5 Every Result Must Be Measurable

Video is diagnostic evidence, not the acceptance criterion. Every experiment
must produce structured metrics, configuration, seed, policy identity, and a
replayable command stream.

## 6. Research Hypotheses

### H1: Bullet Can Support a Stable Active Humanoid

A constrained articulated body using OpenGothic's Bullet build can remain
stable under the required fixed step, joint limits, collision settings, and PD
motor forces.

Failure evidence:

- persistent constraint explosion or unacceptable penetration;
- stability requires solver settings too expensive for several actors;
- equivalent runs diverge too quickly for useful replay and debugging.

### H2: A Small Policy Can Run Inside the Game Budget

A policy using proprioception and compact target commands can run through ONNX
Runtime in C++ without blocking the game loop.

Initial gate:

- `p95` inference below `1 ms` per avatar on the reference CPU;
- fixed-size allocation-free input and output path after initialization;
- numerically close Python and C++ outputs for recorded observations.

The threshold is provisional and must be revised after a multi-avatar budget is
measured.

### H3: Training Can Transfer to Bullet

A controller trained in MuJoCo, Isaac Lab, or another high-throughput simulator
can be transferred to the OpenGothic Bullet body by matching morphology,
actuation, observations, control frequency, and randomized physical parameters.

Fallback:

- train or fine-tune against a headless Bullet environment using the same body
  and motor implementation as the game.

### H4: Strikes Can Be Goal-Conditioned

One policy can generate multiple hand strikes from a compact command containing
limb, target, approach direction, power, and urgency, without selecting a fixed
clip for each direction.

The same hypothesis applies later to kicks, weapon swings, and monster attacks.

### H5: Physical Contact Can Replace Animation Hit Windows

Damage candidates can be generated from physical contact using relative
velocity, impulse, effective mass, weapon properties, and contact direction
without repeated-contact exploits or unstable damage spikes.

### H6: Degraded Bodies Can Retain Bounded Agency

A controller can receive a capability or limb-state mask and produce degraded
behavior such as limping, crawling, one-handed movement, or inability to act.

This is not assumed to emerge from a policy trained only on an intact body.
Damage states require explicit curriculum, data, or specialized policies.

### H7: Weapons Can Transition Between Held and Dynamic State

A weapon can be attached to a hand through a stable grip representation, used
for physical contact, and released into a dynamic rigid body while preserving
linear and angular momentum.

### H8: Embodiment-Specific Policies Are Practical

Humans and monsters can expose a shared intent vocabulary while using separate
body definitions, observations, action dimensions, rewards, and motor policies.

## 7. Proposed System Boundary

```text
Daedalus / player input / NPC policy / optional LLM
                     |
                     v
             PhysicalAvatarIntent
                     |
                     v
          skill planner / action gate
                     |
                     v
        neural or procedural motor controller
                     |
                     v
             joint motor targets
                     |
                     v
            Bullet articulated body
               |              |
               v              v
        render skeleton   contact events
                              |
                              v
                   explicit gameplay rules
```

The initial prototype must use a separate `PhysicalAvatar` or equivalent lab
type. It must not silently change `Npc` behavior.

## 8. First Human Body Model

The first body should be deliberately simpler than a production human rig.

Provisional rigid bodies:

- pelvis;
- lower torso and chest;
- head;
- left and right upper arm, forearm, and hand;
- left and right thigh, shin, and foot.

Provisional joint groups:

- spine and neck;
- shoulders and elbows;
- optional limited wrists;
- hips, knees, and ankles.

Hands do not require finger simulation in the first version. A grip is a
gameplay/controller state plus a physics attachment at the hand.

The body definition must be data-driven and include:

- segment names and parent relationships;
- shape type and dimensions;
- mass and center of mass;
- local joint frames;
- angular limits;
- motor torque limits;
- PD gains;
- self-collision allow/deny pairs;
- render-bone mapping;
- damage-region mapping.

Two Bullet representations must be compared before selection:

- rigid bodies plus constraints, including `btGeneric6DofSpring2Constraint`,
  cone-twist, and hinge constraints;
- `btMultiBody` and its joint motors.

The comparison must measure stability, control quality, contact reporting,
integration complexity, and cost across multiple avatars.

## 9. Motor Policy Contract

### 9.1 Observation

The first observation should contain no rendered pixels and no unbounded world
state. Candidate fields include:

- root height, orientation, linear velocity, and angular velocity;
- gravity vector in root-local space;
- normalized joint positions and velocities;
- foot, hand, and selected body contact flags;
- recent motor command or previous policy action;
- desired planar velocity and facing direction;
- current skill command and phase/time budget where applicable;
- local target position and target velocity for reaching or striking;
- requested strike approach direction and power;
- limb/capability mask;
- weapon presence, grip state, and compact weapon properties.

Observation ordering, units, normalization, and version must be explicit and
shared by Python and C++.

### 9.2 Action

The initial action should be normalized target joint positions or residuals
consumed by bounded PD motors. Direct unconstrained global transforms are not
allowed.

Torque output may be researched later, but it is not the first deployment
choice because it makes transfer and safety harder.

### 9.3 Intent Vocabulary

The initial common command schema should cover:

- `stand`;
- `move(targetVelocity, targetFacing)`;
- `reach(limb, target)`;
- `strike(limb, target, approachDirection, power, urgency)`;
- `evade(direction, distance)`;
- `crouch(height)`;
- `recover(preferredFacing)`;
- `grip(hand, item)`;
- `release(hand)`.

Monsters may reject unsupported commands and add embodiment-specific commands.

## 10. Combat Contact Contract

Every collision-capable body part and weapon must have stable ownership and
damage metadata.

A contact event should expose at least:

- attacker and defender identity;
- source body part or weapon;
- target body region;
- world-space point and normal;
- relative normal and tangential velocity;
- solver impulse or another validated impact estimate;
- current attack command identifier;
- contact timestamp and continuity identifier.

The damage layer must prevent:

- repeated damage from one persistent contact;
- damage caused by low-speed resting contact;
- self-hit and same-owner weapon hit;
- jitter-driven high-frequency damage;
- damage from limbs that are disabled or not participating in an authorized
  action, unless the design explicitly allows accidental impacts;
- unbounded impulse spikes caused by penetration recovery.

Damage formulas remain outside the learned policy.

## 11. Research Environments and Curriculum

### R0: Reproducible Lab Baseline

Research:

- local MuJoCo setup and stock humanoid tasks;
- experiment configuration, seeds, metrics, replay, and policy export;
- reference machine performance.

Exit gate:

- one command launches a deterministic evaluation and produces JSON metrics,
  replay data, video, and model identity.

### R1: Passive and Active Bullet Body

Research:

- body generation from a versioned definition;
- constraints versus `btMultiBody`;
- passive ragdoll behavior;
- PD pose tracking and self-collision filtering;
- fixed-step and solver settings.

Exit gate:

- the body can fall without exploding and track a standing reference pose for
  30 seconds in a controlled scene.

### R2: Balance and Recovery

Research:

- standing policy;
- randomized impulses, friction, mass, and initial pose;
- recovery steps and fall classification;
- scripted PD baseline versus learned policy.

Exit gate:

- success thresholds are defined over a fixed perturbation suite, not selected
  showcase runs.

### R3: Locomotion

Research:

- commanded planar velocity and facing;
- starts, stops, turns, slopes, steps, and low obstacles;
- energy and foot-slip penalties;
- transfer from training simulator to Bullet.

Exit gate:

- command-tracking and fall-rate targets pass over the evaluation suite in both
  the training simulator and Bullet.

### R4: Reaching and Unarmed Strikes

Research:

- target-conditioned reaching;
- hand strikes from top, bottom, left, and right approach directions;
- power control;
- interruption and continuation from noncanonical poses;
- contact-derived damage candidates.

Exit gate:

- a single command interface reaches and strikes randomized targets with
  bounded accuracy, impulse, and fall rate.

### R5: Kicks and Defensive Actions

Research:

- support-foot selection and one-leg balance;
- kicks at different heights and approach directions;
- crouch, lean, sidestep, and jump-back/step-back evasion;
- reactions to incoming physical contact.

### R6: Weapons

Research:

- stable grip attachment;
- weapon inertia in the controller observation;
- physical weapon contact;
- release, disarm, pickup, and momentum preservation;
- one-handed and two-handed command variants.

Exit gate:

- the selected grip representation survives the fixed manipulation and strike
  suite, releases exactly once under authorized conditions, preserves bounded
  linear and angular momentum, and exposes stable hand/capability ownership to
  the controller and gameplay validator.

### R7: Body Damage and Crawling

Research:

- limb capability masks;
- disabled joints and detached constraints;
- specialized versus unified damaged-body policies;
- crawling and degraded locomotion;
- save representation for damage state, without changing the game save format
  during the research phase.

Runtime cut geometry and detached-part rendering remain governed by
`docs/physical-avatar-dynamic-dismemberment-spec.md`. Geometry-only experiments
may begin after R1 selects a body and render-bone mapping, but physical
severing and degraded locomotion may not claim integration readiness until the
R7 exit gate here and the dependent D7 exit gate both pass.

Exit gate:

- at least one declared damaged-body configuration passes a fixed degraded
  locomotion or crawling suite, every unsupported capability mask routes to an
  explicit passive or disabled fallback, and the intact-body policy is never
  invoked for an unsupported mask.

### R8: Monster Embodiments

Research:

- one nonhuman skeleton selected for a materially different topology;
- shared intent schema versus embodiment-specific extensions;
- separate policy and body-definition pipeline.

### R9: OpenGothic Gameplay Integration

Research:

- experimental actor lifecycle inside `World`;
- render skeleton synchronization;
- collision filters against landscape, items, NPCs, and projectiles;
- camera and player-input adapter;
- bounded coexistence with normal NPCs;
- feature flags and deterministic fallback.

This phase does not begin until the controller passes the standalone Bullet
gates.

Exit gate:

- one allowlisted physical avatar can be created, simulated, rendered, failed
  closed, and destroyed in an isolated OpenGothic scene while normal NPC
  behavior remains unchanged with the feature disabled.

## 12. Metrics

Every evaluation suite should report a stable subset of these metrics:

- upright survival time;
- fall and recovery rate;
- root height and orientation error;
- commanded velocity and facing error;
- foot slip and unintended contact count;
- joint-limit violations;
- peak and mean motor torque;
- energy proxy;
- target reach error;
- strike approach-direction error;
- impact impulse distribution;
- false and repeated damage candidate count;
- grip break and unintended release rate;
- policy inference `p50`, `p95`, and maximum latency;
- Bullet physics step cost by avatar count;
- replay divergence from a recorded baseline;
- Python versus ONNX output error;
- training seed success distribution.

No milestone may be accepted from one seed or one successful recording.

## 13. Provisional Toolchain

### Local Exploration

- MuJoCo for quick articulated-body and controller experiments.
- Python and PyTorch for policy development.
- PPO as the first learned-control baseline.
- Blender for body, bone, collision-shape, and detached-part preparation.

### Scaled Training

- Isaac Lab on Linux with an NVIDIA GPU when parallel training becomes the
  bottleneck.
- MimicKit as the current reference implementation for DeepMimic, AMP, ASE,
  and related motion-imitation methods.
- Motion data only after its source, redistribution terms, and conversion path
  are recorded.

### Deployment and Evaluation

- ONNX as the policy interchange format.
- ONNX Runtime C++ for in-process inference.
- OpenGothic's Bullet build for final physics evaluation.
- MLflow or an equivalent tracker for parameters, metrics, artifacts, and run
  identity.

DeepMimic remains useful as a Bullet-oriented reference implementation but is
not the default new training framework because its upstream repository is
deprecated in favor of MimicKit.

## 14. LLM-First Research Workflow

LLM-first means that the research environment exposes explicit, bounded, and
machine-verifiable operations. It does not mean that generated code or an LLM's
visual judgment is accepted without evaluation.

### 14.1 Required Experiment Artifacts

Every experiment must produce:

- hypothesis identifier;
- body, task, reward, and policy configuration hashes;
- source commit identifier;
- dependency and simulator versions;
- random seeds;
- training and evaluation metrics;
- policy checkpoint and optional ONNX artifact;
- replayable command and disturbance stream;
- failure classifications;
- a short decision: retain, reject, or investigate.

### 14.2 LLM-Callable Operations

The lab should eventually expose commands equivalent to:

```text
lab build
lab validate-config <config>
lab train <config>
lab evaluate <policy> <suite>
lab sweep <sweep-config>
lab compare <run-a> <run-b>
lab render-replay <run>
lab export-onnx <policy>
lab verify-onnx <policy> <recorded-observations>
lab evaluate-bullet <policy> <suite>
```

Each command must have a noninteractive mode, bounded runtime where practical,
nonzero exit codes on failure, and JSON output suitable for automated analysis.

### 14.3 LLM Guardrails

- Change one research variable per hypothesis unless an interaction study is
  explicitly declared.
- Do not tune from showcase video alone.
- Do not modify evaluation suites to make a new policy pass.
- Keep training and holdout disturbances separate.
- Record failed and unstable runs, not only successful checkpoints.
- Require tests for observation ordering, normalization, units, and ONNX
  parity.
- Require independent evaluation before promoting a policy to OpenGothic.
- Never allow generated code to call an external model from the physics tick.

## 15. Proposed Repository Layout

The exact layout is not approved by this draft, but the intended ownership is:

```text
docs/
  physical-avatar-research-spec.md
  physical-avatar-decisions.md

tools/physical_avatar_lab/
  configs/
  environments/
  learning/
  evaluation/
  export/
  tests/

game/physics/avatar/
  body definition and Bullet implementation
  motor and contact interfaces
  ONNX inference adapter

ai-data/local/physical-avatar/
  runs/
  policies/
  replays/
  motion-data/
```

Generated runs, proprietary game-derived data, local motion datasets, and model
weights belong under ignored local storage and must not be committed by default.

## 16. OpenGothic Integration Constraints

- Experimental physical avatars are disabled by default.
- Normal `Npc`, `MoveAlgo`, `AiQueue`, animation, dialog, and quest behavior must
  remain unchanged while the experiment is disabled.
- The first integration uses a dedicated test actor or explicitly allowlisted
  NPC, never a global replacement.
- The policy may not directly mutate health, quest state, inventory, Daedalus
  variables, or save data.
- Contact events must pass through an engine validator before becoming damage.
- A missing, invalid, stale, or slow policy must fail closed to a safe physical
  state or disable the experimental actor.
- Physics stepping and policy inference must not wait for an LLM or network
  service.
- Game assets must not be copied into public test fixtures.

## 17. Main Risks and Decision Forks

### Cross-Engine Physics Mismatch

MuJoCo or PhysX training may not transfer to Bullet. The decision fork is:

- retain external high-throughput training with domain randomization;
- fine-tune in a headless Bullet environment;
- train entirely in Bullet if transfer remains unreliable.

### Bullet Articulation Choice

Constraint ragdolls may integrate more easily with existing code, while
`btMultiBody` may provide better articulated dynamics. R1 must select based on
evidence rather than familiarity.

### Reward Exploitation

Pure task rewards may produce effective but visibly nonhuman attacks. The
decision fork is:

- accept procedural/physical behavior;
- add motion priors or imitation data;
- combine a reference-motion prior with target-conditioned residual control.

### Asset and Skeleton Mismatch

Gothic body and armor assets may not support clean per-limb rendering,
detachment, or the required joint topology. Visual damage may require prepared
body variants and caps even when physics detachment works.

### Runtime Cost

Several active ragdolls may exceed physics or inference budgets. Level-of-detail
control, sleep states, reduced-frequency policies, or distance-based fallback
may be required.

### Gameplay Compatibility

Existing animation tags drive combat windows, sounds, interactions, and states.
Physical avatars need explicit replacements before they can participate in
normal Gothic gameplay.

## 18. Research Success and Stop Conditions

The research is successful when evidence supports all of the following:

- a stable Bullet active humanoid exists;
- a compact policy performs balance, locomotion, and at least one
  target-conditioned strike family;
- ONNX inference meets the measured CPU budget;
- contact-derived hit candidates are stable and resistant to simple exploits;
- the controller runs in an isolated OpenGothic scene without changing normal
  NPC behavior;
- the experiment pipeline is reproducible and usable by an LLM agent without
  manual interpretation of every run.

The direction should be reconsidered or narrowed when:

- Bullet cannot meet stability or multi-avatar cost requirements;
- cross-engine transfer fails and Bullet training throughput is impractical;
- acceptable motion requires a volume of authored motion data outside project
  constraints;
- the required replacement of Gothic gameplay contracts is larger than the
  desired project scope;
- combat remains dominated by solver artifacts or reward exploits after
  bounded mitigation experiments.

## 19. Initial Experiment Backlog

1. Record the exact OpenGothic Bullet version, solver configuration, units, and
   fixed-step behavior.
2. Reproduce a stock MuJoCo humanoid stand or walk evaluation.
3. Define the versioned first-human body schema.
4. Build the same passive body in a standalone Bullet executable.
5. Compare constraint ragdoll and `btMultiBody` stability.
6. Implement and benchmark a scripted PD standing baseline.
7. Train a standing policy under randomized impulses.
8. Export the policy to ONNX and verify Python/C++ parity.
9. Evaluate the policy against the Bullet body and quantify transfer error.
10. Decide whether locomotion training proceeds externally, in Bullet, or with
    external training plus Bullet fine-tuning.

Combat, dismemberment, crawling, and monsters remain blocked until this backlog
produces a stable body and a validated policy deployment path.

The dependency gates and permitted preparatory work for dismemberment are
defined in `docs/physical-avatar-dynamic-dismemberment-spec.md`.

## 20. Open Questions

- What is the first reference machine and acceptable actor count?
- Which Gothic human skeleton and body asset should be used for the first render
  mapping?
- Should the first body use metric units internally and convert at the
  OpenGothic boundary, or follow current engine units throughout?
- Which Bullet articulation representation wins the R1 comparison?
- Is MuJoCo-to-Bullet transfer sufficient, or is a Bullet training environment
  mandatory?
- Which motion datasets have acceptable licenses and retargeting quality?
- Should strikes be learned end-to-end, generated by trajectory optimization,
  or use a learned residual over procedural targets?
- What deterministic replay tolerance is useful across platforms?
- What physical and gameplay conditions break a weapon grip?
- Which damaged-body states deserve specialized policies?
- Which monster topology provides the most informative second embodiment?

These questions are intentionally unresolved. Each must be answered by a
bounded experiment or an explicit product decision before its dependent phase
is implemented.

## 21. Primary Research References

- MuJoCo: https://github.com/google-deepmind/mujoco
- MuJoCo Playground: https://github.com/google-deepmind/mujoco_playground
- Isaac Lab: https://github.com/isaac-sim/IsaacLab
- MimicKit: https://github.com/xbpeng/MimicKit
- DeepMimic reference implementation: https://github.com/xbpeng/DeepMimic
- ONNX Runtime C++: https://onnxruntime.ai/docs/get-started/with-cpp.html
- MLflow Tracking: https://mlflow.org/docs/latest/ml/tracking/

Project status, supported backends, licenses, model availability, and setup
instructions must be revalidated from primary sources before a dependency is
adopted. Research papers and demonstrations alone are not sufficient evidence
that a maintained, redistributable implementation exists.
