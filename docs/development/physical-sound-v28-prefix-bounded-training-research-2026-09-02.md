# Physical sound V28 — prefix-bounded training research

| Field | Value |
| --- | --- |
| Date | `2026-09-02` |
| Status | `COMPLETE / H1_CONFIRMED / PREFIX_EQUIVALENCE_EXACT / FRESH_SUCCESSOR_JUSTIFIED / NO_MODEL_VALUE_OPENED` |
| Trigger | V27 R1-A remained CPU-active until its frozen `1,800 s` limit and published no canonical output; R1-B did not start |
| Scope | Value-independent CPU cost attribution and the smallest falsifiable successor hypothesis |
| Out of scope | Reading interrupted model values, changing model capacity/seed/loss weights/gates, starting R1-B, or making a quality claim |

## Question

Why could the frozen five-variant M0b owner not finish inside its declared CPU
budget, and can a fresh successor preserve the exact scientific loss while
removing the dominant cost?

The result is narrow: the failure is attributable to a needlessly long
differentiable render in the synthetic loss. It does not say whether M0b would
have passed any quality gate.

## Competing hypotheses

| ID | Hypothesis | Evidence for | Evidence against | Decision |
| --- | --- | --- | --- | --- |
| H1 | The `144,000`-frame differentiable renderer dominates training cost. | Static dataflow and the random-tensor microbenchmark below. | No operator trace exists from the spent official run. | `CONFIRMED_FOR_VALUE_INDEPENDENT_EXECUTION`; sufficient to design a fresh cost-bounded successor. |
| H2 | Five separately trained variants and `10,000` optimizer steps are the dominant cost even if each step is cheap. | The loop really trains five complete variants. | Parameter-only steps are about `0.00228 s`; the same loop is cheap without the long renderer. | `SECONDARY_MULTIPLIER`, not the root cause. |
| H3 | Preprocessing, MLflow or artifact publication consumed the budget. | These occur in the owner. | R0/preflight finished; R1 produced no MLflow run or canonical artifact and remained in training staging. | `FALSIFIED_AS_PRIMARY_CAUSE`. |
| H4 | The `4 GiB` cgroup killed the job or memory pressure made progress impossible. | Observed RSS was material (`~1.35–1.53 GiB`). | RSS stayed below the limit and the terminal event was the outer wall timeout, not OOM. | `NOT_SUPPORTED`; memory remains a successor gate. |

## Static execution evidence

The frozen official profile declares `1,500` synthetic steps, `500` real
adaptation steps, batch `16` and a `144,000`-sample transfer window
([common.py](../../lab/scripts/physical_sound_v25_m0a_common.py)). The owner
trains candidate A, candidate B, no-geometry, no-contact and no-residual as
independent models. Synthetic replay also runs during all `500` real steps.

The expensive path is mechanically visible:

1. `synthetic_loss` renders exactly `target_waveform.shape[-1]`, which is
   `144,000` in the official profile.
2. `multi_resolution_log_spectrum` is the only waveform consumer in that
   synthetic loss.
3. That consumer evaluates windows `256`, `1,024` and `4,096` only.
4. `_windowed` discards every sample after the requested window.

Therefore samples `4,096..143,999` affect neither a synthetic loss component
nor its gradient. The render is causal and element-wise in time, so rendering
`4,096` frames produces the same consumed prefix as rendering `144,000` and
then slicing it.

## Value-independent measurements

Both experiments used CPU float32, deterministic algorithms, one intra-op and
one inter-op thread, fixed random tensors, batch `16`, the frozen `23,142`
parameter model and precomputed random teacher targets. They used no official
dataset, real signal, protected role, checkpoint or interrupted R1 artifact.

### Training-step cost

Each timed step included model/coarse forward passes, the declared loss,
backward, gradient clipping and AdamW update. One warm-up preceded the samples.

| Path | Repeats | Median | Min–max | Relative to full render |
| --- | ---: | ---: | ---: | ---: |
| Parameter losses only | 5 | `0.002281 s` | `0.002271–0.002639 s` | `464.72×` faster |
| Rendered spectral loss, `4,096` frames | 5 | `0.012665 s` | `0.011426–0.019737 s` | `83.71×` faster |
| Rendered spectral loss, `144,000` frames | 3 | `1.060228 s` | `1.053099–1.070614 s` | baseline |

A deliberately conservative static extrapolation of only the five variants'
`1,500` synthetic steps is `7,951.7 s` (`132.5 min`) at the full horizon,
versus `95.0 s` at `4,096`. It excludes real adaptation, evaluation,
preprocessing and publication, so it explains why the `1,800 s` complete-run
budget could not succeed without making a claim about how far R1 progressed.

### Exact prefix equivalence

For five fixed random seeds, two identical model copies evaluated the frozen
synthetic loss with the same `144,000`-sample target. One prediction rendered
all `144,000` samples; the other rendered only the consumed `4,096` prefix.
For every seed:

- total loss was bit-exact;
- every named loss component was bit-exact;
- every parameter gradient was bit-exact;
- maximum absolute loss and gradient difference was `0.0`.

This is evidence about the frozen eager CPU implementation, not a promise for
another backend or a future loss that consumes late decay.

## External method check

Only primary PyTorch documentation was used:

- [`torch.utils.benchmark`](https://docs.pytorch.org/docs/stable/benchmark_utils.html)
  documents warm-up, controlled thread-pool size and replicate/median-oriented
  measurement. The local discriminator followed those principles, while using
  a bounded explicit loop so a full render could not autorange indefinitely.
- [`torch.profiler`](https://docs.pytorch.org/docs/stable/profiler.html)
  supports CPU operator timing, input-shape capture, memory accounting and
  scheduled windows for long jobs. A successor may use it only on synthetic
  value-independent fixtures; tracing official protected work would add
  overhead and an unnecessary artifact surface.
- The official [Performance Tuning Guide](https://docs.pytorch.org/tutorials/recipes/recipes/tuning_guide.html)
  recommends disabling gradients for inference and `zero_grad(set_to_none=True)`;
  the frozen trainer already uses `set_to_none=True`, so that generic tuning is
  not the missing two orders of magnitude.

## Conclusion and decision

R1 was not under-sized because the neural network was large. It paid autograd
for `35.16×` more waveform samples than its synthetic spectral objective could
observe, then multiplied that cost across five models and all replay steps.

The smallest fresh successor is **M0c prefix-bounded training**:

- inherit M0b preprocessing, data, protected-role order, model, seed,
  optimizer, step counts, losses, weights, controls and quality gates;
- in synthetic loss only, render the maximum consumed window (`4,096`) rather
  than the stored transfer length (`144,000`);
- retain full aligned signals for later evaluation and any metric that actually
  consumes late decay;
- prove value and gradient equivalence on frozen random fixtures before any
  official value opens;
- pass a complete official-shape resource oracle twice under the declared
  wall/RSS envelope before a fresh official A/B run is authorized.

This is not an R1 retry: M0b remains spent. M0c has a new preregistered resource
hypothesis, a mechanically checkable equivalence condition and a new
implementation root. If the oracle fails, close M0c without opening model
values. If official M0c rejects quality, use that result only according to its
fresh protocol; do not tune the horizon, capacity, seed or thresholds.

## Remaining uncertainty

- The benchmark is a value-independent discriminator, not an end-to-end
  official timing result.
- Real-adaptation and evaluation costs were not isolated; they already use
  `4,096`-sample acoustic windows except for synthetic replay.
- No official step cursor or operator trace was published, so the exact R1
  stopping location remains unknowable and irrelevant.
- Prefix equivalence ceases to apply if a future training loss measures late
  decay or another sample after `4,095`; that change requires a new protocol.

## Smallest next action

Freeze the V28 M0c protocol and implement a value-independent full-owner
resource/equivalence gate. Do not open official values until that gate passes
twice and the implementation root is committed.
