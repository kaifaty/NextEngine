# Ceramic Cup multi-output counterfactual result — PS-2 — 2026-08-28

## Outcome

The frozen real-data counterfactual rejects
`spatial-modal-power-15-v1` as a remedy for the Ceramic Cup observation gate.
Two independent executions emit byte-identical report
`9947c427a3416b3a3635c43ea24e074df84ce4db97f0a22cca3be8c93ac96cbf`
with decision `CeramicCupMultiOutputCounterfactualRejected`. The full decoded
block is rehashed before each run and matches `3405843a…e6ca`.

Spatial aggregation passes mode count, persistence, frequency stability and
tail-prediction RMSE. It fails the frozen decay gate and the comparison gate:
decaying-mode fraction is `0.1875`, below both the `0.50` requirement and the
reference-row value `0.25`. The observed improvement is therefore `-0.0625`
rather than at least `0.25`.

No threshold, input row or implementation changed after preflight. The result
grants no observation admission, perceptual-quality, mechanics or runtime
credit.

## Exact identity and measurements

- manifest: `add0017b6350bcfd5a500d1c0e776bb3b56733c1e329001d72d0f8239b1625e1`;
- runner: `b5635c4297c00e67d83dfab094f859b3f7b76cf800b95786224b08f92116fb7a`;
- repeated preflight: `7bf54ec89b837864a9d79bc16ecc9e1275995d945e29fd411759df2dfeb1e0e1`;
- repeated analysis: `9947c427a3416b3a3635c43ea24e074df84ce4db97f0a22cca3be8c93ac96cbf`;
- existing payload hashed per run: `501,348,000` bytes;
- existing rows analyzed per run: `12,533,700` bytes; and
- network, additional payload, physics and Planter counts: zero.

| Measurement | Spatial | Frozen criterion | Result |
| --- | ---: | ---: | --- |
| selected modes | `16` | `>= 6` | pass |
| persistent recall | `0.9375` | `>= 0.50` | pass |
| median repeated-tail frequency error | `12.68295 cents` | `<= 40 cents` | pass |
| decaying-mode fraction | `0.1875` | `>= 0.50` | **fail** |
| median tail-prediction RMSE | `1.90614 dB` | `<= 24 dB` | pass |
| decay-fraction improvement over row 7 | `-0.0625` | `>= 0.25` | **fail** |

Fifteen of the 16 selected spatial peaks are below `500 Hz`; the remaining
peak is `521.61 Hz`. Thirteen fit slopes are positive or no more negative than
`-1 dB/s`. This is consistent with a shared early-window mismatch, not an
isolated listener node.

## Causal audit after rejection

### H1 — the 15 rows came from incompatible impacts

**Rejected.** The paper documents a vertical column of 15 microphones, with
all microphones and the force transducer acquired in a time-synchronized
fashion. The pinned preprocessing source loads one `Force.wav` per
angle/distance condition, then loads microphones `1..15`, and repeats that
condition force over those microphone rows during deconvolution. The fixed
rows `0..14` are therefore a legitimate multi-output measurement of one
condition.

Sources: [REALIMPACT hardware and synchronized acquisition](https://arxiv.org/html/2306.09944v1),
[pinned preprocessing source](https://raw.githubusercontent.com/samuel-clarke/RealImpact/fca2bd6cbb7e9f96ac61328d2a0d51594bf01987/preprocess_measurements.py).

### H2 — one listener node caused the fixed-window decay failure

**Rejected for this object/revision.** The synthetic node counterexample passes
with spatial fraction `1.0`, while all 15 real outputs combined remain at
`0.1875`. Spatial aggregation changes the selected peaks and reduces tail
error, but does not recover negative early slopes.

### H3 — the fixed V2 fit interval is the wrong statistic for these impulse responses

**Supported as the next falsifiable hypothesis, not yet proven.** The
REALIMPACT paper describes bandpassing each picked mode, applying an RMS level
detector and regressing its energy envelope. The authors' pinned analysis
notebook chooses the fit interval per mode from the envelope peak and an
end-of-record noise floor; it does not use a universal `50–900 ms` interval.
The current candidate modes have low tail error but mostly rising early slopes,
the exact symptom expected when a fixed interval begins before a mode-specific
envelope maximum or spans a shared acquisition transient.

Source: [pinned REALIMPACT modal-analysis notebook](https://raw.githubusercontent.com/samuel-clarke/RealImpact/fca2bd6cbb7e9f96ac61328d2a0d51594bf01987/modes_dsp_sweep.ipynb).

### H4 — low-frequency room/deconvolution contamination dominates selection

**Plausible but not isolated.** The paper reports room RT60 below `0.2 s` above
`500 Hz` and longer reverberation below it, while the released preprocessing
performs direct FFT division by the windowed hammer spectrum without a
regularization term. The paper also reports non-negligible measurement noise.
This can explain shared low-frequency structure, but the earlier fixed-axis
diagnostic rejected the simpler claim that low modes alone have uniquely worse
fitted decay. A cutoff remains prohibited.

## Decision and next experiment

Do not tune the V2 interval, select higher-frequency modes or reuse Ceramic
again. First freeze a synthetic source-faithful adaptive-decay control with
known frequency/decay truth, delayed modal build-up and a bounded noise floor.
It should retain deterministic peak inputs, apply narrow mode filtering plus
an RMS envelope, derive the regression interval from the per-mode peak/noise
range, and compare against the rejected fixed-window estimator. Only a
repeatable full pass may authorize another separately frozen Ceramic
counterfactual. Fetching, mechanics and Planter remain blocked.
