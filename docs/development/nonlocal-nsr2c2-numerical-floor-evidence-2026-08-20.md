# Nonlocal NSR2-C2 numerical energy-floor evidence -- 2026-08-20

Status: `PASS / BOUNDED_CPU_SOLVER_CANDIDATE / NSR3_AUTHORIZED`

## Outcome

The guarded `numerical-energy-floor-stop-v1` removes noise-driven trust trials
without changing the objective, Hessian, normal convergence criteria or trust
policy. All exact correspondence, operation, neighborhood-capacity and
conservation gates pass.

| Fixture | Stop | Outer / rejected | Eval / HVP | Scaled residual | Scaled step at floor |
|---|---|---:|---:|---:|---:|
| 2x2x2 correspondence | energy floor | `7 / 0` | `7 / 18` | `6.6704629631456145e-8` | `5.997047806784084e-9` |
| 3x3x3 correspondence | energy floor | `7 / 0` | `7 / 16` | `6.355778919182387e-8` | `1.7178743073556763e-8` |
| 4x4x4 correspondence | ordinary scaled displacement | `8 / 0` | `9 / 24` | `2.2772710983764147e-9` | n/a |
| 5x5x5 scale | energy floor | `10 / 0` | `10 / 35` | `4.958046222928206e-8` | `3.2703445363534714e-8` |
| 8x8x8 scale | energy floor | `12 / 0` | `12 / 46` | `4.201251670579929e-8` | `2.1970490052024613e-8` |
| 10x10x10 scale | ordinary scaled displacement | `13 / 0` | `14 / 45` | `5.3986699290156735e-9` | n/a |

For the formerly failing 512-particle case, work falls from `26` outer trials,
`13` rejects and `153` HVPs to `12`, `0` and `46`. The state is not mutated by
the floor stop. Its residual corresponds to approximately `2.1 nm` at the
`0.05 m` spacing, and its proposed step is approximately `1.1 nm`. Both are
well below the existing `1 um` report quantum.

The modified all-pairs and canonical-neighborhood solvers remain bit-exact on
8/27/64 particles. No floor-eligible trial is accepted. Momentum residuals are
at most `1.052e-15` across the corpus.

## Exact artifacts

| Artifact | SHA-256 |
|---|---|
| raw report, run 1 | `719be1b185584f8a5e954b836450b19671af0f11c9c6c626002faa72ecab3301` |
| raw report, run 2 | `719be1b185584f8a5e954b836450b19671af0f11c9c6c626002faa72ecab3301` |
| semantic result | `ab37ec1790a5717921e8fd3f268480c43ac05b873d094dddc31f03b43c74e51f` |

All NSR0--NSR2-C1 raw hashes remain byte-identical after the remediation.

## Decision

Select the bounded report-only CPU solver candidate:

```text
verified nonlocal objective
  + exact analytic full HVP
  + unpreconditioned Steihaug--Toint trust region
  + canonical cell neighborhood
  + scale-aware ordinary stop
  + guarded numerical energy-floor stop
```

Authorize NSR3 baseline performance measurement and physical-corpus contract
work. This is not production, GPU, multi-step stability or gameplay authority.

