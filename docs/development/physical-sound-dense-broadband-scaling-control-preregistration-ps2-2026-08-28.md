# PS-2 dense broad-band scaling control preregistration — 2026-08-28

## Outcome

Freeze a fresh-seed synthetic control that decides whether the Iron rejection
was caused by an unnecessarily sparse execution envelope rather than a need to
rank opened real regions. The control must process 32 input-driven regions and
96 neighboring bins, recover 64 strong common poles, ignore 16 weak off-region
nuisances and reconstruct the dense modal truth.

The only algorithmic change from the supported broad-band candidate is a
deterministic rank-7 partial SVD. Before the dense fixture receives any credit,
the partial core must reproduce selected order, pole count, frequency, damping
and order-score margin over every bin of the frozen seven-mode parent.

## Research decision

The first Iron counterfactual found 31 regions/91 bins and stopped against
16/48. Three explanations were considered:

- **H-capacity:** real metal legitimately has dense energetic modes and the
  frozen execution envelope is too sparse;
- **H-discovery:** Blackman-Harris leakage or weak nuisances cause most regions;
- **H-both:** dense analysis is required, but repeated complete SVD work also
  makes the envelope impractical.

Primary evidence favors testing capacity before changing selection:

- Sirdey et al. analyze a real metal impact over Gabor energy peaks, initially
  recover about 250 modes and remove duplicate/insignificant modes after
  estimation; their paper does not justify a fixed small top-K.
- Potts and Tasche show that partial SVD/Lanczos plus Hankel structure reduces
  ESPRIT cost while retaining exponential-sum parameter identification.
- Multitaper estimates can reduce leakage, but changing the discovery
  representation now would mix the capacity and leakage hypotheses. It remains
  a later candidate only if the explicit weak-nuisance control fails.

Sources:

- [Gabor/ESPRIT impact analysis](https://www.dafx.de/paper-archive/2011/Papers/61_e.pdf)
- [Fast ESPRIT with partial SVD](https://doi.org/10.1016/j.apnum.2014.10.003)
- [Multitaper leakage bounds](https://arxiv.org/abs/2103.11586)

## Calibration/holdout separation

Discarded seed `20260901` calibrated fixture strength and frozen thresholds.
It is not evidence and produces no stored report. One non-gating calibration
execution on the active host took `19.75 s` and observed:

- 32 regions, 96 bins, 182 raw estimates and 64 clusters;
- 118 duplicate estimates removed, 64/64 truth matches and zero false modes;
- maximum frequency/decay errors `0.102535 Hz/0.575794/s`;
- modal/full/post-transient NRMSE
  `0.047738/0.058920/0.056398`;
- partial/full parent deltas below `1e-12` for frequency/decay and
  `2.04e-8` for order-score margin.

The runner can execute only the previously unseen PCG64 seed `20260902`.
Fixture, gates and thresholds cannot change after preflight.

## Frozen fixture

- 15 outputs, 48 kHz, 60,000 samples, onset 240;
- 32 Gabor region bins `12 + 7*i`, `i=0..31`;
- two strong modes per region at centre ±17 Hz, for 64 truth modes;
- deterministic output participation/phases and bounded varying decays;
- 16 weak nuisance modes three bins above every second strong region,
  amplitude `0.01`;
- a shared broadband transient decaying at `70/s` and independent
  `1e-8` noise.

Weak nuisance bins must not become discovery regions. They are input evidence
against indiscriminate capacity expansion; truth frequencies remain unavailable
to discovery, pole estimation, clustering, amplitude fitting and pruning.

## Frozen candidate and gates

The Gabor/discovery, `50×` order margin, ±1-bin neighborhood, 1 Hz duplicate
clustering, joint amplitude fit and `-25 dB` post-estimation prune remain
unchanged. Complete SVD is replaced only by
`scipy.sparse.linalg.svds(..., k=7, solver="propack", which="LM")` with a
fresh fixed RNG seed zero per bin.

All gates must pass:

- parent order/count exact; maximum parent frequency/decay delta `1e-8`,
  order-margin delta `1e-4`;
- exactly 32 regions/96 bins and zero selected weak regions;
- at least 100 duplicate estimates removed;
- exactly 64 pre-prune clusters, 64 retained, 64 truth matches and zero false
  positives;
- maximum frequency/decay error `0.25 Hz/0.75/s`;
- modal/full/post-transient NRMSE at most `0.10/0.20/0.10`;
- exact discovery under scales `0.125/1/8`.

Success decides `DenseBroadbandScalingSyntheticControlSupported` and
authorizes only a separately frozen Iron V2 over all discovered regions.
Failure rejects the revision without tuning the holdout or reopening Iron.

The runner, external manifest and two byte-identical zero-real preflights MUST
be committed before the holdout. No real payload, network, Iron frequencies,
physics, Planter, quality admission or runtime credit is allowed.

## Frozen evidence identity

Runner SHA-256:
`8e4cc18ba25f90adb43483e5d4ff1ea7be46869e55c28ebd7c9bb5831b29fbbe`.

External manifest SHA-256:
`5ca8b1dad8a1d68f2cea5042c0a6ff305ffa6edc8658ea5739e1a8d0909e1ab0`.
Preflight A/B are byte-identical at
`81b44decea2beb070afd0e69ea51b70c7336430a5c3fcbe6994f783f32e3ac95`
and decide `DenseBroadbandScalingControlFrozen`. Each binds the supported
broad-band parent, the Iron capacity rejection, the dense fixture, partial-SVD
revision and gates while recording zero real payload, network, physics and
Planter access and no quality/admission/runtime credit.

External artifacts remain under
`~/.codex/experiments/nextengine/physical-sound/ps2-dense-broadband-scaling-control-v1/`.
This runner and preregistration MUST be committed before numeric holdout
execution.
