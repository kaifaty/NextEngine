# Locomotion prior art: recover a working end-to-end baseline

2026-09-05. Research only, requested after repeated body/control diagnostics.
Local source: `7e0f20c2`. No new physics experiment, dependency installation,
training, checkpoint selection or Accepted contract change in this work.

## Outcome

Established humanoid learning implementations already supply the coordinated
body, actuator, reset, observation, reward and optimizer setup we need as a
control. The missing discriminator is a reproduced upstream end-to-end result,
not another isolated gain candidate. This is a workflow recommendation, not
proof that our body or explicit PD cannot learn.

Recommend a small **MimicKit human imitation baseline** before further body
changes. Keep ProtoMotions as the relevant richer Kimodo integration reference,
not a second mandatory framework migration. Existing Isaac Lab G1 can serve as
an infrastructure fallback, but robot success is not human-body validation.

## What the sources establish

Sources inspected directly on this date; remote `main` is mutable. HEADs
resolved read-only: MimicKit `2ed1e6c093bb0829f55d33cb4f7a1731cfe6cb69`,
ProtoMotions `607ca7a0bb92e261120bcab8d9f97f28b3130ffc`. These identify the
inspection, not a locally tested/recommended dependency combination.

1. [DeepMimic (2018), sections 5, 8 and 10.4](https://xbpeng.github.io/projects/DeepMimic/DeepMimic_2018.pdf):
   example-guided RL with PD target actions, reference-state initialization and
   fall termination. Its humanoid has 13 links, 34 total DOFs including the
   free root; spherical joints except knee/elbow hinges. Walking succeeds in
   the reported ablations even without RSI; RSI is useful, not a universal
   prerequisite. This is established prior art, not our reproduction.
2. [MimicKit](https://github.com/xbpeng/MimicKit): human imitation implementations,
   trained models and training logs, with separate simulator environments.
   [DeepMimic instructions](https://github.com/xbpeng/MimicKit/blob/2ed1e6c093bb0829f55d33cb4f7a1731cfe6cb69/docs/README_DeepMimic.md)
   describe single-clip training. The inspected default is a **spinkick**, not
   walking; a walking clip/checkpoint must be located and its provenance checked
   before promising a ready walking demo. Assets/models are an external download.
   [Human asset](https://github.com/xbpeng/MimicKit/blob/2ed1e6c093bb0829f55d33cb4f7a1731cfe6cb69/data/assets/humanoid/humanoid.xml)
   has limited hinge axes, rigid box feet and no articulated toes. It is an
   animation-oriented approximation, not a biomechanical gold standard.
3. [Isaac Lab actuator documentation](https://isaac-sim.github.io/IsaacLab/develop/source/concepts/actuators.html)
   distinguishes explicit effort models from implicit solver drives. Both are
   legitimate; gains cannot be transferred without considering timestep,
   limits and dynamics. Implicit drives can still be unstable with contacts.
   Current develop documentation is not the API contract of our installed version.
4. [PhysX 5.4 articulation drive documentation](https://nvidia-omniverse.github.io/PhysX/physx/5.4.0/docs/Articulations.html):
   implicit drives solve against end-of-step state, unlike held external effort.
   This is explanatory prior art, not a native 5.9 compatibility test. Force
   telemetry and force/impulse limit semantics require version-specific care.
5. [legged_gym production environment](https://github.com/leggedrobotics/legged_gym/blob/master/legged_gym/envs/base/legged_robot.py):
   `step` recomputes clipped PD torques each physics step; `check_termination`
   uses selected body contacts and timeout, while soft joint limits contribute
   rewards. This refutes the blanket claim that explicit PD is inappropriate
   for locomotion. It does not validate our gains or safety envelope.
6. [ProtoMotions](https://github.com/NVlabs/ProtoMotions) provides human/robot
   imitation and retargeting. Its current README identifies an Isaac Lab 3.0
   commit, not our installed 2.3.2. Do not upgrade the active environment to try it.
   Code licensing does not automatically license external bodies, data or weights.
7. [Kimodo project](https://research.nvidia.com/labs/sil/projects/kimodo/)
   describes kinematic motion generation, not a torque controller.
   [ProtoMotions conversion guide](https://github.com/NVlabs/ProtoMotions/blob/607ca7a0bb92e261120bcab8d9f97f28b3130ffc/docs/source/getting_started/kimodo_preparation.rst)
   explicitly connects Kimodo SOMA/G1 outputs to motion conversion and physical
   policy training. It includes examples and heuristic contact labeling. This
   supports Kimodo as a reference-motion source, not guaranteed feasible motion
   or plug-compatible NextEngine weights. Rendered docs failed to open; the
   actual upstream RST was read instead.

## Comparison with our implementation

| Boundary | Exact local evidence | Consequence / recommended test |
| --- | --- | --- |
| Body complexity | Current V8: 25 actuated DOFs, articulated forefoot; prior axis/inertia defects repaired (task-state reports 4–16) | No evidence that more joints are the next requirement. A simpler rigid-foot upstream body is a control, not grounds to erase V8. |
| Control | `crates/motor/src/safety_control.rs::step_substep` computes/clips external PD effort; native bridge applies cache joint forces. `lab/next_lab/isaac_reference_env.py` also uses external effort with solver gains zero. | Not an unimplemented PhysX PD drive. Compare the complete upstream controller first; implicit conversion would change effort ownership and needs separate approval/validation. |
| Reset / first failure | Startup-ramp report: knees start at hard minimum; first output -14/-13 microradians, tolerance 10. | This particular result is a limit-check failure, not observed loss of balance. Physical cause remains unresolved; distinguish reset validity, numerical residual and task failure. Do not silently loosen limits. |
| Training route | `lab/next_lab/isaac_reference_env.py` already contains reference clips, phase scheduling, reset and tracking rewards. | We are not missing the idea of imitation. Missing evidence is an upstream control separating custom substrate/retargeting failures from learning failures. |
| Readiness tests | V8 nominal 30-second support passes; manually prescribed loaded transfer fails; V9 also fails before transfer input. | These establish bounded failures, not impossibility of learned walking. A scripted heel-rise success should not be asserted as a universal RL prerequisite. Existing admission remains unchanged pending an explicit decision. |

Installed Isaac Lab was inspected without importing/running simulation:
`/home/kaifaty/NextEngine-training/isaaclab-2.3.2/IsaacLab`, clean HEAD
`37ddf626871758333d6ed89cf64ad702aef127d0`, `git describe` = `v2.3.2`.
Its G1 locomotion uses `G1_MINIMAL_CFG` derived from `G1_CFG`, not `G1_29DOF_CFG`.
`source/isaaclab_assets/isaaclab_assets/robots/unitree.py` sets knee 0.42 rad,
hip pitch -0.20 and ankle pitch -0.23, implicit drives and soft limit factor 0.9.
The G1 rough task preserves those joint positions. Base velocity task runs
physics at 200 Hz and policy at 50 Hz, uses default-relative position targets,
timeout/body-contact termination; G1 adds ankle soft-limit and foot-slide costs.
Those values are evidence of a coordinated setup, **not gains/poses to copy
into a different human body**. MimicKit currently names another tested Isaac Lab
commit; compatibility with our installed environment has not been demonstrated.

## Competing explanations and evidence update

- **Insufficient anatomy/DOFs is the dominant blocker:** not supported by the
  failures at the first or eighteenth step, nor by simpler upstream models.
  Still possible that our particular mass/contact projection is problematic.
  Reconsider on a matched body substitution that alone destroys a working baseline.
- **Explicit PD is intrinsically unsuitable:** refuted as a general claim by
  legged_gym; our particular sampled/clipped controller remains suspect. A local
  linear model passing is not a contact-rich stability guarantee (V9 witness).
- **Reset and strict terminal boundaries censor useful tests:** supported for
  the startup-ramp comparison only. It does not explain all historical failures.
  Quantify first-failure clustering on the next admitted evaluation; do not
  reinterpret any previous FAIL as PASS.
- **Too many custom layers lack an end-to-end control:** strongest actionable
  workflow explanation. Prior reports isolate defects but do not show which
  complete known-learning setup our current system can reproduce.

## Smallest next action (proposal, not executed)

1. Locate one upstream human walking clip and compatible model/config bundle;
   verify asset/data terms and simulator pin. Use a separate external environment.
   First replay an available trained example, then reproduce a single-clip walk
   with upstream defaults. If only spinkick weights are available, label that
   playback an infrastructure check, not walking success. Do not expand into
   whole-dataset learning or migrate the current training environment.
2. Record visible motion plus episode completion, displacement/speed, tracking
   error, contacts/sliding and first failure; retain learning curves and a fixed
   evaluation set. A replayed pretrained model proves inference, not local training.
   This is a control, not a new gate requiring another broad numerical campaign.
3. Only after the control works, choose one substitution: our body with matched
   retargeted references and the reference control loop. Then test our effort
   path separately. CPU runtime admission remains a later explicit comparison,
   not a presumed consequence of GPU success. Request the necessary contract
   decision before changing accepted actuator/safety/admission semantics.

Retain corrected axes, mass tensors, collision visualization, force-schedule
evidence and old checkpoints as controls. Pause additional damping sweeps,
joint-count increases and mandatory manual-transfer refinements. Reconsider
them only after a matched comparison implicates that boundary. Remaining risks:
upstream assets/pins may block reproduction; retargeting is real work; a stock
human's successful gait does not prove anatomical fidelity or engine portability.

Validation: read-only local source/version inspection and primary-source research
performed. No optimizer, simulator or dependency installation run. This report
changes no ProductCheck outcome and does not declare walking solved.
