# PS-2 broad-band common-pole control result — 2026-08-28

## Decision

`BroadbandCommonPoleSyntheticControlSupported`.

Two byte-identical executions on the frozen, previously unseen synthetic seed
pass all fifteen gates. Without receiving truth band centres, the candidate
discovers six energetic Gabor regions, analyzes their neighboring bins,
clusters duplicate common-pole estimates, fits multi-output complex amplitudes,
prunes one weak nuisance mode and reconstructs all seven strong modes.

This is synthetic method support only. It grants no real-transfer, perceptual
quality, domain admission, physics, Planter or runtime credit. The next allowed
step is a separately hash-closed, read-only counterfactual on the already
acquired Iron Skillet rows; no new object or payload is authorized.

## Frozen lineage

| Artifact | SHA-256 / decision |
| --- | --- |
| Runner | `317fa3a2e4f05f6c212ccdbee0575554d3610753573db359a68f3e756a73ec8d` |
| External manifest | `a8f59bdea71862c5d670448bc058f8838bfa41b292d8fea26dead90fb96d040b` |
| Preflight A/B | `eab320847fc1c2b3a18443ec49701e97256ac5f50fa263f32fbff5e3a53ce8bc` / `BroadbandCommonPoleControlFrozen` |
| Run A/B | `dcd832525fbd1955c9122fc52579e4a8ddd10ab76bc5d79393f8f9df9d270094` / `BroadbandCommonPoleSyntheticControlSupported` |
| Observed fixture | `6ee231475039c27b8ef20a3d01641aa40cd7f92fe7b27c50a4dbfea4e81aec1c` |
| Strong modal truth | `8b5f486530d10fa9f9bba8ca191bd2b438dab8fed99e1dcab3bb61d7c8a2a678` |
| Gabor coefficients | `863f89dabef524a4388019bfc01195f355a09555141db047d6c2492001633a17` |
| Parent subband report | `3fcb5fc864283576add71192afcf4b73b8ddd9ceccde5b3dead9c8acc442284f` |

The manifest and reports remain external under
`/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-broadband-common-pole-control-v1`.
Every report records zero real-payload bytes, network requests, physics runs and
Planter payload bytes, and explicitly denies quality/admission/runtime credit.

## Measurements

| Gate family | Frozen limit | Observed |
| --- | ---: | ---: |
| Input-driven discovery | 6 regions / 18 bins | `6 / 18` |
| Duplicate removal | at least 10 | `12` from 20 raw estimates |
| Cluster/prune counts | 8 pre-prune / 7 retained / 1 pruned | `8 / 7 / 1` |
| Strong truth / false positives | 7 / 0 | `7 / 0` |
| Maximum frequency error | at most `0.25 Hz` | `0.095876 Hz` |
| Maximum decay error | at most `0.75/s` | `0.190671/s` |
| Strong modal-truth NRMSE | at most `0.08` | `0.033913` |
| Full observed NRMSE | at most `0.20` | `0.130840` |
| Post-transient observed NRMSE | at most `0.10` | `0.049273` |
| Discovery under scales `0.125/1/8` | exact bins | exact |

The two close truth pairs are separated at `1373/1401 Hz` and `3479/3512 Hz`,
despite the `46.875 Hz` Gabor-bin spacing. All seven strong frequencies match.
The deliberately weak 7313 Hz nuisance is discovered and estimated before its
fitted energy (`-27.568 dB`) causes the frozen post-estimation prune; truth
labels are not used by discovery, estimation, amplitude fitting or pruning.

## Interpretation

This closes the synthetic end-to-end method gap left by the parent control:

- active analysis regions are selected from input energy, not truth centres;
- adjacent-bin copies are merged after common-pole estimation;
- per-output cosine/sine coefficients provide complex modal amplitudes;
- significance pruning occurs after estimation and amplitude fitting;
- resynthesis is checked against both strong modal truth and the noisy,
  transient-bearing observed fixture.

It does not show that the fixed discovery floor, score margin, cluster radius or
energy floor generalize to real internet recordings. It also does not establish
the correct onset/tail span for Iron, identify material or object class, judge
perceptual quality, or authorize a shorter universal survival gate.

The next experiment must therefore bind the existing Iron payload and decoded
block by hash, derive its Gabor regions from the input, run the frozen candidate
without truth, and publish descriptive stability/residual metrics plus declared
controls. The result may support or reject method transfer, but cannot tune the
candidate, open Planter, access physics, admit a domain or replace clip fallback.

## Sources

- [Sirdey et al., Gabor/ESPRIT impact analysis](https://www.dafx.de/paper-archive/2011/Papers/61_e.pdf)
- [Badeau, David and Richard, ESTER perturbation/order analysis](https://perso.telecom-paristech.fr/grichard/Publications/SP06_Badeau1.pdf)
- [SVD-based EDS order selection](https://ftp.esat.kuleuven.be/stadius/ida/reports/05-107.pdf)
