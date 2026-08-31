# Physical sound R3A V10 A1R result and V11 bounded research

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Status | `REPEAT_EXACT_REJECT / FORCE_ONSET_VALID / V9_REAL_REPRESENTATION_CLOSED` |
| Object | ObjectFolder Real `51 / Fruit_Bowl / Glass` |
| Product effect | None; authored clips remain authoritative |

## Exact result

Two independent fit runs are byte-identical:

- manifest SHA-256:
  `a50e4ade09301dc438fe3393a003c9f52b0f4a62147acd3c7af99568b8e98ca3`;
- model SHA-256:
  `e64ec94dc58ac6cf67d0afced81efeffbb9daa982d56745f1bb17e584b70c42d`;
- report SHA-256:
  `5c9e87e064f9104d01d03ffcb99ee199c4c4d9e7c09378f926070dfc1646c2ca`;
- decoded microphone/force samples: `1,152,000 / 1,152,000`;
- development and sealed decoded samples: `0 / 0` for both channels;
- contact records: `5,904` bytes each;
- every source, finiteness, count, mode, band, temporal-variation, repeat and
  storage hard gate passed.

Measured force finds the event at sample `48,000` for contacts `27/15/4` and
`47,999` for contact `3`. Therefore the A1 failure was genuinely repaired:
synchronization is no longer ambiguous. The unchanged V9 representation then
fails every fit contact.

| Endpoint | Frozen maximum | Observed range | Contacts passed |
| --- | ---: | ---: | ---: |
| RMS level error | `0.5 dB` | `0.63–10.50 dB` | `0/4` |
| Log-spectrum RMSE | `4.0 dB` | `9.39–10.27 dB` | `0/4` |
| Envelope RMSE | `0.20` | `0.018–0.217` | `3/4` |
| Modal-frequency error | `100 cents` | `1,807–3,870 cents` | `0/4` |
| T60 relative error | `0.35` | `0.086–0.752` | `1/4` |

Decision: `REJECT_A1R_V9_REAL_REPRESENTATION`. Development contact `9` and
sealed contact `18` remain closed. No exact-object field, validator, atlas,
material formula, public contract or runtime path is authorized.

## Competing hypotheses

### H1 — missing excitation/transfer separation is causal

**For:** V9 estimates modes directly from microphone output and uses force only
as a clock. A measured microphone waveform is the convolution of contact force,
object response, radiation, room and sensor response. Different impacts can
therefore move output peaks without changing the object poles. The RealImpact
authors explicitly force-deconvolve each microphone signal before peak picking,
damping fitting and transfer-map construction.

**Against:** direct frequency-domain division is unstable where force spectrum
is weak; room and microphone effects remain. A regularized estimator and
coherence/conditioning gates are required.

### H2 — V9 residual capacity is merely too small

**For:** all records meet the small `5,904`-byte budget, so more capacity could
reduce reconstruction error.

**Against:** the dynamic residual does not materially recover stable modal
identity, all four spectra miss by more than `9 dB`, and modal error is measured
in thousands of cents. V8 and V9 have already moved capacity and temporal
structure without closing this real-data criterion. Another nearby bank/DCT
tune is not falsifiable enough.

### H3 — a pure prompt/waveform network should replace the physics

**For:** a sufficiently large network can imitate a training distribution.

**Against:** it does not provide the force/contact/geometry factorization needed
for controllable impacts or bounded OOD. DiffImpact reports that physics-based
priors improve efficiency and expressiveness over pure learning alternatives;
NeuralSound uses ML to accelerate modal and radiation solvers rather than
discarding their structure.

### H4 — physics plus constrained ML is the smallest viable successor

**For:** DiffImpact separates contact force and object impulse response;
DiffSound performs differentiable physical modal inverse rendering; NeuralSound
learns approximations to vibration and acoustic-transfer solvers. These are
consistent with an offline model that learns contact-to-modal residues or
radiation while exact frequencies/damping remain mechanically testable.

**Against:** no NextEngine experiment has yet recovered a stable transfer
response from internet data or shown that geometry predicts it on a held object.

## Decision

Close V9 for real impact reconstruction. V11 starts with a successful
known-truth control for a regularized force-to-microphone transfer estimator,
then freezes a fresh internet source before real selection. Object `51` may be
used only as opened diagnostic data; it cannot select the V11 capacity or prove
generalization.

The target factorization becomes:

```text
measured/tracked contact force f(t)
  -> object/listener transfer H(contact, listener, frequency)
  -> explicit poles + damping + spatial residues
  -> optional bounded stochastic remainder
```

ML is retained, but in constrained roles: predict modal residues/radiation from
geometry and contact, accelerate a numerical solver, or estimate uncertainty.
The runtime still receives only baked clips (or a later separately authorized
bounded coefficient record), never a neural model.

## Primary evidence

- [RealImpact CVPR 2023 paper](https://openaccess.thecvf.com/content/CVPR2023/papers/Clarke_RealImpact_A_Dataset_of_Impact_Sound_Fields_for_Real_Objects_CVPR_2023_paper.pdf): measured force deconvolution, mode fitting and spatial transfer maps.
- [DiffImpact CoRL 2021 paper](https://proceedings.mlr.press/v164/clarke22a/clarke22a.pdf): separate differentiable force, impulse-response and environmental models.
- [DiffSound SIGGRAPH 2024 project](https://hellojxt.github.io/DiffSound/): differentiable FEM/modal inverse rendering.
- [NeuralSound paper](https://arxiv.org/abs/2108.07425): ML approximations for vibration modes and acoustic transfer with numerical refinement.

## Smallest next action

Implement V11-B1 on synthetic known truth: generate a known modal transfer,
convolve it with several force pulses, add bounded noise, and compare direct
division, regularized spectral deconvolution and an H1-style estimator. A valid
method must recover poles/damping, reconstruct held force responses and expose
low-coherence/OOD bands before any fresh real waveform is opened.
