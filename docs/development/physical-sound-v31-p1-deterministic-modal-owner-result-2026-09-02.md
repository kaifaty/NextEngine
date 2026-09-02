# Physical sound V31 P1 — deterministic modal owner result

| Field | Value |
| --- | --- |
| Date | `2026-09-02` |
| Decision | `PASS / REPEAT_EXACT / SYNTHETIC_ONLY / FALLBACK_REQUIRED` |
| Parent | [V31 P0 causal baseline](physical-sound-v31-p0-causal-baseline-result-2026-09-02.md) |
| Product effect | None; T0 is authorized, authored clips remain authoritative |

## Result

The external P1 owner implements the frozen P0 plate and beam formulas,
mode-shape participation, ten-mode modal records, nested remesh fields and the
three-second `48 kHz` float32 reference render. Two complete processes emit the
same 56 files byte-for-byte.

The owner evaluates two baseline fixtures and all seven isolated interventions:
Young's modulus, density, thickness, uniform scale, impulse, contact and
support. It also executes one unsupported probe for every frozen reason code.
Those probes return `FallbackOutOfDomain`, retain the authored clip class and
publish no partial candidate.

This is a causal mechanics result, not an audio-quality result. No real
recording, protected signal, model, dataset or checkpoint was opened. The
synthetic material is not Steel, Glass or Wood, and the emitted WAVs are
controls rather than product assets.

## Exact identities

| Artifact | SHA-256 |
| --- | --- |
| Frozen P0 profile | `c6b7f816d65bdcaa18618a72d40cfedb6ed970359d3a1cf8f28825cd4c686a3d` |
| P1 owner | `04b44d77ce073d07f30e94c3c361ca4c199cb554ecbe017947aa423842781650` |
| A/B `evidence.json` | `fd51f384b1745749f0ac11a1f7609c18dbc34d8532d524881b50e8065270b37a` |
| A/B `report.json` | `f3452250b3ba205cbfeda7a6dd05bc954819c7429636ad3ccfca55f1e33a3089` |
| Baseline plate WAV | `3cc882dd9e367c5fd24047d9a711049583db1ee02bb722e3723c26eae60ef25b` |
| Baseline beam WAV | `911f765ea08f02a21e6d847f457d5fe9a32b31b6873b5cf7df58b775e0311e62` |

Each complete output is `6,418,813` bytes. Generated meshes, gain fields, modal
records and WAVs remain in external temporary storage and do not enter Git.

## Gate evidence

| Gate | Result |
| --- | --- |
| P1 frequencies versus inherited V24 plate/cantilever teacher | `Exact` |
| Plate boundary displacement | `0` |
| Beam clamp displacement | `0` |
| Maximum cantilever characteristic residual | `4.66861256532609e-15` |
| `E`, density, thickness, scale and impulse frequency-ratio error | `0` |
| Support first-mode frequency-ratio error | `0` |
| Impulse response | `2x binary64 exact`; participation unchanged |
| Target plate `(m=2,n=1)` contact participation | `0 exact`; frequencies unchanged |
| Coarse/fine common-vertex gain fields | `9/9 exact` |
| Finite/no-clipping/decaying modal-envelope bounds | `9/9 Pass` |
| Typed authored fallback probes | `6/6 Pass` |
| Signal/network/model access | `0 / 0 / 0` |

## Resource evidence

The two measured owner processes completed in `0.48 s` and `0.50 s`; peak RSS
was `52,340 KiB` and `52,164 KiB`. The frozen limits are `300 s`, `1 GiB` RSS
and `256 MiB` output. The deterministic live-array bound reported by the owner
is below that memory ceiling, and every case remains below one by construction:
ten modes, 825 fine vertices and 144,000 render frames.

## Verification

```text
lab/.venv/bin/python -m py_compile \
  lab/scripts/physical_sound_v31_p1_modal_owner_v1.py \
  lab/tests/test_physical_sound_v31_p1_modal_owner_v1.py

lab/.venv/bin/python -m unittest \
  lab.tests.test_physical_sound_v24_t0_teacher \
  lab.tests.test_physical_sound_v31_p0_causal_baseline_v1 \
  lab.tests.test_physical_sound_v31_p1_modal_owner_v1

cargo run -p xtask -- boundary-scan
```

The P1-focused suite passes `8/8`; the inherited V24 + P0 + P1 suite passes
`23/23`. It covers full-process A/B identity, exact
modal/WAV/mesh formats, V24 equivalence, analytic controls, all interventions,
remesh and energy bounds, every fallback reason, dependency/profile/numeric
corruption, artifact corruption, unsafe paths and atomic failure cleanup.

The boundary scan still fails on the pre-existing
`SOURCE_LAYOUT_ESCAPE_HATCH` at
`tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`.
That file is unchanged from the P1 parent commit, and no P1 path appears in the
diagnostic. This remains a reported repository-wide baseline risk rather than
being relabelled as a pass.

## Consequence

P1 closes only the deterministic classical owner. T0 is the smallest next
action: freeze clean causal cases plus wrong-decay, frozen-carrier,
shuffled-envelope, mode-collapse, spectral-copy, clipping and provenance
mutations with exact expected validator outcomes. Real validator calibration,
ML training, admission, cooking, demo integration and runtime promotion remain
blocked by their later gates.
