# Physical sound R2D context trainability V1 result

| Field | Value |
| --- | --- |
| Date | 2026-08-30 |
| Scope | Query-free objective/optimizer/cooker gate over the 420-row Green Goblet context field |
| Decision | `REJECT_TRAINING_SUBSTRATE_V1 / DECAYED_STEP_REVISION_NEXT` |
| Public/runtime authority | None; no spatial model or query audio ran, SPEC-45 remains `Proposed`, and clips remain mandatory |

## Question and answer

R2D V1 asked whether an energy-preserving coefficient objective and the real
complex-STFT cooker can fit a context-only rank-96 representation without the
R2C silence collapse.

Mostly, but not enough to pass the frozen gate. Identity, one-row, coefficient,
gradient, oracle-proximity and zero/global-mean comparisons pass. The
small-block and full-context tasks miss only the preregistered mean absolute
log-energy bound. The immutable decision is `RejectTrainingSubstrate` and
N0.3E remains unauthorized.

## Frozen context factorization

The freeze reads only the already validated 420-row R2C context cache. It
builds an exact context Gram matrix, canonicalizes eigenvector phase, streams a
rank-96 complex basis and applies a symmetric re-orthonormalization while
transforming coefficients inversely so reconstruction is unchanged.

| Artifact or observation | Exact value |
| --- | --- |
| Manifest SHA-256 | `3fe4129584da5395b1502c0f55e6061603c530ed9e214c48bfd13f3166c05f28` |
| Freeze report SHA-256 | `729cbf42ae25efd2e6e25b6cb6534d997f7e644a6f08d47bc8ab973a96542e51` |
| Retained total energy | `0.9963964439551598` |
| Best linear Frobenius NRMSE | `0.06002962639264227` |
| Basis orthonormal max error | `1.6689305084582884e-6` |
| Basis size | `321,177,600` bytes, external |
| Context cache read by freeze | `1,405,152,000` bytes |
| Query/method-holdout/shadow reads | `0 / 0 / 0` bytes |
| Optimizer steps before freeze | `0` |

Frozen PCM controls over the declared context probes behave as expected:
identity is exact; the global mean has `23.44 dB` mean level error; the silent
zero control has `188.86 dB`; and the rank-96 oracle has `0.0724 dB` mean level
error, `8.4610 dB` mean spectrum error and `-22.1692 dB` waveform NRMSE.

## Repeated optimizer result

Runs A and B produce byte-identical `run-report.json` with SHA-256
`350a1e6bb2421ca8bd7c58dbd6218e6e323052e74f8ed229caebc7c070f30afd`.
All three checkpoint hashes also match across runs. MLflow params, metrics and
artifacts remain in each external run directory; run IDs are non-deterministic
lineage and are excluded from the exact report comparison.

| Task | Coefficient RMSE | Raw NMSE | Mean abs log-energy | Oracle aggregate delta max | Result |
| --- | ---: | ---: | ---: | ---: | --- |
| One row, 800 steps | `0.000592` | `7.64e-6` | `0.003391` | `0.0371 dB` | PASS |
| Eight-row block, 1,200 steps | `0.002245` | `5.27e-5` | `0.007785` | `0.0456 dB` | FAIL energy only |
| Full context, 1,600 steps | `0.001741` | `4.41e-6` | `0.005062` | `0.0225 dB` | FAIL energy only |

The frozen log-energy limit is `0.005`. Every task is strictly better than both
zero and global-mean controls on all five cooker endpoints; every trained
aggregate is within `0.15 dB` of its rank oracle; normalized coefficient RMSE,
raw NMSE, objective ratio and clipping fraction all pass. No cook failure or
non-finite output occurs.

## Conclusion and next revision

V1 disproves neither the rank representation nor the energy-preserving
objective. The final fixed AdamW step leaves a small deterministic oscillation:
the eight-row miss is material and the full-context miss is marginal, while
coefficient and emitted-PCM evidence already sit close to the oracle.

Preserve V1 and do not relax its threshold or add steps at the same learning
rate. The smallest falsifiable successor changes one hypothesis only:

1. retain the same basis, targets, objective, tasks, thresholds, initialization
   and cooker;
2. replace the fixed learning rate with one preregistered deterministic decay
   schedule ending near zero;
3. repeat twice without query access;
4. authorize N0.3E only if all original gates pass unchanged.

This result creates no learned listener-field quality, admission, public schema
or runtime authority.
