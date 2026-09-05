# R8b final-policy regression: closed result and bounded diagnosis

Status: `REPORT_ONLY / V7_AND_CORRECTED_V8_FINAL_QUALITY_FAILED`.
No optimizer is active. This report does not select an intermediate checkpoint,
change physics/safety, or admit another training budget.

## Observed result

V7 generation-01/TRAIN-1 completed all 10,000 updates / 40,960,000 transitions
at 2026-09-05 01:34:29 UTC. Source closure, all artifact hashes and consecutive
metrics verify. The predeclared final model 9999 fails all five original
episodes at tick 152 (2.5333 s), with -0.176714 m forward displacement.
The separately admitted ADR-111 corrected V8 evaluation also completes and
fails all five episodes identically. Every recorded V7/V8 NPZ array is equal;
the schedule repair first acts later, so it cannot explain this early failure.

Native replay identifies `terminal.self-collision` at tick 152, substep 0:
right forearm (`collider.right-forearm`, shape 10017, actor 1023) contacts the
head (`collider.head`, shape 10011, actor 1015). Separation is 0 micrometres;
impulse is [-333125, -107322, -4246] micro-newton-seconds. All four substeps
complete; the actuator safety error is null. Corrected longest support is
4/0 ticks, with zero qualified switches. This is not successful walking.

The previously predeclared diagnostic 3999 remains a useful positive physical
control: 1,200 safe ticks, 6.135033 m, actual whole-foot lift and 24 corrected
support switches. It is not substituted for the final checkpoint or promoted
by this report. Performance has regressed; the optimization mechanism causing
that regression is not yet established.

## Competing hypotheses and minimal interventions

| Hypothesis | Discriminator / evidence | Update |
| --- | --- | --- |
| Loading, adapter or V8 evaluation changes caused the failure | Exact final V7/V8 arrays and native action replay agree | Falsified for the observed early trajectory; preserve closure controls |
| Only learned arm residuals prevent an otherwise valid gait | Zero only eight arm residuals of the unchanged final policy; same nominal reset and 1,200-tick ceiling | Insufficient: tick 170 self-collision between left/right shanks, 0.009483 m, longest corrected support 2/0, no switches |
| Mean-action evaluation alone hides a reliable stochastic policy | Sixteen fixed noise seeds 0..15, unchanged final Gaussian/std, same environment seed 1001, no optimizer or normalization updates | Narrowed: only 3/16 complete safely, all below 3 m; none meets the complete task |
| Later PPO updates lose deterministic quality despite improved training return | Existing training windows and frozen milestone outcomes | Supported as an outcome, not a root-cause explanation; update-level KL/clip/gradient evidence is missing |

The arm diagnostic masks channels [2,7,8,9,12,17,18,19] after inference;
leg/torso outputs, body, PD, safety and reward remain unchanged. Its independent
native replay matches all retained state arrays. At tick 170, substep 1,
`collider.left-shank` / `collider.right-shank` (shapes 10003/10008, actors
1004/1010) contact with impulse [-3061591,-251388,98485] micro-newton-seconds
and 0 micrometres separation. Suppressing arm motion is not a sufficient fix.

The stochastic intervention preserves the full policy state byte-for-byte
before/after. Completed episodes are noise seeds 0, 1 and 13, travelling
1.694144 / 1.714931 / 1.783131 m, with velocity MAE 0.279810 / 0.277775 /
0.274698 m/s (limit 0.2). Six episodes end on impact, five on joint safety,
two on self-collision. Some failed episodes exceed 3 m before termination;
none establishes safe walking. Their raw-presence gait measurements are not
the corrected native support matrix, and no gait pass is claimed from them.
Noise changes outcomes substantially, but is neither a fix nor acceptance.

## Training signals and primary research

Window means, taken from existing metrics, not additional checkpoint searches:

| Iterations inclusive | Episode return | Episode ticks | Action std | Entropy |
| --- | ---: | ---: | ---: | ---: |
| 0–199 | 292.719 | 135.379 | 0.24747 | 0.50804 |
| 3800–3999 | 1194.794 | 515.259 | 0.28413 | 3.47856 |
| 5800–5999 | 1469.904 | 591.192 | 0.29542 | 4.22194 |
| 7800–7999 | 1667.104 | 640.336 | 0.31303 | 5.42871 |
| 9800–9999 | 1656.378 | 621.985 | 0.33205 | 6.72850 |

Metrics contain finite losses but do not retain approximate KL, clipping
fraction or pre-clip gradient norm. Installed RSL-RL 3.1.2 uses KL to adapt
learning rate within [1e-5,1e-2], not a hard KL early stop. Most late-window
rates are near its lower bound. This is not proof of excessive update size:
the missing KL values cannot be reconstructed from final weights alone.
Mean and sampled actions use the same actor and observation normalizer;
`act` samples its Normal distribution while `act_inference` returns its mean.

Primary sources opened 2026-09-05:

- [Spinning Up PPO](https://spinningup.openai.com/en/latest/algorithms/ppo.html):
  stochastic on-policy learning; clipping does not guarantee a bounded policy
  change, and this implementation uses KL early stopping. This supports an
  update-stability diagnostic, not automatic adoption of another optimizer.
- [Huang et al., 2022 implementation study](https://iclr-blog-track.github.io/2022/03/25/ppo-implementation-details/):
  precise implementation and evaluation details matter; its reproduced tasks
  do not establish learnability of this humanoid or optimal parameters here.
- [Moalla et al., NeurIPS 2024](https://proceedings.neurips.cc/paper_files/paper/2024/hash/81166fbd9cc5adf14031cdb69d3fd6a8-Abstract-Conference.html):
  representation degradation can accompany PPO performance collapse. No
  representation-rank measurement exists here, so that mechanism remains
  unresolved and does not justify an algorithm change.

The [earlier sole/method research](r8b-sole-support-and-training-method-research-2026-09-05.md)
answers the original toe-standing question and compares phase/contact/clearance,
procedural references and imitation methods. Its essential lesson remains:
physical motion and held-out behavior must be evaluated separately from reward.

## Decision and next discriminator

Retain failed final results, exact safety and the original task gates. Do not
restart unchanged, switch to stochastic deployment, freeze arms as a presumed
fix, or retrospectively select model 3999. No current evidence requires new
body geometry, an all-phase flat-foot constraint or a different RL algorithm.

Passive telemetry and a local normalization discriminator now pass, as below.
Next: predeclare one bounded fixed-buffer gradient diagnostic with explicit
source weights, optimizer initialization, RNG and buffer/return closure; it is
not a resume or another full walking budget. The unresolved choice is
whether update instability, normalization/distribution shift, or an objective
that rewards unreliable behavior dominates. A subsequent bounded experiment
must discriminate those, rather than merely adding samples or reward terms.

## Passive update instrumentation and normalization discriminator

`lab/next_lab/ppo_diagnostics.py`, introduced at `b7330152`, observes the
installed RSL-RL 3.1.2 minibatches and pre-step distributions through scoped
instance/optimizer/parameter hooks. It records analytic old-to-current Gaussian
KL, likelihood-ratio clipping fraction, saturation, learned std, actual learning
rate and pre/post-clipping gradient norms, including actor/critic components.
It adds no sampling, backward pass, global monkeypatch, weight update or
normalizer update. Hooks are removed on success and exception; unsupported
recurrent/RND/symmetry/multi-GPU paths and nested observation fail closed.
It is an opt-in private diagnostic context, not enabled in frozen old trainers.

The hook contract is checked against installed PyTorch and its
[2.7 backward-hook documentation](https://docs.pytorch.org/docs/2.7/generated/torch.Tensor.register_hook.html).
Detached float64 arithmetic is report-only. Analytic KL deliberately excludes
the scheduler expression's log epsilon, so identical Gaussians yield zero.
Ratio clipping measures likelihood ratios outside the PPO interval, not the
fraction whose clipped surrogate actually wins. Gradient norms precede the
existing clip, not a substituted clipping algorithm.

PASS: two complete baseline/observed updates have exactly equal losses, weights,
Adam state, adaptive learning rate and CPU/CUDA RNG states; normalizers do not
change during updates. Controls cover small CPU/CUDA networks and the actual
88-input/23-action, [256,128,64] ELU network with 128 × 32 transitions and
5 × 4 minibatches on CUDA. These use synthetic transitions and fresh networks,
not another humanoid optimizer run or a checkpoint continuation. Seven focused
diagnostic tests plus 26 adapter/evaluator tests pass (33 total), with Ruff,
formatting and diff checks. No Rust/public-contract changes require host-check.

The no-optimizer normalization discriminator uses unchanged final model 9999,
V8, 128 environments/eight shards, 32 steps, environment seed 44 and Torch noise
seed 44001 set after weight loading. It follows the trainer's explicit
post-step normalization updates, retains pre-action observations/actions/old
distributions, and recomputes their distribution under the end-of-rollout
normalizer. Trainable parameters remain exact; all state is restored afterward.

- Normalizer count: 40,960,000 -> 40,964,096.
- Analytic KL: mean 1.1117264101339589e-7, max 3.685534865072171e-7,
  versus the profile's target 0.008; likelihood-ratio clipping fraction zero.
- Maximum absolute log-ratio change 0.0024394989013671875; mean/std parameter
  maximum changes 0.0001221299171447754 / 0.00011980533599853516.
- Six real terminals within the 4,096 transitions (2 joint safety, 3 impact,
  1 self-collision). They are retained, not hidden by resetting the report.
- Outcome: within-one-rollout normalization is far too small in this measured
  late-policy/reset batch to explain a large pre-gradient distribution mismatch.
  Cumulative drift over thousands of updates and later walking-phase batches
  remain unresolved; this is not a global normalization-invariance claim.

External `final-rollout-normalization-01/control-manifest.json` SHA-256:
`23ce8e522896998585a76c66f18e94421c904a3b8bdab74ff324c0ceceb5a8db`;
report `6aa1ca6f4cde458e7f5ffbc2fe71a837e7c630b5d42cbcf919b20090f1dc8607`;
rollout NPZ `a9d2a6d66cc47a1bddfce344e0240ce07b8f6585c420654dc1241b49470accf9`.
The manifest binds code, source checkpoint/manifest, descriptor and native
executable hashes. Optimizer steps are exactly zero. The retained buffer is
for distribution diagnostics; it does not close final/terminal value inputs
for a PPO return calculation and must not be silently reused as training data.

Method research also confirms that predeclared periodic validation and retaining
validated checkpoints are ordinary tools, distinct from requiring the last
optimizer state: see [SB3 EvalCallback and stopping callbacks](https://stable-baselines3.readthedocs.io/en/master/guide/callbacks.html).
A future selection protocol may use that approach if frozen before its run;
it does not retrospectively admit model 3999 or weaken physical quality gates.

## Exact external evidence and verification

Source root: `/home/kaifaty/NextEngine-training/r8b-canonical-walking-v3`.
Corrected/diagnostic root: `/home/kaifaty/NextEngine-training/r8b-walking-stop-window-v8`.

| Artifact | SHA-256 |
| --- | --- |
| Source `generation-01/runs/TRAIN-1/run-manifest.json` | `3bcd5ee7a6cd6cd5582454da40c951c143e642bceec4b5f25d721d11f9d76e9e` |
| Source `generation-01/runs/TRAIN-1/model_9999.pt` | `108e372cd45e7d7ca30eae6bfaafb1116a549c10a131c1b6a2582fb719d928de` |
| `final-evaluation-01/run-manifest.json` | `f21a9e6be3b435e6309964950b300142fefe7870a66cadffe3c73f38fe025bc0` |
| `final-arm-mask-control-01/control-manifest.json` | `689f91153ed3fc52ad4ad13e39a975a6bf887a3e44bdc19045c0013251a25a5d` |
| `final-arm-mask-native-01/report.json` | `8d94751d0745e70304a12112f90af26e64b2c4edf52e7ababfa7eae3baf759fa` |
| `final-stochastic-control-01/control-manifest.json` | `e1d4a5069c900b976a8070b0e00f4ecae2ebddde8524c805bbbc50121ca746d8` |
| `final-video-01/video-manifest.json` | `02e07a521b7ae5113117695c6df5ec4195fb0c52df2906ef8d6623400002990d` |

PASS: source and result artifact closure, five exact native final replays,
masked-arm native replay, unchanged stochastic-control policy state, video
hashes and ffprobe (152 frames / 60 fps / 2.533333 s / 1200×800). Final preview
was inspected; the video is the complete failed episode, not a selected prefix.
FAILED: original and corrected final walking quality, both causal remedies as
sufficient fixes. NOT RUN: new optimizer, runtime/export or renewed mirror gate.
The initial final-result transition changed documentation only. Subsequent
passive telemetry passes the focused checks above; native host-check results
remain in the linked evaluator/schedule reports for the unchanged plant.
