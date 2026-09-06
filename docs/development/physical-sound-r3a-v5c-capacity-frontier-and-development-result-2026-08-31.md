# Physical sound R3A V5-C capacity frontier and development result

| Field | Value |
| --- | --- |
| Date | 2026-08-31 |
| Result | `REJECT_NEURAL_REPRESENTATION` |
| Frozen evaluator commit | `58171558bcb74d015d0bef0cf865e1483bba1536` |
| Evaluator implementation SHA-256 | `64bf1604718ebb76c8453cce8d92c0617a35353cbf267c452501a309e7db26f0` |
| Capacity-frontier report SHA-256 | `2db6e7c62f8d20bb3d0350b3f5ad677b3b1408e77877a683cc14677ea1caa911` |
| Repeated development-report SHA-256 | `74a6f4eab92f360e483d10035c092510f16fa5ed3c7cd201dc2d112fe7a8bbb4` |
| Consequence | Stop V5, keep authored clips, do not open V5 holdout or R3B |

## Question and frozen decision rule

The experiment asked whether one of the preregistered `6/12/24 kbps`
task-specific neural impact representations could preserve four already-opened
REALIMPACT development contacts well enough to replace the nearest fit-contact
waveform under the unchanged V4 five-endpoint gate.

Before any development waveform was read, the implementation froze:

- the three paired step-4000 checkpoints and their exact run/report hashes;
- peak alignment and the five unchanged V4 endpoint implementations;
- nearest fit contact by Euclidean mesh position as the baseline;
- one target-RMS-matched `float32` output-gain sidecar in each encoded record;
- selection of the smallest capacity passing every endpoint and every object;
- a `64 KiB` per-contact budget, no runtime inference and authored-clip
  fallback;
- `REJECT_NEURAL_REPRESENTATION` with no holdout authority if all capacities
  fail.

A fit-only 144,000-sample control first exercised the worst-case 24 kbps path.
It returned exactly 144,000 finite samples, used about `256 MB` peak allocated
CUDA memory and produced a `9,004`-byte record. This control did not read a
development waveform.

## Paired capacity frontier

All capacity runs used the same stateless paired item/crop stream and the same
frozen continuous encoder trajectory. Development and sealed reads were zero.

| Capacity | Full internal-validation total | Log spectrum | Minimum code use | Checkpoint SHA-256 |
| --- | ---: | ---: | ---: | --- |
| 24 kbps | `400.0453698` | `24.3436535` | `84` | `32b862094d8f0cb93bbf9fccfd474626402238336612794699df7dc411fe1a73` |
| 12 kbps | `406.0681799` | `24.5426359` | `84` | `7ac1b9d8ce2737d94adeeada7c78705489c8f9a7676f4246d25a71a2eda37735` |
| 6 kbps | `411.6117643` | `24.7456575` | `73` | `e1cbeb167319b5a60d563a3be491f07d48429f2ae07890a51ed48b61fe77e310` |

The frontier selected 24 kbps by the preregistered internal metric, but
authorized the single development comparison for all three capacities so the
smallest genuinely passing record could win.

## Development result

The evaluator decoded the already-opened development contact for Blue Bowl,
Large Swan, Plastic Bin and Purple Scoop. It decoded `879,465` source samples
in total and no row `2407`, method holdout or admission-shadow sample. Two
identical invocations produced the same canonical report SHA-256.

Every candidate passed its record budget: `2,254`, `4,504` and `9,004` bytes
for `6`, `12` and `24 kbps`. Acoustic quality failed decisively:

| Endpoint | Frozen limit | Result across 12 capacity/object comparisons |
| --- | ---: | --- |
| Absolute RMS level error | `0.5 dB` | `12/12` pass after the frozen gain sidecar |
| Gain-matched multiresolution log-spectrum RMSE | `4 dB` | `12/12` fail; `13.9869–24.1748 dB` |
| Normalized envelope RMSE | `0.20` | `12/12` pass; `0.0419–0.0788` |
| Modal-frequency median error | `100 cents` | `12/12` fail; `1317.23–5733.73 cents` |
| Decay T60 relative error | `0.35` | `11/12` fail; only 6 kbps Plastic Bin passes |

No capacity/object candidate strictly beat nearest fit on more than two of the
five endpoints; the gate requires at least four plus every absolute threshold
and the normalized-error improvement rule. Consequently all twelve candidate
gates and all three capacity gates failed.

## Conclusion and decision

The paired frontier proves that increasing this latent from 6 to 24 kbps
slightly improves the internal training objective, but does not recover the
modal or spectral identity of unseen real impacts. The byte budget is not the
limiting resource. Internal anti-collapse and codebook-use success were
necessary substrate checks, not evidence of transferable acoustic quality.

R3A V5 therefore closes as `REJECT_NEURAL_REPRESENTATION`. Do not select a
capacity, run a longer checkpoint, change thresholds or losses using these
opened contacts, spend a new source-disjoint representation holdout, start the
R3B contact-to-latent field, or claim an offline baked atlas. SPEC-45 remains
`Proposed` and the authored clip path remains authoritative.

Reconsider only with a materially different preregistered representation
hypothesis and a new source-disjoint development protocol that does not reuse
these four opened contacts for model selection. A new hypothesis must explain
how it directly preserves stable resonant modes and decay rather than merely
raising neural bitrate or continuing the rejected checkpoints.
