# Ceramic Cup observation causal diagnostic result — PS-2 — 2026-08-28

## Outcome

The frozen 27-row offline diagnostic repeats byte-identically at report
`47b578ac3706743778fba753f5260594c75a9fff46c4cba6761d7f8def8d2603`
with decision `CeramicCupSharedDecayMismatchSupported`. Twenty-three of 27
preselected rows fail the unchanged V2 gate. This rejects a row-7-local
explanation and supports a field-wide mismatch between the current early-decay
statistic and this force-deconvolved multi-listener observation.

The frozen low-frequency association criterion does not pass. Across all 432
selected row-modes, fitted-decay fraction is `0.3583` below `500 Hz` and
`0.2252` at/above `500 Hz`; the low/high ratio is `1.5907`, not `<= 0.5`.
Sub-`500 Hz` dominance remains a source limitation, but it does not explain the
decay-gate failure observed here. A high-pass cutoff is therefore not the next
repair.

No network request, new payload, threshold change, denoising, mechanics or
Planter access occurred. Four passing rows remain diagnostic only and cannot
replace frozen reference row 7.

## Exact result

| Scope | Failed / total | Failed fraction | Decaying-mode fraction min / median / max |
| --- | ---: | ---: | ---: |
| Union | `23 / 27` | `0.851852` | `0.1875 / 0.3125 / 0.5625` |
| Height | `15 / 15` | `1.0` | `0.1875 / 0.3125 / 0.4375` |
| Angle | `7 / 10` | `0.7` | `0.1875 / 0.3125 / 0.5625` |
| Distance | `3 / 4` | `0.75` | `0.1875 / 0.3125 / 0.5` |

Only `4/26 = 0.153846` non-reference rows pass, far below the frozen `0.8`
listener-local criterion. Shared failure passes both its `0.8` union threshold
and `0.5` minimum on every axis.

| Frequency band | Modes | Fitted decay | Tail decay | Positive fit then decaying tail |
| --- | ---: | ---: | ---: | ---: |
| `< 500 Hz` | `321` | `0.358255` | `0.429907` | `0.252336` |
| `>= 500 Hz` | `111` | `0.225225` | `0.459459` | `0.378378` |

The higher band actually has fewer modes classified as early-decaying, while
its negative-tail fraction is similar. This falsifies the preregistered simple
frequency-band explanation and points at the early single-output statistic.

## Hypothesis disposition

- **H1 low-frequency room/deconvolution cause:** rejected by the frozen band
  association, though low-frequency contamination remains an evidence caveat.
- **H2 row-7 listener-local cancellation/build-up:** rejected; all height rows
  and most angle/distance rows fail.
- **H3 single-output early-decay mismatch:** supported as the smallest surviving
  explanation. Positive early fits followed by negative tails recur across the
  field.
- **H4 acquisition/identity failure:** remains rejected by the hash/CRC/metadata
  lineage and byte-identical analysis.

The result does not prove a replacement estimator. It only identifies the
layer that the next synthetic counterfactual must test.

## External evidence and candidate direction

The [REALIMPACT paper](https://arxiv.org/html/2306.09944v1) measures room RT60
across all 600 listener positions and warns that generic denoising both removes
low-frequency noise and can shorten real modes, biasing damping. Denoising and
a simple `500 Hz` cutoff are therefore unsafe repairs.

The original [Complex Mode Indication Function paper](https://doi.org/10.1016/0888-3270(88)90060-X)
uses multiple response functions to identify global modal parameters and modal
participation. For one excitation and multiple microphone outputs, the first
singular magnitude reduces to the Euclidean norm of the response vector. This
motivates a bounded spatial-energy decay counterfactual; it does not by itself
validate CMIF or a production estimator for Next Engine.

## Decision and next action

Do not tune V2, filter below `500 Hz`, select one of four passing rows, open a
third object or run mechanics. Freeze a synthetic-only multi-output control:

1. generate 15 output channels from the same known 16 exponentially decaying
   modes with fixed spatial participation, including modal nodes and phase;
2. retain the exact V2 frequency/window conventions;
3. replace per-row modal power only with summed spatial modal power;
4. require recovery of known frequency/decay signs and improvement over a
   deliberately node-contaminated single-output control; and
5. run no real Ceramic row until that synthetic control is committed.

If the control fails, reject spatial-energy aggregation. If it passes, freeze a
separate read-only Ceramic counterfactual, still without admission or mechanics
credit, then require an object-disjoint calibration/holdout before changing the
validator.
