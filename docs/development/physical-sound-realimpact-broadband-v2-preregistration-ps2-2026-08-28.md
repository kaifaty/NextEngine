# PS-2 existing-Iron broad-band V2 preregistration — 2026-08-28

## Outcome

Freeze one read-only transfer counterfactual over the complete broad-band region
set already discovered in the existing `17_IronSkillet`, impact-zero block. The
only algorithmic change from rejected V1 is the rank-7 partial-SVD core supported
by the dense synthetic control. The execution envelope expands from 16/48 to
32 regions/96 bins; no opened-region ranking or top-K selection is allowed.

This is a method-transfer test. It cannot establish material identity,
perceptual quality, domain admission, physical correctness or runtime fitness.

## Frozen parents and hypotheses

V1 report `7635f8ac…075d` stopped before pole estimation because input-driven
discovery produced 31 regions/91 neighboring bins. Dense synthetic report
`b9a5b226…978b` then proved full/partial parent equivalence and recovered 64/64
modes over 32 regions/96 bins with no false modes or selected weak nuisances.

The V2 result discriminates:

- **H-capacity:** all-region partial estimation, post-fit pruning and unchanged
  controls pass on the existing real block;
- **H-leakage/nuisance:** the larger execution envelope admits too many clusters
  or the fitted poles fail spatial/predictive controls;
- **H-representation:** a common-pole damped sinusoid bank cannot explain the
  observation better than the unchanged undamped ablation.

## Frozen observation and candidate

- existing decoded block SHA-256 `e26d1df1…c7eeb`, 600 × 230,549 f32 samples;
- only rows 0…14, onset 29, first 60,000 post-onset samples;
- unchanged 2,048-sample Blackman-Harris Gabor transform, 48-sample hop,
  150–12,000 Hz discovery, −30 dB local-region floor and ±1-bin neighborhoods;
- every region and analysis bin must exactly reproduce the V1 discovery;
- deterministic rank-7 `scipy.sparse.linalg.svds`/PROPACK with RNG seed zero;
- unchanged `50×` order-score margin and 1 Hz duplicate clustering;
- joint 15-output amplitude fit, then unchanged −25 dB mode-energy pruning;
- at most 64 pre-prune modes, therefore at most 128 amplitude-design columns.

The 64-mode limit is the largest fit demonstrated by the dense synthetic
control. Exceeding it rejects V2; it does not authorize another cap increase.

## Frozen controls and gates

Every gate must pass:

- discovery exact to V1: 31 regions/91 bins, within the supported 32/96 cap;
- exact discovery under amplitude scales `0.125/1/8`;
- at least one duplicate removed;
- 6…64 pre-prune clusters and 6…64 retained modes;
- at least 50% of retained full-output frequencies independently reproduced by
  each even/odd microphone partition within 40 cents;
- full-observation NRMSE at most `0.95`;
- damped/undamped full NRMSE ratio at most `0.95`;
- fit amplitudes only on samples 0…8,191, then damped/undamped squared-error
  ratio at most `0.95` separately on samples 8,192…16,383 and
  16,384…32,767.

Success decides `ExistingIronBroadbandV2MethodTransferSupported` and authorizes
only a separately frozen independent internet-sourced real holdout. Failure is
published without tuning, subset selection, retry or new payload.

## Data and claim boundary

The runner may read only the hash-bound decoded block already present in the
external experiment store. New payload, network, object, physics solver and
Planter access are forbidden. Reports record zero such access and grant no
quality, admission or runtime credit.

The runner, manifest and two byte-identical preflights MUST be committed before
numeric analysis. Analysis runs twice without threshold or variant changes.

## Frozen evidence identity

Runner, manifest and preflight hashes are filled only after the runner is
committed and the zero-analysis preflight repeats. External artifacts remain
under
`~/.codex/experiments/nextengine/physical-sound/ps2-realimpact-iron-skillet-broadband-v2-counterfactual-v1/`.
