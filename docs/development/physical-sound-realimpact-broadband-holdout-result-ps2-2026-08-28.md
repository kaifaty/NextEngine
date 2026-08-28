# PS-2 Iron Mortar broad-band independent holdout result — 2026-08-28

## Decision

`BroadbandIndependentHoldoutMethodTransferSupported`.

Two byte-identical offline analyses pass all 15 frozen gates on the previously
unopened, object- and name-family-disjoint REALIMPACT `43_IronMortar` holdout.
The complete input-driven region set is analyzed without top-K selection.

This supports the rank-7 partial-SVD common-pole extractor across one opened
development object and one independently frozen real holdout. It does not prove
perceptual quality, material identity, exact-domain admission, physical
parameter recovery or production runtime fitness.

## Frozen lineage

| Artifact | SHA-256 / decision |
| --- | --- |
| Protocol runner / manifest / preflight | `219c6279…ae85` / `73c218ed…a824` / `c6943d12…34fd` |
| Execution runner | `53f637789b2fa2f665f3f6eee1bddc8b9cfc1af6ea91dd70510d792ab6b7a8e9` |
| Execution manifest | `6915846dc63687ca6f9982a90eb43866203c8c79114c5a70b80fd4156737caaf` |
| Execution preflight A/B | `67e83d24b11933a325e0cf2951d446ed4cb1827509cd610250c5a02de853a67e` / `BroadbandIndependentHoldoutExecutionFrozen` |
| Acquisition | `1ac0fd3914103800e06c6227a8c2312e793ec74df88b189baac45dfc0d49fb41` / `BroadbandIndependentHoldoutInputsAcquired` |
| Decode | `5029930e271c27485f2d8e4110bdc0d0576d28277a8e7a3dec1565b29d42f127` / `BroadbandIndependentHoldoutObservationDecoded` |
| Decoded 15-row block | `19108dc93b4b3ee5fcc43fd1777454e25cfa99043483d93e636f68c69928d948` |
| Analysis A/B | `64bc63aa0cd02cfe9ad3959670102e883893f7d53a1427fddb5f660a8951d0cc` / `BroadbandIndependentHoldoutMethodTransferSupported` |

External artifacts remain under
`/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-realimpact-iron-mortar-broadband-holdout-v1`.

## Bounded access and identity

The single acquisition used exactly four requests: 1,355 metadata bytes and a
32 MiB observation deflate prefix. No retry or prefix growth occurred.
Metadata proves all 600 impact-zero rows share vertex 1315, cover 40
angle/distance groups and order microphones 0…14 inside every group. The
analysis condition is rows 0…14 at angle/distance `0/0`.

The prefix decoded exactly 15 × 208,375 f32 samples. Decode and both analyses
made zero new network, audio-payload, physics or Planter accesses.

## Measurements

| Gate family | Observed |
| --- | ---: |
| Input-derived onset | sample `27`, `0.5625 ms` |
| Discovery | 26 regions / 77 bins |
| Raw / duplicate removed / pre-prune clusters | `27 / 12 / 15` |
| Retained / pruned modes | `13 / 2` |
| Retained frequency range | `2,831.221…10,746.985 Hz` |
| Retained decay range | `16.7529…90.4263/s` |
| Even / odd partition match fraction | `0.846154 / 0.846154` |
| Full damped / undamped NRMSE | `0.762138 / 0.993895` |
| Full damped/undamped NRMSE ratio | `0.766820` |
| Predictive damped/undamped SSE ratio, 171–341 ms | `0.040533` |
| Predictive damped/undamped SSE ratio, 341–683 ms | `0.021967` |
| Discovery under scales `0.125/1/8` | exact |

The deterministic work envelope is 77 rank-7 partial SVDs and a 30-column
amplitude design. Non-gating active-host analysis measurements were
`4.72 s / 253,440 KiB` peak RSS and `5.88 s / 252,336 KiB`. They grant no
production performance credit.

## Interpretation

The independent result supports a bounded modal-observation extractor:

- region discovery remains under the synthetic 32/96 capacity envelope;
- post-estimation clustering/pruning produces a compact 13-mode observation;
- `11/13` retained frequencies reproduce in each disjoint microphone half;
- fitted damping improves full and future-window error relative to the frozen
  undamped ablation.

It does **not** support a perceptually complete generator. Absolute damped NRMSE
is `0.762138`; predictive damped NRMSE is `0.999703/0.999999`. The small
damped/undamped ratios mainly show that an undamped bank diverges badly, while
the damped bank leaves nearly all late-window signal unexplained. A transient,
residual/noise or richer radiation component remains necessary, and an
automatic quality validator must keep fallback enabled meanwhile.

## Next action

Freeze a report-only `ModalObservationV0` registry record and deterministic
builder from the exact Iron Skillet and Iron Mortar reports. Each record must
carry source/runner/report hashes, object/impact/listener identity, frequencies,
decays, complex spatial amplitudes, fit/prune provenance, spatial replication,
absolute residual metrics and admission state.

The builder must never convert method support into quality/domain admission.
After two exact seed records reproduce, extend the same builder to a frozen
object-grouped batch split and evaluate residual/transient model candidates
without per-object human validation. Public contracts and runtime use remain
blocked until a later Accepted decision and real ProductCheck.
