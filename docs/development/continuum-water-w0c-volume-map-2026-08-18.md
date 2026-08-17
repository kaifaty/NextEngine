# Continuum water W0C analytical volume-map evidence — 2026-08-18

Status: `REPORT_ONLY / CANDIDATE_REJECTED / W0C_RESEARCH_ONLY`.

## Scope and claim

This report records the final bounded W0C discriminator:
`volume-map-box-bender2019-ref-v1`. It is a private serial-oracle research
path, not a successor water profile or runtime implementation. The original
W0B roots, public contracts, 20-iteration ceiling, `100,000 ppb` density
tolerance and canonical integer position/velocity authority remain unchanged.

Production and a separately written brute-force calculator agree exactly, and
the 97-frame free-fall control remains byte-identical. The candidate still
fails the initial partition and the first hydro step by large margins. Under
the predeclared W0C stop rule, this closes W0C as `RESEARCH_ONLY`; W1 cannot
resume and no successor roots or ProductCheck credit exist.

## Frozen candidate

The method is based on the primary
[Volume Maps paper](https://animation.rwth-aachen.de/media/papers/65/2019-MIG-VolumeMaps.pdf)
and pinned SPlisHSPlasH commit
[`eccce861`](https://github.com/InteractiveComputerGraphics/SPlisHSPlasH/tree/eccce86155776f6ac52d5080b1f720a52bf29450).
The paper defines a boundary volume from a cubic signed-distance extension and
represents it by one virtual boundary sample. The pinned reference code adds a
`0.8` volume factor and places the virtual sample at
`max(distance + 0.5 particle_radius, 2 particle_radius)` to avoid strong
pressure forces.

The candidate freezes those reference operations and replaces only the
regular-grid SDF interpolation with the exact signed-distance field of the
selected closed axis-aligned box:

- signed distance is positive inside the fluid box and negative in exterior
  solid; exact face ties use the normalized symmetric subgradient;
- the extension is `1` in solid,
  `CubicKernel::W(distance) / CubicKernel::W(0)` inside one support radius and
  `0` beyond it;
- degree-30 tensor Gauss-Legendre quadrature uses 16 frozen points per axis in
  `i-j-k` order over the support cube, with a spherical support cutoff;
- the volume is multiplied by the reference `0.8` constant;
- density, factor and pressure operators consume at most one virtual sample
  per fluid row through the unchanged cubic SPH kernel;
- every query is rebuilt from canonical integer position. There is no map
  cache in authority, warm start, retained float state or public schema.

The implementation commit is
`ea122208ca102c5fd63febc229f7d14b72de5b19`. The clean exact-profile report
uses schema `nextengine.continuum-water.volume-map-boundary-candidate.v1` and
has SHA-256
`df03f05e573ea464d7786677b355501b8b0b218685aa2f4609a58d11e3d258c7`.
The bounded JSON remains outside Git.

Exact invocation shape:

```text
CARGO_ENCODED_RUSTFLAGS=<exact W0B flags> \
cargo run --locked --profile water-oracle \
  --target x86_64-unknown-linux-gnu -p xtask -- \
  continuum water evaluate-hydro-candidate \
  --candidate volume-map-box-bender2019-ref-v1 \
  --output <absolute-path-outside-Git>
```

## Independent equality

The production path and independent calculator share only report types. The
independent implementation duplicates the analytical SDF, frozen quadrature
tables, cubic extension, virtual-point rule, fluid reconstruction and complete
320-iteration diagnostic solve. They match exactly for:

- face, edge, corner, interior, off-grid and support-boundary field probes;
- signed distance, volume, virtual displacement, kernel value/gradient,
  volume-weighted gradient and density-contribution bits;
- all four selected reconstruction rows;
- all 320 residuals and five prospective checkpoints.

The analytical box normal has the expected axis switch across an exact edge
bisector. This is reported explicitly; it is not the cause of the much earlier
partition failure.

## Partition failure

The selected lattice places the first fluid centres `25,000 µm` from the wall:
one particle radius, while support is `100,000 µm`. At that distance the
reference volume approximation assigns substantially more boundary density
than the discrete fluid partition is missing.

| Row | Self + fluid (ppb) | Volume-map boundary (ppb) | Reconstructed density (ppb) | Partition error (ppb) | Required post-hoc scale |
| --- | ---: | ---: | ---: | ---: | ---: |
| Corner | 606,560,836 | 1,993,008,656 | 2,599,569,492 | +1,599,569,492 | 197,409,661 ppb |
| Edge | 719,660,965 | 1,784,445,394 | 2,504,106,360 | +1,504,106,360 | 157,101,492 ppb |
| Face | 850,287,875 | 1,288,740,400 | 2,139,028,274 | +1,139,028,274 | 116,169,343 ppb |
| Interior | 999,972,466 | 0 | 999,972,466 | -27,534 | n/a |

The local gate is the unchanged `100,000 ppb` tolerance. The maximum absolute
selected error is `1,599,569,492 ppb`, so the candidate fails before any
corpus claim. The different face/edge/corner correction factors also reject a
uniform fitted multiplier.

## Solver and control results

The first normal hydro step fails with
`WATER_DENSITY_NONCONVERGENCE`: iteration 20 ends at `70,690,915 ppb`. The
required 24-step unchanged-ceiling soak therefore accepts zero steps.

The extended curve is diagnostic only and cannot earn gate credit:

| Iteration | Residual (ppb) | Prospective minimum clearance | Prospective penetration | Outer escape |
| ---: | ---: | ---: | ---: | --- |
| 20 | 70,690,915 | 4,447 µm | 20,553 µm | no |
| 40 | 51,393,897 | -6,955 µm | 31,955 µm | yes |
| 80 | 29,290,620 | -21,670 µm | 46,670 µm | yes |
| 160 | 10,037,028 | -34,971 µm | 59,971 µm | yes |
| 320 | 1,211,465 | -41,194 µm | 66,194 µm | yes |

Even at iteration 320 the residual is more than twelve times the fixed gate
and the prospective state has escaped. A larger ceiling is not a remedy.

The pre-impact free-fall control matches all 97 frames exactly. Baseline and
candidate final frame root are both
`cbe47b57dbb819e44eabe53049a1b9cb44c6a94626d560421c73deb6b1db4011`.
This proves that the new path is isolated when no boundary is in support; it
does not offset the interacting hydro failure.

## Decision

- Reject `volume-map-box-bender2019-ref-v1`; issue no successor document,
  float-profile, corpus or scenario roots.
- Close W0C as `RESEARCH_ONLY`. Do not start W1 corpus continuation, W2,
  coupling, GPU work or public continuum contracts.
- Do not fit the `0.8` factor, change the virtual-point offset, increase the
  iteration ceiling, loosen density tolerance or retain hidden float state.
- Continuing requires a new explicit architecture/profile decision. The
  recommended option is a separately scoped density-map discriminator that
  stores the kernel-convolved missing density and its gradient directly while
  preserving canonical integer authority. It must define exact field,
  interpolation or analytical evaluation, feature/aperture composition,
  capacities and an independent calculator before code starts.
- Revising the lattice phase or wall clearance is the alternative, but it
  changes sample count, fill geometry and corpus identity and therefore needs
  a successor product/profile closure rather than an in-place W0C tweak.

## Verification

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | `PASS` |
| Strict all-target Clippy for `next_continuum_water` | `PASS` |
| `next_continuum_water` tests | `PASS` — 42/42 |
| `xtask` tests | `PASS` — 99/99 library and 44/44 binary |
| Strict all-target Clippy for `xtask` | `PASS` |
| `cargo run --locked -p xtask -- boundary-scan` | `PASS` — all six checks |
| Clean exact-profile candidate report | `EXACT_MATCH / CANDIDATE_REJECTED / NOT_SELECTED` |
| `CONTINUUM-WATER-REF-P1` | `NOT_RUN` |

The broad workspace `host-check` and full water corpus were not run because
the local discriminator rejected the candidate. No production, performance,
persistence, coupling or activation claim is made.
