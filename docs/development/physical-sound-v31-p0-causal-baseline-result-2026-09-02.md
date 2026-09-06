# Physical sound V31 P0 — causal baseline result

| Field | Value |
| --- | --- |
| Date | `2026-09-02` |
| Decision | `PASS / PROTOCOL_FROZEN / REPEAT_EXACT / ZERO_SIGNAL` |
| Protocol | [V31 P0](physical-sound-v31-p0-causal-baseline-protocol-2026-09-02.md) |
| Product effect | None; P1 implementation is authorized, authored clips remain authoritative |

## Result

The external P0 owner accepted the frozen profile twice and emitted
byte-identical `contract.json`, `report.json` and CLI report bytes. It opened no
solver, audio, protected signal, waveform/feature or model value. No WAV,
dataset, checkpoint or generated value entered Git.

P0 therefore closes its roadmap criterion: geometry/material/contact/support
inputs, modal ordering, deterministic numeric behavior, resources, typed
fallback and all seven isolated causal interventions were frozen before P1
solver values. This result does not evaluate a solver or sound.

## Exact identities

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| Frozen profile | `7453` | `c6b7f816d65bdcaa18618a72d40cfedb6ed970359d3a1cf8f28825cd4c686a3d` |
| Bound protocol | `6842` | `fc5ca56495ef0f21d50685707a95d0253013228b4df3bf4abdc185a7fb37cd5a` |
| Contract owner | `23299` | `47f548dac2de3e1dbdde30c44d59a9f466bc1f02bf78b469e61ee3cfaa7be374` |
| A/B `contract.json` | `6978` | `9e77790a6aa7e0651f48b3902b4e2a8b68299d92d5568f79598f57beebcbf0c7` |
| A/B `report.json` | `1384` | `c56f643700ec192f3b0dae41cc481ee69e0651703dce8c26ad48655df4ee7619` |

The profile binds planning commit
`362378eb40e4e4a9c1b1789426ace1cbbab9f7d0`. Generated A/B artifacts remain in
external temporary storage and are not release assets.

## Gates

| Gate | Result |
| --- | --- |
| Two synthetic plate/beam fixtures and three analytic support/formula bindings | `Pass` |
| Isolated `E`, density, thickness, scale, impulse, contact and support interventions | `7/7 Pass` |
| Independent frequency-ratio and plate nodal-rule recomputation | `Pass` |
| Deterministic numeric and CPU/resource envelope | `Pass` |
| `FallbackOutOfDomain`, stable reason codes and no partial publication | `Pass` |
| Full owner A/B bytes | `Pass / exact` |
| Signal/network/model access | `0 / 0 / 0` |
| Focused P0 positive/failure tests | `9/9 Pass` |
| P0 plus inherited V24 analytic-teacher tests | `15/15 Pass` |
| SPEC-45 boundary scan | `Known baseline failure; no new P0 path finding` |

## Verification

```text
lab/.venv/bin/python -m py_compile \
  lab/scripts/physical_sound_v31_p0_causal_baseline_v1.py \
  lab/tests/test_physical_sound_v31_p0_causal_baseline_v1.py

lab/.venv/bin/python -m unittest \
  lab.tests.test_physical_sound_v24_t0_teacher \
  lab.tests.test_physical_sound_v31_p0_causal_baseline_v1

cargo run -p xtask -- boundary-scan
```

The focused suite covers exact A/B publication, formula/profile/resource
drift, missing/duplicate/non-isolated interventions, wrong support and contact
semantics, duplicate/non-finite/non-canonical JSON, protocol-hash drift and
unsafe output paths.

The boundary scan still reports the pre-existing
`SOURCE_LAYOUT_ESCAPE_HATCH` at
`tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`.
That file is byte-unchanged relative to the bound planning commit; none of the
new P0 paths appears in the diagnostic. The check is therefore reported as
failed, not silently converted to a pass, and remains a repository-wide
baseline risk outside this P0 boundary.

## Consequence

P1 is now the smallest authorized action: implement the deterministic modal
owner under this exact profile, including analytic plate/beam controls, nested
remesh identity, bounded energy and typed OOD fallback. P1 may fail or narrow
its supported domain; P0 grants no real material, validator, perceptual,
admission, cooker, demo or runtime credit.
