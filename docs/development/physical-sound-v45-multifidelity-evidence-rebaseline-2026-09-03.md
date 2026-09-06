# Physical sound V45 — multi-fidelity evidence rebaseline

| Field | Value |
| --- | --- |
| Date | `2026-09-03` |
| Status | `COMPLETE / MULTI_FIDELITY_REBASELINE / NO_NEW_CORPUS_CREDIT` |
| Scope | Public internet evidence for modal structure, force-to-vibration transfer and microphone-domain acoustic validation |
| Supersedes | V44 assumption that one new descriptor-to-microphone project must precede every useful learning experiment; it does not supersede V44 corpus, power, PSEL, validator or admission gates |
| Product constraint | No local recording, instrumented hammer or per-sound human approval |

## Question

V44 correctly found that the disclosed real-audio corpus cannot yet support a
material pack: it has `71/105` supported physical parents, a deficit of `34`,
and all five strict-Steel IETeasy parents come from one project. The next search
therefore asked two distinct questions:

1. is there another public project that alone supplies independent physical
   descriptors and microphone waveforms at the required power;
2. if not, can several sources support different bounded claims without being
   falsely pooled as equivalent evidence?

The answer to the first question remains **no**. The answer to the second is
**yes**, provided that observation masks, source lineage and claim-specific
gates are frozen before any target values or model candidates are opened.

## Method and access boundary

The search inspected publisher metadata, dataset cards, recording notes and
source code. No protected role, existing candidate output or hidden holdout was
opened. No newly found audio, feature archive or model checkpoint was decoded.
One read-only shallow clone of Clatter was inspected at commit
`79cac6cbe3f7c452ba28b56c7da4a0124ad04806`; it remains outside the repository.

This document records source capability, not source admission. A later profile
must pin the exact revision, hashes, aliases, usage terms, cost and allowed
claim mask before any payload access. Dataset size, repeated impacts and
synthetic variants never count as independent physical parents.

## Candidate disposition

| Source | Published observations | Admissible V45 role | Claims it cannot support |
| --- | --- | --- | --- |
| [Clatter](https://github.com/alters-mit/clatter) | Physics-event synthesizer backed by 84 material-size parameter files: 14 material classes × 6 size buckets. The impact path samples ten modal frequencies, onset powers and RT60 values, sums damped sinusoids and convolves them with a short half-sine contact force. | Frozen empirical-prior/control teacher after a value-free decoder protocol. Run externally; store only hashes, neutral summaries and results allowed by provenance. | It is synthetic output, not an independent real-acoustic validator, physical-parent corpus increment or runtime dependency. Its own README calls the library alpha and notes weaker soft-material and large-object output. |
| [NISR dataset](https://huggingface.co/datasets/BumsooKim00/nisr-dataset) | 1,100 ObjectFolder geometries, eight material parameter sets, up to 20 FEM modes, modal resynthesis, PyBullet-derived contacts and randomized/blended synthetic sounds. Current card reports no train/validation/test split. | Bounded synthetic teacher for geometry/material-to-mode counterfactuals, only after exact revision and generation-provenance preflight. | It derives from ObjectFolder geometry and synthetic modal audio. It adds zero independent real-project, microphone-naturalness, validator or protected power. |
| [TU Delft aluminium plate](https://zenodo.org/records/7758683) | One Aluminium 6082-T6 plate with exact dimensions and material constants; 25 hammer locations, three responses each, measured force and acceleration under free support. | Structural-transfer control for contact location, force-to-vibration response and damping consistency. | One parent, one material and no microphone radiation: no material-pack power or final acoustic-quality claim. |
| [Cello bridge study](https://zenodo.org/records/20797149) | Nine bridge configurations with published material/composition and mass; impact hammer and three accelerometers measure installed-instrument response. | Prospective structural-transfer and representation control after an axis/cost audit. | Whole-cello assembly response is not isolated object radiation; no direct microphone-naturalness or generic shape-transfer claim. |
| [CMU Sound Events Database](https://www.auditorylab.org/sound-events-database) and [impact record](https://doi.org/10.1184/R1/20205035) | Controlled impact recordings, videos and notes for repeated exemplars under broadly consistent microphone conditions. Notes expose object/action/support context but not a complete mass/dimension/material-grade descriptor plane. | Prospective disclosed acoustic/OOD or validator-calibration source after exact archive-to-note binding and alias audit. | No generator descriptor completeness and no corpus credit before signal-blind physical-parent accounting. |
| [Three-object impact dataset](https://zenodo.org/records/2563718) | Repeated microphone recordings of a glass beer bottle, plastic bucket and paper CD case at several locations. | Small acoustic/OOD control. | Only three physical objects and incomplete physical descriptors; no generator power. |
| [Giordano/McAdams plates](https://www.mcgill.ca/mpcl/files/mpcl/blg_smc_2006_jasa.pdf) | Twenty exact plate configurations across steel, soda-lime glass, walnut and PMMA; dimensions, thickness, support and striker are published. | Existing metadata-only physics control. | Search did not locate a public, exact stimulus-to-plate binding; zero waveform/corpus credit. |
| [Sounding Object](https://www.soundobject.org/) | Historical modal/contact synthesis examples and reports. | Literature/formula control. | Published examples are modeled demonstrations, not a measured multi-parent physical corpus. |
| Waste material recordings, DOI [`10.1016/j.array.2026.100913`](https://www.sciencedirect.com/science/article/pii/S2590005626002365) | Paper reports 6,000 recordings and 12,641 events over six materials. | `EXCLUDED_UNAVAILABLE`. | The publisher record says the data are confidential; there is no public payload to bind. |
| [VibraVerse](https://huggingface.co/datasets/technetium66/VibraVerse) | Public synthetic dataset over roughly 46,000 Objaverse-derived and generated shapes with geometry, material parameters, computed modes and synthesized impact sounds. | Second bounded synthetic modal-teacher candidate after exact lineage, generation and sample preflight. | It is synthetic and partly uses inferred/generated assets; it adds zero real microphone, validator, protected or physical-parent power. |

## Clatter audit details

The Clatter repository is especially valuable because it exposes a compact,
testable baseline rather than a black-box neural generator:

- `ImpactMaterial` contains 84 sized classes: ceramic, glass, metal, three
  wood hardnesses, cardboard, paper, hard plastic, soft foam, rubber, fabric,
  leather and stone, each with six size buckets;
- each `.bytes` payload stores arrays `cf`, `op` and `rt`; the runtime consumes
  the first ten entries and perturbs frequency by 10%, power by 10 dB and RT60
  by 10% using a seeded normal sampler;
- synthesis uses cosine modes with positive RT60-derived exponential decay,
  combines both colliding objects, convolves the response with a half-sine
  force whose duration depends on mass and is normally capped at 2 ms, then
  scales by collision amplitude;
- public inputs include both materials, mass, resonance, amplitude, relative
  speed, random seed and sample rate;
- the inspected parameter directory contains 84 files. The ordered aggregate
  path-neutral ordered aggregate hash of their individual SHA-256 output is
  `8230a6192f189806b899a9113c08fefaf458e9e7f26322e2fc6fe5434a4c065e`.

This is a stronger comparator than another hand-tuned recipe, but it remains a
coarse stochastic material prior. Size is bucketed from bounding extents or
volume, material constants are not the recipe input, radiation geometry is not
explicit and the synthesized signal is not measured microphone truth. V45
therefore compares against it but does not copy its implementation into the
runtime or let it validate itself.

## New evidence model

Every source row receives an immutable `claim_mask`; missing targets remain
missing and contribute no loss. The minimum lanes are:

| Lane | Observed relation | Typical evidence | Allowed use |
| --- | --- | --- | --- |
| `modal_teacher` | geometry/material → frequencies and mode shapes | NISR, VibraVerse, analytic/FEM fixtures | Pretraining and counterfactual modal controls only |
| `empirical_prior` | material/size → modal statistics | Clatter | Frozen baseline and optional prior regularization only |
| `structural_transfer` | force/contact/support → vibration response | Delft plate, prospective cello bridge rows | Transfer, damping and contact-location losses only |
| `real_acoustic` | physical parent/event → microphone waveform/target | IETeasy and disclosed real recordings | Radiation/residual calibration and disclosed evaluation |
| `validator_calibration` | independent real audio + corruptions → risk label | Future frozen CMU/other roles | Validator calibration only; never generator fitting |
| `protected_admission` | untouched independent real audio → final decision | Future source-power-qualified roles | One-shot holdout, validator qualification and joint shadow |

No loss may silently translate one lane into another. In particular:

- FEM modes do not prove microphone naturalness;
- accelerometer response does not prove acoustic radiation;
- a material label without geometry does not prove shape transfer;
- repeated hits do not multiply physical-parent power;
- a synthetic teacher or baseline never joins validator/protected roles;
- disclosed validator calibration never returns to generator training;
- a source derived from ObjectFolder stays in its alias component for project
  independence even when its rendering pipeline is new.

## Factorized recipe and training consequence

V45 should replace a monolithic audio target with three bounded heads under one
deterministic projection:

1. **Modal body:** ordered frequency ratios, modal participation and positive
   damping, supervised by synthetic modes, empirical priors and real acoustic
   targets where available.
2. **Excitation/transfer:** onset, contact duration, impact-strength response and
   bounded location/support modifiers, supervised by structural measurements
   and real events where available.
3. **Radiation/residual:** spectral tilt, band energy and short coloured
   residual envelope, supervised only by real microphone recordings.

Rows train only the heads they observe. Shared latent features are allowed only
after ablations prove that they do not degrade a directly observed lane. The
analytic projector owns ordering, positivity, energy and resource bounds; the
model predicts a recipe, never unrestricted PCM.

Synthetic pretraining may begin before the real `105`-parent floor is closed,
but it can return only `SyntheticTeacherUseful` or `SyntheticTeacherRejected`.
It cannot select the first material pack, qualify the validator, open protected
roles or authorize runtime use. Real disclosed fine-tuning still requires PSEL
and descriptor-signal evidence; production still requires independent one-shot
admission.

## Decision

Adopt [Roadmap V45](../plans/physical-sound-synthesis-roadmap-v45.md) with a
claim-masked multi-fidelity lane. Continue the V44 real-source search in
parallel, preserving the exact `71/105`, deficit `34`, project-independence and
protected-power gates. The first implementation checkpoint is a value-free
source-claim ledger; the first numeric external-prior experiment is Clatter,
but only after recipe V3 and a deterministic external decoder protocol are
sealed.

## Reconsideration conditions

Reconsider this split only if a public, revision-pinned source supplies enough
independent physical parents with exact runtime descriptors, event/force
correspondence and microphone waveforms to pass the existing signal-blind power
and lineage gates. Discovery of more files, repeated hits or synthetic renders
alone is insufficient.
