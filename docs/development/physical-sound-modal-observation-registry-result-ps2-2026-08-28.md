# PS-2 modal-observation registry V0 result — 2026-08-28

## Decision

`ModalObservationRegistryV0BuiltFallbackOnly`.

Two builds emit byte-identical registry and report bytes from the frozen Iron
Skillet and Iron Mortar reports. The registry is a reproducible experimental
observation base: it preserves method, modal, spatial and residual facts while
making non-admission and fallback machine-readable.

It is not a formula base, quality validator, material model, public content
schema or runtime source. Both entries remain authored-clip fallback only.

## Frozen lineage

| Artifact | SHA-256 / value |
| --- | --- |
| Runner / manifest / preflight | `a958c795…c064b` / `0f473f4d…05c20` / `fbffbe1e…441c0` |
| Registry A/B | `e0bec857edee2af087ff970631dcc8c389803c3fcde24b2947edbce224e09e98` |
| Build report A/B | `34a7f209e446cb0261bff4231f7401899501000543eb554501f10ddd80cdaed0` |
| Registry bytes / entries | `277,667 / 2` |
| Retained / pruned modes | `43 / 3` |

Artifacts remain external under
`/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-modal-observation-registry-v0`.

## Seed entries

| Entry | Modes | Frequency range | Full NRMSE | Future-window damped NRMSE |
| --- | ---: | ---: | ---: | ---: |
| `17_IronSkillet`, impact 0, outputs `0..14` | 30 retained / 1 pruned | `1,292.367…11,980.269 Hz` | `0.796572` | `0.792251 / 0.963143` |
| `43_IronMortar`, impact 0, outputs `0..14` | 13 retained / 2 pruned | `2,831.221…10,746.985 Hz` | `0.762138` | `0.999703 / 0.999999` |

Every fitted cluster retains its source members, representative frequency and
decay, energy, prune state and 15 complex amplitudes. The records also carry
discovery, scale invariance, even/odd replication, the complete source method
gate, full reconstruction and two future-window residual controls.

## Admission interpretation

Both entries expose exactly these states:

- method supported by the frozen source gate;
- quality disabled because no absolute residual threshold is validated;
- exact domain disabled because no exact-domain evidence exists;
- runtime disabled because V0 is a Proposed report-only experiment;
- authored clip fallback required.

This closes the deterministic adapter/storage step. It does not close absolute
prediction quality: Mortar leaves effectively all late-window signal
unexplained, and the two opened seeds cannot define their own acceptance bar.

## Next action

Freeze an object-grouped batch manifest before new candidate evaluation. Reuse
exact internet-derived observations where possible, keep object/family parents
within one partition, and bind train/calibration/holdout roles before comparing
an explicit modal-only baseline with transient and stochastic/residual
candidates. Candidate selection and reporting must be automatic; uncertainty
selects fallback rather than requesting per-sound human approval.
