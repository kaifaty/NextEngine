# PS-2 subband common-pole control result — 2026-08-28

## Decision

`SubbandCommonPoleSyntheticControlSupported`.

Two byte-identical synthetic-only executions pass all eleven frozen gates. A
15-output Gabor/subband signal-subspace estimator recovers all six declared
common damped poles, including two pairs closer than the ordinary 1024-point
FFT-bin spacing, without a fixed 900 ms survival observation.

This supports only the preselected high-energy-subband mathematical core. It
does not yet discover active subbands, merge duplicates from adjacent bins,
estimate/prune perceptual energy, resynthesize the full sound or transfer to a
real recording.

## Frozen lineage

| Artifact | SHA-256 / decision |
| --- | --- |
| Runner | `4acf8016791a0072ea68ce8902d3e4e9d0230ec4438f26a55f0254b6a02bf141` |
| External manifest | `38025d254a24596bb742b188c032cec8fa610968a7ed9a0db050abf5eebf8703` |
| Preflight A/B | `9fb1c48896b456934595b4e4b108286122e3fb51274565e5f89ad69858f64e4a` / `SubbandCommonPoleControlFrozen` |
| Run A/B | `3fcb5fc864283576add71192afcf4b73b8ddd9ceccde5b3dead9c8acc442284f` / `SubbandCommonPoleSyntheticControlSupported` |
| Synthetic fixture | `787f91e5b65c7bab7717ed6661acf60dfbca0416945340fe190512b30ead4931` |
| Gabor coefficients | `faafc76b9ab15a7498fbb596a9fe5e57f4493d470f1bb200525eb578478109f2` |
| Fixed-tail parent | `02551f2995764197575d5dd30bb36bd44da5452e2af79533b43922f44fec48d1` |

All artifacts other than source and documentation remain under
`/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-subband-common-pole-control-v1`.
Each report records zero real payload, network, physics and Planter access and
no quality/admission/runtime credit.

## Measurements

| Band centre | Truth `(Hz, decay/s)` | Selected order | Score margin | Maximum errors in band |
| ---: | --- | ---: | ---: | --- |
| 1125 Hz | `(1108,2)`, `(1142,4)` | 2 | `46380×` | `0.0097 Hz`, `0.0309/s` |
| 3000 Hz | `(2986,3)`, `(3017,7)` | 2 | `8800×` | `0.0179 Hz`, `0.1463/s` |
| 6000 Hz | `(5992,5)` | 1 | `97664×` | `0.0164 Hz`, `0.1166/s` |
| 9000 Hz | `(8990,12)` | 1 | `2691×` | `0.0431 Hz`, `0.2591/s` |

Global maximum frequency error is `0.043014 Hz` against the frozen `0.25 Hz`
limit; maximum amplitude-decay error is `0.259092/s` against `0.50/s`. Orders
and estimates are exactly invariant after linear scales `0.125`, `1` and `8`.

Output 7 has exactly zero participation in the 1142 Hz truth mode. Its
single-output analysis selects order one and estimates only about
`(1108.008 Hz, 2.0408/s)`. The 15-output estimator selects order two and
recovers both 1108 and 1142 Hz modes. This is the declared evidence that common
multi-output poles survive a listener/node null where one microphone cannot
identify the complete mode set.

## Interpretation

The control closes one mathematical question: a Gabor coefficient time series
retains enough common-pole structure for bounded multichannel ESPRIT-family
frequency/damping recovery, and the direct rotational-invariance score selects
the declared order in this fixture.

It does not close the engineering path to arbitrary internet recordings:

- band bins are declared by the fixture rather than discovered from input;
- the same pole may appear in several adjacent Gabor bins and is not merged;
- only stable in-band poles are filtered; amplitude, phase, mode energy,
  significance and transient/noise residual are not estimated;
- there is no reconstruction-error or perceptual-transparency gate;
- the order score is an ESTER-family residual, not the complete published fast
  recurrence or perturbation-bound implementation.

The next bounded implementation is therefore a separate synthetic broad-band
control. It must select high-energy Gabor regions without truth frequencies,
run the frozen common-pole core, cluster adjacent-bin duplicates, fit
multi-output complex amplitudes, prune only after estimation and pass
frequency/damping/order plus reconstruction/residual gates on a new seed. No
real counterfactual, new object, mechanics or Planter access is authorized yet.

## Sources

- [Sirdey et al., Gabor/ESPRIT impact analysis](https://www.dafx.de/paper-archive/2011/Papers/61_e.pdf)
- [Badeau, David and Richard, ESTER perturbation/order analysis](https://perso.telecom-paristech.fr/grichard/Publications/SP06_Badeau1.pdf)
- [SVD-based EDS order selection](https://ftp.esat.kuleuven.be/stadius/ida/reports/05-107.pdf)
