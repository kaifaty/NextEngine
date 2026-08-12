# Reference PPO diagnostic playbook

Use this playbook with the exact frozen environment and training profiles. It
does not authorize changing code, profiles or running another experiment when
the user asked only for diagnosis.

## Order of evidence

| Order | Boundary | First questions |
| --- | --- | --- |
| 1 | Artifact | Do hashes, counts, commit, generation and checkpoint lineage close? |
| 2 | Body | Are IDs, ordering, frames, limits, gains and action ownership exact? |
| 3 | Data | Is the expected corpus/split/clip/phase actually selected? |
| 4 | Environment | Are reset, observation, reward, terminal and truncation semantics frozen? |
| 5 | Safety | Which phase and applied action channel first causes non-finite, ROM or contact failure? |
| 6 | Optimization | Are KL, clipping, gradients, value estimates and exploration behaving coherently? |
| 7 | Evaluation | Does the final checkpoint improve the same predeclared matrix? |
| 8 | Performance | Where does time go after correctness is established? |

Stop at the earliest broken boundary. Later signals can be consequences. A high
value loss caused by invalid terminal facts is not first an optimizer defect.

## Current metric semantics

- `approximate_kl` is compared with the frozen PPO target, but a single spike is
  weaker evidence than sustained pressure.
- `early_stop_kl` is a per-update boolean encoded as a numeric value. Its window
  mean is the early-stop rate.
- `gradient_norm` is the pre-clip norm. Repeatedly exceeding
  `maximum_gradient_norm` means clipping pressure, not necessarily divergence.
- `clip_fraction` is the sample fraction outside the PPO ratio interval. Read it
  together with KL and policy loss.
- `action_standard_deviation_mean` is the transformed policy's learned scale
  summary; compare collapse against the exponential of the frozen minimum log
  standard deviation.
- `rollout_mean_reward` is a training diagnostic. It is not an acceptance
  metric and is not comparable across changed reward profiles.
- `rollout_failure_count` and `rollout_reference_complete_count` need their
  fixed vector size and horizon context.

## Safety and evaluation

Any final-evaluation `non_finite`, `hard_rom` or `forbidden_contact` count is a
promotion blocker for that matrix. Preserve:

- exact clip ID and source phase;
- action channel and post-safety applied target;
- hard-ROM excess or forbidden-contact mask;
- the first failing motor tick;
- environment, descriptor, corpus, USD and checkpoint hashes.

`reference_tracking_lost` is not interchangeable with a fall. Stratify it by
clip and phase before attributing it to optimizer quality.

Initial and final evaluation must use identical matrix ID/hash, deterministic
mean action and the declared final checkpoint. Both frozen acceptance measures
must improve when the profile requires both. A selected best intermediate
checkpoint is a new experiment unless declared in advance.

## One-variable experiments

Choose the smallest experiment that can falsify the primary diagnosis:

- lineage defect: regenerate a new closed run; never edit old evidence;
- phase-local safety: run the phase safety audit with the unchanged checkpoint;
- correspondence suspicion: collect the required P1/P2 CPU and Isaac evidence;
- sustained KL pressure: revise only learning rate first, after safety is clean;
- critic-only pathology: verify returns/bootstrap, then revise one critic
  variable;
- exploration collapse: verify phase/data coverage before changing the
  distribution floor;
- throughput regression: use the frozen exclusive-device performance method,
  keeping changes report-only.

Use a new hash-bound profile for every causal change. Keep seed, clip/phase
selection, budget and evaluation matrix fixed unless one of those is the single
variable under test.

## Secondary analysis tools

Jupyter, TensorBoard, MLflow or Trackio-style dashboards are useful for plots
and alerts, especially during long runs. Keep them outside Git and always label
views with the underlying run/generation/profile hashes. They must not become
the only copy of a metric, selection decision or checkpoint identity.
