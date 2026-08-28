# Physical sound PS-2 — analytical boundary-solver control

Date: `2026-08-28`

Status:
`CLASSICAL_BOUNDARY_SOLVER_ANALYTICAL_CONTROL_REJECTED / FOUR_OF_FIVE_NUMERIC_GATES_PASS / REFINEMENT_GATE_FAIL / BYTE_IDENTICAL_REPEAT / SYNTHETIC_FIXTURE_ONLY / NO_FRESH_REALIMPACT_DATA_OPENED`

## Question and frozen protocol

The preceding REALIMPACT diagnostic showed that listener-coordinate
multipoles cannot replace the missing surface mode shape. This experiment asks
a narrower falsifiable question: can a deterministic classical surface
boundary solver recover the known complex radiation field of a pulsating
sphere before it is trusted as an offline target for object modes?

External manifest
`physical-sound/ps2-surface-bem-v1/feasibility-manifest.json`, SHA-256
`9a26ca137681b319b2ffb490dd1e4b897aeaf4de19349c7cbb669c97669c8095`,
was frozen before either run. It declares:

- a `0.1 m` sphere with unit outward pressure derivative;
- `ka = 0.25, 0.75, 1.5`;
- listener radii `1.5a`, `3a`, `10a` and ten directions;
- outward icospheres with `80` and `320` constant panels;
- an indirect single-layer Helmholtz boundary equation,
  symmetric three-point panel quadrature and deterministic complex pivoted
  elimination;
- absolute complex, magnitude, phase and symmetry gates plus a required
  fine/coarse median-error ratio of at most `0.8`.

The fixture is generated analytically. The protocol forbids fresh REALIMPACT
payload access, network training and runtime or quality-admission credit.

## Implementation

`physical-sound-registry bem-feasibility` validates the exact external
manifest hash, generates both meshes, solves the complex boundary density for
each `ka`, evaluates all 180 mesh/field conditions and atomically publishes a
canonical JSON report. Unit controls cover panel counts/orientation, the
complex linear solver and the analytical boundary derivative.

The implementation is repository-owned Rust and does not execute upstream
code. Its source lineage freezes NeuralSound revision
`b18e81b1e3dba7e963b09a7d9a45b584707d805f`, whose classical pipeline maps
surface-normal vibration through BEM to near/far acoustic transfer, and the
Precomputed Acoustic Transfer publication. These sources justify the family
of offline reference computation, not this implementation's correctness.

## Result

Runs `run-a` and `run-b` publish byte-identical manifests and reports:

| Artifact | SHA-256 |
| --- | --- |
| manifest | `9a26ca137681b319b2ffb490dd1e4b897aeaf4de19349c7cbb669c97669c8095` |
| report A/B | `6f74a30988ff349c270d6cb0ba37cfbf905e1dc1b078868660d01fb1c622a689` |

| Metric | 80 panels | 320 panels | Frozen fine gate |
| --- | ---: | ---: | ---: |
| Median relative complex error | `0.0071243` | `0.0203677` | — |
| Maximum relative complex error | `0.0170479` | `0.0212507` | `<= 0.15`, pass |
| Maximum magnitude error | `0.061141 dB` | `0.182647 dB` | `<= 0.75 dB`, pass |
| Maximum phase error | `0.885858°` | `0.299201°` | `<= 10°`, pass |
| Maximum direction span | `0.005062 dB` | `0.000196 dB` | `<= 0.25 dB`, pass |
| Fine/coarse median error ratio | — | `2.858899` | `<= 0.8`, **fail** |

Decision:
`ClassicalBoundarySolverAnalyticalControlRejected`.

The fine solution is accurate in absolute amplitude, phase and symmetry, but
the preregistered convergence claim is false. Coarse phase error is larger
while its total complex error is smaller; fine low-`ka` conditions show a
nearly direction-independent positive magnitude bias. This is evidence of
non-monotonic error cancellation, not permission to delete the gate after
inspection.

## Conclusion and next discriminator

This package establishes a reproducible boundary-solver harness and a useful
absolute accuracy point. It does **not** establish a trustworthy classical
BEM/FFAT oracle, surface-mode transfer, non-spherical geometry, material
identity, perceptual quality or admission.

Keep both ceramic REALIMPACT holdouts sealed. The smallest next experiment is
still synthetic: freeze a second protocol that distinguishes low-order panel
geometry/quadrature error from a boundary-equation defect, using an additional
resolution or an independent classical BEM result and a successful analytical
control. Do not fit another empirical listener-coordinate basis.

Primary references:

- [NeuralSound classical acoustic-transfer source at the frozen revision](https://github.com/hellojxt/NeuralSound/blob/b18e81b1e3dba7e963b09a7d9a45b584707d805f/dataset_scripts/acousticTransfer.py)
- [Precomputed Acoustic Transfer](https://graphics.stanford.edu/~djames/publication/precomputed-acoustic-transfer-output-sensitive-accurate-sound-generation-for-geometrically-complex-vibration-sources/)
