# Physical sound R3A V11 B1 — force/response oracle result

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Protocol | [V11-B1 force/response oracle](physical-sound-r3a-v11-b1-force-response-oracle-protocol-2026-08-31.md) |
| Status | `REPEAT_EXACT_REJECT / DEVELOPMENT_FAILED / HOLDOUT_UNOPENED` |
| Decision | `REJECT_ESTIMATOR` |
| Product effect | None; authored clips remain authoritative |

## Outcome

The first force-normalized H1 revision is numerically useful but does not pass
its preregistered known-truth boundary. It predicts twelve unseen development
responses with `0.025706` mean NRMSE, versus `0.436192` for the
impulse-assumption control and `0.816787` for input-ignorant raw-output modal
fitting. It also recovers six shared poles with at most `0.034 Hz` frequency
and `0.307/s` damping error.

The conjunctive decision is nevertheless `REJECT_ESTIMATOR`:

- minimum contact-local valid coverage is `0.566656`, below `0.90`;
- only `6/7` truth modes survive the conditioned transfer, below `7/7`.

Every other development, identity, control-ratio and OOD gate passes. The
runner therefore stops before generating holdout. This result opens no real
source and grants no transfer-quality, validator, atlas or runtime credit.

## Exact identity and access accounting

Two complete runs are byte-identical for every emitted JSON and NPY file.

| Artifact | SHA-256 |
| --- | --- |
| Frozen manifest | `4de87ee4433e86d8f83791f73a0e2c406d53723e824550829bfdcc2f31dab3db` |
| Repeated preflight report | `c057a71aba19f633464b381de4cb326a098ac217f7590f71d895a6c7f91d8c29` |
| Model | `1e699d34cd8c1b2f00521ffe636519ca8fcac9703a98401d2919cde34aa1909a` |
| Run report | `700ada20a72bf9a9457e30a8c301e220ace935e255579a0376d20eace67ff376` |
| H1 transfer array | `57dfc1133c4a742fe26f01dc4ba0d0589366abd02fd3d6f96b3b1340347c8b53` |
| Conditioning array | `5866bb396d3b2111a9b8b7b4292b0edc18cff76bc98728d09f746a336333dc06` |
| Coherence array | `1fa7313444a7e2835b4345dfbec123298653ac1281088441ef98367ef1e69ed2` |
| Valid mask | `98a7b7262c9ca2efcabba0cec952cc2a5b1579c62bb758e5ddb90970885c6be3` |
| Modal reconstruction | `ed89761911e39dff32f4e039776f90a1102b4078fcb4602f651bf2ac9015c84d` |

External roots:

- `r3a-v11-b1-force-response-freeze`;
- `r3a-v11-b1-force-response-preflight-a` and `-b`;
- `r3a-v11-b1-force-response-run-a` and `-b`;
- all under `/home/kaifaty/.codex/experiments/nextengine/physical-sound/`.

The run generates `576,000` exact transfer samples, evaluates `24` fit and
`12` development trials, generates `0` holdout trials, reads `0` real samples
and performs `0` network requests. Role access order is exactly
`fit -> development`; holdout is absent from both outputs.

## Measurements

| Endpoint | Gate | Observed | Result |
| --- | ---: | ---: | --- |
| Noiseless held-response identity NRMSE | `<= 1e-11` | `0` | pass |
| Minimum valid coverage | `>= 0.90` | `0.566656` | **fail** |
| Truth modes / false positives | `7 / 0` | `6 / 0` | **fail** |
| Maximum frequency error | `<= 1.0 Hz` | `0.033959 Hz` | pass |
| Maximum absolute damping error | `<= 1.5/s` | `0.307196/s` | pass |
| Maximum relative damping error | `<= 0.20` | `0.010971` | pass |
| Mean / maximum held NRMSE | `<= 0.06 / 0.10` | `0.025706 / 0.040443` | pass |
| Mean gain-matched spectrum RMSE | `<= 1.5 dB` | `0.638725 dB` | pass |
| H1 / impulse NRMSE | `<= 0.65` | `0.058933` | pass |
| H1 / raw-modal NRMSE | `<= 0.75` | `0.031472` | pass |
| H1 / direct-division NRMSE | `<= 1.00` | `0.944497` | pass |

Contact-local valid coverages are `0.711646, 0.679154, 0.653650, 0.688160,
0.618113, 0.566656`. In contrast, the force-conditioning-only coverage is
`0.999758…0.999870`; almost all lost coverage is caused by the response
coherence half of the combined mask.

All seven input-driven Gabor energy regions are discovered, including the
`9,140.625 Hz` region near the `9,137 Hz` truth. The conditioned transfer
retains only six modes. Around the seventh truth mode, input conditioning is
still `-30.0…-33.1 dB`, but contact-local coherence crosses the hard `0.98`
boundary inconsistently; only `55.7…65.3%` of the `±20 Hz` bins survive. The
resulting fragmented ringdown is not retained as a common pole.

## OOD behavior

The fail-closed controls work as intended:

| Control | Observation | Decision |
| --- | --- | --- |
| weak excitation | high-band valid coverage `0.000069`; median reconstruction NRMSE `0.999981` | `OOD_WEAK_EXCITATION`, zero modes |
| low coherence | median coherence `0.219952`; valid coverage `0.002685` | `OOD_LOW_COHERENCE`, zero modes |

These passes matter: the estimator does not manufacture confident modes when
the force is genuinely uninformative or the response violates the coherent
single-input model. They do not override the clean-development failures.

## Bounded conclusion

The force/response factorization is not rejected. The selected H1 transfer is
already much better than input-ignorant controls on unseen force profiles, and
the six recovered poles are accurate. The rejected hypothesis is narrower:
one contact-local mask cannot use a `0.98` response-coherence requirement both
as a broad-band source-conditioning gate and as the support from which shared
poles are identified.

The clean source is broadly excited, while quiet response bins and
contact-specific antiresonances legitimately have weaker coherence. Conflating
those cases removes valid common-modal evidence, especially for the weakest
high-frequency mode.

## Next discriminator

A fresh B1R revision must be preregistered before another numeric run. It may
not relax the two failed gates on this opened revision. The new mathematical
hypothesis is a two-level joint modal FRF estimator:

1. keep a force-only conditioning mask for source observability and OOD;
2. discover shared poles from pooled cross-contact evidence where at least a
   declared contact subset is coherent;
3. solve each contact's residues only on its valid modal neighborhoods and
   publish per-mode/contact uncertainty;
4. evaluate on fresh synthetic phases/noise seeds with the same held-force and
   corruption families, then open its holdout only after development passes.

Only a repeat-exact `PASS_KNOWN_TRUTH_FRF` from that fresh revision may open
V11-B2 zero-decode internet-source work. Directly lowering `0.98`, reducing the
`0.90` gate, dropping the seventh mode or reading new real data is forbidden.
