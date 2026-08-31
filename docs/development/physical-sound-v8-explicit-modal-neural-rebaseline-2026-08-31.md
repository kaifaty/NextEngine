# Physical sound R3A V8 — explicit-modal neural rebaseline

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Status | `RESEARCH_DECISION_FROZEN / V8_SYNTHETIC_PREFLIGHT_NEXT / REAL_QUALITY_NOT_AUTHORIZED` |
| Previous result | [R3A V5-C development rejection](physical-sound-r3a-v5c-capacity-frontier-and-development-result-2026-08-31.md) |
| Architecture | [SPEC-45](../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md), `Proposed` |
| Product effect | None; authored clips remain authoritative and runtime neural inference remains forbidden |

## Decision

Do not continue the V5 waveform-RVQ family. R3A V8 tests a materially different
representation in which object resonance remains explicit:

```text
object-global frequencies + object-global damping
  + neural contact -> per-mode displacement/gain field
  + bounded deterministic filtered residual
  + explicit excitation/onset
  -> differentiable impact renderer
  -> offline baked contact clip atlas
```

The first V8 step is an opened synthetic engineering preflight, not an acoustic
quality gate. It must prove that the renderer can recover known modal truth and
that a small coordinate network can predict held surface mode-shape values from
published FEM labels. A pass permits only a fresh real-data protocol. It does
not reopen the four V5 development objects, authorize R3B, establish realistic
glass, or change the clip fallback.

## Why this is a new hypothesis

V5 learned a generic waveform bottleneck and then asked the latent to preserve
modal identity. V8 instead makes the quantities that V5 lost—frequency,
damping and contact-dependent modal response—first-class parameters. Neural
learning is used for the spatial field and later bounded residual, not as an
opaque replacement for the resonator.

The following primary sources support that direction without proving it for
Next Engine:

- [DiffImpact](https://proceedings.mlr.press/v164/clarke22a/clarke22a.pdf)
  uses a differentiable impact model built from exponentially decaying modes,
  excitation and residual/reverberation components, and demonstrates fitting
  it as a physics-informed decoder. The exact inspected official code revision
  is `c9f5b949fc5b4194ee861b0c29490b33cdfb4072`.
- [Neural Resonator](https://arxiv.org/abs/2210.15306) predicts stable
  differentiable resonator banks rather than unrestricted waveforms. Its
  reported high-frequency decay limitation is a warning against unconstrained
  pole prediction. The exact inspected official code revision is
  `ceab3770d88caae1c9ee208bea127ec0d0a1e763`.
- [AV-MSF](https://arxiv.org/abs/2608.05145) separates object-global
  frequency/damping, a position-dependent neural gain field and a filtered
  residual; it also reports that modal initialization and residual are
  necessary ablations. Its [project page](https://zisenshao.github.io/AV-MSF/)
  reports real ObjectFolder/RealImpact experiments. The associated
  [repository](https://github.com/ZisenShao/AV-MSF) has no usable code at this
  checkpoint, so its numbers are prior art, not a reproducible baseline.
- The older [Sound Synthesis for Impact Sounds in Video Games](https://www.microsoft.com/en-us/research/publication/sound-synthesis-impact-sounds-video-games/)
  reports that measured amplitude envelopes and residual structure can improve
  on a single ideal exponential. V8 therefore does not claim that global
  exponential damping alone is sufficient for every real object.

## Competing hypotheses

| ID | Falsifiable explanation | Evidence for | Evidence against / missing | Smallest discriminator |
| --- | --- | --- | --- | --- |
| H8-A | V5 failed primarily because the representation did not preserve explicit resonances. | All `12/12` V5 development comparisons fail spectrum and modal frequency; differentiable modal prior art succeeds on related tasks. | V4 also failed despite explicit poles, although its fixed factorization was much narrower. | Recover known modes and held spatial mode shapes before reading fresh real audio. |
| H8-B | The tiny 73-clip V5 corpus and source mismatch dominate representation choice. | V5 internal validation did not predict any real development endpoint; newer real datasets contain tens of impacts per object. | More data cannot repair a representation that already discards modal identity. | After synthetic pass, compare explicit-modal and honest KNN/modal controls on a new ObjectFolder Real development source. |
| H8-C | V5 only needed more steps or bitrate. | Internal objective improved with capacity. | All three frozen capacities fail the same real endpoints by large margins; the opened contacts can no longer select training changes. | No retry. Only a new source-disjoint protocol may reconsider this explanation. |
| H8-D | One global exponential modal bank is itself insufficient on real impacts. | Prior work benefits from measured/spatial damping and filtered residual; AV-MSF ablates both initialization and residual. | Added components can overfit or hide incorrect modes. | Preregister global damping first, then at most one spatial-damping ablation and one bounded residual on fresh data. |

Decision: test H8-A first while preserving H8-B and H8-D as explicit competing
causes. Reject H8-C for the opened V5 family.

## Internet data roles

No local microphone or instrumented impact capture is permitted.

### V8-SYNTH — opened engineering preflight

Use [NISR v5](https://huggingface.co/datasets/BumsooKim00/nisr-dataset)
revision `20368791bcd7829e04ae3eb07c10aa0bb370e38a`. It provides FEM frequencies,
boundary coordinates and 3D mode shapes. It is synthetic and cannot establish
realism. The repository currently exposes the following two opened Glass label
files for the first preflight:

| Object | Role | Path | Bytes | SHA-256 |
| --- | --- | --- | ---: | --- |
| `1` | format/control development | `training_dataset/1/feat/feat_Glass.npz` | `156935` | `6548be1d819cef4c010aed787d5277e0c4de8021fa901b5345ad1dac115f36aa` |
| `2` | scale/control development | `training_dataset/2/feat/feat_Glass.npz` | `866616` | `d527053ef8c06f96ba4435a2e46ac03cbc690138f37dd2b9273f5496828322b9` |

These files are opened synthetic development inputs. They are not a sealed
holdout. The dataset card says randomized WAV parameters live beside the WAVs,
but those paths are absent at this exact revision while a second top-level WAV
tree exists. V8-SYNTH therefore uses only the hash-closed `feat_*.npz` labels
and makes no claim from the randomized audio metadata.

### V8-REAL — future representation development

[ObjectFolder Real](https://objectfolder.stanford.edu/objectfolder-real-download)
is the preferred fresh real source because it publishes object geometry,
multiple impact positions, force profiles and real recordings. Its complete
archive is too large for the engineering preflight, and no V8 real object is
selected or opened in this revision. Before access, V8-REAL must freeze exact
objects, source hashes, canonical listener/excitation policy, grouped roles,
baselines and gates. Objects used by prior REALIMPACT work cannot select V8.

### V8-HOLDOUT — future source-disjoint evidence

[X-Capture](https://huggingface.co/datasets/swistreich/XCapture) is a candidate
source-disjoint holdout because it publishes many real objects, contact
conditions and local geometry. It has no official split, so any use requires a
new hash-closed grouped projection. It remains unopened and unauthorized.

## Frozen V8-SYNTH protocol

All outputs, weights and source payloads remain outside Git.

### Modal recovery control

- Renderer: sum of four signed `gain * exp(-damping*t) * sin(2*pi*f*t)` modes.
- Signal: `8,000` float64 samples at `16 kHz`.
- Frequencies: `[333, 817, 1511, 2879] Hz`.
- Dampings: `[3, 7, 15, 30] s^-1`.
- Gains: `[0.8, -0.5, 0.32, 0.18]`.
- Initialization is a bounded perturbation of known synthetic truth; this is
  an optimizer/renderer control, not blind audio decomposition.
- Optimizer: deterministic CPU Adam, `2,000` updates, fixed per-parameter
  learning rates and seed `20260831`.
- Pass: finite output, maximum frequency error `<= 1 cent`, maximum relative
  damping error `<= 0.02`, maximum relative gain error `<= 0.02`, waveform MSE
  `<= 1e-6`, and exact repeated result.

### Neural mode-shape field control

- Input: normalized boundary coordinate, four fixed Fourier bands
  `[1, 2, 4, 8]`, and the published six-value surface encoding.
- Target: all `3 x 20` signed boundary mode-shape components. No unpublished
  force direction or normal-column meaning is inferred.
- Split: deterministic farthest-point context set with `ceil(20%)` of surface
  points and at least `32` points; all remaining points are query.
- Normalization: per-target RMS from context only.
- Candidate: CPU float32 MLP `input -> 128 -> 128 -> 60`, SiLU, AdamW,
  learning rate `0.002`, weight decay `1e-5`, `1,500` full-batch updates, seed
  `20260831` independently for each object.
- Frozen baselines: context mean and one-nearest context point in normalized
  coordinate plus published surface encoding.
- Pass per object: finite outputs, nonzero query diversity, neural normalized
  RMSE `<= 0.50 *` nearest-neighbour RMSE and below the constant baseline.
- Exact repeat is required for metrics, predictions and parameter hash.

### Decision boundary

`READY_FOR_FRESH_REAL_MODAL_PROTOCOL` requires every synthetic control to pass.
It authorizes only a new preregistered V8-REAL fit/development revision.
`STOP_V8_SYNTH_KEEP_CLIPS` retains authored clips and triggers diagnosis before
any real data access. Neither result authorizes R3B, a validator release, a
baked atlas or runtime integration.

## V8 sequence after the preflight

1. `V8-SYNTH`: prove explicit renderer recovery and neural mode-shape field on
   exact FEM labels.
2. `V8-REAL-FIT`: freeze a small fresh ObjectFolder Real slice and test whether
   analytic STFT/Hilbert initialization plus bounded residual can fit the
   authorized fit recordings before interpolation.
3. `V8-REAL-DEV`: evaluate global damping first and at most one preregistered
   spatial-damping capacity on new object/source-disjoint development.
4. `V8-REAL-HOLDOUT`: only a development pass may open one new source-disjoint
   holdout.
5. `V8-FIELD`: only a representation holdout pass may train an exact-object
   contact field on real positions and compare with KNN.
6. `V8-ATLAS`: accepted predictions are rendered offline to ordinary clips;
   the engine still loads no model.

## Remaining uncertainty

Synthetic mode-shape interpolation may be easy while real audio fitting still
fails due to support, excitation, microphone response, spatial damping or
non-modal residuals. Conversely, real nearest-neighbour may remain as good as
the neural field. The V8-SYNTH gate deliberately cannot resolve those risks.

The smallest next action is to implement and repeat the frozen V8-SYNTH
preflight without reading any real V8 development waveform.
