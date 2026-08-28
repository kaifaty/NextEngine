# Multi-output decay estimator control result — PS-2 — 2026-08-28

## Outcome

The frozen synthetic control supports `spatial-modal-power-15-v1` on its
declared counterexample. Two independent executions emitted byte-identical
report
`a099f50d017e4f86ac7e2519663c457622e755ddd4a1969dcbd629fc0e0e8bea`
with decision `MultiOutputSpatialDecayControlSupported`. Every frozen gate
passes without changing the manifest, runner, fixture or threshold.

This result proves only that spatial power aggregation can recover known
shared modal decay when one listener is near a node and contains a delayed
local response. It does not establish that the REALIMPACT Ceramic failure has
that cause, admit any observation, validate perceptual quality or authorize a
production estimator.

## Exact identity

- manifest: `cd8ee8565d391a5543d39901c35b223c710f71968308db1e52ba7e50694b2078`;
- runner: `f0483c397843994d23793a4f3392dd49d9ce5d90ec218caa1b10f2a846a2068e`;
- repeated preflight: `88b017a1b35b1ff1bf350fc52fb30e62b38acbfc5e796913fe0980e4c83b1153`;
- repeated execution report: `a099f50d017e4f86ac7e2519663c457622e755ddd4a1969dcbd629fc0e0e8bea`;
- generated 15-channel fixture: `e9ac0ca845bba2943f5ae2936d48a51b4aefb57ec69119712d2787cb31e2be7d`;
- generated fixture size: `23,040,000` bytes; and
- both reports record zero network requests, real-payload bytes, physics runs
  and Planter-payload bytes.

The generated fixture remains an external transient result rather than a
repository asset.

## Frozen-gate result

| Measurement | Observed | Frozen criterion | Result |
| --- | ---: | ---: | --- |
| selected modes | `16` | `= 16` | pass |
| persistent recall | `1.0` | `>= 0.75` | pass |
| median repeated-tail frequency error | `0.00043 cents` | `<= 40 cents` | pass |
| spatial decaying-mode fraction | `1.0` | `>= 0.75` | pass |
| median tail-prediction RMSE | `0.34787 dB` | `<= 24 dB` | pass |
| median error against known decay | `0.35979 dB/s` | `<= 3 dB/s` | pass |
| node-channel decaying-mode fraction | `0.0` | `<= 0.49` | pass |
| spatial-minus-node improvement | `1.0` | `>= 0.25` | pass |

All 16 ground-truth associations are complete. Median selected-frequency error
against source truth is `0.00273 cents`.

## Decision boundary

The result falsifies the narrow claim that the unchanged V2 windows and
regressions necessarily prevent recovery of shared decay from multi-listener
power. It does not distinguish sensor nodes, room response, denoising or
another acquisition effect in Ceramic Cup.

No real-row result is authorized by this report. The next experiment must
first hash-close a read-only counterfactual that reuses the existing decoded
Ceramic Cup block and fixed reference row, aggregates all 15 microphones at
that same impact position, preserves the V2 thresholds, and declares a result
regardless of outcome. Fetching, row selection, mechanics and Planter remain
prohibited.

