# Shared Embodiment Control
## A Player-Avatar Control Architecture for Physics-Driven Neural Characters

**Document type:** architecture and research paper
**Status:** working foundation for ADRs, specifications, prototypes, and user studies
**Context date:** August 2026
**Target:** custom Rust game engine with physics-driven articulated characters and learned motor control
**Related documents:**
- `physical_world_layer_architecture_for_specs.md`
- `arcane_world_layer_magic_architecture_paper.md`
- `vegetation_physics_destructible_trees_research.md`
- `continuum_physics_water_mud_offroad_research_brief.md`
- Motor System / BodySchema research documents

---

# Abstract

This paper proposes a player-avatar control architecture for a game engine in which characters are not primarily driven by animation clips, but by learned and physics-constrained motor systems. Non-player characters may use strategic agents, tactical controllers, skill policies, and low-level motor policies to act autonomously. The player avatar presents a different problem: the player must retain authorship of every important decision while controlling a body with dozens or hundreds of physical degrees of freedom through a low-dimensional device such as a gamepad, keyboard, mouse, touch interface, or motion controller.

Conventional action games solve this mismatch by mapping buttons to pre-authored actions. This is responsive and readable, but it hides most of the expressive capacity of a physically embodied neural character. The opposite solution—direct control of limbs and joints—is too difficult for general play. A fully autonomous neural controller may generate excellent movement, but risks taking agency away from the player.

The proposed solution is **Shared Embodiment Control**:

> **The player controls task-space intent, targets, timing, effort, direction, commitment, and allowed action families. The learned motor system controls redundant joint coordination, balance, contacts, foot placement, biomechanical execution, and recovery.**

The central interface between the player and the body is a semantic `SkillContract`. A contract does not name an animation clip. It describes the physical result the player wants: strike this region along this line, protect this spatial sector, put this hand at this target, pull this object toward the body, move with this velocity, or maintain this weapon tip near this trajectory. The body then solves the high-dimensional motor problem under physical, anatomical, stylistic, and skill constraints.

A dedicated `AgencyArbiter` limits autonomy. The body may decide how to place the feet, but not whom to attack. It may use a free arm for balance, but not consume an item. It may convert an impossible movement into the closest safe execution, but must not silently replace a requested parry with a dodge. This separation allows neural motion completion without surrendering player authorship.

The architecture supports progressive specificity: accessible controls can specify broad intent, standard controls can specify continuous attack and guard directions, and expert controls can directly manipulate hands, weapons, or end-effectors. All layers compile into the same contract system and use the same motor controller.

The result is intended to support not only combat, but locomotion, climbing, grappling, manipulation, tools, vehicles, physical magic, injuries, prosthetics, and non-humanoid morphologies.

---

# 1. Motivation

The engine's character architecture already separates:

```text
Strategic Agent
→ Tactical Controller
→ Skill / Motion Policy
→ Low-level Motor Policy
→ Joint Controllers / Actuators
→ BodySchema
→ Physics Engine
```

For NPCs, this hierarchy is natural:

- the strategic layer selects goals;
- the tactical layer selects maneuvers;
- the skill layer selects or synthesizes movement;
- the motor layer controls the physical body.

The player avatar cannot simply reuse the NPC hierarchy unchanged.

A player does not want an autonomous tactical agent to decide:

- which enemy to attack;
- when to dodge;
- whether to consume a potion;
- whether to continue a combo;
- which spell to cast;
- whether to grapple;
- whether to retreat.

At the same time, the player cannot manually control:

- pelvis pose;
- center of mass;
- each foot;
- each knee;
- each shoulder;
- every finger;
- weapon inertia;
- support contacts;
- balance recovery.

The player has perhaps four to six comfortable continuous control dimensions. The body may have fifty to several hundred relevant degrees of freedom.

The problem is therefore:

> How do we preserve high player agency while using a learned controller to expand low-dimensional input into physically coordinated full-body motion?

---

# 2. Why conventional action control is insufficient

A conventional action game often uses:

```text
button
→ action state
→ animation montage
→ root motion
→ hit window
```

Examples:

```text
LightAttack
HeavyAttack
Block
Dodge
SpecialAttack
```

This gives:

- responsiveness;
- predictability;
- animation readability;
- straightforward networking;
- manageable content production.

But it also imposes limitations:

- actions exist only if authored;
- physical context has limited influence;
- target geometry is often approximate;
- transitions are state-machine edges;
- a weapon's mass is often cosmetic;
- contact is frequently resolved through hitboxes rather than mechanics;
- skill progression is expressed by animation-set replacement;
- the player cannot continuously shape an action.

A physics-driven neural body can do much more:

- adjust footwork to terrain;
- redirect a strike after partial contact;
- recover from unusual collisions;
- use weapon inertia;
- strike from arbitrary poses;
- adapt to injuries;
- coordinate both hands around real object geometry;
- perform new transitions.

If the input remains only:

```text
press X to play attack 03
```

most of that capability is invisible to the player.

---

# 3. Why direct limb control is insufficient

The other extreme is direct puppeteering.

For example:

```text
left stick  → left leg
right stick → right leg
triggers    → hands
buttons     → joints
```

This can create expressive physical comedy or specialized simulation, but has serious problems:

- high cognitive load;
- weak camera control;
- poor combat readability;
- long learning time;
- difficulty with simultaneous locomotion and manipulation;
- accessibility issues;
- poor transfer across morphologies;
- low reliability in fast situations.

Even expert players generally should not need to manage every support contact.

The body should know how to stand.

---

# 4. Why full neural autonomy is insufficient

A neural controller can choose a plausible whole-body action from a high-level goal.

For NPCs this is desirable.

For the avatar, it risks agency loss:

```text
player requests block
model dodges instead

player aims at arm
model strikes torso

player cancels
model completes attack

player approaches object
model grabs it automatically
```

These may be tactically sensible, but they feel like the game is playing itself.

The avatar therefore requires a stricter autonomy contract than NPCs.

---

# 5. Central thesis

The proposed control law is:

```text
Player:
    chooses intent
    chooses target
    chooses action family
    chooses timing
    chooses direction
    chooses effort
    chooses commitment
    chooses risk

Motor System:
    chooses joint coordination
    chooses foot placement
    chooses balance strategy
    chooses support contacts
    chooses biomechanical realization
    chooses micro-corrections
    chooses recovery motion
```

This can be summarized as:

> **The player owns decisions; the body owns coordination.**

---

# 6. Terminology

## 6.1. Intent

A desired semantic outcome, such as:

- move left;
- reach the handle;
- strike the opponent's right shoulder;
- guard the upper-left sector;
- pull the object;
- lower the body;
- jump across the gap.

---

## 6.2. Task space

A low-dimensional space defined by meaningful physical variables rather than joint angles.

Examples:

- hand position;
- weapon-tip position;
- desired velocity;
- gaze direction;
- protected sector;
- target contact normal;
- desired impulse;
- grip force.

---

## 6.3. Skill contract

A structured, time-bounded description of the physical result the body is authorized to produce.

A contract contains:

- goal;
- target;
- allowed action family;
- timing;
- tolerances;
- resource budget;
- cancellation rules;
- autonomy boundaries.

---

## 6.4. Agency

The degree to which the player remains the author of meaningful action.

Agency is not the same as direct joint control.

A player can have strong agency while the body automatically balances.

---

## 6.5. Assistance

Motor decisions delegated to the controller in order to make intent physically executable.

Examples:

- moving a foot;
- bending the knees;
- counter-rotating the torso;
- stabilizing the head.

---

## 6.6. Intervention

A controller modification of the player's requested action.

Some intervention is necessary for safety and feasibility. Unnecessary intervention is harmful to agency.

---

## 6.7. Commitment

How much the action may invest:

- momentum;
- time;
- posture;
- stamina;
- exposure;
- support changes.

High commitment allows stronger actions but reduces reversibility.

---

## 6.8. Effort

How strongly the avatar should attempt the action within current capability.

Effort is not necessarily equal to final force because force depends on:

- skill;
- strength;
- leverage;
- fatigue;
- weapon inertia;
- contact.

---

## 6.9. Style

A prior over physically valid solutions.

Style changes how a contract is executed, not what the contract means.

---

# 7. High-level architecture

```text
Input Device
    │
    ▼
Input Sampling
    │
    ▼
Control Context
    │
    ▼
Semantic Input Mapper
    │
    ▼
PlayerIntentState
    │
    ▼
Agency Arbiter
    │
    ▼
SkillContract Compiler
    │
    ▼
Contract Scheduler / Transition Planner
    │
    ▼
Skill Prior / Expert Gating
    │
    ▼
Contact and Motion Planner
    │
    ▼
Low-level Motor Policy
    │
    ▼
Joint / Muscle / Impedance Controllers
    │
    ▼
BodySchema
    │
    ▼
Physics Engine
    │
    └────────────── feedback ──────────────┐
                                          │
Input Haptics / Camera / UI ◄──────────────┘
```

---

# 8. Source of truth

The source of truth for avatar movement is:

```text
BodySchema + Physics Engine
```

The control system does not directly set render transforms.

The visual character follows physical state through the `CharacterEmbodiment` layer.

The player-control layer emits:

- intents;
- contracts;
- constraints;
- permissions.

The motor layer emits:

- actuator targets;
- joint torques;
- PD targets;
- muscle activations.

---

# 9. `PlayerIntentState`

A unified state should represent the current player-authored intent.

Conceptual Rust type:

```rust
struct PlayerIntentState {
    locomotion: LocomotionIntent,
    gaze: GazeIntent,

    focus: Option<FocusTarget>,
    interaction: InteractionIntent,

    left_hand: HandIntent,
    right_hand: HandIntent,

    combat: CombatIntent,
    traversal: TraversalIntent,
    magic: Option<ArcaneIntent>,

    style: StyleSelection,

    effort: f32,
    commitment: f32,
    precision: f32,
    risk_tolerance: f32,

    cancel: CancelIntent,
}
```

This is a semantic state, not a device-specific state.

---

# 10. Device-independent control

A gamepad, keyboard/mouse, touch screen, VR controller, or accessibility device should map into the same semantic layer.

```text
Gamepad
Keyboard/Mouse
Gyro
Touch
Motion Controllers
Eye Tracking
Adaptive Controller
      ↓
Semantic Input Mapper
      ↓
PlayerIntentState
```

This avoids designing the motor architecture around one controller.

---

# 11. Control context

The same physical input may have different semantic meaning depending on context.

Examples:

```text
right stick:
    camera in exploration
    guard direction in combat
    hand direction in precision mode
    tool direction while working
```

Context must be explicit and stable.

Possible contexts:

```rust
enum AvatarControlContext {
    Exploration,
    Combat,
    Precision,
    Grapple,
    Climb,
    ToolUse,
    RangedAim,
    SpellShaping,
    Vehicle,
}
```

The system should avoid rapid hidden context switching.

---

# 12. Progressive specificity

The player can provide different levels of detail.

The controller fills the unspecified dimensions.

## 12.1. Broad intent

```text
attack that enemy
```

The body has substantial freedom.

Suitable for assisted mode.

---

## 12.2. Directional intent

```text
attack from right to left
```

The body chooses exact trajectory and footwork.

---

## 12.3. Targeted intent

```text
attack the opponent's weapon arm from right to left
```

---

## 12.4. Timed physical intent

```text
contact this region within 250–350 ms
with high edge velocity
```

---

## 12.5. Direct task-space intent

```text
move weapon tip through this path
while keeping guard near this plane
```

The body only fills in full-body coordination.

---

# 13. Why progressive specificity matters

This allows one controller to support:

- accessibility;
- standard action gameplay;
- expert analog control;
- precision interaction;
- VR;
- AI;
- scripted cinematics.

The difference is the number of constraints supplied.

```text
fewer constraints
→ more motor autonomy

more constraints
→ more direct player control
```

---

# 14. Skill contracts

A `SkillContract` is the central runtime interface.

Conceptual base:

```rust
struct SkillContract {
    id: ContractId,
    family: SkillFamily,

    target: ContractTarget,
    spatial_goal: SpatialGoal,
    temporal_goal: TemporalGoal,

    effort: f32,
    commitment: f32,
    precision: f32,

    style: StyleId,

    permissions: AgencyPermissions,
    constraints: ContractConstraints,
    abort_policy: AbortPolicy,
}
```

---

# 15. Contract families

Possible families:

```rust
enum SkillFamily {
    Locomotion,
    Reach,
    Grab,
    Push,
    Pull,
    Carry,
    Throw,

    Strike,
    Thrust,
    Kick,
    Guard,
    Parry,
    Brace,
    Evade,

    Grapple,
    Climb,
    Vault,
    Jump,

    ToolAction,
    RangedAim,
    SpellShape,
}
```

Families are semantic boundaries.

A `Guard` contract must not silently become `Evade`.

---

# 16. Locomotion contract

```rust
struct LocomotionContract {
    desired_velocity: Vec3,
    desired_facing: Vec3,

    gait_preference: GaitPreference,
    speed_preference: f32,

    stance_height: f32,
    stability_bias: f32,

    path_corridor: Option<PathCorridor>,
    forbidden_contacts: ContactMask,
}
```

The body selects:

- step timing;
- stride length;
- foot placement;
- torso lean;
- arm motion;
- recovery.

---

# 17. Reach contract

```rust
struct ReachContract {
    effector: EffectorId,
    target_pose: TargetPose,

    position_tolerance: f32,
    orientation_tolerance: f32,

    time_window: TimeWindow,

    support_policy: SupportPolicy,
    collision_policy: CollisionPolicy,
}
```

Useful for:

- touching;
- pressing buttons;
- using handles;
- aiming tools;
- spell gestures.

---

# 18. Strike contract

```rust
struct StrikeContract {
    weapon_effector: EffectorId,
    target_region: TargetRegion,

    desired_path: StrikePath,
    desired_contact_normal: Vec3,

    contact_time_window: TimeWindow,

    desired_contact_speed: f32,
    desired_impulse: f32,

    edge_alignment: Option<Vec3>,

    follow_through: f32,
    commitment: f32,

    allowed_support_changes: SupportPolicy,
    miss_policy: MissPolicy,
}
```

The contract specifies the result, not an animation.

---

# 19. Guard contract

```rust
struct GuardContract {
    protected_sector: SpatialSector,
    defense_family: DefenseFamily,

    interception_window: TimeWindow,

    stiffness: f32,
    mobility_bias: f32,

    counter_preference: f32,
}
```

Possible defense families:

```rust
enum DefenseFamily {
    Block,
    Parry,
    Evade,
    Brace,
    Deflect,
    Counter,
}
```

The player selects the family.

The body selects the exact mechanics.

---

# 20. Grab contract

```rust
struct GrabContract {
    hand: HandId,
    target: TargetRegion,

    grip_type: GripType,
    grip_force: f32,

    approach_direction: Option<Vec3>,
    time_window: TimeWindow,

    followup: GrabFollowup,
}
```

Follow-ups:

- hold;
- pull;
- push;
- lift;
- drag;
- throw;
- restrain;
- climb.

---

# 21. Tool contract

Tools expose semantic action geometry.

```rust
struct ToolInteractionProfile {
    grip_points: Vec<GripPoint>,
    action_effectors: Vec<EffectorDefinition>,
    action_axes: Vec<ActionAxis>,
    supported_contracts: Vec<SkillFamily>,
}
```

Examples:

## Axe

- blade edge;
- cutting normal;
- swing plane;
- desired impact speed.

## Hammer

- head center;
- face normal;
- desired impulse.

## Saw

- blade line;
- reciprocal motion axis;
- contact pressure.

## Bow

- grip hand;
- draw hand;
- draw axis;
- target direction;
- release event.

---

# 22. Spell shaping contract

The same system can control physical magic.

```rust
struct SpellShapeContract {
    spell_instance: SpellInstanceId,

    focus_pose: TargetPose,
    shape_parameters: SpellShapeParameters,

    release_direction: Vec3,
    release_time: TimeWindow,

    hand_assignments: HandAssignment,
    stability_bias: f32,
}
```

The body handles:

- hand coordination;
- stance;
- balance;
- staff or wand manipulation;
- gaze alignment.

The `ArcaneDomain` handles the spell itself.

---

# 23. Contract compilation

Raw player intent is not immediately executable.

The compiler performs:

1. semantic interpretation;
2. target resolution;
3. coordinate-frame selection;
4. capability check;
5. constraint generation;
6. autonomy permission generation;
7. fallback generation.

```text
PlayerIntentState
      ↓
SkillContractCompiler
      ↓
Validated SkillContract
```

---

# 24. Contract feasibility

Before or during execution, estimate feasibility.

Possible outputs:

```rust
struct ContractFeasibility {
    reachable: bool,
    expected_error: f32,

    minimum_time: f32,
    expected_effort: f32,

    fall_risk: f32,
    joint_risk: f32,

    confidence: f32,
}
```

The UI may communicate:

- target unreachable;
- not enough room;
- insufficient stamina;
- poor footing;
- low confidence.

---

# 25. Contract scheduler

The body may have:

- an active contract;
- a queued contract;
- persistent background contracts.

Example:

```text
background:
    move forward
    look at target
    maintain guard

active:
    strike

queued:
    recover into left guard
```

Contracts can overlap if compatible.

---

# 26. Contract priority

Suggested priority classes:

```text
1. physical safety
2. explicit cancel
3. explicit player action
4. persistent player posture/guard
5. locomotion
6. stylistic preferences
7. cosmetic motion
```

Safety should not become an excuse to ignore intent. It should only prevent clearly invalid states.

---

# 27. Action lifecycle

A physical action is continuous.

Suggested phases:

```text
Acquire
→ Prepare
→ Load
→ Commit
→ Contact
→ FollowThrough
→ Recover
→ Complete
```

---

# 28. Prepare phase

The body:

- selects support;
- aligns stance;
- moves weapon into a feasible region;
- prepares balance;
- raises guard.

Player can still redirect cheaply.

---

# 29. Load phase

The body begins investing:

- muscle tension;
- weight transfer;
- elastic energy;
- weapon momentum.

Cancellation remains possible but increasingly costly.

---

# 30. Commit phase

The action crosses a commitment threshold.

The system is authorized to:

- generate substantial momentum;
- change support contacts;
- expose posture;
- consume stamina.

The player still owns cancel intent, but physics limits immediate reversal.

---

# 31. Contact phase

The physics engine determines:

- exact contact;
- impulse;
- sliding;
- weapon deflection;
- penetration;
- recoil.

The controller adapts around the actual outcome.

---

# 32. Follow-through

Follow-through depends on:

- remaining momentum;
- contact result;
- commitment;
- next contract;
- terrain;
- opponent movement.

The player may guide follow-through.

---

# 33. Recovery

The body returns to a valid state or transitions into the next contract.

Recovery should not always mean returning to idle.

Possible targets:

- guard;
- new attack;
- retreat;
- grapple;
- balanced stance.

---

# 34. Cancellation

Cancel does not mean animation reset.

It means:

```text
stop pursuing the previous task
and minimize further commitment
```

The body immediately begins:

- braking;
- lowering stiffness;
- redirecting momentum;
- restoring support;
- protecting joints.

Physical inertia remains.

---

# 35. Cancel policy

Contracts define:

```rust
enum AbortPolicy {
    ImmediateSafe,
    BrakeAndRecover,
    RedirectAllowed,
    CompleteMinimumContact,
    NonCancelableAfterCommit,
}
```

The game should use non-cancelable actions sparingly and communicate them clearly.

---

# 36. Commitment and reversibility

A useful relationship:

```text
higher commitment
→ higher possible impulse
→ lower reversibility
→ greater exposure
```

This creates physical combat risk without arbitrary animation lock.

---

# 37. Agency Arbiter

The `AgencyArbiter` is mandatory.

It determines what the controller may decide.

```rust
struct AgencyPermissions {
    may_move_feet: bool,
    may_use_free_hand_for_balance: bool,
    may_change_support: bool,

    may_adjust_target_point: bool,
    max_target_adjustment: f32,

    may_change_action_family: bool,
    may_select_new_target: bool,

    may_consume_resources: bool,
    may_initiate_contact: bool,
}
```

For player avatar contracts:

```text
may_change_action_family = false
may_select_new_target    = false
```

by default.

---

# 38. Allowed autonomous behavior

The controller may generally:

- position feet;
- bend knees;
- stabilize balance;
- use trunk counter-rotation;
- avoid self-collision;
- adjust trajectory within tolerance;
- use a free hand for balance;
- select a safe recovery;
- perform a protective fall if collapse is unavoidable;
- adapt to terrain;
- respect joint limits.

---

# 39. Forbidden autonomous behavior

Without explicit permission, the controller must not:

- initiate an attack;
- change targets;
- switch weapons;
- cast a spell;
- consume an item;
- choose dodge instead of block;
- grab an entity;
- continue aggression after cancel;
- spend a scarce resource;
- select a new tactical objective.

---

# 40. Minimal intervention principle

The controller should solve:

\[
\min_u
\left[
w_I E_{\text{intent}}
+
w_S E_{\text{stability}}
+
w_P E_{\text{physics}}
+
w_T E_{\text{style}}
+
w_A E_{\text{assistance}}
\right]
\]

Where:

- `E_intent` = contract error;
- `E_stability` = balance/fall risk;
- `E_physics` = joint/contact violations;
- `E_style` = deviation from style prior;
- `E_assistance` = unnecessary deviation from user task-space command.

The assistance term penalizes over-correction.

---

# 41. Player-authoritative dimensions

The system should explicitly label contract dimensions as:

```text
hard player constraints
soft player preferences
controller-owned dimensions
```

Example:

```text
target entity       = hard
action family       = hard
strike direction    = hard/medium
exact elbow angle   = controller-owned
foot placement      = controller-owned
style               = soft
```

This prevents accidental agency leakage.

---

# 42. Confidence and uncertainty

The controller should estimate whether the requested contract is within its training distribution.

```rust
struct MotorConfidence {
    policy_confidence: f32,
    feasibility_confidence: f32,
    contact_prediction_confidence: f32,
}
```

Low confidence should trigger conservative behavior.

---

# 43. Low-confidence fallback

When uncertain:

1. preserve action family;
2. preserve target if safe;
3. reduce speed/commitment;
4. use direct task-space tracking;
5. choose stable foot placement;
6. avoid novel flourish;
7. communicate uncertainty if needed.

Do not replace an unfamiliar player motion with a familiar but semantically different move.

---

# 44. Predictability

For the avatar:

```text
same intent
+
similar physical state
+
same style
→ similar semantic result
```

The exact foot pose may vary. The action category and contact path should remain consistent.

---

# 45. Deterministic latent selection

If internal latent variables or experts are used:

- default sampling should be deterministic;
- action-start latent should be held through the action;
- switching should use hysteresis;
- random variation should remain small;
- major stylistic variation should require player choice.

NPCs may use more stochasticity.

---

# 46. Control modes

The system should support several control densities.

## 46.1. Assisted mode

Player controls:

- movement;
- target;
- high/mid/low;
- action family;
- light/heavy timing.

The compiler supplies more constraints automatically.

---

## 46.2. Standard mode

Player controls:

- continuous direction;
- target region;
- effort;
- timing;
- guard vector;
- commitment.

Recommended default.

---

## 46.3. Expert mode

Player controls:

- end-effector target;
- weapon orientation;
- hand assignments;
- grip;
- contact path;
- release timing.

The controller still manages the body.

---

## 46.4. Precision interaction mode

For:

- tools;
- levers;
- crafting;
- climbing;
- grappling;
- spell shaping.

The input directly controls one or two end-effectors.

---

# 47. One controller, different densities

These modes should not require unrelated motor systems.

They produce different amounts of contract information.

```text
Assisted
→ sparse contract

Standard
→ medium-detail contract

Expert
→ dense task-space contract
```

The same whole-body controller completes the motion.

---

# 48. Example gamepad semantics

A possible baseline:

| Input | Semantic meaning |
|---|---|
| Left stick | desired locomotion / center movement |
| Right stick | camera, guard vector, or active-effector direction |
| Left trigger | engage left hand / left guard / grip |
| Right trigger | engage right hand / attack / grip |
| Trigger pressure | effort, stiffness, or commitment |
| Left bumper | precision modifier / left-hand mode |
| Right bumper | precision modifier / right-hand mode |
| Face button | jump, explicit evade, context action |
| Stick click | stance/focus modifier |
| Gyro | optional gaze/camera/aim |

This is a design space, not a frozen layout.

---

# 49. Hand-semantic triggers

Mapping:

```text
left trigger  → left hand
right trigger → right hand
```

has several advantages:

- intuitive body mapping;
- supports weapons and tools;
- supports grappling;
- supports two-hand objects;
- supports magic gestures;
- generalizes beyond combat.

---

# 50. Camera conflict

When the right stick controls an effector, camera control must be preserved.

Possible solutions:

- soft target focus;
- gyro camera;
- shoulder buttons for camera;
- context-relative camera follow;
- temporary frozen action frame;
- mouse for camera, keys for task direction.

The chosen scheme requires user studies.

---

# 51. Stable control frames

The meaning of a directional gesture must not rotate unpredictably with the camera during execution.

Possible frames:

```rust
enum ControlFrame {
    CameraRelative,
    AvatarRelative,
    TargetRelative,
    WeaponRelative,
    FrozenAtActionStart,
    WorldRelative,
}
```

For attacks, `FrozenAtActionStart` or `TargetRelative` may be best.

---

# 52. Target-relative combat plane

A continuous direction can be mapped around the focused opponent.

```text
up
→ high line

down
→ low line

left/right
→ lateral lines
```

Unlike discrete guard directions, the value can remain continuous.

---

# 53. Guard representation

A guard can be represented as a plane or sector.

```rust
struct SpatialSector {
    origin: Vec3,
    axis: Vec3,
    angular_extent: f32,
    radial_extent: Range<f32>,
}
```

The body places weapon, shield, or limbs to cover it.

---

# 54. Block, parry, evade, brace

These should remain distinct player-authored families.

## Block

- high stiffness;
- accepts impulse;
- prioritizes coverage.

## Parry

- moving interception;
- tight timing;
- redirects momentum.

## Evade

- removes body from predicted path;
- changes support.

## Brace

- prepares body to absorb impact;
- may use environment or shield.

The controller chooses only the detailed realization.

---

# 55. Strike direction

Strike direction should not be a discrete animation index.

Possible representation:

```rust
struct StrikePath {
    start_region: TaskSpaceRegion,
    end_region: TaskSpaceRegion,
    curvature: f32,
    plane_normal: Vec3,
}
```

The player can specify it through a stick gesture or pointer movement.

---

# 56. Gesture interpretation

A gesture may define:

- initial direction;
- path;
- speed;
- release;
- commitment.

The compiler converts the gesture into a stable contract.

It should not continuously reinterpret the gesture after commitment unless the action allows redirection.

---

# 57. Prepare, hold, release

An intuitive action pattern:

```text
hold
→ prepare/load

move stick
→ define line

release
→ commit/contact
```

This can support:

- sword swings;
- throws;
- spell release;
- bow shots;
- hammer blows.

---

# 58. Analog effort

Trigger pressure may control:

- grip stiffness;
- muscular effort;
- attack commitment;
- shield stiffness;
- pull force.

It should not necessarily map linearly to damage.

---

# 59. Haptics

The control system should return physical feedback.

Possible haptic channels:

- increasing load during preparation;
- contact impulse;
- weapon deflection;
- grip slip;
- channel overload for magic;
- fatigue;
- blocked joint or unreachable target.

Haptics help the player understand a high-dimensional body through low-dimensional input.

---

# 60. Combat styles

A style is a prior over valid solutions.

```rust
struct StyleProfile {
    stance_bias: StanceBias,
    distance_bias: f32,

    movement_compactness: f32,
    commitment_bias: f32,

    guard_priority: f32,
    counter_rotation: f32,

    preferred_contacts: ContactPreference,
    recovery_profile: RecoveryProfile,
}
```

---

# 61. Style does not change intent

Contract:

```text
strike right-to-left at shoulder
```

Style changes:

- step size;
- torso rotation;
- weapon arc;
- guard preservation;
- follow-through;
- recovery.

It must not change the strike into a kick.

---

# 62. Skill progression

Character skill should improve execution quality without changing control vocabulary.

Player input meaning remains stable.

```text
same contract
→ novice execution
→ trained execution
→ master execution
```

---

# 63. Novice behavior

A novice may:

- use wider preparation;
- have larger trajectory error;
- overcommit;
- expose guard;
- recover slowly;
- use excess stamina;
- have poor foot placement;
- fail complex simultaneous constraints.

---

# 64. Trained behavior

A trained character may:

- execute more accurately;
- maintain guard;
- change support efficiently;
- use weapon inertia;
- transition quickly;
- recover from partial contact.

---

# 65. Master behavior

A master may:

- preserve several constraints simultaneously;
- redirect late;
- exploit opponent force;
- use compact motion;
- execute from awkward poses;
- maintain balance under perturbation;
- recover from misses;
- perform advanced contact plans.

---

# 66. Player skill versus avatar skill

The outcome should depend on:

```text
player decision quality
×
avatar motor competence
×
physical capability
×
world state
```

Player skill includes:

- timing;
- target selection;
- direction;
- distance;
- commitment;
- reading momentum;
- risk.

Avatar skill includes:

- accuracy;
- coordination;
- recovery;
- technique availability;
- energy efficiency.

---

# 67. Avoiding stat domination

High avatar skill must not completely automate success.

A master avatar should execute good player intent better.

It should not independently make the good tactical decision.

---

# 68. Avoiding player-only domination

A skilled player should not entirely ignore avatar limitations.

A weak or injured character may:

- fail to reach required speed;
- lose balance;
- lack throughput;
- tire;
- have limited joint range.

---

# 69. Stamina and effort

Stamina should emerge from motor state where possible.

Inputs:

- muscle activation;
- torque;
- contraction duration;
- cardiovascular state;
- carried load;
- injury.

High effort may produce:

- stronger acceleration;
- higher fatigue;
- lower precision;
- longer recovery.

---

# 70. Injury

The contract system remains stable under injury.

The motor controller adapts execution.

Examples:

```text
injured right arm
→ lower achievable impulse
→ two-hand compensation
→ different guard

injured leg
→ reduced movement
→ altered support
→ increased fall risk
```

The player does not need a new control scheme.

---

# 71. Prosthetics and morphology changes

Because input targets semantic effectors and contracts, the same control language can work with:

- prosthetic limb;
- missing hand;
- extra arm;
- tail;
- wing;
- non-humanoid body.

The `EmbodimentMap` determines available effectors.

---

# 72. Grappling

Grappling is a strong use case for shared embodiment.

Player semantics:

```text
trigger → grip
stick   → pull/push/twist
target  → body region
```

The controller handles:

- foot placement;
- torso mechanics;
- grip maintenance;
- balance;
- force distribution.

---

# 73. Grapple contract

```rust
struct GrappleContract {
    grips: Vec<GripConstraint>,

    desired_relative_motion: RelativeMotionGoal,
    desired_force: Vec3,

    body_region_priority: BodyRegionPriority,

    takedown_family: Option<TakedownFamily>,
    release_policy: ReleasePolicy,
}
```

The player must authorize the takedown family.

The model may not invent a throw.

---

# 74. Two-body control

Grappling requires coupled controllers.

Possible architectures:

- independent controllers with shared contact state;
- a joint interaction policy;
- leader/follower task constraints;
- short-horizon coupled optimizer.

This is a research-heavy subsystem.

---

# 75. Manipulation

The same contract system supports:

- doors;
- levers;
- carrying;
- dragging;
- throwing;
- cooperative objects;
- construction.

The player controls the meaningful action axis.

---

# 76. Climbing

Player specifies:

- movement direction;
- desired hand/foot target;
- release;
- commitment.

Controller selects:

- exact holds;
- body orientation;
- weight transfer;
- contact order.

Assisted mode may choose holds automatically.

Expert mode may allow direct hold selection.

---

# 77. Parkour

Contracts:

- vault obstacle;
- grab ledge;
- mantle;
- wall-run;
- jump to region.

The player authorizes the family and target.

The controller solves the body.

---

# 78. Ranged weapons

Player controls:

- aim;
- draw;
- release;
- stance preference;
- breath/precision.

The controller handles:

- full-body alignment;
- recoil;
- weapon stabilization;
- support.

---

# 79. Physical magic

Player controls:

- target;
- shape;
- direction;
- release;
- effort;
- hand assignment.

Motor control handles body gestures and stance.

Arcane control handles mana and spell fields.

The two systems should be separate but synchronized.

---

# 80. Intent horizon

The controller should receive a short-horizon estimate of desired intent.

```rust
struct IntentHorizon {
    samples: Vec<TimedIntentSample>,
}
```

This allows anticipatory:

- weight shift;
- step preparation;
- guard movement;
- tool alignment.

The horizon should not predict tactics beyond player input.

---

# 81. Input prediction

For latency reduction, predict continuation of:

- stick direction;
- hand path;
- aim;
- trigger pressure.

Prediction must be conservative and immediately correctable.

---

# 82. Input buffering

The player may queue the next contract.

```text
active strike
+
queued guard
```

The transition planner seeks a physically efficient transition.

---

# 83. Combos as contract transitions

A combo is not a sequence of animation IDs.

It is a sequence of compatible physical contracts.

```text
contract A
→ residual state
→ contract B
```

Transition cost depends on:

- weapon pose;
- support;
- momentum;
- fatigue;
- target position.

---

# 84. Flow

A good transition reuses momentum.

A poor transition requires braking and repositioning.

This creates physically meaningful move chaining.

---

# 85. Feints

A feint is:

```text
prepare/load
→ change or cancel before full commit
```

The opponent may observe the physical preparation.

No special feint animation is required.

---

# 86. Redirection

Some contracts permit redirection.

```rust
struct RedirectPolicy {
    latest_redirect_time: f32,
    max_angular_change: f32,
    additional_effort_cost: f32,
}
```

Avatar skill can improve late redirection.

---

# 87. Motion priors

Pure optimization may generate awkward but valid motion.

A learned motion prior provides:

- natural coordination;
- cultural technique;
- style;
- realistic transitions;
- human-like regularization.

---

# 88. Universal motor executor

A useful architecture:

```text
universal low-level executor
+
skill/style conditioning
+
specialized experts
+
transition-aware gating
```

The executor owns general body knowledge.

Experts handle difficult contact regimes.

---

# 89. Expert policies

Possible experts:

- locomotion;
- sword;
- spear;
- shield;
- grapple;
- climb;
- recovery;
- swimming;
- flight.

Experts should share lower-level representations if possible.

---

# 90. Gating

The gate selects or blends experts based on:

- contract family;
- morphology;
- equipment;
- contact state;
- style;
- confidence.

The gate must not change player intent.

---

# 91. Transition training

Experts must be trained on:

- random interruption;
- arbitrary start states;
- changing targets;
- misses;
- collisions;
- partial completion;
- injury;
- terrain.

Without transition training, the controller will fail at the exact moments where player control matters.

---

# 92. Training pipeline

```text
Motion Data
    ↓
Tracking Policy
    ↓
Universal Motor Prior
    ↓
Contract Extraction
    ↓
Contract-conditioned Skill Training
    ↓
Task RL
    ↓
Perturbation Training
    ↓
Transition / Interruption Training
    ↓
Self-play for tactics
    ↓
Player-in-the-loop Calibration
    ↓
Production Validation
```

---

# 93. Motion data

Sources may include:

- mocap;
- keyframed animation;
- video reconstruction;
- procedural demonstrations;
- optimization-generated motion;
- teleoperation;
- existing policy rollouts.

---

# 94. Contract extraction from motion

For each demonstration, derive:

- active effector;
- target;
- trajectory;
- contact timing;
- contact normal;
- impulse;
- support sequence;
- style features;
- recovery.

This creates:

```text
contract
→ full-body solution
```

training pairs.

---

# 95. Imitation learning

Imitation establishes:

- natural motion;
- recognizable techniques;
- timing;
- coordination;
- style.

It should not be the only objective.

---

# 96. Reinforcement learning

RL expands robustness across:

- terrain;
- target variation;
- weapon mass;
- body parameters;
- perturbations;
- injuries;
- unusual poses.

Rewards should prioritize contract satisfaction and agency.

---

# 97. Reward components

Possible reward:

\[
R =
w_C R_{\text{contract}}
+
w_B R_{\text{balance}}
+
w_E R_{\text{energy}}
+
w_P R_{\text{prior}}
+
w_A R_{\text{agency}}
+
w_S R_{\text{safety}}
\]

Where:

- contract reward measures task-space intent;
- balance reward prevents unnecessary falls;
- energy reward encourages efficiency;
- prior reward preserves style;
- agency reward penalizes semantic deviation;
- safety reward prevents injury/limits.

---

# 98. Agency reward

Penalize:

- wrong action family;
- wrong target;
- unauthorized contact;
- continued action after cancel;
- excessive automatic target correction;
- unnecessary movement.

Agency must be evaluated during training, not added only as runtime logic.

---

# 99. Self-play

Self-play is useful for:

- tactical timing;
- spacing;
- reactions;
- counterplay;
- opponent modeling.

It should not define the entire movement style from scratch.

Recommended:

```text
motion prior
+
contract-conditioned motor system
+
self-play tactical policy
```

For the avatar, tactical decisions remain player-owned.

---

# 100. Player-in-the-loop calibration

Human traces should be collected.

Data:

- input;
- generated contract;
- motor output;
- corrections;
- cancels;
- unexpected actions;
- subjective rating.

This can improve:

- input mapping;
- contract inference;
- control-frame choice;
- assistance strength;
- latency.

---

# 101. Personal control calibration

Different players may prefer:

- more assistance;
- more directness;
- different stick response;
- different commitment curves;
- different camera behavior.

A per-player calibration layer may adjust semantic mapping while preserving contract meaning.

---

# 102. Active learning

The controller will encounter unusual player-created motions.

Pipeline:

```text
runtime uncertainty
→ record state and intent
→ cluster failures
→ generate teacher solutions
→ retrain
→ validate
```

This is especially important because the player explores action space differently from mocap actors.

---

# 103. Out-of-distribution detection

Signals:

- policy disagreement;
- high action variance;
- critic uncertainty;
- constraint violation prediction;
- distance from training embedding;
- repeated corrective intervention.

---

# 104. Multi-morphology support

The input should target semantic effectors, not fixed humanoid joints.

```rust
enum SemanticEffector {
    PrimaryHand,
    SecondaryHand,
    PrimaryFoot,
    Head,
    TailTip,
    WingTip,
    WeaponTip,
    Custom(EffectorId),
}
```

A morphology maps semantics to actual body nodes.

---

# 105. Morphology-conditioned contracts

A `StrikeContract` may be realized by:

- human arm;
- claw;
- tail;
- wing;
- weapon.

The player may explicitly select an effector or allow a permitted set.

For the avatar, automatic effector substitution should be conservative.

---

# 106. Character Embodiment Map

```rust
struct EmbodimentMap {
    semantic_effectors: HashMap<SemanticEffector, EffectorId>,
    available_skill_families: SkillFamilyMask,
    support_effectors: Vec<EffectorId>,
}
```

This connects control semantics to morphology.

---

# 107. Interface continuity under body change

If a character loses an arm:

- right-hand contracts become unavailable;
- the same locomotion controls remain;
- two-hand skills may become unavailable or adapted;
- one-hand alternatives may appear.

The control vocabulary should degrade gracefully.

---

# 108. Neural controller input

Conceptual:

```rust
struct AvatarMotorInput {
    body_state: BodyState,
    environment: LocalEnvironmentState,

    active_contracts: ContractSet,

    style: StyleState,
    skill: SkillState,

    physiology: PhysiologyState,
    damage: BodyDamageState,

    agency_permissions: AgencyPermissions,
}
```

---

# 109. Neural controller output

Possible output spaces:

- joint torque;
- target joint positions;
- target velocities;
- impedance parameters;
- muscle activations;
- contact intent.

For stability, a hierarchy may be used:

```text
policy
→ PD / impedance targets
→ low-level actuator controller
```

---

# 110. Contact planner

Some skills require explicit contact planning.

Inputs:

- geometry;
- predicted target motion;
- active contract;
- body capability.

Outputs:

- candidate contact;
- timing;
- support sequence;
- approach region.

The planner must remain inside agency permissions.

---

# 111. Short-horizon motion planner

A short-horizon optimizer can refine:

- contact timing;
- end-effector path;
- collision avoidance;
- balance.

The neural policy provides warm starts and priors.

---

# 112. Hybrid neural + optimization control

Recommended research architecture:

```text
contract
→ neural skill prior
→ short-horizon constrained refinement
→ motor policy
```

Benefits:

- predictable contract satisfaction;
- physical constraints;
- learned natural motion;
- adaptability.

Cost must be measured.

---

# 113. Latency budget

Player control is sensitive to latency.

Pipeline components:

```text
input sampling
semantic mapping
contract compilation
policy inference
physics step
render interpolation
```

The action should show visible response immediately.

Even if a strong strike requires preparation, the body should begin preparing on the first frame.

---

# 114. Response versus completion

Separate:

- **response latency:** time before body visibly reacts;
- **completion latency:** physical time to achieve result.

Response latency must be low.

Completion latency may reflect physics.

---

# 115. Multi-rate control

Possible frequencies:

```text
input sampling:
120–1000 Hz device-dependent

contract compilation:
60–120 Hz

high-level skill planning:
30–60 Hz

motor policy:
60–120 Hz

joint control:
120–480 Hz

physics:
60–240 Hz

render:
display frequency
```

---

# 116. Pose interpolation

Visual interpolation may be used, but the render pose must derive from physics.

Do not hide large control latency with animation-only anticipation that contradicts physical state.

---

# 117. Network architecture

For multiplayer, replicate player intent and authoritative physical outcomes.

Possible model:

```text
client:
input → local predicted contracts → local motor simulation

server:
validates contracts → authoritative physics

server:
sends corrections / topology events
```

---

# 118. Replication unit

Do not replicate every neural latent.

Replicate:

- semantic contract;
- start tick;
- style;
- commitment;
- target;
- seed if needed;
- authoritative contacts;
- corrections.

---

# 119. Server authority

Server decides:

- valid target;
- resource use;
- hit/contact outcome;
- topology-changing injury;
- disarm;
- item use;
- spell release.

Clients may predict body motion.

---

# 120. Reconciliation

Corrections should preserve action semantics.

Bad correction:

```text
snap avatar to server pose
```

Better:

```text
adjust contract timing
blend motor state
apply impulse correction
preserve active action family
```

---

# 121. Replay

A replay can store:

- contracts;
- input traces;
- authoritative events;
- periodic body snapshots.

This is more compact and interpretable than storing all joints every frame.

---

# 122. Physics LOD

Avatar motor physics and remote-character rendering may use different LODs.

Local player:

- full policy;
- high-rate control;
- detailed contact.

Remote players:

- contract-driven reconstruction;
- lower-rate policy;
- state corrections.

Far NPCs:

- reduced motor representation.

---

# 123. Control LOD

Control specificity may also change by context.

Example:

```text
crowded large battle
→ standard directional contracts

close duel
→ precision weapon control
```

This should remain player-selected or clearly communicated.

---

# 124. Accessibility

Accessibility is not a separate engine.

It is a different contract compiler.

Possible assists:

- target-region snapping;
- direction quantization;
- timing assistance;
- guard prediction;
- lower commitment by default;
- automatic camera focus;
- simplified grip;
- one-handed controller layout.

The `AgencyArbiter` still preserves player ownership.

---

# 125. Assistance strength

A continuous parameter:

```text
assistance = 0..1
```

may influence:

- target tolerance;
- timing tolerance;
- path smoothing;
- footwork autonomy;
- camera assistance.

It must not silently authorize new action families.

---

# 126. Readability

Physics-driven movement must remain readable.

Tools:

- stable styles;
- clear preparation;
- limited action variance;
- visible commitment;
- camera framing;
- consistent guard semantics;
- audio/haptic cues;
- opponent telegraph policies.

---

# 127. Fairness

If attacks are fully continuous, competitive fairness requires:

- bounded acceleration;
- reliable telegraph cues;
- consistent contact rules;
- controlled latency;
- limited hidden assistance;
- explicit skill capabilities.

---

# 128. Enemy readability

NPC motor policies may need presentation constraints:

- minimum telegraph time;
- readable preparation;
- style-consistent contact path;
- bounded redirection;
- no invisible instant transitions.

Physical possibility alone is not sufficient for good combat.

---

# 129. Avatar readability

The player's body should visually communicate:

- load;
- commitment;
- available recovery;
- fatigue;
- imbalance;
- blocked path;
- grip loss.

The `CharacterEmbodiment` layer can expose muscle tension and body state.

---

# 130. UI

Prefer world-space and embodied feedback over large HUD indicators.

Possible feedback:

- weapon ghost path;
- target-sector highlight;
- guard arc;
- subtle contact-time marker;
- body balance indicator;
- grip state;
- haptic load.

Expert mode may remove some aids.

---

# 131. Debugging

Essential visualizations:

- active contracts;
- hard/soft constraints;
- task-space targets;
- predicted contacts;
- support polygon;
- center of mass;
- controller-owned joints;
- player-owned dimensions;
- agency interventions;
- confidence;
- selected expert/latent;
- cancel state;
- predicted trajectory.

---

# 132. Agency debugging

Log every intervention:

```rust
struct AgencyInterventionEvent {
    contract_id: ContractId,
    dimension: AgencyDimension,

    requested: SemanticValue,
    executed: SemanticValue,

    reason: InterventionReason,
    magnitude: f32,
}
```

This is critical for tuning trust.

---

# 133. Metrics

## 133.1. Intent fidelity

- target error;
- direction error;
- contact-time error;
- impulse error;
- guard-sector coverage;
- release-time error.

---

## 133.2. Agency

- unauthorized action family changes;
- unauthorized target changes;
- continued action after cancel;
- unwanted contacts;
- unnecessary intervention magnitude.

---

## 133.3. Predictability

Measure variance of semantic outcome for similar:

- intent;
- state;
- style.

---

## 133.4. Correctability

- time to redirect;
- time to cancel;
- counter-input required;
- overshoot after cancel.

---

## 133.5. Responsiveness

- input-to-visible-response latency;
- input-to-contract latency;
- input-to-actuator latency.

---

## 133.6. Learnability

- time to basic competence;
- error rate;
- subjective workload;
- retention after break.

---

## 133.7. Skill ceiling

- expert accuracy;
- advanced techniques;
- performance separation;
- intentional variability.

---

## 133.8. Physical quality

- falls;
- joint-limit violations;
- contact penetration;
- energy spikes;
- controller instability;
- foot sliding;
- self-collision.

---

# 134. User-study structure

Compare:

## Scheme A

Conventional action buttons.

## Scheme B

Directional semantic contracts.

## Scheme C

Direct task-space hand/weapon control.

Measure:

- enjoyment;
- agency;
- accuracy;
- fatigue;
- learning time;
- expressive variety;
- competitive performance.

A likely result is:

```text
B = default
C = precision/expert
A = accessibility
```

This must be experimentally validated.

---

# 135. MVP 1 — Locomotion and hand embodiment

Requirements:

- physical humanoid;
- player locomotion intent;
- gaze;
- one hand target;
- balance completion;
- push and touch.

Demo:

```text
walk over uneven terrain
while moving right hand toward arbitrary targets
```

---

# 136. MVP 2 — Unarmed semantic duel

Contracts:

- strike;
- push;
- guard;
- parry;
- explicit evade;
- grab.

Inputs:

- direction;
- target region;
- effort;
- timing.

---

# 137. MVP 3 — One-handed weapon

Requirements:

- weapon grip;
- strike path;
- guard sector;
- physical contact;
- misses;
- cancel;
- recovery.

---

# 138. MVP 4 — Transition robustness

Test:

```text
strike → cancel
strike → guard
guard → thrust
miss → recovery
collision → adaptation
```

This milestone is more important than adding many techniques.

---

# 139. MVP 5 — Styles and character skill

Implement:

- novice;
- trained;
- master.

Same player contract language.

Different execution quality.

---

# 140. MVP 6 — Two-hand interaction

Add:

- shield;
- two-handed sword;
- spear;
- carry;
- grapple.

---

# 141. MVP 7 — Physical magic

Add:

- hand assignment;
- spell shaping;
- staff/wand;
- release;
- balance under recoil.

---

# 142. Recommended implementation roadmap

## Phase A — Semantic input model

Define:

- `PlayerIntentState`;
- control contexts;
- coordinate frames;
- device mapping.

No neural controller change yet.

---

## Phase B — Skill contract schema

Implement:

- locomotion;
- reach;
- strike;
- guard;
- grab;
- cancel.

Use a simple existing controller or scripted solver.

---

## Phase C — Agency Arbiter

Implement:

- permissions;
- intervention logging;
- forbidden autonomy;
- cancellation guarantees.

---

## Phase D — Contract-conditioned motor policy

Train controller to satisfy sparse contracts.

Start with locomotion and hand reach.

---

## Phase E — Continuous attack and guard

Add:

- path contracts;
- contact timing;
- guard sector;
- physical weapons.

---

## Phase F — Transition training

Train:

- interruption;
- redirects;
- misses;
- collisions;
- arbitrary initial states.

---

## Phase G — Styles

Condition execution on style priors.

---

## Phase H — Avatar skill progression

Add competence parameters:

- accuracy;
- control bandwidth;
- recovery;
- efficiency.

---

## Phase I — Expert task-space mode

Direct hand/weapon targets.

---

## Phase J — Grapple and tools

Introduce persistent contacts and object action axes.

---

## Phase K — Player-in-the-loop calibration

Collect user traces and adjust mappings.

---

## Phase L — Multiplayer

Contract prediction, authority, and reconciliation.

---

# 143. Rust module proposal

```text
crates/
├── avatar-control-core
│   ├── intent
│   ├── contexts
│   ├── control_frames
│   ├── targets
│   └── semantics
│
├── avatar-input
│   ├── gamepad
│   ├── keyboard_mouse
│   ├── gyro
│   ├── touch
│   └── accessibility
│
├── avatar-contracts
│   ├── schema
│   ├── compiler
│   ├── feasibility
│   ├── scheduler
│   └── transitions
│
├── avatar-agency
│   ├── permissions
│   ├── arbiter
│   ├── intervention
│   └── audit
│
├── avatar-skills
│   ├── locomotion
│   ├── reach
│   ├── combat
│   ├── grapple
│   ├── traversal
│   ├── tools
│   └── magic
│
├── avatar-motor
│   ├── policy
│   ├── priors
│   ├── experts
│   ├── gating
│   ├── contact_planner
│   └── fallback
│
├── avatar-training
│   ├── contract_extraction
│   ├── imitation
│   ├── rl
│   ├── transitions
│   ├── self_play
│   └── active_learning
│
├── avatar-network
├── avatar-debug
├── avatar-validation
└── avatar-user-study
```

Initially, use modules inside fewer crates.

---

# 144. Runtime types

```rust
struct AvatarControlState {
    context: AvatarControlContext,

    intent: PlayerIntentState,

    active_contracts: ContractSet,
    queued_contracts: ContractQueue,

    agency: AgencyState,
    motor_confidence: MotorConfidence,

    selected_style: StyleId,
    assistance_profile: AssistanceProfile,
}
```

---

# 145. Contract target

```rust
enum ContractTarget {
    None,
    WorldPoint(Vec3),
    WorldPose(Transform),
    Entity(EntityId),
    EntityRegion {
        entity: EntityId,
        region: BodyRegion,
    },
    Trajectory(TrajectoryId),
    SpatialRegion(SpatialRegion),
}
```

---

# 146. Contract constraints

```rust
struct ContractConstraints {
    hard: Vec<HardConstraint>,
    soft: Vec<SoftConstraint>,

    support_policy: SupportPolicy,
    contact_policy: ContactPolicy,

    stamina_budget: Option<f32>,
    risk_limit: f32,
}
```

---

# 147. Agency dimensions

```rust
enum AgencyDimension {
    TargetEntity,
    TargetRegion,
    ActionFamily,
    Effector,
    Direction,
    Timing,
    Effort,
    Commitment,
    ContactPermission,
    ResourceUse,
}
```

Every intervention should identify the affected dimension.

---

# 148. Motor fallback interface

```rust
trait MotorFallback {
    fn generate_safe_action(
        &self,
        body: &BodyState,
        contract: &SkillContract,
        permissions: &AgencyPermissions,
    ) -> FallbackAction;
}
```

Fallback must preserve semantic boundaries.

---

# 149. Contract result

```rust
struct ContractResult {
    outcome: ContractOutcome,

    intent_error: IntentError,
    contact_result: Option<ContactResult>,

    energy_used: f32,
    stamina_used: f32,

    interventions: Vec<AgencyInterventionEvent>,
}
```

This supports training and analytics.

---

# 150. Integration with BodySchema

`BodySchema` must expose:

- current joints;
- available effectors;
- contact state;
- actuator limits;
- injuries;
- support contacts;
- body morphology.

The avatar layer must not assume a fixed humanoid joint list.

---

# 151. Integration with CharacterEmbodiment

The visual system receives:

- joint torques;
- muscle activations;
- contact impulses;
- commitment;
- fatigue.

This lets the avatar visibly communicate control state.

---

# 152. Integration with PhysicalWorld

Contracts target real world state:

- actual rigid bodies;
- actual water;
- actual trees;
- actual tools;
- actual terrain;
- actual spell fields.

The control layer uses world queries rather than scripted interaction markers only.

---

# 153. Integration with AI

NPCs may use the same contract vocabulary.

Difference:

```text
player avatar:
    player generates contracts
    strict Agency Arbiter

NPC:
    tactical controller generates contracts
    broader autonomy
```

This creates a unified movement interface across players and NPCs.

---

# 154. Integration with animation assets

Animation is still useful as:

- training data;
- style examples;
- motion priors;
- presentation constraints;
- fallback clips;
- cinematics.

Animation clips are not the source of truth for physical action.

---

# 155. Failure modes

## 155.1. Autopilot feeling

Symptoms:

- model chooses actions;
- excessive target snapping;
- unexpected movement.

Mitigation:

- stronger hard constraints;
- intervention penalty;
- agency metrics;
- lower assistance.

---

## 155.2. Puppet feeling

Symptoms:

- body is unstable;
- player must micromanage balance.

Mitigation:

- stronger motor prior;
- automatic support planning;
- higher body-owned coordination.

---

## 155.3. Random feeling

Symptoms:

- same input creates different attacks.

Mitigation:

- deterministic latent;
- stable style;
- hysteresis;
- action-family hard constraints.

---

## 155.4. Sluggish feeling

Symptoms:

- long delay before visible response.

Mitigation:

- immediate preparation;
- lower pipeline latency;
- intent horizon;
- local prediction.

---

## 155.5. Animation lock in disguise

Symptoms:

- contract always resolves to one fixed trajectory;
- cancel unavailable;
- context has little effect.

Mitigation:

- wider state distribution;
- transition training;
- task-space evaluation;
- physical contact adaptation.

---

## 155.6. Unreadable combat

Symptoms:

- attacks redirect too late;
- no telegraph;
- excessive variety.

Mitigation:

- presentation constraints;
- minimum preparation;
- bounded redirection;
- style stability.

---

## 155.7. Skill system hides player input

Symptoms:

- high-level character auto-wins.

Mitigation:

- character skill improves execution, not decisions;
- target and family remain player-owned.

---

# 156. Research questions

## Input and agency

1. Which dimensions should be player-owned in each context?
2. How much automatic target correction is acceptable?
3. Can assistance be continuous without becoming unpredictable?
4. Which control frame is easiest to learn?
5. How should camera and hand control share the right stick?

## Contracts

6. What is the minimal universal contract schema?
7. Should contact time be explicit or inferred?
8. How should contract tolerances scale with skill?
9. How should multiple concurrent contracts compose?

## Motor control

10. One universal policy or mixture of experts?
11. How should expert transitions be trained?
12. Is short-horizon optimization necessary?
13. Which action output is best: torques, PD targets, or muscles?
14. How should OOD confidence be estimated?

## Combat

15. What continuous guard representation is most readable?
16. How much redirection should be possible after commitment?
17. How should telegraph requirements constrain a physical policy?
18. How should weapon bind and sliding contact affect control?

## Progression

19. Which capability dimensions represent novice-to-master growth?
20. How can progression remain visible without changing input semantics?
21. How should injuries modify feasible contracts?

## User experience

22. Which mode should be default?
23. How quickly can players learn directional contracts?
24. Does expert task-space control improve skill ceiling?
25. What haptic signals communicate physical state best?

## Networking

26. Can contracts be predicted reliably?
27. How should reconciliation preserve agency?
28. Which physical events require server authority?

---

# 157. Candidate ADRs

1. `ADR-Avatar-Control-Thesis.md`
   - player owns decisions;
   - body owns coordination.

2. `ADR-PlayerIntent-State.md`
   - semantic device-independent input.

3. `ADR-Skill-Contract-Architecture.md`
   - contract schema and lifecycle.

4. `ADR-Agency-Arbiter.md`
   - autonomy boundaries.

5. `ADR-Progressive-Specificity.md`
   - assisted, standard, expert control.

6. `ADR-Avatar-Control-Frames.md`
   - camera/target/frozen frames.

7. `ADR-Avatar-Motor-Policy.md`
   - universal policy vs experts.

8. `ADR-Avatar-Style-Priors.md`

9. `ADR-Avatar-Skill-Progression.md`

10. `ADR-Avatar-Cancel-And-Commitment.md`

11. `ADR-Avatar-Network-Authority.md`

12. `ADR-Avatar-Accessibility.md`

13. `ADR-Avatar-Confidence-Fallback.md`

---

# 158. Candidate specifications

- `SPEC-PlayerIntentState.md`
- `SPEC-Avatar-Control-Contexts.md`
- `SPEC-Avatar-Control-Frames.md`
- `SPEC-SkillContract-Core.md`
- `SPEC-LocomotionContract.md`
- `SPEC-ReachContract.md`
- `SPEC-StrikeContract.md`
- `SPEC-GuardContract.md`
- `SPEC-GrabContract.md`
- `SPEC-ToolContract.md`
- `SPEC-SpellShapeContract.md`
- `SPEC-Contract-Scheduler.md`
- `SPEC-Contract-Feasibility.md`
- `SPEC-Agency-Permissions.md`
- `SPEC-Agency-Arbiter.md`
- `SPEC-Avatar-Cancel.md`
- `SPEC-Commitment-Model.md`
- `SPEC-Avatar-Style.md`
- `SPEC-Avatar-Skill-State.md`
- `SPEC-Motor-Confidence.md`
- `SPEC-Motor-Fallback.md`
- `SPEC-Avatar-Input-Gamepad.md`
- `SPEC-Avatar-Input-Keyboard-Mouse.md`
- `SPEC-Avatar-Haptics.md`
- `SPEC-Avatar-Networking.md`
- `SPEC-Avatar-Debug-Tools.md`
- `SPEC-Avatar-Validation.md`

---

# 159. Candidate research spikes

- `SPIKE-Contract-Conditioned-Masked-Motion.md`
- `SPIKE-Continuous-Guard-Space.md`
- `SPIKE-Weapon-Tip-Task-Control.md`
- `SPIKE-Modal-Shared-Autonomy.md`
- `SPIKE-Agency-Metric.md`
- `SPIKE-Player-Correction-Model.md`
- `SPIKE-Deterministic-Latent-Control.md`
- `SPIKE-Avatar-Motor-Confidence.md`
- `SPIKE-Neural-Expert-Transitions.md`
- `SPIKE-Grapple-Control.md`
- `SPIKE-Contract-Network-Prediction.md`
- `SPIKE-Gamepad-Control-Study.md`
- `SPIKE-Gyro-Hand-Camera-Separation.md`
- `SPIKE-Avatar-Skill-Progression.md`

---

# 160. Normative requirements

## MUST

- The player avatar MUST preserve player ownership of target, action family, timing intent, and significant resource use.
- The motor system MUST own redundant joint coordination, balance, and foot placement.
- Player input MUST be mapped into semantic intent before motor inference.
- Physical actions MUST be represented by contracts or equivalent task-space constraints, not only animation identifiers.
- The `AgencyArbiter` MUST prevent unauthorized tactical decisions.
- Cancel input MUST be acknowledged immediately, even when physical momentum prevents instant stopping.
- The controller MUST expose confidence or an equivalent uncertainty signal.
- Low-confidence fallback MUST preserve action-family semantics.
- Player-avatar latent selection MUST be sufficiently deterministic for predictable control.
- Training MUST include interruptions, misses, collisions, and arbitrary starting states.
- The system MUST log agency interventions.
- The physics engine and `BodySchema` MUST remain the source of truth for movement.

## SHOULD

- The system SHOULD support progressive specificity.
- Standard control SHOULD expose continuous direction, effort, timing, and guard.
- Expert control SHOULD expose task-space hand or weapon targets.
- Character skill SHOULD change execution quality rather than input meaning.
- Styles SHOULD be priors over solutions, not separate input vocabularies.
- Haptics SHOULD communicate load, contact, and blocked motion.
- The controller SHOULD minimize unnecessary intervention.
- The same contract vocabulary SHOULD be reusable by NPCs and players.
- Networking SHOULD replicate contracts and authoritative events rather than all policy latents.
- Accessibility SHOULD be implemented through alternate contract compilers.

## MAY

- A short-horizon optimizer MAY refine neural motion.
- Gyro MAY separate camera and effector control.
- Player-specific mapping calibration MAY be learned.
- VR controllers MAY provide dense task-space contracts.
- Motion clips MAY be used as fallback or training data.
- Expert policies MAY be used for specialized contact skills.

---

# 161. Reference directions

The following are research and production directions relevant to this architecture. Exact versions, bibliographic details, licenses, and code availability should be verified before implementation.

## Learned physical motion completion

- NVIDIA `MaskedMimic`
  - sparse targets and partial-body constraints;
  - full-body physical motion completion.
  - https://research.nvidia.com/labs/par/maskedmimic/

- NVIDIA `MaskedManipulator`
  - sparse whole-body manipulation intent;
  - hand/head/object constraints.
  - https://research.nvidia.com/labs/par/maskedmanipulator/

- NVIDIA `CALM`
  - controllable low-dimensional motion/style latent.
  - https://research.nvidia.com/labs/par/calm/

## Shared autonomy and latent action control

- Research on low-dimensional shared control for high-dimensional robotic systems.
- Relevant themes:
  - latent action spaces;
  - user correction;
  - context-conditioned mappings;
  - shared autonomy.
- https://arxiv.org/abs/2107.02907

## Production control precedents

- `For Honor`
  - directional attack/guard readability.

- `Skate`
  - analog semantic control and multiple control profiles.

- `Sifu`
  - distinct semantic defensive actions and readable multi-opponent design.

- `Overgrowth`
  - combination of procedural, physical, and authored motion rather than purely procedural control.

These are interface precedents, not direct implementations of the proposed neural physical avatar architecture.

## Physical skill and transition research

Relevant directions include:

- universal physics-based character controllers;
- structured combat motion priors;
- transition randomization;
- user-controlled physical sports skills;
- contact-rich interaction and grappling.

The implementation agent should verify current 2025–2026 papers before selecting dependencies or reproducing claims.

---

# 162. First prototype recommendation

The first prototype should not begin with a full RPG combat system.

Build:

```text
physical humanoid
+
locomotion intent
+
right-hand task target
+
Agency Arbiter
+
contract-conditioned whole-body completion
```

Scenario:

1. player walks around;
2. player points the right hand toward arbitrary targets;
3. body keeps balance and selects footwork;
4. player pushes objects;
5. player cancels and redirects;
6. system logs intervention and intent error.

This prototype answers the foundational question:

> Can the player feel that they directly control an embodied hand and intention, while trusting the model to control the rest of the body?

Only after this feels good should the project add sword combat.

---

# 163. Second prototype recommendation

Implement a one-handed sword duel with only:

- locomotion;
- focus target;
- continuous strike direction;
- target region;
- guard vector;
- effort;
- commit/release;
- explicit cancel;
- explicit evade.

No large move list.

Measure:

- agency;
- predictability;
- control latency;
- strike accuracy;
- learning time;
- physical quality.

---

# 164. Core project thesis

> **Shared Embodiment Control turns the player's low-dimensional input into explicit task-space contracts while a physics-trained motor system supplies only the redundant coordination required to execute them. The player remains the tactical and semantic author of action; the neural body becomes an adaptive motor instrument rather than an autonomous agent.**

---

# 165. Final recommendation

Do not choose between:

```text
Souls-like action buttons
```

and:

```text
manual joint puppeteering
```

Build a layered control language:

```text
default:
    move
    look
    target
    direction
    effort
    timing

technique:
    target region
    guard sector
    action plane
    commitment
    follow-through

precision:
    hand pose
    weapon tip
    grip
    exact contact
```

All layers compile into the same `SkillContract` representation.

The motor system then solves:

```text
feet
balance
joint coordination
contact timing
biomechanical execution
recovery
```

under strict `AgencyArbiter` permissions.

This architecture can expose the expressive power of neural physical characters without forcing the player to control every limb and without reducing the body back to a library of animation clips.
