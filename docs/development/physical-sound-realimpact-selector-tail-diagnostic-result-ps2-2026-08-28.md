# PS-2 Iron Skillet selector/tail diagnostic result — 2026-08-28

## Decision

`FixedTailTimingMismatchSupported`.

The repeated read-only diagnostic isolates the rejected `900 ms` persistence
observation as a cause of the Iron Skillet failure. It does **not** support the
competing source-selector-composition hypothesis:

- the source-derived onset set has `13/17 = 0.764705…` valid adaptive fits and
  therefore meets the unchanged `0.75` threshold;
- the comparator onset set has `15/16 = 0.9375` valid fits;
- source persistence is `12/17 = 0.705882…` at 100 ms, `13/17 = 0.764705…`
  at 200 ms and `11/17 = 0.647058…` at 400 ms, but falls to
  `6/17 = 0.352941…` at the rejected 900 ms baseline;
- the diagnostic comparator independently shows the same direction:
  `10/16`, `8/16`, `6/16`, then `2/16` across the same starts.

The previous `0.50` adaptive-valid result was measured only after the late
survival filter. That subset contains three invalid low-frequency candidates
among six survivors. The onset set itself passes adaptive validity, so the
parent report remains a correct rejection but no longer supports blaming the
±10% dominance composition.

## Frozen lineage

| Artifact | SHA-256 / decision |
| --- | --- |
| Runner | `8e8155986e57d99791300c65338d216de3f2f80fb0586332b0200ce4fec6b9bf` |
| External manifest | `00d0507426fbcadfbc2a84ee4cc08eea4c433e64df52e334f0ed9f0fb8c0281e` |
| Repeated preflight A/B | `4fc075c406ea19759d345d0eb6fff156f77e4ace8ae45b4959d3335ecba5ff2c` / `IronSkilletSelectorTailDiagnosticFrozen` |
| Repeated analysis A/B | `02551f2995764197575d5dd30bb36bd44da5452e2af79533b43922f44fec48d1` / `FixedTailTimingMismatchSupported` |
| Rejected parent A/B | `a9c4ae36ae04bf9a9772cb8a1a6c55e2862ed918eaff3ae6d9a2043acb07206d` |
| Decode report / block | `47acdc35449390f639cda34990520c5f74cb8627d0932079345ca35735ffc099` / `e26d1df1d547d059bbcbc57453c522b8c23402461feec09ff6149bc37a9c7eeb` |

Preflight and analysis reports repeat byte-identically. Both analyses use only
the already opened impact-zero rows `0..14`, onset sample `29`, unchanged
selectors, adaptive estimator, FFT and 40-cent injective matching. They report
zero network requests, zero additional payload, zero physics runs, zero
Planter bytes and no quality/admission/runtime credit. External reports remain
under:

`/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-realimpact-iron-skillet-selector-tail-diagnostic-v1`

## Frozen measurements

| Tail start | FFT support end | Source persistence | Comparator persistence | Source median match error |
| ---: | ---: | ---: | ---: | ---: |
| 100 ms | 1465.333 ms | `12/17 = 0.705882` | `10/16 = 0.625` | `0.01155 cents` |
| 200 ms | 1565.333 ms | `13/17 = 0.764706` | `8/16 = 0.50` | `0.02212 cents` |
| 400 ms | 1765.333 ms | `11/17 = 0.647059` | `6/16 = 0.375` | `0.17630 cents` |
| 900 ms | 2265.333 ms | `6/17 = 0.352941` | `2/16 = 0.125` | `0.13099 cents` |

Every FFT is still 65,536 samples, or `1365.333… ms`; the table changes only
the start. Therefore the result supports a timing/support mismatch in the
current long-window persistence test, not a precise instantaneous mode death.

The four source-onset candidates with invalid adaptive fits are approximately
`251.20`, `293.04`, `473.07` and `580.06 Hz`, with only `5.23–15.35 dB`
dynamic range. The other thirteen, from about `751.79` through `9787.59 Hz`,
are valid; their median fit `R²` is `0.993553`.

## Interpretation and next implementation

No observed early-tail value becomes a replacement parameter. Selecting
100/200/400 ms after observing Iron would tune the method on the opened object.
The preserved conclusions are narrower:

1. keep the synthetic ±10% selector control;
2. keep the complete real-transfer rejection and clip fallback;
3. retire `900 ms` as an admissible universal survival gate;
4. do not fetch another object or run mechanics yet.

Primary impact-analysis work models the signal directly as exponentially
damped components and performs order estimation in time-frequency subbands,
then removes insignificant or duplicate modes after estimation rather than by
a universal late survival observation. The next code package will therefore
freeze a synthetic multichannel Gabor/subband ESPRIT common-pole control with
known frequency, damping, model-order and node-channel truth. It must use no
Iron values or payload. Only a successful synthetic control may authorize a
separately preregistered real counterfactual; no result here grants material,
mechanics, perceptual quality, corpus admission, runtime or P1 credit.

## Sources

- [Pinned audio_dspy modal selector](https://raw.githubusercontent.com/jatinchowdhury18/audio_dspy/2ad0b05f81b014c27612f6f91087265ee52e9238/audio_dspy/modal_tools.py)
- [Pinned REALIMPACT modal notebook](https://raw.githubusercontent.com/samuel-clarke/RealImpact/fca2bd6cbb7e9f96ac61328d2a0d51594bf01987/modes_dsp_sweep.ipynb)
- [Sirdey et al., modal analysis of impact sounds with ESPRIT in Gabor transforms](https://www.dafx.de/paper-archive/2011/Papers/61_e.pdf)
- [Badeau, David and Richard, ESTER model-order analysis](https://perso.telecom-paristech.fr/grichard/Publications/SP06_Badeau1.pdf)
