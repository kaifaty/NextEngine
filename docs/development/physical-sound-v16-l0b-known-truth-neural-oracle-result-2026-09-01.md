# Physical sound V16 L0b — known-truth neural oracle result

| Field | Value |
| --- | --- |
| Date | `2026-09-01` |
| Status | `COMPLETE / REPEAT_EXACT / L0_CAPABILITY_REJECT` |
| Protocol | [V16 L0a](physical-sound-v16-l0a-known-truth-neural-oracle-protocol-2026-09-01.md), SHA-256 `d4f18e7a…21bbecc` |
| Implementation | Git `e1f7d48a`; three deterministic Python modules and 14 focused tests |
| Allowed claim | The frozen generic structured MLP learns useful geometry-conditioned gain structure but does not recover the complete unseen-object modal field or coverage OOD gate |
| Product effect | None; no real-data credit, public schema, cooked atlas, demo or runtime inference |

## Execution

Two complete executions wrote only to fresh external directories:

- `/home/kaifaty/.codex/experiments/nextengine/physical-sound/v16-l0b-known-truth-run-a`;
- `/home/kaifaty/.codex/experiments/nextengine/physical-sound/v16-l0b-known-truth-run-b`.

Each contains `26` files and `2,944,320` file bytes: six deterministic model
parameter streams, eighteen prediction arrays, one manifest and one report.
`diff -qr` found no difference. The ordered complete-file-map digest is
`95ac0bf868c965e335b0fc9148878391d14c1e4fff223ea4861af7091f8301b3`
for both executions. Manifest SHA-256 is `ca3d1e92…68ad613`; report SHA-256 is
`0993c0d5…93b2f7b` in both.

The pinned CPU environment was CPython `3.12.13`, NumPy `2.5.2`, SciPy
`1.18.0`, PyTorch `2.13.0+cu130`, float64 deterministic algorithms, one
intra-op thread and one inter-op thread. All real/source/protected/network
access counters are exactly zero.

An earlier attempt published no directory: a compatible constant control
produced an exact-zero prediction for a non-nodal query and the evaluator
aborted instead of assigning the declared worst spectrum score. Commit
`e1f7d48a` made that representation fail-closed at `80 dB`; it changed no
model, seed, update budget or quality threshold. No aggregate test metric was
available to select the correction.

## Result

Only `7/21` immutable gates passed. The final decision is
`L0_CAPABILITY_REJECT`.

| Endpoint | Candidate | Required / comparison | Result |
| --- | ---: | ---: | --- |
| Frequency median / p95 | `164.11 / 431.94 cents` | `<=30 / <=80` | fail |
| Maximum modal peak error | `433.54 cents` | `<=80` | fail |
| Damping median / p95 | `0.1807 / 0.3098` | `<=0.12 / <=0.30` | fail |
| Gain NRMSE mean / maximum | `0.3489 / 0.4518` | `<=0.25 / <=0.40` | fail |
| Gain vs best classical | `0.3489 / 0.6432 = 0.542x` | `<=0.98x` | pass |
| Gain vs geometry-agnostic | `0.3489 / 0.8120 = 0.430x` | `<=0.95x` | pass |
| Spectrum median | `15.891 dB` | `<=2.5 dB` | fail |
| Spectrum vs best classical | `1.036x` | `<=0.98x` | fail |
| Spectrum vs geometry-agnostic | `1.0003x` | `<=0.95x` | fail |
| Waveform NRMSE mean | `1.3212` | `<=0.20` | fail |
| Envelope NRMSE p95 | `1.1411` | `<=0.15` | fail |
| Surface continuity p99 | `0.8592` | `<=0.20` | fail |
| Hard / ordinary mutations | `100% / 100%` rejected | `100% / >=95%` | pass |
| Collapsed coverage | `52.08%` rejected | `>=95%` | fail |
| Valid test OOD | `0%` rejected | `<=10%` | pass |
| Isolation / finite stability | exact zero access / finite | required | pass |

Per-object behavior was not uniformly bad. The unseen Glass Cylinder recovered
global frequencies within `6.7 cents`, while several Plate/Bowl objects missed
by roughly `150–434 cents`. That clustering is evidence against random training
failure and for an underconstrained cross-object global mapping. Conversely,
the candidate's gain error is `45.8%` lower than the best classical error and
`57.0%` lower than the equal-budget geometry-agnostic ablation. Geometry therefore
contains learnable contact information, but the current pooled-context decoder
does not preserve it with the required continuity or waveform accuracy.

## Decision

The V16 primary model family is closed before real-data training. The opened
test objects, their metrics and these six checkpoints cannot select a wider
network, seed, training duration, threshold or contact split. R1 remains
unauthorized for this family, and all real/protected roles remain sealed.

The result separates three issues that a successor must test independently:

1. object-global poles/damping should use an explicit dimensionless physical
   factorization or residual-on-analytic-baseline, not ask nine labels to teach
   a generic MLP the complete multiplicative law;
2. surface gains need a topology-aware, query-to-context mechanism with an
   explicit continuity objective rather than one mean/max pooled context;
3. coverage OOD needs geodesic/topological coverage features that remain
   sensitive on periodic surfaces, independent of ensemble disagreement.

The smallest next action is a bounded primary-source research cycle and a new
unopened known-truth protocol that discriminates these hypotheses separately.
Source-adapter work may continue in parallel, but no disclosed real tournament
or protected validator threshold opens from this rejection.
