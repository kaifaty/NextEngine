# Physical Sound V13 — canonical modal-field research

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Status | `BOUNDED_RESEARCH_COMPLETE / ROADMAP_V13_JUSTIFIED` |
| Trigger | V12-C4a repeat-exact `DATA_INSUFFICIENT_FORCE_COVERAGE` |
| Product effect | None; clips remain authoritative |

## Falsifiable rebaseline

V12 tried to prove a true measured `force -> response` transfer before any
real contact model. ObjectFolder object `41` has valid, high-SNR impacts, but
their useful force spectra do not overlap: maximum target-bin support is two
contacts against a frozen minimum of four. Lowering the gate would manufacture
confidence rather than information.

The product does not need to wait for that stronger claim. It can first build
a **canonical-impact sound field**:

```text
exact object + canonical normalized impact + surface contact
    -> global modal frequencies/damping
    -> contact-dependent modal gains
    -> deterministic baked clips
```

This model cannot accept an arbitrary measured force waveform. A later record
may add that capability only after a new paired source passes acquisition
coverage. The claim distinction is stored in every research record and checked
by the validator.

## Current primary-source evidence

- The official [RealImpact repository](https://github.com/samuel-clarke/RealImpact)
  still publishes preprocessed per-object archives and says its raw dataset is
  not yet packaged. The public preprocessing code shows force-windowed
  deconvolution and emits `deconvolved_0db.npy`; the public archives expose the
  derived response but not raw paired channels.
- The [RealImpact paper](https://arxiv.org/abs/2306.09944) documents five
  impact locations and 600 listener measurements per impact for each of 50
  objects, plus calibrated hammer acquisition and force deconvolution.
- The newly published [AV-MSF paper](https://arxiv.org/abs/2608.05145)
  models object-global frequencies/damping and a spatial neural modal-gain
  field. It evaluates ObjectFolder Real with about 20% of 30–50 impacts for
  training and RealImpact with leave-one-impact-out evaluation.
- [DiffSound](https://hellojxt.github.io/DiffSound/) supports differentiable
  modal inverse problems, but its physical-parameter error can place modes or
  damping incorrectly; it is an initializer/control, not a validator.
- [NeuralSound](https://arxiv.org/abs/2108.07425) shows a safer ML role:
  approximate solver output is warm-started into a convergent eigensolver and
  learned radiation is compared against BEM. This supports offline acceleration
  with numerical residual checks, not unconstrained waveform generation.

## Important conflict with prior Next Engine evidence

AV-MSF includes a static filtered-noise residual and currently lists its code
as coming soon. Next Engine already rejected stationary random-phase magnitude
residuals and nearby neural codec/residual families under stricter spectral
and modal gates. V13 therefore does not copy that residual or treat the paper's
reported metrics as Next Engine admission.

The reusable part is narrower:

- object-global `frequency + damping`;
- a geometry/contact-conditioned modal-gain field;
- few-shot initialization before end-to-end refinement;
- held-contact comparison against KNN and geometry interpolation.

Any residual is a separate candidate with its own held evidence. Until then,
the baked atlas uses explicit modal output plus an authored/recorded clip
fallback. Unreleased upstream code is not a dependency.

## Decision

Roadmap V13 becomes a promotion pipeline rather than one all-or-nothing
experiment. Every exact object can end in one of four durable states:

1. `SourceQualified` — provenance/axes are usable, signal unopened;
2. `FallbackOnly` — clips can be catalogued, but no formula passes;
3. `FormulaValidated` — explicit modal field wins on held contacts;
4. `AtlasAdmitted` — independent validator and deterministic cooker pass.

Failures continue to populate negative/OOD evidence instead of triggering
manual threshold tuning. Human listening is report-only. The next action is an
experimental Research Record V0 schema plus a complete exposure/source ledger
before selecting any fresh object.
