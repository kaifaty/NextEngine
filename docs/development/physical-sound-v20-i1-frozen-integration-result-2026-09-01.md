# Physical sound V20 I1 — frozen integration result

| Field | Value |
| --- | --- |
| Date | `2026-09-01` |
| Status | `COMPLETE / REPEAT_EXACT_REJECT / ACTUAL_ACOUSTIC_PASS / F0_REMESH_REJECT / P0C_COUNTERFACTUAL_DEFECT` |
| Protocol | [V20 P0c](physical-sound-v20-p0c-i1-integration-protocol-2026-09-01.md), SHA-256 `b591164a09c4a9aeb078cef7ab5f4dbebd1c2117b09a8fd21277be79427218c5` |
| Implementation | Git `85f0f91eb506b9ba351c5e9168635271f372de1f` |
| Allowed claim | The frozen B0+C0+F0 composition passes all four phase-consistent acoustic thresholds, but V20 I1 rejects on one frozen remesh field gate and one incorrect P0c counterfactual conjunction |
| Product effect | None; no real role, cooker, demo, public contract or runtime path opens |

## Execution identity

The implementation and eight metadata-only focused tests were committed before
any I1 mesh or value was generated. Two official runs wrote independent roots:

- `/home/kaifaty/.codex/experiments/nextengine/physical-sound/v20-i1-frozen-integration-run-a`;
- `/home/kaifaty/.codex/experiments/nextengine/physical-sound/v20-i1-frozen-integration-run-b`.

All nine corresponding files are byte-identical. Each root contains
`10,522,371` file bytes and has complete-file-map digest
`a46fe6ea85889233a4c660b31f61018d9e61a67529fba99fbb40483f886a732c`.

| Artifact | SHA-256 |
| --- | --- |
| `access-ledger.json` | `9fe292bcbda421cda2e9a41821b8042aa2932208b7f8ee13cd49010bbf5bac31` |
| `corpus.json` | `9a88f1b0440460ed246ff3bb1619f945801bdaa6333fbb959e766be81d0f9539` |
| `decisions.jsonl` | `bfcac5ea424237116bf29407c7318a226588fb1f860730b8e1fc85d51ad92521` |
| `endpoints.npz` | `0bcddf27facec5e72450ba4803956597c1ba1f95057316490184d9f470681322` |
| `fallback.json` | `841c57768357e9eea01a006e51d82503f7b18f04d8e86b72348178f01c701c89` |
| `geometry.npz` | `63c233f6327b6b0c3a86475ee98a8624bf33d11450f81989d478dba45e0ad418` |
| `manifest.json` | `1026dbeafaa76d114284d176662bcec1318c207586a9371732501f578d8affcc` |
| `metrics.jsonl` | `454526508a02ead75e3d4646d235e5edf2ea1d4f7fefb4ec9c8f36b603ac9929` |
| `report.json` | `743e9e9c44bf03bec76f26ab3204e9b0c46a36baf512075d65354b16ada9c638` |

The manifest pins B0 tree `bffd8bf5…1111`, F0 tree `7918d8b4…a718`, M0b
tree `7eb5c75e…b22`, row root `f1709914…b7b8` and implementation hashes:

| File | SHA-256 |
| --- | --- |
| `physical_sound_v20_i1_common.py` | `c9ba3846c3d28052b21282244ba87e5b2d41e8e5ff01c9de50066070d6b7c299` |
| `physical_sound_v20_i1_integration.py` | `a88df61cf1e508ee331318e2b3546aae5e986896022ddd35fd3d1162a76153b3` |
| `test_physical_sound_v20_i1_integration.py` | `aa13ebddabb176b874dc0a8d30cf604aa6ca96b1e0a015e2a7e5cdc0eba05d7b` |

## What passed

All 24 views, 12 primary/twin groups and 192 waveform cases are complete and
finite. Lineage, serialization, resource ceilings, fallback equality, zero
forbidden access and every hard mutation pass. B0 global quality passes:

| Metric | I1 value | Gate |
| --- | ---: | ---: |
| frequency cents median / p95 / max | `5.4692 / 14.4215 / 25.1764` | `<=20 / 60 / 60` |
| damping relative median / p95 | `0.001841 / 0.004691` | `<=0.08 / 0.20` |

F0 also passes its absolute field claims: gain NRMSE mean/max is
`0.154924/0.248288`, while edge-gradient p99 mean/max is
`0.331393/0.558422`. Every topology, control-ratio, paired-win, mutation,
coverage, structural and round-trip gate passes except the one remesh metric
described below.

Most importantly, the complete combined sound passes every M0b acoustic gate:

| Blocking p95 metric | I1 | Threshold |
| --- | ---: | ---: |
| MRSC | `0.3922929855` | `0.6860209113` |
| mean-centered log magnitude | `0.4328596893` | `0.4616213252` |
| normalized decay-slope residual | `0.1148117464` | `0.1471435733` |
| transient energy | `0.0211287551` | `0.0921538629` |

This is evidence that the phase-consistent instrument works and that the
combined endpoint is acoustically inside its frozen synthetic envelope. It is
not real-material or perceptual admission.

## Why I1 rejects

### 1. One F0 remesh gate

Eleven groups pass the frozen `gain_metric_drift <=0.10` gate. Wood/Bowl group
`v20-i1-integration-wood-bowl-baseclamped-1707` reaches
`0.1344067209`. Its primary/twin gain NRMSE values are approximately
`0.162233/0.184039`; the absolute change is small, but the frozen relative
metric is binding. Direct canonical-probe disagreement still passes at
`0.0896566`; corpus mean/max probe disagreement is `0.0817470/0.1266995`
against `<=0.13/0.18`.

The result therefore attributes a narrow, real F0 discretization-consistency
deficiency. The opened I1 values may diagnose it but may not choose a model,
hyperparameter or replacement threshold.

### 2. P0c assigned an extra owner to alternating signs

The alternating modal-sign mutation is correctly rejected by its signed-gain
physical owner: gain NRMSE mean/p95 is `1.309048/1.843776`. P0c additionally
required MRSC or mean-centered log magnitude to reject, but M0b never calibrated
either metric as the owner for signed gain. They remain below their unrelated
thresholds at `0.221247/0.396639`; DSR happens to exceed its threshold at
`0.150774` but is not a signed-gain owner.

This is a P0c protocol defect, not evidence for loosening M0b metrics. A
successor protocol must restore the M0b ownership rule: signed-gain rejection
is blocking and acoustic values are diagnostic for this mutation. Historical
P0c and this I1 result remain immutable rejects.

## Access and verification

The ledger records 12 generated physical groups, 24 views, 192 waveform cases
and 11 controls after the implementation commit. Real waveform, force,
generator-real, protected calibration, method holdout, shadow, successor-band,
dataset, checkpoint, source-body and network counters are all zero.

- focused unittest: `8/8 PASS` before value generation;
- Ruff `0.16.3`, formatting and `py_compile`: pass;
- official runs A/B: same `REJECT` and nine byte-identical files;
- old B0/C0/F0/M0b artifacts: read-only exact-hash dependencies.
- mapped `boundary-scan`: blocked by the pre-existing
  `SOURCE_LAYOUT_ESCAPE_HATCH` in `realimpact_transfer_fixture.rs`; no changed
  document or I1 implementation introduces that finding.

## Decision

V20 closes as `REPEAT_EXACT_REJECT`. The next roadmap may open only:

1. a protocol-only P0d correction that removes the uncalibrated spectral
   conjunction from alternating signs without changing any metric threshold;
2. a preregistered F1 field family on fresh `1801…2101` bands whose explicit
   target is representation/remesh consistency;
3. a fresh integration after F1 passes its own unopened test.

Real roles, validator, shadow, cooker, demo and runtime remain sealed. Authored
clips remain the complete fallback.
