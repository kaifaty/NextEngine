# TRAIN-4 causal research decision — 2026-08-14

| Field | Value |
|---|---|
| Status | `COMPLETE / OPTIMIZER_FREE` |
| Decision | Build a bounded contact-consistent reference/reset prototype before another full corpus identity |
| Requirement | `REQ-HUM-DATA-005`, `REQ-HUM-DATA-007` |
| Source audit | V18/R14, `12518/12518`, SHA-256 `ded76473905f7f26e4db0dfaa236508f649d46a692ff28938203f92dcf1c9b61` |
| Result | TRAIN-4 remains `FAIL`; TRAIN-5 and optimizer execution remain forbidden |

This report closes the bounded research checkpoint opened after repeated
ankle/contact remediations moved the failure counts without reaching exact
zero. It is a decision record, not an Accepted ADR, a new corpus identity or a
gate result. Generated reports, corpus bytes and captures remain in the
external training store.

## Guard rails and question

The frozen question was: which smallest intervention can distinguish a local
controller defect from an invalid reference/reset/contact state without
weakening any hard safety criterion?

The research kept the V18 BodySchema, USD, fixed PD, safety thresholds,
11-motor-tick horizon and all `12518` canonical start cases. Every intervention
used zero learned action, executed no optimizer and was compared with a
zero-residual control that had to reproduce R14 case-by-case on status,
required-safety reasons and terminal tick. Formal visual review remains
pending and cannot waive a safety failure.

Frozen input identities were V18 corpus profile
`91d60064444f155084ec0b793876b208336747b7dcaa03712290ca1ef9bdb4b0`,
canonical corpus manifest
`bd6164180a33585ed9231fd6a41cb3943b20e51da8b0e06ac0c978406e204456`,
tracker V11
`19a8e5266f12c5c1ae9e307bd0f2f1a76bb6ea23a6ae9912e1ccec401eeaf28c`,
descriptor
`f1f2be6a486367038f605709ebf54edb4fa6ef400fa797dd772d7590e06f3014`,
USD `951fc04bed0ff3e15414bfd06d067636ee8db18ef70e6e4a0ced28c624fcefa4`
and TRAIN-4 remediation gate
`2fc1d6c3d312a1e88a3820bdd851fd0eecf3230795a1162b55f13dd65d45cdb6`.

## Baseline localization

R14 completes the whole canonical schedule and leaves `204` failed cases:

- `121` hard-ROM cases, localized to ankle pitch (`61` left, `73` right joint
  events);
- `67` hard-impact cases, localized to the sole/ankle ground pairs (`37` left,
  `30` right);
- `40` overlapping joint-safety/joint-velocity cases, localized to ankle roll
  (`27` left, `13` right);
- zero effort-envelope, fall, forbidden-contact, self-collision, world-bound
  or non-finite failures.

Only `10` cases fail in the declared reset window, but the remaining failures
are still early: hard-ROM occurs at ticks `8..11`; hard-impact begins at tick
`1`; ankle-roll velocity failures occupy ticks `3..11`. This clustering is
consistent with one coupled ankle/sole transient rather than independent
corpus-wide failures.

## Code and semantics audit

The implementation audit found four relevant facts.

1. [`NextEngineReferenceDirectEnv`](../../lab/next_lab/isaac_reference_env.py)
   writes exact reference root pose/velocity and joint pose/velocity on reset,
   initializes the applied target to the reset pose, then commands the current
   reference and advances the cursor before four physics substeps. At the first
   exact-reference tick, position error is zero, so the fixed controller begins
   with damping against the non-zero reference joint velocity.
2. The environment calls Isaac Lab `write_root_state_to_sim`. In the pinned
   Isaac Lab 2.3.2 implementation this writes a root-link pose but interprets
   the supplied velocity as root **center-of-mass** velocity. The corpus
   `root_linear_velocity_um_s` is the finite difference of the retargeted root
   link/pelvis position. This is a real frame-semantics defect; the root-link
   velocity writer is the semantically correct API.
3. [`_detect_contacts`](../../lab/next_lab/motion_retarget.py) independently
   takes the minimum heel/forefoot height and the minimum heel/forefoot speed.
   The two minima can come from different points, so a declared sole contact
   need not have one physical point that is both low and slow. V18 then admits
   entry/retention speeds of `0.6/0.9 m/s` and validates declared support up to
   `1.0 m/s` planar and `0.75 m/s` vertical in the
   [V18 corpus profile](../../lab/profiles/humanoid-motion-corpus-cmu-ankle-pitch-velocity-closure.v18.json).
4. [ADR-070](../architecture/adr/070-biomechanics-reference-tracking-training-environment.md)
   says a reset constructs a fresh PhysX scene at the selected state and
   forbids post-create root teleport. The current vector environment instead
   performs indexed root/joint writes into a running scene and clears Isaac
   Lab sensor/actuator buffers. This is an architecture/implementation
   contradiction; clearing a contact sensor buffer is not evidence that the
   underlying PhysX contact state is equivalent to a fresh scene.

The fourth item is deliberately recorded as an unresolved causal possibility.
PhysX documents persistent contact manifolds and internal contact caches, but
that alone does not prove that a particular indexed reset reuses a harmful
manifold. A paired fresh-scene experiment is required before changing the
Accepted reset contract.

## Counterfactual matrix

The committed
[`isaac_reference_causal_probe.py`](../../lab/scripts/isaac_reference_causal_probe.py)
replayed the complete R14 schedule. All three reports first matched
`12518/12518` baseline cases with zero mismatches.

Counts below are required-safety **case/event counts**; categories can overlap.
`Recovered` and `regressed` compare case status with R14.

| Intervention | Failed | Impact | ROM | Velocity | Self collision | Recovered | Regressed | Decision |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| zero residual control | 204 | 67 | 121 | 40 | 0 | 0 | 0 | exact reproduced control |
| target lead 1 tick | 203 | 54 | 125 | 47 | 0 | 69 | 68 | reject: churn, no closure |
| target lead 4 ticks | 242 | 72 | 121 | 71 | 0 | 86 | 124 | reject |
| target lead 6 ticks | 265 | 78 | 109 | 104 | 0 | 99 | 160 | reject |
| `D/K * qdot` feed-forward | 265 | 95 | 121 | 79 | 1 | 89 | 150 | reject |
| zero joint reset velocity | 208 | 54 | 122 | 46 | 1 | 74 | 78 | reject |
| zero root reset velocity | 179 | 150 | 6 | 23 | 0 | 190 | 165 | reject: ROM becomes impact |
| zero all reset velocities | 52 | 15 | 7 | 30 | 0 | 194 | 42 | diagnostic only; invalid reference-state initialization |
| root-link velocity writer | 220 | 65 | 129 | 49 | 0 | 46 | 62 | semantic fix confirmed, not sufficient |
| root-link velocity projected by declared stance speed | 194 | 108 | 64 | 32 | 0 | 152 | 142 | supports contact coupling; approximation rejected |

External report identities under
`/home/kaifaty/NextEngine-training/generations/humanoid-motor-rebuild-v1/evaluations/TRAIN-4/`:

- `causal-counterfactual-v18-r15.json`, controller/reset matrix:
  `a49f84b98e8b642933492a8eadafbb8b0d915d09777e6f0ea272978231ec4a7a`;
- `root-link-counterfactual-v18-r16.json`, root-link semantics:
  `9ae54fadeb63b286b3d46a8d529e4488dcfeba2d71307c8e6c2f6ec21fa6c97a`;
- `contact-projected-reset-counterfactual-v18-r17.json`, contact-projected reset:
  `60ce03b6fbc2f213d82c5f95b0f4e831ca81dd50b52af1750890d44566cfc25c`.

The important result is not the lowest raw count. Zeroing all velocities
changes the reference state and lets the fixed controller start from an
artificially static body; it therefore cannot satisfy reference-state
initialization. Root-only zeroing and contact projection visibly transfer
failures between ROM and impact and regress many passing controls. They are
causal probes, not candidate fixes.

## Contact-trajectory evidence

An offline event/NPZ correlation pass supplied supporting, not dispositive,
evidence. It used the emitted heel/forefoot `effector_position_um`, defined the
sole center as their integer mean, differentiated it at the corpus's exact
60 Hz cadence, and joined each R14 start/horizon with the emitted per-foot
contact flags:

- cases whose maximum declared-contact sole-center planar speed lies in
  `0.8..0.9 m/s` fail `70/763` (`9.17%`), versus `134/11643` (`1.15%`) below
  `0.8 m/s`; the final `0.9..1.0 m/s` bin contains `112` passes, so the relation
  is not monotonic and must not be treated as a standalone classifier;
- `cmu05-walk-validation`, `cmu91-walk-slow-validation` and
  `cmu139-walk-heldout` declare double support for approximately `41%`, `54%`
  and `50%` of frames respectively, while `cmu16-walk-nominal-b` declares
  approximately `0.4%`; broad per-clip variation is not currently constrained
  against source contact phase;
- affected hard-ROM sides have median initial planar sole-center speed about
  `0.68 m/s`, while cases combining ROM and impact are about `0.89 m/s`.

These measurements explain why another ankle-limit or smoothing adjustment is
unlikely to be the right next move: the reference can be statically within ROM
while asking PhysX to reconcile a moving foot with a declared sticking support
and a reset velocity field that does not satisfy the same contact constraint.

## Hypothesis disposition

| Hypothesis | Evidence | Disposition |
|---|---|---|
| Target indexing or missing PD velocity feed-forward is primary | Lead/feed-forward modes recover some failures but regress equally or more controls and increase total failures. | `REJECTED_AS_PRIMARY` |
| Joint reset velocity alone is primary | Zero joint velocity is worse overall and adds a collision. | `REJECTED_AS_PRIMARY` |
| A statically valid reference is dynamically invalid only because of actuator envelope | The baseline has zero effort-envelope violations, while failures track contact/ROM/velocity and respond to root/contact interventions. | `REJECTED_AS_PURE_ACTUATOR_CAUSE`; dynamic contact viability remains supported |
| Root velocity participates in the transient | Root/all-velocity interventions radically change outcomes and transfer ROM to impact. | `SUPPORTED`, but naive zeroing rejected |
| Root link/CoM semantics are wrong | Direct code/API mismatch; root-link writer is semantically correct. Corrected run still has `220` failures. | `CONFIRMED_DEFECT`, not sufficient cause |
| Retargeted contact mode and velocity field are inconsistent | High declared support speeds, unconstrained duty/double-support structure, mixed-point classifier and contact projection response all point here. | `STRONGLY_SUPPORTED`, next implementation target |
| Running-scene partial reset differs materially from fresh-scene reset | Accepted ADR and implementation conflict; PhysX maintains contact manifolds/caches. No paired result yet. | `OPEN`, next discriminating experiment |
| More local ankle reserve/smoothing is the next best intervention | V15–V18 already moved the symptom; remaining categories are coupled and counterfactuals churn controls. | `REJECTED_FOR_NEXT_INCREMENT` |

The interventions establish causality of reset/reference velocities, but do
not prove a unique root cause. The chosen next step targets the smallest layer
that can jointly explain foot impact, ankle-roll excitation and late
plantarflexion ROM without relaxing safety.

## Primary-source findings

- Isaac Lab's [DirectRLEnv source](https://isaac-sim.github.io/IsaacLab/v2.3.0/_modules/isaaclab/envs/direct_rl_env.html)
  defines action preprocessing, decimated physics, terminal evaluation and
  indexed autoreset ordering. Its current
  [environment API](https://isaac-sim.github.io/IsaacLab/develop/source/api/lab/isaaclab.envs.html)
  also warns that vectorized environments do not independently recreate each
  sub-environment.
- The official Isaac Lab
  [articulation reset tutorial](https://isaac-sim.github.io/IsaacLab/main/source/tutorials/01_assets/run_articulation.html)
  writes root pose, root velocity, joint pose and joint velocity, then resets
  internal buffers. The
  [articulation implementation](https://isaac-sim.github.io/IsaacLab/main/_modules/isaaclab/assets/articulation/articulation.html)
  distinguishes root-link velocity from root-CoM velocity.
- PhysX [articulation documentation](https://nvidia-omniverse.github.io/PhysX/physx/5.6.0/docs/Articulations.html)
  defines stiffness against target position and damping against target
  velocity. This supports the first-tick damping audit; it does not make the
  rejected feed-forward intervention admissible.
- PhysX documents that
  [persistent contact manifolds reuse contacts from previous frames](https://nvidia-omniverse.github.io/PhysX/physx/5.1.2/docs/AdvancedCollisionDetection.html)
  and that [contact caches are enabled by default](https://nvidia-omniverse.github.io/PhysX/physx/latest/_api_build/structPxSceneFlag.html).
  Inferring reset contamination from those facts is a hypothesis, not a
  documented guarantee.
- Standard rigid-contact mechanics models a sticking contact with a shared
  position constraint and zero relative tangential velocity; see MIT's
  [multibody contact formulation](https://underactuated.mit.edu/multibody.html).
  DeepMimic's
  [reference-state initialization](https://xbpeng.github.io/projects/DeepMimic/DeepMimic_2018.pdf)
  likewise initializes from the full reference state rather than discarding
  velocities.
- Recent primary research independently reports that purely kinematic
  retargeting introduces foot sliding/penetration and that enforcing dynamics
  and contact constraints improves downstream feasibility; see
  [Kinodynamic Motion Retargeting](https://arxiv.org/abs/2603.09956) and
  [SPIDER](https://arxiv.org/abs/2511.09484). These papers support the direction,
  not the project-specific thresholds or a claim that V18's cause is proven.

## Decision: bounded V19 contact-manifold prototype

Do not create another whole-corpus ankle/PD identity yet. The next immutable
implementation candidate is a bounded V19 prototype with these rules:

1. Represent per-foot contact mode explicitly as `Flight`, `HeelSticking`,
   `ForefootSticking` or `FlatSticking`. A single physical point must satisfy
   both height and speed predicates; approach/exit hysteresis cannot be
   reported as already-sticking support.
2. For every sticking point, solve root and stance chain together so the final
   trajectory approximates `h(q)=0` and `J(q)v=0`. Recompute root/joint
   velocities only after the final pose/root correction, then cross-check
   analytic point velocity against finite differences of the emitted effector
   trajectory.
3. Freeze a conservative prototype tolerance before PhysX results are viewed:
   at 60 Hz, maximum sticking-point tangential displacement is `2 mm/frame`
   (`0.12 m/s`) and normal displacement is `1 mm/frame` (`0.06 m/s`), with a
   maximum `5 mm` normal position residual. The theoretical target remains
   zero; these are quantization/discrete-sample tolerances, not permission for
   skating. A changed tolerance creates a new prototype identity and rationale.
4. Use root-link velocity semantics. Do not hide a contact impulse with zero
   velocities, an uncounted settling interval, a safety grace window or a
   looser impact/ROM/velocity bound.
5. Pair the same selected start states under a freshly constructed scene and
   the current indexed partial reset. If case outcomes or first-tick contact
   impulses differ, stop and resolve ADR-070 through the architecture workflow
   before a full V19 build. If they are equivalent, record the evidence and
   then propose the smallest ADR clarification that permits the proven reset.

The bounded prototype uses mixed failure mechanisms from
`cmu05-walk-validation`, `cmu16-walk-nominal-b` and
`cmu139-walk-heldout`, plus start-phase-matched passing controls. It advances to
a full 27-clip rebuild only when:

- every selected contact satisfies the new pointwise position/velocity
  invariants;
- no previously passing control becomes a required-safety failure;
- every targeted failure class strictly decreases without introducing a new
  required-safety category;
- fresh-scene versus partial-reset disposition is explicit and
  architecture-compliant.

The eventual full identity still requires deterministic import, all local and
native checks, formal visual review, complete exhaustive phase coverage and
exactly zero required-safety events. Until then `REQ-HUM-DATA-007` is `Fail`,
TRAIN-4 is reopened, and optimizer/training runs remain unauthorized.
