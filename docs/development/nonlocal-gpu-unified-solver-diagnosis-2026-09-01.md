# NCGP15 unified-solver work-ceiling diagnosis — 2026-09-01

## Question

Why does the admitted NCGP15 Revision-5 pressure mask stop at the frozen
`64`-outer-update ceiling, and what is the smallest solver change worth
testing before CUDA work resumes?

## Exact local observation

- Source commit: `ff0ea1fe0711016874fe4b62a2501d33223480d7`.
- Release binary SHA-256:
  `6bf468537bcd91dc2f8bbf650452785d1d3c98fa351ba6aec780f8ae7ad66e3d`.
- Raw stdout SHA-256:
  `151394d8046b34b1bb8b435c290e657af8bce2be09165e52cf623c665864e440`.
- Result root:
  `ff9070f7bdd065fa9106633a7e104922f041f98120f2023228f75dfdbd30f926`.
- Phase A and all nine mutation controls pass. Candidate, independent
  all-pairs oracle, input permutation, work receipts and roots agree exactly.
- The first physical mask `P` stops as
  `SOLVER_WORK_CEILING_INCONCLUSIVE` after `64` multiplier updates and `1548`
  accepted inner iterations. Later masks and the trajectory are typed skips.
- The final state already has maximum/RMS positive density strain
  `3.8860627938404769e-7 / 9.8278377932749008e-8` and projected position KKT
  `1.8274000293777388e-10 m` maximum. The remaining failures are normalized
  complementarity `2.6227877691848134e-7` and multiplier fixed point
  `4.0826452996374767e-4`, both against `1e-8`.
- The 96 active multipliers lie in `[0, 1.0074041939455616]`. Inner work per
  outer update falls from 93 accepted iterations initially to roughly
  13--20 near the cap, so this is not a single late line-search accident.

## Primary-source comparison

The published method does not use the NCGP15 nested projected-gradient PHR
solver. The paper splits each pair coefficient into a non-negative implicit
part and a non-positive explicit part and updates each particle position by a
local `3x3` inverse (Equation 26). Algorithm 1 runs a fixed number of these
SISSM substitutions. The authors explicitly describe this as GPU-parallel;
their current PeriDyno implementation recomputes density, assembles local
source/matrix terms and applies the local inverse each iteration.

- [A Nonlocal Unified Variational Framework for Free Surface Flows](https://doi.org/10.1145/3799902.3811196),
  SIGGRAPH 2026, Sections 5--5.2 and Algorithm 1.
- [Official PeriDyno source](https://github.com/peridyno/peridyno/tree/5d5a081b1c30d09499c100ffaaf6745be632f546/src/Dynamics/Cuda/ParticleSystem/SIUnifiedFluid),
  commit `5d5a081b1c30d09499c100ffaaf6745be632f546`.
- Downloaded paper SHA-256:
  `610047ef32e895026c3661c57999f14ae550d8e2719ba2fcd95742371ad031c1`.
- Upstream header/source SHA-256:
  `42ff10957a5e497c67ab916bb5397fc6a937624b1ab6b529f90daa7c2d57878c` /
  `23b78d69690eaf765598d3ccfe923051e5d411d6e94feb1152a86513ba80c516`.

This matters because the paper's incompressibility term is a finite squared
density penalty, whereas NCGP15 deliberately replaced it with a unilateral
augmented-Lagrangian constraint. The paper and code therefore support the
coefficient split and local semi-implicit update, but not the claim that the
current PHR schedule is the intended optimizer.

## Bounded SISSM counterfactual

A temporary standalone long-double probe reconstructed the exact TIGHT-128
dynamic/ghost lattice, corrected kernel, gravity and analytic box. It applied
the paper's pressure coefficient split and componentwise box projection;
it did not modify repository sources or evidence.

| Stiffness | Observation |
| --- | --- |
| `kappa=1` (upstream default) | position updates converge by about 10 iterations, but maximum/RMS positive density strain settle near `1.3377e-3 / 5.2808e-4`, outside the frozen NCGP15 gates |
| `kappa=1226.25` (NCGP15 pressure scale) | the direct substitution is stiff: after 100/200 iterations maximum strain is about `3.283e-2 / 3.008e-2`; copying upstream verbatim is not an admissible repair |

The probe is a discriminator, not evidence: it uses a simplified owner/ghost
mapping and has no independent receipt apparatus. It is sufficient to reject
“copy upstream SISSM unchanged” and “raise PHR outer cap” as next actions.

## Competing hypotheses

| Hypothesis | Prediction | Update |
| --- | --- | --- |
| H16A: the PHR cap is just too small | more outer updates eventually close the dual residual without changing the method | plausible but not selected; increasing the observed cap is post-hoc and does not address the optimizer mismatch |
| H16B: upstream SISSM is the direct repair | the unchanged coefficient split closes the frozen density gates rapidly | falsified by the bounded TIGHT-128 probe |
| H16C: the final gates are unnecessarily strict | the state is physically adequate despite dual residuals | not admissible; complementarity and fixed-point gates were frozen before the result |
| H16D: exact pressure projection is the missing nonlinear block | the retained NCGP14 QP closes the same fixture while preserving pressure/contact invariants | selected for the next discriminator |

## Decision

Do not increase the PHR cap and do not weaken complementarity/fixed-point
gates. Replace the next experiment's pressure subproblem with the independently
reviewed NCGP14 linearized nonnegative-pressure QP. Couple viscosity and
surface through an outer semi-implicit/SQP iteration evaluated on the same
private position state, with one transactional commit only after all
correspondence and physical gates pass.

The first lane must be pressure-only and reproduce the NCGP14 TIGHT-128
pressure/contact result within newly frozen correspondence tolerances. Only
then run `PV`, `PS` and `PVS`, followed by the short trajectory. A QP or outer
iteration cap remains `SOLVER_WORK_CEILING_INCONCLUSIVE`, never physics.

## Smallest next action

Freeze an NCGP16 CPU-long-double discriminator for a pressure-QP-preconditioned
semi-implicit/SQP solve. Reuse the exact NCGP15 profile, TIGHT fixture, term
oracles, ordered masks and physical gates; add only the solver-specific work,
pressure-block correspondence and transactional receipts. CUDA, 4k, 16k,
50k and timing remain `NOT_RUN`.

## Pressure-QP/SQP feasibility probe

A second temporary long-double probe tested the selected construction before
contract freeze. For each outer iterate it evaluated the unchanged NCGP15
non-pressure gradient, formed

```text
z = P_box(y - dt^2/m * gradient_nonpressure(y))
```

and then applied the unchanged NCGP14 CAP16384 pressure/contact projection to
`z`. The next outer iterate was the projected state. Fixed-point gates were
the existing NCGP15 relative RMS/maximum inner gates. The probe source/binary
SHA-256 were
`7c427015043ddcf89ee3a0f594bcbe1e8d50998b0f9c2cc96d78722687d62b3f` /
`e6b94812ed4b2f65a1a468ac6b23a66f546946296f5ee4aa8baf152c0fbf1250`;
both temporary files were deleted after the run.

| Mask | Outer map evaluations | Pressure rounds per evaluation | Total QP sweeps | Final max/RMS positive strain |
| --- | ---: | ---: | ---: | ---: |
| `P` | 2 | `5, 5` | `67,744` | `6.973935643299492e-4 / 2.200404888408005e-4` |
| `PV` | 3 | `5, 5, 5` | `101,616` | `6.973933315244889e-4 / 2.200404369278360e-4` |
| `PS` | 3 | `5, 5, 5` | `101,550` | `6.973163729218396e-4 / 2.199743883787805e-4` |
| `PVS` | 3 | `5, 5, 5` | `101,550` | `6.973161412652373e-4 / 2.199743368597816e-4` |

The first pressure projection uses the exact retained trial-1 sweep total
`33,872`; the `P` map is byte-stationary on its second evaluation. The
`PV/PS/PVS` second-to-third RMS changes fall to
`3.10e-16 / 1.57e-15 / 2.68e-15 m`. This is not admitted correctness evidence:
the probe has no independent oracle, roots, work receipt or transaction
apparatus. It is a successful pre-freeze discriminator showing that the
selected pressure-block map closes all four one-step masks with margin and
without changing physical coefficients or gates.
