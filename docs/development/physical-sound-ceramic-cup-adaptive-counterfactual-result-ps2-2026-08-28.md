# Ceramic Cup adaptive-decay counterfactual result — PS-2 — 2026-08-28

## Outcome

The frozen adaptive Ceramic counterfactual is rejected. Two executions emit
byte-identical report
`2ec3b03e08792c34ca5113c63ea8d6174c845fddd6c1aa2d4bac54e9ee33d6b7`
with decision `CeramicCupAdaptiveDecayCounterfactualRejected`. Only `6/16`
selected components have the frozen `20 dB` dynamic range and minimum fit
length; valid-fit fraction is `0.375`, below `0.75`.

The valid six fits are negative and highly linear, but their median slope is
`-361.86 dB/s` and their envelope peaks occur at `6..9 ms`. The result improves
decaying fraction from fixed-window `0.1875` to `0.375`, short of both the
absolute `0.50` gate and required `0.25` improvement. Spectral count,
persistence, repeated-tail frequency error and median `R²` pass.

This rejects the estimator/input combination. It does not justify weakening
dynamic-range gates or treating the six extremely short fits as mechanical
damping.

## Exact measurements

- manifest `48001fb7…ff9a`;
- runner `e50bec23…056b`;
- repeated preflight `4358d4da…220f`;
- repeated analysis `2ec3b03e…d6b7`;
- full existing block rehashed per run: `501,348,000` bytes;
- existing rows analyzed per run: `12,533,700` bytes; and
- zero network, additional-payload, physics and Planter counts.

| Measurement | Observed | Frozen criterion | Result |
| --- | ---: | ---: | --- |
| selected modes | `16` | `>= 6` | pass |
| persistent recall | `0.9375` | `>= 0.50` | pass |
| median frequency error | `12.68295 cents` | `<= 40 cents` | pass |
| valid adaptive fits | `0.375` | `>= 0.75` | **fail** |
| decaying-mode fraction | `0.375` | `>= 0.50` | **fail** |
| median valid-fit `R²` | `0.997701` | `>= 0.95` | pass |
| improvement over fixed | `0.1875` | `>= 0.25` | **fail** |

Ten modes between `251.87` and `400.03 Hz` have only `10.40..19.61 dB`
dynamic range and are invalid. The six valid modes span `416.84..521.61 Hz`
and fit only approximately `12..80 ms` after the record start.

## Interpretation

The most economical inference is that current V2 spatial peak selection is
filling its 16 slots with a dense low-frequency deconvolution/filter transient
family, not a stable set of object modes. This inference is supported, but not
proved, by the extremely early peaks, very steep slopes and insufficient
dynamic range.

The pinned REALIMPACT notebook supplies a falsifying contrast. Its
`find_freqs` suppresses a candidate when a larger peak exists within ±`10%`
(`frac_off=0.1`). In the notebook's separate full-sweep ceramic-bowl example,
the resulting nine modes are widely spaced from about `830 Hz` to `21.2 kHz`,
rather than densely filling the low-frequency region. Those frequencies do not
transfer to Ceramic Cup, but the salience rule does.

Source: [pinned REALIMPACT modal notebook](https://raw.githubusercontent.com/samuel-clarke/RealImpact/fca2bd6cbb7e9f96ac61328d2a0d51594bf01987/modes_dsp_sweep.ipynb).

The published object archive contains no ready modal or transfer-map product;
its twelve entries end in `deconvolved_0db.npy`. A source-faithful selector
therefore has to be independently implemented and validated rather than
imported as hidden ground truth.

## Decision and next action

Close Ceramic Cup to further method development. Do not tune its frequency
floor, peak spacing, dynamic range or fit interval.

Next freeze a synthetic salience-selection control with known widely separated
object modes plus stronger dense low-frequency artifacts. Compare V2's fixed
`12 Hz/45 cents` separation against the source-derived ±`10%` dominance rule,
then require adaptive-valid mode recovery and known-truth precision. Only a
repeatable pass may proceed directly to a separately preregistered unopened
REALIMPACT object. Mechanics, admission and Planter remain blocked.

