# NSR3-B3R owned-residual boundary composition evidence -- 2026-08-21

Status: `PASS / STATIC_BOUNDARY_SMOKE_CANDIDATE / B4_CONTRACT_DESIGN_AUTHORIZED`

The frozen
[B3R retry](../plans/nonlocal-nonlinear-solver-research/03b3r-owned-boundary-composition-contract.md)
passes twice byte-identically. It reuses the complete original B3 composition,
adaptive controller, fixed references, contact order and strict per-substep
ledger; only the selected D5 numerical state/globalization is different.

## Adaptive composition

| Fixture | accepted / executed / discarded substeps | spectral / nonlinear HVP | floor accepts | max ledger | contacts |
|---|---:|---:|---:|---:|---:|
| face slab | `72 / 108 / 36` | `96 / 597` | 100 | `5.35e-10` | 1526 |
| corner column | `10 / 16 / 6` | `0 / 90` | 6 | `5.93e-12` | 105 |

Every frame accepts the fine member of its first passing pair. No candidate
rejects, no negative-curvature exit and no penetration occurs. Floor merit
uses at most two accepts per smooth solve on the face and one on the corner.
All executed comparator paths retain exact floor conditions; maximum active
stationarity ratios are `0.9634/0.6729` and inactive defect/bound ratios are
`0.00488/0`.

The strict original B3 ledger is not replaced by D5's diagnostic certificate:
its normalized `1e-9` gate passes directly on every executed substep.

## Reference and final accuracy

Fixed remediated references pass their `96/192/384` ladder:

| Fixture | position ratio | velocity ratio |
|---|---:|---:|
| face | `1.9517` | `1.8969` |
| corner | `1.9999` | `1.9134` |

Adaptive final gates pass with margin:

| Fixture | position error | velocity error | kinetic error | contact-time error |
|---|---:|---:|---:|---:|
| face | `0.00536 dx` | `0.000419 c` | `0.0363` | `1.46e-4 s` |
| corner | `0.00395 dx` | `0.000102 c` | `0.00207` | `1.58e-4 s` |

Terminal contacted particle/feature sets are exact. First contact occurs at
`0.00935 s` for the face and `0.01293 s` for the corner.

## Architectural result

B3's split ordering is now validated for these bounded static fixtures:

```text
owned smooth variational solve
  -> virtual static-support reaction
  -> post-solve swept hard contact
  -> contact reaction
  -> velocity from owned displacement
  -> cache rebuild
```

The original B3 failure remains important evidence: trajectory convergence
alone was insufficient until displacement, inertia gradient and reaction
globalization shared one numerical state. B3R validates the repaired
composition; it does not validate broader free-surface behavior.

## Repeatability

```text
B3R raw SHA-256 (two identical runs):
89ede039a67b8e0f7f4a27d5ecd412f4691245ee8acaec612dd0894b2153edad

B3R JSON-without-newline SHA-256:
792dceb540d7998d62f9c290b4e4215832ec9e1a6ba6ecf9e16b3200f1e30da9

B3R semantic SHA-256:
2310c531dc7ab8883e83ab015742ce439d7923c21687eef9fac8f6b0014a0d10

D5 raw preserved:
38845883a1f689aa1f126f58633d94605d8662e914c2165211c3b52577ec4261

original B3 raw preserved FAIL:
c64ad0b8d73f7ada62364a3daa2d3bed66a8bfc5a1fe6c0148b7c1f8f2e5fb2f

B2 raw preserved:
d6ba5f8e802966c25283d0c8384ed01beec20b347acb343cf5c7c2bf360d69d9
```

PASS authorizes B4 physical-corpus contract design only. No hydrostatic or
dam-break execution, moving solids, CUDA, performance, runtime or production
authority is granted.
