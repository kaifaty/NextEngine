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

Next: inspect the pinned update/normalization path and define passive KL,
clip-fraction, gradient and action-saturation telemetry with a matched
non-regression control before any new optimizer run. The unresolved choice is
whether update instability, normalization/distribution shift, or an objective
that rewards unreliable behavior dominates. A subsequent bounded experiment
must discriminate those, rather than merely adding samples or reward terms.

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
This transition changes documentation only; focused implementation tests and
native host-check results remain in the linked evaluator/schedule reports.
