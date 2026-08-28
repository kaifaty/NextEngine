# REALIMPACT Ceramic Cup observation result — PS-2 — 2026-08-28

## Outcome

The frozen observation-first protocol rejects official REALIMPACT
`78_CeramicCup` before any mechanics comparison. Acquisition report
`b6d25bc67db09e1b63798e30061cd357106ec2de33b5847fb7795934f4a16a0c`,
decode report `9f1c23118ac2d451bd423d2f2a8deb604ad5ccd085900501682b6e31e71adaf0`
and two byte-identical analyses
`56591bb82ab50470296013c432c57e2b31b5addbae002ccbc833d595f3823fd9`
establish decision `CeramicCupObservationRejected`.

Four of five unchanged V2 gates pass. Decaying-mode fraction is `0.25`, below
the frozen `0.50` minimum. Thirteen of sixteen selected modes are below
`500 Hz`. The runner therefore authorizes no scalar, shell, volumetric, BEM or
cooker execution and no further request. Planter remains sealed and authored
clips remain the fallback.

This is the second independent object whose observation fails the same decay
gate: opened Pitcher measured `0.4375` and selected `11/16` modes below
`500 Hz`. The repeated failure invalidates “try another object unchanged” as
the next action. It does not authorize weakening the gate or selecting a
passing listener after looking at the data.

## Exact lineage

| Artifact | Result |
| --- | --- |
| Frozen manifest / runner | `71123b21…cae5` / `bad26592…60e2` |
| Acquisition report | `b6d25bc6…6a0c` |
| Spatial metadata block | `579a5f36…379a`, `3756` bytes |
| Condition metadata block | `62fafadf…3360`, `1355` bytes |
| Compressed audio prefix | `80b38cc9…cf56`, `536870912` bytes |
| Decode report / raw rows | `9f1c2311…daf0` / `3405843a…e6ca` |
| Decoded block | 600 rows × 208895 `<f4` samples, `501348000` bytes |
| Analysis A/B | `56591bb8…3fd9`, byte-identical |
| Network requests | `3`, then closed |
| Additional network / physics / Planter access | `0 / 0 / 0` |

The metadata proves one impact vertex (`1024`) at
`[0.00146395, -0.02246722, -0.00105312] m`, ten angles
`0..180°` in `20°` steps, distances `0/333/666/1000 mm`, microphones
`0..14` and all 600 unique listener positions. Frozen row `7` is microphone 7
at angle/distance `0/0` and listener coordinate `[0.23, -0.04345, 0.0] m`.

## Observation gate

| Check | Observed | Frozen threshold | Result |
| --- | ---: | ---: | --- |
| Selected modes | `16` | `>= 6` | pass |
| Persistent recall | `0.75` | `>= 0.50` | pass |
| Median frequency error | `19.3476079145 cents` | `<= 40` | pass |
| Decaying-mode fraction | `0.25` | `>= 0.50` | **fail** |
| Median tail-prediction RMSE | `9.1420193738 dB` | `<= 24` | pass |

The first selected mode is `251.65 Hz`; only modes at approximately
`507.61`, `539.26` and `645.81 Hz` are at or above `500 Hz`. Twelve modes have
a persistent tail match, yet twelve of sixteen early fitted slopes are not
more negative than `-1 dB/s`. The failure is therefore specifically an
early-decay interpretation problem, not missing peaks or tail-frequency drift.

## Bounded causal hypotheses

### H1 — low-frequency room/deconvolution energy dominates the extractor

**For.** Both failed objects select mostly sub-`500 Hz` peaks. The REALIMPACT
paper reports longer room reverberation below `500 Hz`, while its room RT60 is
below `0.2 s` above that boundary. It also reports that denoising particularly
removes low-frequency noise for ceramic objects.

**Against / uncertainty.** Room reverberation normally extends decay rather
than necessarily producing a positive `50..900 ms` fitted slope. The spatial
distribution of the failure has not yet been measured.

### H2 — row 7 is a listener-specific spatial cancellation/build-up case

**For.** REALIMPACT exists precisely because transfer varies over 600 listener
locations. A fixed microphone may sit near modal nodes or reflection extrema.

**Against / uncertainty.** Pitcher and Ceramic Cup both fail at the same
central reference microphone, but that alone cannot distinguish a listener
effect from a shared extractor bias.

### H3 — the V2 early-decay statistic is mismatched to force-deconvolved fields

**For.** Ceramic Cup has `0.75` persistent recall, `19.35 cents` median
frequency error and `9.14 dB` tail RMSE, yet only `0.25` early decaying modes.
Many fitted slopes are positive while later slopes are negative. The paper
warns that generic denoising can shorten modes and overestimate damping, so
blind denoising is not a valid repair.

**Against / uncertainty.** The statistic passed the earlier object-disjoint V2
holdout. Its failure may be domain- or listener-local rather than universal.

### H4 — acquisition or row identity is wrong

**Against.** HTTP identity, ranges, member CRCs, NPY shapes/dtypes, the full
`10 × 4 × 15` product, prefix/decode hashes and A/B reports all agree. This is
retained only as a falsification target, not the leading explanation.

## External evidence

The [REALIMPACT paper](https://arxiv.org/html/2306.09944v1) states that room
RT60 is below `0.2 s` above `500 Hz` and longer below it. Its denoising appendix
reports strong low-frequency noise removal for ceramic examples, but also that
denoising can shorten real modes and bias damping upward. The official
[preprocessing source](https://github.com/samuel-clarke/RealImpact/blob/fca2bd6cbb7e9f96ac61328d2a0d51594bf01987/preprocess_measurements.py)
constructs the force-deconvolved `<f4` response and the repeated 15-microphone
metadata used by the local decoder.

## Decision and next action

Do not open a third object, retune the five gates, choose a listener post hoc,
denoise, or run mechanics. Freeze one offline-only diagnostic over a
value-independent subset of the already decoded 600 rows:

1. all 15 microphones at angle/distance `0/0`;
2. centre microphone 7 at all ten angles and distance `0`; and
3. centre microphone 7 at angle `0` and all four distances.

Report unchanged V2 gates, selected-frequency bands and early-versus-tail
slopes for every frozen row. Resume implementation only if this discriminates
listener-local failure from a shared low-frequency/decay-statistic failure.
