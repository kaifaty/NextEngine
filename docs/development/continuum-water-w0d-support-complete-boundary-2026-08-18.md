# Continuum water W0D support-complete boundary evidence — 2026-08-18

Status: `REPORT_ONLY / CANDIDATE_REJECTED / PROFILE_RECLOSURE_REQUIRED`.

## Scope and result

This report records the bounded W0D discriminator
`support-complete-lattice-complement-v1`. It tests whether the dynamic failure
of the one-layer ghost shell was caused by omitting an exterior lattice layer
that enters kernel support after particles move toward a wall. It does not
change the DFSPH equations, authored fluid lattice, 20-iteration ceiling,
`100,000 ppb` density threshold, canonical integer state or failure policy.

The hypothesis is falsified. The two-layer boundary restores the same exact
initial discrete partition as the one-layer shell, but does not improve the
accepted trajectory: the first step passes and step 2 fails. Production and a
separately generated brute-force input/solver path agree exactly, and the
free-fall control remains byte-identical. No successor roots, corpus credit or
ProductCheck result are issued.

## Formula audit

The pressure operator is not a transcription accident. Equations 29–34 of
[Consistent SPH Rigid-Fluid Coupling](https://animation.rwth-aachen.de/media/papers/84/2023-VMV-SPH_ConsistentBoundaryHandling.pdf)
add boundary volume-gradient terms to the density constraint and central
constraint gradient, omit independent boundary-gradient squares from the
diagonal, apply only the fluid-row multiplier to a static boundary, and use
zero boundary acceleration in the matrix action. The serial oracle follows
those operations.

The same operations appear in the scalar path of pinned SPlisHSPlasH commit
[`eccce861`](https://github.com/InteractiveComputerGraphics/SPlisHSPlasH/tree/eccce86155776f6ac52d5080b1f720a52bf29450):
`computeDFSPHFactor`, `computeDensityAdv`, `computePressureAccel` and
`compute_aij_pj` match the local factor, predicted-density, acceleration and
matrix-action ordering. The upstream default maximum of 100 iterations does
not validate that setting here; the W0D `max100` trajectory also fails.

This audit narrows the problem from “wrong equations” to an invalidated
combination of boundary field, authored initial state and solver profile.

## Frozen two-layer discriminator

At the unchanged minimum centre clearance of `22,500 µm`, exterior lattice
centres in layers one and two are `47,500 µm` and `97,500 µm` from a fluid
centre. Both can enter the `100,000 µm` kernel support. Layer three is at least
`147,500 µm` away and cannot contribute to an admitted state. W0D therefore
uses exactly two layers, not an arbitrary thicker shell.

For the `1 m` hydro box the boundary has
`24³ − 20³ = 5,824` samples. Each has the unchanged `REST_VOLUME`, the origin
is `bounds.min + PARTICLE_RADIUS`, and generation is stable `x-y-z` order.
The free-fall box has `9,344` samples. Both remain below the unchanged
`16,384` boundary-sample capacity.

Implementation commit:
`89567e14a6205fbec4705c8bf5de40be674eb234`.

The clean exact-profile report uses schema
`nextengine.continuum-water.hydro-boundary-candidate.v1` and has SHA-256
`67412e759bb80551f030fa8ef53c06efd77411d216ca259b973ae26ee48bbd9c`.
Its tool tree is `CLEAN`; the bounded JSON remains outside Git.

## Independent equality and initial partition

The production generator accepts arbitrary exact box spans. The independent
path hard-codes the hydro lattice, two exterior layers, expected count and
brute-force reconstruction without calling the production generator. They
match exactly for all boundary position/volume records, selected contribution
rows and the complete 320-iteration initial diagnostic trace.

The boundary-input root is
`16b70d3db32f099630101021e542aaa6d7d95d9be6c931207f6521df8056251a`.

| Row | Boundary contribution | Reconstructed density | Partition error |
| --- | ---: | ---: | ---: |
| Corner | `393,411,630 ppb` | `999,972,466 ppb` | `-27,534 ppb` |
| Edge | `280,311,501 ppb` | `999,972,466 ppb` | `-27,534 ppb` |
| Face | `149,684,592 ppb` | `999,972,466 ppb` | `-27,534 ppb` |
| Interior | `0 ppb` | `999,972,466 ppb` | `-27,534 ppb` |

The second layer lies exactly at the zero-valued support boundary for the
authored first fluid row, so the initial values intentionally equal the
one-layer result. It becomes non-zero only after inward motion.

## Dynamic result

The normal first step converges at iteration 2 with `95,755 ppb`, producing
frame root
`10fa8338768a8a980ff274067b88516a00446b61dddfa79aa615be60a2428ddf`
and `171 µm` maximum penetration.

| Density ceiling | Accepted of 24 steps | Terminal result | Maximum accepted iterations | Maximum accepted penetration |
| ---: | ---: | --- | ---: | ---: |
| 20, unchanged | 1 | step 2: `192,430 ppb` | 2 | `171 µm` |
| 100, diagnostic | 3 | step 4: `108,211 ppb` | 63 | `1,149 µm` |
| 160, diagnostic | 4 | step 5: `126,883 ppb` | 108 | `1,935 µm` |

The corresponding one-layer terminal values were `192,429`, `108,150` and
`126,814 ppb`, with the same accepted-step counts. A support-complete field
changes the failing residual only minimally and does not change the outcome.
This activates the W0D stop rule: no third layer or another particle-shell
scale is admissible.

The free-fall control matches all 97 frames exactly. Baseline and candidate
final roots are both
`cbe47b57dbb819e44eabe53049a1b9cb44c6a94626d560421c73deb6b1db4011`.

## Why a published density map is not a drop-in repair

The continuous model in
[Density Maps for Improved SPH Boundary Handling](https://animation.rwth-aachen.de/media/papers/kb17.pdf)
does not merely convolve a constant solid half-space. Its Equation 8 uses a
linear basis density that equals rest density at the surface, increases inside
solid and remains positive through a fluid-side buffer of one support radius.
That deliberate buffer prevents visible penetration, but it also selects an
equilibrium wall offset.

A degree-30 exact-box quadrature pre-design calculation at the current
`25,000 µm` clearance gives boundary-density contributions of approximately
`0.750010`, `0.904510` and `0.982893` for face, edge and corner queries. Added
to the current discrete fluid terms, the density ratios are approximately
`1.600298`, `1.624171` and `1.589454`: all fail the current initial gate.
Matching each missing discrete contribution separately would require wall
clearances of approximately `92,641`, `88,442` and `83,883 µm`. One uniform
lattice phase therefore cannot make the published field exactly partition
all three feature ranks.

The paper's reported experiments also use XSPH viscosity, while W0B
explicitly disables viscosity and warm start. Published stability results
cannot validate that untested combination. A genuine density-map candidate
must jointly define its field and gradient, predicted-position evaluation,
free-surface/solid composition, equilibrium generator, stabilization terms
and authority state. Reusing only the word “density map” under the rejected
roots would repeat the architectural error.

## Decision and next architecture step

- Reject `support-complete-lattice-complement-v1`; keep W1 blocked and issue no
  successor roots.
- Retain the consistent DFSPH boundary equations as a supported formulation,
  but withdraw the claim that the current cold-start/no-stabilization profile
  is validated.
- Do not implement a density map under the unchanged `25,000 µm` authored
  lattice or treat a new wall offset as a local constant tweak.
- Reclose the boundary field, initial equilibrium and non-pressure profile as
  one successor decision. The recommended candidate is an exact convolved
  density field plus a deterministic equilibrium generator and an explicitly
  frozen, accounted stabilization model. The alternative is a separately
  derived unilateral geometric non-penetration constraint coupled to the
  pressure solve; it is a different solver architecture, not a repair pass.
- Keep canonical sample ID/position/velocity authority unless evidence proves
  that pressure continuation is required. An upstream-style warm start may be
  evaluated only as a state-contract discriminator, never hidden in runtime
  cache state.
- Require the new local gate to cover face/edge/corner field values and
  gradients, equilibrium generation, 24 unchanged-profile hydro steps,
  free-fall exactness and boundary reaction closure before full corpus work.

`CONTINUUM-WATER-REF-P1` remains `NOT_RUN`.

## Verification

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | `PASS` |
| Strict all-target Clippy for `next_continuum_water` | `PASS` |
| Exact-profile focused W0D tests | `PASS` — 2/2 |
| `next_continuum_water` tests | `PASS` — 44/44 |
| `xtask` tests | `PASS` — 99/99 library and 44/44 binary |
| Strict all-target Clippy for `xtask` | `PASS` |
| `cargo run --locked -p xtask -- boundary-scan` | `PASS` — all six checks |
| Clean exact-profile candidate report | `EXACT_MATCH / CANDIDATE_REJECTED / NOT_SELECTED` |
| `CONTINUUM-WATER-REF-P1` | `NOT_RUN` |

The broad workspace `host-check`, full water corpus and performance/coupling
checks were not run because the local discriminator rejected the candidate.
