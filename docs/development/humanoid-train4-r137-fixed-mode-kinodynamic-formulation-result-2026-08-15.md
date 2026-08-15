# TRAIN-4 R137 fixed-mode kinodynamic formulation result — 2026-08-15

| Field | Value |
| --- | --- |
| Scope | Sole clean report-only fixed-mode controller-reachable formulation |
| Status | `R137_COMPLETE_R138_CONFORMANCE_ONLY` |
| Gate decision | `PERMIT_SEPARATE_REPORT_ONLY_R138_KINODYNAMIC_FORMULATION_CONFORMANCE_ONLY` |
| Claim ceiling | Symbolic graph and exact inventory only; no real assembly, solve, candidate, PhysX or training |

## Immutable result

The sole R137 process ran from clean commit `c647eba`, passed all six frozen
validations and completed without importing a numeric/solver implementation or
evaluating the real trajectory. Canonical/file/profile SHA-256 is
`f0db8b516897671461161cac299a9e81e9405a1a3ae38778fa6d3659c898381f` /
`a1da712723a3fde29dbbd20496da6cd5937dc7a575b3d70f390cd5fe4b93c733` /
`8468d67c02671de27e57cc6a550a01d16e2cac36ac4496124e875d96026176c9`.
The canonical hash independently reproduces exactly.

R137 hash-closes the tracked post-R136 decision, R136 valid-infeasible report
and R131 contact-edge formulation. It independently reproduces R131's motor
mode-sequence SHA-256
`ab30a912550e06e1219b1fab87e0d93044eba6b0cdf8e7377f7b27a0e1915ca4`
from all `3200` R136 collocation metadata rows.

## Frozen hybrid inventory

The exact fixed-mode transcription has `800` motor intervals, `3200` physics
intervals and `3201` state nodes. Its `799` motor boundaries split into `770`
unchanged and `29` changed boundaries:

- `11` activation boundaries introduce `18` source point activations and 54
  rigid impulse scalars;
- `18` deactivation boundaries remove `18` points with exact zero release
  impulse;
- two points are initially active, two are terminally active and the complete
  schedule owns 20 fixed source contact-point anchors;
- `4956` active point-force rows retain the unchanged individual unilateral
  circular Coulomb cones.

The primary symbolic inventory contains `311780` scalars: `92829`
configuration-local, `92829` generalized velocity, `92800` generalized
acceleration, `18400` integer commanded-target, `14868` active contact-force
and `54` activation-impulse scalars. Applied target and all `73600` effort
values are exact derived controller outputs. Free effort and contact-anchor
decision counts are both zero.

## Preserved semantics

Contact mode selection, point deletion, sliding, compliance, penetration,
material/friction changes and unscheduled impulses are structurally absent.
Every regular step uses `1/240 s` manifold integration and rigid dynamics; only
a newly activated point may produce a post-integration rigid momentum jump.
Release is continuous with exact zero impulse.

The motor graph retains ties-to-even observation quantization, one 60 Hz target
held for four ordered 240 Hz steps, persistent effort-rate state and per-motor-
tick positive-work reset. A smooth approximation may generate a future
proposal but has zero acceptance authority; byte-exact fixed-PD replay remains
mandatory.

## Exact stop boundary

All real-work counters are zero: state reconstruction, controller derivation or
evaluation, dynamics/integration/impulse residual evaluation, kinodynamic
assembly/solve, factorization, optimization, cache/candidate build, PhysX and
training. R137 neither proves that the feasible set is nonempty nor selects an
objective or numerical method.

The exact transition is `R137_COMPLETE_R138_CONFORMANCE_ONLY`. It authorizes
one separate report-only R138 implementation-conformance increment with zero
real reconstruction, controller derivation, assembly or solve. R139 solve
formulation, a real kinodynamic solve and every downstream candidate/native/
corpus/training action remain unauthorized.
