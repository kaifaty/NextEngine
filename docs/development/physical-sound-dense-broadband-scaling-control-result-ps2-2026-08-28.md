# PS-2 dense broad-band scaling control result — 2026-08-28

## Decision

`DenseBroadbandScalingSyntheticControlSupported`.

Two byte-identical new-seed executions pass all 18 frozen gates. A rank-7
partial-SVD common-pole core processes 32 input-driven regions and 96
neighboring bins, removes 122 duplicate estimates, recovers all 64 dense truth
modes and reconstructs the fixture while admitting none of 16 weak off-region
nuisances.

This supports a larger synthetic offline analysis envelope and the numerical
equivalence of the partial core on the sparse parent. It does not prove Iron
transfer, perceptual quality, material identity, domain admission or runtime
fitness.

## Frozen lineage

| Artifact | SHA-256 / decision |
| --- | --- |
| Runner | `8e4cc18ba25f90adb43483e5d4ff1ea7be46869e55c28ebd7c9bb5831b29fbbe` |
| External manifest | `5ca8b1dad8a1d68f2cea5042c0a6ff305ffa6edc8658ea5739e1a8d0909e1ab0` |
| Preflight A/B | `81b44decea2beb070afd0e69ea51b70c7336430a5c3fcbe6994f783f32e3ac95` / `DenseBroadbandScalingControlFrozen` |
| Run A/B | `b9a5b226e2f96ecca7830d3adc6374f3c11baceed3b78e06eb71a2e58201978b` / `DenseBroadbandScalingSyntheticControlSupported` |
| Fixture | `a6a7366f7da344086a25ead482c0d1506c4c515b8c35d8788abc4d007089f7a6` |
| Modal truth | `b19a25c82ba7f8fa7b528daf11dca585e6aa74caabf3148bab06d714ea742eac` |
| Gabor coefficients | `18f2f96f3c1c95616f1b508d67d9b37033c0b14b984614eb73b5c5290f11f742` |
| Iron capacity rejection parent | `7635f8aca44286e0a46709aa3268afd049d1038023768a8a265d6a0b24a5075d` |

External artifacts remain under
`/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-dense-broadband-scaling-control-v1`.
Every report records zero real payload, network, physics and Planter access and
no quality/admission/runtime credit.

## Measurements

| Gate family | Observed |
| --- | ---: |
| Full/partial parent order and pole counts | exact over all 18 bins |
| Maximum parent frequency/decay delta | `9.09e-13 Hz / 1.34e-12/s` |
| Maximum parent order-margin delta | `2.04e-8` |
| Dense discovery | 32 regions / 96 bins |
| Weak nuisance regions admitted | 0 of 16 |
| Raw / duplicate removed / clusters | `186 / 122 / 64` |
| Truth matches / false positives | `64 / 0` |
| Maximum frequency/decay error | `0.080465 Hz / 0.610466/s` |
| Modal-truth NRMSE | `0.041496` |
| Full-observed NRMSE | `0.054329` |
| Post-transient NRMSE | `0.049039` |
| Discovery under scales `0.125/1/8` | exact |

The deterministic work envelope is 96 rank-7 partial SVDs, maximum selected
order 6 and a 128-column amplitude design for 64 clusters. Non-gating active-host
measurements were `39.25 s / 1,180,560 KiB` peak RSS for run A and
`19.94 s / 1,180,608 KiB` for warm-cache run B. Wall time and RSS are not
inside the byte-identical report and grant no production performance credit.

## Interpretation and next action

The control supports **H-capacity** as the cheapest next explanation:

- the original 16/48 Iron envelope was not a mathematical limit;
- partial SVD is numerically equivalent on the supported sparse parent;
- a 32/96 dense fixture remains fully recoverable without selecting a top-K;
- explicit weak off-region nuisances do not force extra discovery regions.

It does not eliminate leakage in the real Iron spectrum. That question can be
answered only after the complete real region set is estimated and
post-estimation energy, spatial replication and predictive controls are
observed.

The next allowed package is an Iron V2 counterfactual that changes only complete
SVD to the supported rank-7 partial core and expands the operational envelope to
32 regions/96 bins. It must analyze all 31 already discovered regions, retain
the unchanged amplitude/pruning and damped-versus-undamped controls, run twice
from the existing decoded block and avoid any opened-region ranking. No new
payload, object, physics or Planter is authorized.

## Sources

- [Sirdey et al., dense real-metal Gabor/ESPRIT analysis](https://www.dafx.de/paper-archive/2011/Papers/61_e.pdf)
- [Potts and Tasche, fast ESPRIT with partial SVD](https://doi.org/10.1016/j.apnum.2014.10.003)
- [Karnik, Romberg and Davenport, multitaper leakage bounds](https://arxiv.org/abs/2103.11586)
