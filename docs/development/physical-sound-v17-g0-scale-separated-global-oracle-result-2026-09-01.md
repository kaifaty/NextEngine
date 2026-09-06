# Physical sound V17 G0 — scale-separated global oracle result

| Field | Value |
| --- | --- |
| Date | `2026-09-01` |
| Status | `COMPLETE / REPEAT_EXACT / G0_CAPABILITY_REJECT` |
| Protocol | [V17 P0a](physical-sound-v17-p0a-disjoint-factorized-truth-protocol-2026-09-01.md) |
| Implementation | Git `9d5ebe10`; three deterministic Python modules and 10 focused tests |
| Allowed claim | Deterministic dimensional scale separation is strongly supported; this neural head does not earn the frozen complexity margin over compatible low-order regression |
| Product effect | None; no field/integration execution, real-data credit, public schema, cooked atlas, demo or runtime inference |

## Execution

Two complete executions wrote only to fresh external directories:

- `/home/kaifaty/.codex/experiments/nextengine/physical-sound/v17-g0-scale-separated-run-a`;
- `/home/kaifaty/.codex/experiments/nextengine/physical-sound/v17-g0-scale-separated-run-b`.

Each directory contains `11` files and `652,872` file bytes: six model
parameter streams, two prediction arrays, the complete synthetic corpus, one
manifest and one report. Every corresponding file is byte-identical. The
ordered complete-file-map digest is
`217cd08b9022b3c7cf331f6af845aa4737a23547934b1ed959738d0b98f64c23`;
manifest SHA-256 is
`598ab1df3ee821e0147d87b0976fbe130941e903056dcee204f10d7c4c7b179e`
and report SHA-256 is
`ebbbb9f411615e112d243f0d5f136d2b1c67ddf6969058a7509ca1783ddfc532`.

The pinned environment was CPython `3.12.13`, NumPy `2.5.2`, SciPy `1.18.0`,
PyTorch `2.13.0+cu130`, float64 deterministic CPU algorithms and one
intra/inter-op thread. Real, source, protected, network and V16-artifact access
counters are exactly zero.

## Result

The frozen decision is `G0_CAPABILITY_REJECT`: `10/12` gates pass. Absolute
quality, raw-dimensional comparison, mutations, OOD preservation, stability,
isolation and repeatability all pass. Only the two required `<=0.90x`
complexity-margin gates against the best compatible non-neural control fail.

| Endpoint | Scale-separated neural | Comparator / gate | Result |
| --- | ---: | ---: | --- |
| Frequency median / p95 | `4.238 / 19.891 cents` | `<=20 / <=60` | pass |
| Damping median / p95 | `0.001370 / 0.004173` | `<=0.08 / <=0.20` | pass |
| Scale-transfer frequency mean | `4.715 cents` | raw MLP `33.015`; ratio `0.1428x` | pass |
| Scale-transfer damping mean | `0.001449` | raw MLP `0.014074`; ratio `0.1030x` | pass |
| Paired scale-transfer cells | `22/24` | `>=20/24` | pass |
| Overall frequency mean | `5.317 cents` | ridge `5.598`; required `<=5.039` (`0.9497x`, not `<=0.90x`) | fail |
| Overall damping mean | `0.001546` | ridge `0.001674`; required `<=0.001506` (`0.9239x`, not `<=0.90x`) | fail |
| Material/support/scale mutations | `72/72` rejected | `72/72` | pass |
| Valid static OOD | `0/48` rejected | `<=10%` | pass |
| Exact repeat / isolation | byte-identical / zero access | required | pass |

This is not evidence that scale separation failed. Compared with the
equal-budget raw-dimensional MLP, it reduces scale-transfer frequency error by
`85.7%` and damping error by `89.7%`. It also stays well inside every absolute
mode/damping limit. The negative result is narrower: on this smooth
dimensionless law, a degree-two ridge already reaches `5.598 cents` and
`0.167%` mean damping error, so the neural candidate's additional machinery
does not buy the preregistered ten-percent margin.

## Decision

Close the V17 G0 neural family without a wider network, extra updates, seed
selection or threshold change. Per the P0a stop rule, do not execute V17 O0,
F0 or I0 under the failed dependency chain.

Preserve two positive pieces of evidence:

1. dimensional scale separation is the correct global representation for the
   frozen synthetic domain;
2. the preregistered degree-two ridge is a viable, simpler global scaffold.

V18 may test that unchanged deterministic scaffold on fresh global rows, then
spend neural capacity only where relational surface structure is not captured
by a low-order formula. This is a new dependency graph, not a retry on the
opened G0 test. Authored clips remain the complete product fallback.
