# PS-2 existing-Iron broad-band V2 result — 2026-08-28

## Decision

`ExistingIronBroadbandV2MethodTransferSupported`.

Two byte-identical executions pass all 17 frozen gates. The rank-7 partial-SVD
core analyzes every one of the 31 input-driven regions and 91 neighboring bins
from the existing Iron Skillet block, produces 31 pre-prune clusters and retains
30 modes without ranking opened regions.

This supports the common-pole method on one opened real multi-listener object.
It does not prove metal identity, perceptual quality, exact-domain admission,
physical parameter recovery or production runtime fitness.

## Frozen lineage

| Artifact | SHA-256 / decision |
| --- | --- |
| Runner | `3f1c07b501dec97645cc0f1ad2fec5b3960312eacde88a291b64cc740a056aa5` |
| External manifest | `876e125dcef4a8c38e5bf7630b1115013cc02deb0c1dd5303d5bed244a5aa4bf` |
| Preflight A/B | `81f1b0898b1fdc72642700e9d2ff5d71e6e02d2cc4e012a61fe83ea9240f3fcf` / `ExistingIronBroadbandV2CounterfactualFrozen` |
| Analysis A/B | `f2fb359faaccb36544203780f07cc1d359c5d529be7f7f0d533ae39bd50b6a73` / `ExistingIronBroadbandV2MethodTransferSupported` |
| V1 capacity parent | `7635f8aca44286e0a46709aa3268afd049d1038023768a8a265d6a0b24a5075d` |
| Dense synthetic parent | `b9a5b226e2f96ecca7830d3adc6374f3c11baceed3b78e06eb71a2e58201978b` |

External artifacts remain under
`/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-realimpact-iron-skillet-broadband-v2-counterfactual-v1`.
Every report records zero additional payload, network, physics and Planter
access, no opened-region ranking and no quality/admission/runtime credit.

## Measurements

| Gate family | Observed |
| --- | ---: |
| Parent discovery identity | exact, 31 regions / 91 bins |
| Raw / duplicate removed / pre-prune clusters | `59 / 28 / 31` |
| Retained / pruned modes | `30 / 1` |
| Retained frequency range | `1,292.367…11,980.269 Hz` |
| Retained decay range | `6.6156…48.5388/s` |
| Even / odd partition match fraction | `0.833333 / 1.0` |
| Full damped / undamped NRMSE | `0.796572 / 0.991326` |
| Full damped/undamped NRMSE ratio | `0.803542` |
| Predictive damped/undamped SSE ratio, 171–341 ms | `0.005848` |
| Predictive damped/undamped SSE ratio, 341–683 ms | `0.002251` |
| Discovery under scales `0.125/1/8` | exact |

The deterministic work envelope is 91 rank-7 partial SVDs and a 62-column
amplitude design. Non-gating active-host measurements were `8.34 s / 282,756
KiB` peak RSS for analysis A and `19.88 s / 282,776 KiB` for analysis B. Their
variation is outside the byte-identical report and grants no production
performance credit.

## Interpretation

The result supports **H-capacity** over the alternatives tested here:

- the original 16/48 stop was an execution-envelope failure, not evidence
  against the modal representation;
- post-estimation clustering and energy pruning reduce 59 pole estimates to 30
  retained modes without a pre-estimation top-K;
- most retained frequencies reproduce independently in both microphone halves;
- fitted damping improves both the full observation and the two future windows
  relative to the frozen undamped ablation.

The absolute fit remains deliberately modest: full NRMSE is `0.796572`, and the
second predictive-window damped NRMSE is `0.963143`. The exceptionally small
predictive ratios arise partly because the undamped model diverges badly, not
because the damped reconstruction is perceptually transparent. Consequently
this report is not a sound-quality pass.

## Next action

Freeze an object- and family-disjoint internet-sourced real holdout before
opening its audio. Reuse the complete V2 algorithm and thresholds without
retuning. The holdout must have multiple synchronized listener rows, stable
object/impact identity and enough duration for both predictive windows. If no
eligible unopened source exists, publish that acquisition blocker rather than
reuse an opened calibration object.

Only a byte-identical independent holdout pass may support broader real-method
transfer. Material/domain admission and perceptual quality remain separate,
later gates.
