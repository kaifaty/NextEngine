# Physical sound V29 — validator-first ML rebaseline research

| Field | Value |
| --- | --- |
| Date | `2026-09-02` |
| Status | `RESEARCH_COMPLETE / METAL_ROLE_REBASELINE_REQUIRED / VALIDATOR_FIRST / CAUSAL_HYBRID_GENERATOR_SELECTED_FOR_PROTOCOL / RUNTIME_ML_NOT_AUTHORIZED` |
| Scope | Internet-only rigid-impact evidence, independent automatic validation and the smallest fresh generator hypothesis after V28 M0c rejection |
| Architecture | [SPEC-45](../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md), `Proposed`; authored clips remain authority |

## Question

After V28 made training resource-feasible but rejected M0c's causal
representation, what program can still reach automatic material-sound
generation without asking the user to record or approve every sound?

The answer must preserve three independent facts:

1. a generator can fit disclosed data without being physically causal;
2. a validator can classify familiar recordings without safely admitting a
   generated candidate;
3. only a frozen generator, an independently qualified validator and one
   untouched joint admission can authorize an offline cooked result.

## Evidence carried forward

V28 R2 is not a resource or repeatability failure. Both official runs are
byte-identical and the prefix-bounded owner finishes inside its frozen budget.
The representation itself is rejected: it improves frequency, gain and
spectrum relative to ridge, but fails decay, remesh consistency and every
physical counterfactual. Nearby M0c tuning and a direct waveform/codec switch
remain closed.

The earlier project-disjoint E3 split is also not a Metal validator split. Its
semantic target is Glass; Wood and Metal were imported as `reject_parent`
evidence. Reusing those labels for Metal would invert frozen ground truth and
invalidate every threshold and risk estimate. That split remains immutable
Glass diagnostic evidence.

AV-P0C additionally shows why a small embedding or temporal-distance head is
not enough. It covers only one of three real parents in holdout, false-passes a
shuffled-envelope mutation and reports a 95% Wilson upper false-pass bound of
`0.7923`. V29 therefore treats validator sample power as an entry gate, not a
metric to inspect after implementation.

## Primary-source review

The following sources are candidates for a new signal-blind inventory. A page
or paper establishes only the stated capability; the Q0 adapter must still
freeze exact source identity, object/material labels, payload hashes and
missing axes before any signal is opened.

### Real material and object evidence

- [ObjectFolder Real official download](https://objectfolder.stanford.edu/objectfolder-real-download)
  publishes 100 real household objects. Each object has three mesh
  resolutions and impact recordings at 30–50 surface points with strike
  coordinates and contact-force profiles. This is the strongest current
  internet-native source for geometry/contact-conditioned generator training,
  subject to an exact bounded-access audit.
- The official [ObjectFolder material-classification repository](https://github.com/objectfolder/material-classification)
  declares seven material classes, including `iron` and `steel`, and an
  object-level real split. It can supply source-declared Metal labels; it does
  not by itself prove acoustic quality, support equivalence or statistical
  independence.
- The official YCB-impact publication page and
  [paper](https://www.iri.upc.edu/publications/show/2619) describe more than
  3,000 impact recordings from 75 objects and explicit material labels,
  including steel and aluminium. The acquisition has human, teleoperated and
  autonomous robot modes, so action mode must remain a declared axis rather
  than being pooled silently.
- The [REALIMPACT paper](https://ai.stanford.edu/~rhgao/publications/RealImpact.pdf)
  and [official repository](https://github.com/samuel-clarke/RealImpact)
  remain E2 transfer evidence: scanned objects, impact/listener locations and
  force-deconvolved responses. They do not supply the missing absolute
  force-to-amplitude or exact material-composition claims.
- The official [RWCP-SSD distribution page](https://www.nii.ac.jp/dsc/idr/speech/submit/RWCP-SSD.html)
  provides a large research-only corpus of isolated real-world sounds. Unless
  its exact entries expose stable physical-object and material identity, V29
  may use it only for background/event OOD and hard-negative diagnostics, not
  as a Metal positive or independent physical object count.

No source is admitted by this review. Q0 must reject any source whose object,
material, action or project grouping cannot be represented honestly.

### Frozen feature candidates

- Microsoft's [BEATs publication](https://www.microsoft.com/en-us/research/publication/beats-audio-pre-training-with-acoustic-tokenizers/)
  and [official implementation](https://github.com/microsoft/unilm/tree/master/beats)
  provide pretrained general-audio representations. A hash-pinned BEATs
  encoder is the primary neural embedding candidate because the validator can
  consume it without sharing a generator checkpoint or feature head.
- [LAION-CLAP](https://github.com/LAION-AI/CLAP) provides audio/text
  representations. Its text-aligned semantics are useful as a report-only
  ablation, but a prompt similarity such as “metal impact” cannot grant
  admission and cannot replace object-group risk evidence.
- DSP temporal/spectral descriptors remain mandatory because AV-P0C found a
  concrete shuffled-envelope failure that a material embedding alone need not
  detect.

The V29 validator is therefore an ensemble of independent evidence, not a
single neural judge: hard PCM/provenance checks, temporal-decay specialists,
spectral/material specialists, one frozen audio embedding candidate and an OOD
abstention layer. Every component may veto or return OOD; disagreement never
creates a human review queue.

### Risk control

[Distribution-free risk-controlling prediction sets](https://www.gsb.stanford.edu/faculty-research/publications/distribution-free-risk-controlling-prediction-sets)
show that a black-box predictor can be calibrated with finite-sample risk
control on held data. V29 uses this as a method candidate, not as an automatic
guarantee: object recordings are nested inside publisher/acquisition projects,
so exchangeability and grouping must be declared and checked.

The release contract retains the already measured conservative minima:

- at least `35` independent reject parents for a 95% upper false-pass target
  of `0.10` under the frozen grouped method;
- at least `16` in-domain object groups for a useful-coverage lower target of
  `0.80` when every group passes;
- source-stratified reporting and leave-project-out diagnostics in addition to
  object-parent counts;
- no protected partition dominated by one publisher/project revision.

If internet evidence cannot populate the frozen groups, the result is
`FallbackOutOfDomain`; thresholds and confidence targets are not weakened.

## Fresh generator hypothesis

[DiffSound](https://hellojxt.github.io/DiffSound/) demonstrates a
differentiable modal pipeline combining implicit shape, high-order finite
elements and an audio synthesizer for material/geometry/impact inference. It
supports a narrower next hypothesis than another unconstrained student:

> keep geometry, eigenfrequency scaling and counterfactual material response
> in a differentiable classical modal owner; let ML predict only bounded
> damping, radiation and modal-participation corrections that cannot bypass
> those physical coordinates.

This is not a claim that DiffSound or any upstream code is production-ready.
The first V29 tournament compares:

1. a deterministic classical/differentiable inverse-fit control;
2. one physics-locked neural correction model around the same cooked modes;
3. only after a discriminating failure, one separately preregistered
   mesh-spectral successor.

The model must pass remesh-paired and Young's-modulus/density/thickness/scale
counterfactuals before reading disclosed real training signals. A neural
waveform decoder remains report-only because it can hide rather than repair
the V28 causal failure.

## Decision

Adopt a validator-first V29 roadmap with two independent lanes:

- **Q lane:** rebuild Metal source roles from metadata, prove sample power,
  implement and qualify the automatic validator without generator outputs;
- **P lane:** preregister and prove a physics-locked hybrid generator on
  synthetic known truth before external Metal training.

The lanes meet only after both have immutable release hashes. One untouched
joint Metal shadow is then opened exactly once. Pass cooks deterministic clips;
Reject or OOD keeps the authored fallback. Runtime inference, public acoustic
schemas and a raw PhysX callback path remain unauthorized.

## Rejected options

| Option | Reason |
| --- | --- |
| Re-label the frozen Glass split as Metal | Inverts immutable target/reject semantics and leaks threshold evidence. |
| Let CLAP or BEATs alone decide naturalness | A semantic embedding has no independent false-pass/coverage certificate and can miss temporal physics failures. |
| Train the validator on generated candidates | Couples the judge to the generator's shortcuts and destroys independent admission. |
| Retry M0c with new weights, seed or capacity | V28 opened repeat-exact causal failures; nearby tuning is spent. |
| Jump directly to a waveform codec | Does not structurally enforce remesh or material counterfactuals and repeats a previously rejected family. |
| Ask the user for recordings or per-sound approval | Violates the standing internet-only and automatic-validation constraint. |

## Smallest next action

Freeze Q0-Metal source identities and one-use roles from metadata only. The
freeze must prove or reject the `35` reject-parent, `16` positive-group and
multi-project minima before downloading or decoding protected signal values.
