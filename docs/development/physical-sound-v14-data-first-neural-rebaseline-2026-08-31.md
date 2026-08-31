# Physical Sound V14 — data-first neural rebaseline

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Status | `BOUNDED_RESEARCH_COMPLETE / ROADMAP_V14_JUSTIFIED` |
| Trigger | Two complete source lineages failed before real fitting: V12 object 41 lacks shared force-spectrum coverage; V13 object 92 lacks one required metadata parent. |
| Product effect | None; ordinary authored clips remain authoritative and SPEC-45 remains `Proposed`. |

## Problem restatement

The project has already proved that it can acquire public evidence, freeze
roles before signal access, reproduce experiments byte-for-byte and stop on a
failed source contract. It has not yet trained a real contact-conditioned
generator. The blocking assumption was that one perfect exact object with
geometry, contact, raw force and audio had to be complete before model work
could begin.

That assumption is stronger than the first useful product claim. An offline
authoring model only needs to produce a bounded, plausible dry impact field and
an honest OOD/fallback decision. It does not need to identify the true material
constants of an unseen prop or accept an arbitrary measured force waveform.

The next falsifiable target is therefore:

```text
mesh + material/scale/support hints + impact position/energy
    -> learned stable modal field + uncertainty
    -> deterministic 48 kHz clip atlas
```

The neural network learns the hard mapping. The explicit oscillator/rendering
layer is retained as a stable output constraint and cooker, not as a manually
tuned theory of every object.

## Current primary-source evidence

- The official [ObjectFolder-Real download page](https://objectfolder.stanford.edu/objectfolder-real-download)
  describes 100 real household objects, with 30–50 six-second impact recordings
  per object, strike coordinates on the mesh and ground-truth force profiles.
  This is sufficient for grouped real contact learning even if individual
  objects or archive members fail a stricter measured-transfer contract.
- The official [ObjectFolder 2.0 download page](https://objectfolder.stanford.edu/objectfolder2-0-download)
  exposes 1,000 implicit object models queryable by surface coordinate and force.
  It is useful as a synthetic teacher and coverage augmenter, but not as an
  independent real-quality validator.
- The official [RealImpact project](https://samuelpclarke.com/realimpact/) reports
  150,000 recordings across 50 objects with impact and listener locations,
  material labels and contact-force profiles. The current
  [publisher repository](https://github.com/samuel-clarke/RealImpact) distributes
  preprocessed archives while still describing the raw dataset as forthcoming.
  It can support derived-response and listener controls, not a new raw-force
  provenance claim.
- [AV-MSF](https://zisenshao.github.io/AV-MSF/) demonstrates the same useful
  factorization—object-global frequencies/damping and a spatial neural modal-
  gain field—from about 20% of each object's impacts. Its
  [repository](https://github.com/ZisenShao/AV-MSF) still contains only a
  “code coming soon” placeholder, so Next Engine must implement and validate
  the hypothesis independently.
- The official [ModalSound code and dataset page](https://www.cs.columbia.edu/cg/modalsound/)
  publishes a solver pipeline and 4,968 preprocessed watertight meshes. This is
  a viable known-truth/synthetic curriculum, not real-sound admission evidence.

## Competing hypotheses

| ID | Hypothesis | Evidence for | Evidence against | Discriminator |
| --- | --- | --- | --- | --- |
| H1 | A hybrid neural modal field can learn plausible impact variation from synthetic pretraining plus sparse real calibration. | AV-MSF reports few-shot real-object success; ObjectFolder supplies contact-labelled real data; the constrained output is controllable and cookable. | Next Engine's previous coordinate fields and compact representations failed; upstream AV-MSF code is unavailable. | Known-truth oracle, then one frozen multi-object few-shot tournament against classical controls. |
| H2 | A direct waveform generator is simpler and will sound better. | It avoids explicit pole extraction and may represent transients/residuals better. | It is harder to make contact-continuous, deterministic, OOD-aware and independently testable; prior generic codec attempts lost modes/spectrum. | Keep one direct decoder as report-only rate-distortion upper bound after the structured baseline is frozen. |
| H3 | True arbitrary-force transfer must precede useful synthesis. | It is the strongest physical claim and would support arbitrary excitations. | Two exact acquisition lineages failed before response fitting, while the product can use canonical energy bins and baked clips. | Move measured transfer to an optional later upgrade; it no longer blocks canonical authoring. |
| H4 | One perfect exact-object source must be recovered before any model work. | A complete source simplifies lineage and interpretation. | Source completeness is independent of representation feasibility; synthetic truth and multi-object real data can test the model without reopening failed objects. | Freeze an object-disjoint dataset contract and allow source-qualified training pools while keeping evaluation objects strict and sealed. |

## Decision

Adopt H1 as Roadmap V14. Do not continue V13-M2d as the sequential critical
path. Preserve object 41 and object 92 as permanent negative/OOD fixtures and
preserve RealImpact Green Goblet as a derived-response control.

The source policy becomes claim-scoped:

- synthetic solvers and ObjectFolder 2.0 may pretrain representation and
  invariants, but cannot validate real quality;
- partially usable real objects may contribute to the training pool only after
  a signal-blind quality mask and object-group split are frozen;
- method holdout and admission shadow require complete declared axes and are
  never repaired or reduced after opening;
- audio-only internet corpora may train or calibrate validator specialists but
  cannot fabricate geometry, force or contact evidence;
- measured raw-force transfer remains a separate future record type.

## Selected model boundary

The first candidate is deliberately small:

```text
mesh samples/local surface descriptors
        + material, scale and support hints
        + optional few-shot reference encoder
        -> object-global stable frequency/damping bank

contact coordinate + local descriptors + object latent
        -> bounded complex modal gains + confidence/OOD score

modal bank + gains + canonical impact-energy bin
        -> deterministic differentiable renderer
        -> ordinary cooked clips
```

The first pass has no unconstrained waveform residual. A deterministic
transient/residual head is a separately ablated successor only after the modal
branch passes held real contacts. Mesh-only features precede images, DINO or
3D Gaussian Splatting. A direct waveform decoder remains an upper-bound
experiment and cannot silently become runtime inference.

## Validation boundary

There is no per-sound human approval loop. One frozen validator release combines:

1. provenance, split, schema, finite-value and byte-repeat hard gates;
2. stability, positive damping, energy, continuity and excitation monotonicity;
3. held real spectrum, transient, envelope, decay and modal metrics;
4. object/source-disjoint material and real-versus-mutation specialists;
5. calibrated OOD/selective-risk bounds on untouched parent groups.

Human listening may appear only as a non-authoritative research audit of a
validator release, never as the decision for every generated clip.

## Consequences and stopping rules

- If the known-truth oracle cannot recover the chosen representation, real
  training does not start.
- If KNN/RBF/local-linear controls match the neural contact field within the
  frozen margin, the simpler classical field wins.
- If object-specific few-shot fitting fails, shared zero-shot training does not
  start; another source hunt is not an automatic retry.
- If the shared prior passes material identity but not contact continuity, it
  may produce only a fallback clip family, not a physical field claim.
- If no candidate passes automatic shadow admission, V14 ends with reproducible
  `FallbackOnly` records. Runtime and public schemas remain unchanged.
- A successful V14 produces external research records and deterministic clips,
  not a production feature. Promotion still requires a concrete consumer and
  a later Accepted ADR under ADR-046.
