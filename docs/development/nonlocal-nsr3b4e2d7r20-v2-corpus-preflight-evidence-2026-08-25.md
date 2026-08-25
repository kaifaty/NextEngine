# NSR3-B4E2D7R20 v2 corpus/operator preflight evidence

Date: `2026-08-25`

Status: `PASS / GENERALIZATION_OPERATOR_PREFLIGHT_CANDIDATE / NO SOLVER RUN`.

Implementation commit: `6cbf10ad`.

## Reproducibility

Manifest v2:

```text
schema    nextengine.nonlocal.nsr3b4e2d7r20_corpus_manifest.v2
semantic  420031730236445859426960bc5540ae8b8f9522d4b9263fe5e86cf5f13f85b4
stdout    cf9af006da05434233436458fca7eebbd6dc67831d41d64f5752a143d57543c7
```

Operator preflight:

```text
schema    nextengine.nonlocal.nsr3b4e2d7r20_operator_preflight.v1
semantic  16a24ef3040411ef0bd29294f0887b29a1cce7690c7d8a8bee511ea58880b1f6
stdout    a7afb864fc6d437af0dba59e76b50a819fc798adf7af24589a3dffc2340bf613
```

Each path is byte-identical across two executions. Candidate iterations,
oracle cycles and timing are all exactly zero.

## Corpus roles and excitation

| role | state | rows | projected-positive rows | projected maximum |
|---|---|---:|---:|---:|
| transfer | face | 75 | 0 | 0 |
| transfer | corner | 45 | 0 | 0 |
| transfer | supported column | 48 | 16 | `1.3453e-3` |
| transfer | released/pre-impact | 27 | 0 | 0 |
| negative | v1 oblique edge | 24 | 0 | 0 |
| negative | v1 opposed corner | 24 | 0 | 0 |
| blind | filled upper-x/lower-y edge | 60 | 38 | `5.1099e-3` |
| blind | filled opposed corner | 64 | 42 | `6.0713e-3` |

The v2 admission gate passes exactly: one excited transfer case, both blind
holdouts excited and both negative controls quiet. The holdout source roots
are:

```text
edge    7da5dc21c97eff9fa8e4b96cfeb6b37043a39c7e4c0a2fc8433c327464da0092
corner  796e96d6b5e909b8338e5216f84e469cccd39eaf64f60d34afef742677b23f11
```

All eight R64 operators pass slot/entry/incidence ownership. The two blind
operators contain `5,626/2,138` and `5,984/2,392` slots/entries. Six selected
row checks per case under factors `2^-8` and `2^8` have zero primal, projected-
dual and row-scale gaps.

## Claim boundary

This closes only source excitation, operator ownership, row scaling and
lifecycle. It does not show feasibility of the full intersection, oracle
convergence, candidate convergence, a work advantage, nonlinear transfer,
runtime speed or production readiness.
