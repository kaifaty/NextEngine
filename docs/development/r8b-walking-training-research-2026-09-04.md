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
