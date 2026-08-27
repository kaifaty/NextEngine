# Physical sound validator AV-P0A — 2026-08-27

| Field | Result |
| --- | --- |
| Scope | External deterministic rigid-impact validator; no runtime or public content contract |
| Command | `cargo run -p xtask -- physical-sound-eval --manifest <external-json> --output <external-empty-directory>` |
| Manifest | `nextengine.experimental-physical-sound-validator.manifest.v1` |
| Report | `nextengine.experimental-physical-sound-validator.report.v1` |
| Outcomes | `Pass`, `Reject`, `FallbackOutOfDomain` |
| Claim | Physical-control conformance only; no subjective naturalness, material identity or shipping-quality claim |

## Implemented boundary

AV-P0A validates an external hash-closed generator probe set. It does not
listen to sounds, call a network/model provider, load a learned evaluator,
modify engine content or replace the clip baseline.

The automatic decision is:

- `Reject` when an impact fails the hard signal gate or a declared causal/
  metamorphic relation fails;
- `FallbackOutOfDomain` when the declaration/control matrix is incomplete, the
  source family is not `rigid-impact`, reference evidence is unusable or the
  validator's deterministic mutation controls fail;
- `Pass` only when every object/material pair has a valid zero-force silence,
  exact repeated WAV bytes, ordered force response and local position-continuity
  control, every impact entry participates in a relation, all relations pass,
  the internal mutation suite repeats exactly and the authored fallback itself
  passes the hard signal gate.

References and the existing spectrum/modal/decay distances remain diagnostic.
They do not become an uncalibrated quality threshold. A blind A/B bundle is
written only with explicit `--write-blind-bundle` and never changes the
decision.

## Current manifest shape

The format is current-only experimental data under ADR-046. A v0 quality
manifest is rejected rather than silently interpreted with the new decision
semantics.

```json
{
  "schema": "nextengine.experimental-physical-sound-validator.manifest.v1",
  "split": "glass-vessel-controls-v1",
  "validator": {
    "source_family": "rigid-impact",
    "generator_revision": "example.glass-vessel.v1",
    "generator_sha256": "<64 lowercase hex digits>",
    "deterministic_probe_seed": 7,
    "fallback": {
      "path": "fallback.wav",
      "sha256": "<64 lowercase hex digits>"
    }
  },
  "relations": [
    {
      "kind": "exact_wav_repeat",
      "id": "repeat-center-medium",
      "left": "repeat-a",
      "right": "repeat-b"
    },
    {
      "kind": "force_response",
      "id": "response-force-center",
      "ordered_entries": ["force-low", "force-medium", "force-high"]
    },
    {
      "kind": "position_continuity",
      "id": "response-position-medium",
      "ordered_entries": ["position-left", "force-medium", "position-right"]
    }
  ],
  "entries": [
    {
      "id": "force-high",
      "object_id": "thin-glass-vessel",
      "material": "glass",
      "impact_position": "center",
      "force_band": "high",
      "expected_signal": "impact",
      "control": {
        "impact_position_micrometres": [0, 0, 0],
        "impulse_micronewton_seconds": 4000000
      },
      "candidate": {
        "path": "force-high.wav",
        "sha256": "<64 lowercase hex digits>"
      },
      "reference": null
    },
    {
      "id": "force-low",
      "object_id": "thin-glass-vessel",
      "material": "glass",
      "impact_position": "center",
      "force_band": "low",
      "expected_signal": "impact",
      "control": {
        "impact_position_micrometres": [0, 0, 0],
        "impulse_micronewton_seconds": 1000000
      },
      "candidate": {
        "path": "force-low.wav",
        "sha256": "<64 lowercase hex digits>"
      },
      "reference": null
    },
    {
      "id": "force-medium",
      "object_id": "thin-glass-vessel",
      "material": "glass",
      "impact_position": "center",
      "force_band": "medium",
      "expected_signal": "impact",
      "control": {
        "impact_position_micrometres": [0, 0, 0],
        "impulse_micronewton_seconds": 2000000
      },
      "candidate": {
        "path": "force-medium.wav",
        "sha256": "<64 lowercase hex digits>"
      },
      "reference": null
    },
    {
      "id": "position-left",
      "object_id": "thin-glass-vessel",
      "material": "glass",
      "impact_position": "left",
      "force_band": "medium",
      "expected_signal": "impact",
      "control": {
        "impact_position_micrometres": [-10000, 0, 0],
        "impulse_micronewton_seconds": 2000000
      },
      "candidate": {
        "path": "position-left.wav",
        "sha256": "<64 lowercase hex digits>"
      },
      "reference": null
    },
    {
      "id": "position-right",
      "object_id": "thin-glass-vessel",
      "material": "glass",
      "impact_position": "right",
      "force_band": "medium",
      "expected_signal": "impact",
      "control": {
        "impact_position_micrometres": [10000, 0, 0],
        "impulse_micronewton_seconds": 2000000
      },
      "candidate": {
        "path": "position-right.wav",
        "sha256": "<64 lowercase hex digits>"
      },
      "reference": null
    },
    {
      "id": "repeat-a",
      "object_id": "thin-glass-vessel",
      "material": "glass",
      "impact_position": "center",
      "force_band": "medium",
      "expected_signal": "impact",
      "control": {
        "impact_position_micrometres": [0, 0, 0],
        "impulse_micronewton_seconds": 2000000
      },
      "candidate": {
        "path": "repeat-a.wav",
        "sha256": "<same exact WAV hash as repeat-b>"
      },
      "reference": null
    },
    {
      "id": "repeat-b",
      "object_id": "thin-glass-vessel",
      "material": "glass",
      "impact_position": "center",
      "force_band": "medium",
      "expected_signal": "impact",
      "control": {
        "impact_position_micrometres": [0, 0, 0],
        "impulse_micronewton_seconds": 2000000
      },
      "candidate": {
        "path": "repeat-b.wav",
        "sha256": "<same exact WAV hash as repeat-a>"
      },
      "reference": null
    },
    {
      "id": "zero-force",
      "object_id": "thin-glass-vessel",
      "material": "glass",
      "impact_position": "center",
      "force_band": "zero",
      "expected_signal": "silence",
      "control": {
        "impact_position_micrometres": [0, 0, 0],
        "impulse_micronewton_seconds": 0
      },
      "candidate": {
        "path": "zero-force.wav",
        "sha256": "<64 lowercase hex digits>"
      },
      "reference": null
    }
  ]
}
```

Entries and relations are strictly sorted by ID. Relation members use declared
causal order rather than lexical order. Force relations stay within one object,
material and exact micrometre impact position, with strictly increasing
micronewton-second impulse. Position relations keep one impulse and use
distinct neighboring coordinates no more than 250,000 micrometres apart.
Exact-repeat relations preserve the labels, physical control values and WAV
bytes. Missing physical values make the domain ineligible for `Pass`.

Relation thresholds are frozen in the hashed AV-P0A evaluator profile rather
than supplied by the generator manifest: force RMS must rise by at least 1 dB
per declared step while modal assignment stays at or below 0.2; neighboring
positions must differ by at least 0.1 dB RMS while log-spectrum RMSE stays at
or below 8 dB and modal assignment at or below 0.25. These are deterministic
control tolerances, not learned naturalness thresholds. AV-P0B/P0C must measure
grouped real-corpus risk before `Pass` can mean more than the limited AV-P0A
claim.

## Deterministic mutation controls

Every run evaluates and repeats a fixed internal 48 kHz control profile:

- clipping below/above the hard threshold;
- DC offset below/above the hard threshold;
- duration above/below 50 ms;
- silence with missing onset.

The report stores both control-profile hashes and requires byte-identical
results. These controls verify the deterministic hard-gate implementation;
they are not evidence that a generator sounds natural.

## Remaining work

AV-P0B still needs a license-reviewed real-impact subset and grouped
leave-object/position/generator-out benchmarks. AV-P0C still needs a specialist
head, an OOD detector and a separately calibrated risk-coverage threshold.
Until those steps pass, autonomous optimization may use AV-P0A to eliminate
numerical and causal failures but must not optimize or claim perceptual quality
from its `Pass` alone.
