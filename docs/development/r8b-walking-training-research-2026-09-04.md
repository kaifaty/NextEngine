# R8b walking training research — 2026-09-04

## Question and observed failure

The first forward start/stop run learned safe standing rather than locomotion:
the final GPU policy achieved `0.084 m` against `3.944 m` commanded. At the
steady `0.5 m/s` command, the V1 compact tracking term still paid
`(1 - 0.5 / 2.5)^2 = 0.64`; independent upright, height and yaw bonuses then
made stationary survival a strong local optimum.

## Primary-source findings

- NVIDIA Isaac Lab defines planar velocity tracking as an exponential kernel
  over squared command error. Its current H1 locomotion configuration uses
  `std = 0.5`, positive linear/yaw tracking, a biped feet-air-time term and
  negative orientation/action-rate costs rather than independent positive
  survival bonuses:
  <https://isaac-sim.github.io/IsaacLab/develop/source/api/lab/isaaclab.envs.mdp.html>,
  <https://github.com/isaac-sim/IsaacLab/blob/release/3.0.0-beta2/source/isaaclab_tasks/isaaclab_tasks/manager_based/locomotion/velocity/config/h1/rough_env_cfg.py>.
- The official Isaac Lab direct locomotion implementation uses
  `exp(-squared_error / 0.25)`, gates feet-air-time reward on a non-zero
  command and tracks actual episode-mean velocity error:
  <https://github.com/isaac-sim/IsaacLab/blob/release/3.0.0-beta2/source/isaaclab_tasks/isaaclab_tasks/direct/anymal_c/anymal_c_env.py>.
- Rudin et al. report that simple observations/actions/rewards can learn flat
  locomotion with massively parallel PPO, but a biped needs an additional
  single-foot signal for a walking gait; they also use a staged curriculum:
  <https://arxiv.org/abs/2109.11978>.

## Competing hypotheses

| Hypothesis | Prediction | Result before new optimization |
| --- | --- | --- |
| H1 — V1 tracking is too permissive | A stationary body receives a large fraction of the moving optimum | Confirmed: `0.64` raw planar tracking at `0.5 m/s` error, plus independent posture bonuses |
| H2 — PPO/exploration is the first failure | Metrics should diverge, collapse exploration or fail to improve survival | Not supported: all 250 records are finite, action noise remains about `0.34`, and episode length rises materially |
| H3 — evaluation does not prove start/stop | The generated command path may omit the required final stop | Confirmed: V1 counter schedules do not guarantee a final 180-tick zero interval |
| H4 — the body cannot produce a biped gait | A corrected objective still cannot acquire signed forward travel | Open; only a bounded successor run can discriminate it |

## Smallest evidence-backed successor

Freeze a distinct V2 profile; do not mutate V1 or tune PPO.

- Use one deterministic first curriculum lesson: 120-tick warm-up, ramp to
  `0.5 m/s`, ramp down early enough to provide exactly 180 final zero-command
  ticks. Its integrated commanded distance is greater than `3 m`.
- Replace the `2.5 m/s` tracking width with a deterministic Q16 compact-square
  kernel of width `0.5 m/s`. This approximates the discrimination intent of
  the exponential kernels while preserving exact CPU/GPU integer parity.
- Make planar/yaw tracking the positive task reward; express tilt and height
  as costs. Preserve bounded effort, target-rate, slip, one-sole support and
  fall facts.
- Keep the model, optimizer and PPO hyperparameters unchanged; initialize
  only actor/critic and observation-normalizer weights from the admitted
  standing checkpoint.

Before training, exact goldens must show that stationary motion earns at most
10% of the ideal moving reward, the schedule commands at least `3 m`, and its
last 180 commands are zero. Canonical CPU zero-action and standing-parent
controls must fail motion acceptance. A trained candidate passes only if all
five canonical CPU episodes time out safely, each achieves at least `3 m`
signed forward travel, and mean absolute forward speed during the final stop
is at most `0.15 m/s`. Failed `MODEL-MIRROR-P1` continues to prohibit runtime
promotion regardless of this result.

## V2 outcome and bounded belief update

V2 completed all `1,024,000` transitions with finite metrics. Its final GPU
checkpoint completes five `1,199`-step episodes without a safety terminal and
stops with mean final-180-tick speed MAE `0.0216 m/s`, but achieves only
`0.123 m` against `7.258 m` commanded. On canonical CPU the final checkpoint
falls at tick `116`; checkpoint 25 survives all five horizons but moves
`-0.176 m`. No saved checkpoint walks.

This refutes H4 for the compact objective, not for the body/controller. At the
initial stationary error, V2's width equals the `0.5 m/s` command, so
`square(max(0, 1 - error / width))` is exactly zero and locally flat on the
wrong side. The final GPU component mean (`0.226`) is explained by the
zero-command warm-up and stop rather than forward tracking.

ADR-105 therefore selects one minimal V3 counterfactual. It changes only the
tracking kernel to the exact Q16 dense bounded function
`square(1 / (1 + (error / width)^2))`. At `0.5`, `0.25` and `0 m/s` error its
raw planar reward rises strictly from `16384` to `65536`, so PPO receives a
monotonic signal from the inherited standing policy. The command schedule,
coefficients, PPO, seed, body, controller, safety and acceptance gates remain
unchanged. Failure next distinguishes reward sparsity from a command-
curriculum, action or body limitation.

## V3 outcome and stop decision

V3 completed all `1,024,000` samples. The final deterministic GPU evaluation
(manifest SHA-256 `1fe4c2cc…b81`) completed all five `1,199`-step horizons with
zero declared safety terminal and final-stop speed MAE `0.0185 m/s`, but mean
forward travel was only `0.104 m` against `7.258 m` commanded. The canonical
CPU evaluation (manifest SHA-256 `2dbe861e…968`) terminated all five episodes
at tick `156` with `terminal.fall`, mean signed displacement `-0.822 m` and
final pitch about `-59.85°`.

The dense kernel fixes V2's local zero-gradient defect but does not produce a
gait. That rejects reward-density alone as the next explanation and completes
three failed command-only walking cycles. No further full optimizer run or PPO
tuning is authorized until a bounded research cycle discriminates missing
contact-phase/gait curriculum, action/observation limitations and the already
failed CPU/Isaac dynamics correspondence using smaller successful controls.

## Post-V3 causal audit

### Exact policy/contact evidence

The final V3 checkpoint was recorded for one `600`-tick Isaac episode in the
external training store. The NPZ has SHA-256
`ee9bc8386f0f473fa501afff5d8c70b972404558ed481d015e21c1a5f40e224f`.
The first non-zero forward command occurs at tick `120`, but both soles remain
in contact for all `600/600` ticks: there are zero left-only, right-only or
flight samples and zero contact-state switches. During the moving interval the
mean forward velocity is `0.00620 m/s`, velocity MAE is `0.47872 m/s`, and
total forward displacement is only `0.05967 m`.

This also explains the apparently non-zero V3 support component. The reward is
a binary predicate: exactly one contacting sole while any command is non-zero,
or exactly two soles while the command is zero. It has no air/contact duration,
touchdown, clearance or alternation term. The learned rollout receives support
credit only during the zero-command portions; it never enters the rewarded
moving support state.

The policy is not globally action-saturated: moving mean absolute action is
`0.2716` and only `1.80%` of action samples are saturated. One suspicious
exception is torso yaw, whose mean action is `0.9086` and which is saturated in
`41.37%` of moving samples despite a zero yaw command. Therefore exploration
collapse or global output clipping is not the first failure.

### Walking action base conflicts with translation

The walking profile does not act around a neutral/default joint pose. It adds
small residuals to `procedural-standing.v1`. That standing controller computes
both ankle-pitch targets from the absolute world-forward displacement from the
reset position:

`ankle = -0.14 + pitch/2 + pitch_rate/20 + forward_position/10 + forward_velocity/50`.

At an upright `0.5 m/s` target, the walking ankle residual is only
`+/-0.15 rad`. Ignoring pitch feedback for the discriminator, the reachable
ankle-pitch target interval therefore changes with travelled distance:

| Forward displacement | Standing reference | Reachable target after residual/soft ROM |
| ---: | ---: | ---: |
| `0 m` | `-0.13 rad` | `[-0.28, 0.02] rad` |
| `1 m` | `-0.03 rad` | `[-0.18, 0.12] rad` |
| `2 m` | `0.07 rad` | `[-0.08, 0.22] rad` |
| `3 m` | `0.17 rad` | `[0.02, 0.32] rad` |
| `5 m` | `0.37 rad` | `[0.22, 0.436] rad` |
| `7 m` | `0.57 rad` | `[0.42, 0.436] rad` |

The acceptance gate requires at least `3 m`, while the fixed command integrates
to `7.258 m`. Thus the action meaning drifts as the character moves and nearly
loses ankle authority by the end of the commanded path. Absolute root position
is not an explicit policy observation, so the policy must infer this hidden
controller state indirectly from the previous applied target. This is a
structural locomotion-contract conflict, not a PPO hyperparameter issue.

The official Isaac Lab velocity environment instead applies joint-position
actions around the robot's default joint offsets, with a generic `0.5 rad`
scale. Its H1 profile adds dense velocity tracking, biped feet-air-time and
feet-slide terms; it does not reuse a world-position-anchored standing
controller. Unitree's H1 profile is another valid design point: `0.25 rad`
default-offset actions, an explicit `0.6 s` gait phase, phase-conditioned foot
gait reward, foot-clearance reward and command curriculum:

- <https://github.com/isaac-sim/IsaacLab/blob/release/3.0.0-beta2/source/isaaclab_tasks/isaaclab_tasks/manager_based/locomotion/velocity/velocity_env_cfg.py>
- <https://github.com/isaac-sim/IsaacLab/blob/release/3.0.0-beta2/source/isaaclab_tasks/isaaclab_tasks/manager_based/locomotion/velocity/config/h1/rough_env_cfg.py>
- <https://github.com/unitreerobotics/unitree_rl_lab/blob/main/source/unitree_rl_lab/unitree_rl_lab/tasks/locomotion/robots/h1/velocity_env_cfg.py>

This does not mean a phase clock is mandatory: the official Isaac Lab H1
profile learns without one. It means that successful systems provide either a
translation-invariant direct action basis plus duration-aware contact credit,
or an explicit gait phase/foot objective. Current V3 provides neither.

### Optimizer-free reachability and CPU split

Two bounded canonical CPU probes were run without optimization:

1. `128` slots received zero action for `30` ticks and then either a fixed
   full-range leg residual or a deterministic random full-range leg vector
   through tick `90`. The zero-action control survived. No slot reached
   single-sole support; `83` survived, `36` ended in self-collision and `9` in
   joint safety.
2. `64` slots used the admitted final standing policy as a successful
   stabilizing control. Eight unchanged controls and `56` deterministic
   alternating hip-pitch/knee/ankle-pitch/hip-roll overlays were run for `120`
   ticks. All `64` survived, but none reached single-sole support. The best
   forward displacement was `0.02499 m`; unchanged controls averaged
   `0.01245 m`.

These probes do not prove that a time-varying policy can never lift a foot.
They do refute the assumption that the current narrow residual basis makes an
alternating support state readily reachable around the standing controller.
The body itself still has separate anatomical hip, knee and ankle axes, real
ROM and non-spherical leg/foot geometry, so an incapable body is not the
leading hypothesis.

A separate one-episode CPU trace of the final V3 policy shows backward drift
before the walking command begins: pitch is about `-8.22 deg` at tick `30`,
`-10.43 deg` at `60`, `-12.68 deg` at `90`, and `-19.31 deg` at tick `119`;
the command begins at tick `120` and the episode falls at tick `156`. Both
soles remain in contact throughout. This makes the failed CPU/Isaac mirror an
independent blocker: the CPU failure is already present in the zero-command
warm-up and cannot be caused by the forward lesson.

### Updated hypotheses

| Hypothesis | Evidence update | Status |
| --- | --- | --- |
| H1 — world-position standing feedback is a valid walking action base | The same action maps to progressively dorsiflexed ankles and almost no bidirectional ankle authority near the command's `7 m` endpoint | Strongly refuted |
| H2 — current action basis readily exposes a step | Learned GPU trace is double-support for `600/600`; `128` fixed/random and `56` periodic CPU probes produce no single support | Disfavoured; absolute impossibility not claimed |
| H3 — current support reward supplies gait credit | It is a one-tick binary occupancy predicate and the policy collects it only during zero-command intervals | Refuted |
| H4 — PPO instability/exploration collapse is first | Finite run, stable entropy/noise, improved survival, little global action saturation | Disfavoured |
| H5 — biomechanics body is intrinsically incapable of walking | Anatomical axes/ROM/geometry are credible; no translation-invariant reachability test has been run | Open, not leading |
| H6 — CPU failure is caused by the walking command | CPU divergence is already material before tick `120` | Refuted; mirror mismatch remains independent |

The literature is consistent with this split. Periodic Reward Composition
shows that a generic reference-free "move forward" objective is underspecified
for reliable biped gait discovery and uses periodic foot force/velocity costs
instead: <https://arxiv.org/abs/2011.01387>. Residual-RL biped work that starts
from an analytical controller places RL on top of an analytical **walking**
planner/controller, not an origin-anchored standing controller:
<https://arxiv.org/abs/2104.10592>.

## Pre-registered next discriminator

Do not start another PPO run yet. The smallest next experiment is an
optimizer-free walking action-basis audit:

1. Freeze a walking-only, translation-invariant reference candidate by removing
   the absolute forward-position term while leaving body, PD/safety, residual
   scales, command and reward unchanged.
2. Replay a small deterministic bank of alternating leg targets on CPU and
   Isaac. Require both left-only and right-only support, at least one complete
   alternation, positive forward COM displacement and zero declared safety
   terminal; require the unchanged standing control to remain upright.
3. If this fails, vary only residual range/direct default-offset action scale.
   If it passes, replace the binary support predicate in a separate change with
   a duration/alternation-aware foot objective (or a pre-registered phase
   objective) before a tiny overfit run.
4. A full `1,024,000`-sample run remains blocked until the action-basis audit
   passes and the same action tape has an acceptable CPU/Isaac correspondence
   result.

This order separates body capability, action authority, gait credit and mirror
dynamics. It avoids spending another full run on a controller whose action
semantics change with travelled distance.
