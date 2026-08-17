# Continuum water W0C hydro-calibration evidence — 2026-08-17

Status: `REPORT_ONLY / BOUNDARY_AND_INITIALIZATION_CANDIDATES_REJECTED / SUPERSEDED_BY_W0C_CLOSURE`.

## Scope and claim

This report records the first two W0C counterfactual cycles. They add
diagnostics, not a successor water profile: the original W0B roots, W1
production path, 20-iteration ceiling and failure semantics remain unchanged.
All generated artifacts declare `NO_CORPUS_CREDIT`;
`CONTINUUM-WATER-REF-P1` remains `NOT_RUN`.

The cycles answer three questions:

1. Is the original hydro failure plausibly fixed by a larger iteration
   ceiling?
2. Does a deterministic one-cell layer of cell-centred ghost volumes provide
   a viable boundary replacement?
3. Can bounded zero-velocity settling turn that corrected initial partition
   into a stable canonical hydro state?

The answer to all three is no. Persistent failure now requires an
adjacent-layer boundary-formulation discriminator, not another ceiling, fitted
scale or position-only settling variant.

That follow-up discriminator is complete. The
[analytical volume-map report](continuum-water-w0c-volume-map-2026-08-18.md)
rejects the adjacent-layer candidate and closes W0C `RESEARCH_ONLY`. The
historical conclusions below remain evidence for the earlier cycles, not a
current authorization to implement another candidate.

## Reproducible artifacts

The first two reports were produced from clean commit
`328d01c70004bd0c9572ca4dc49891d141f6ba7f`; the initialization report was
produced from clean commit `0c3acb4969f32cac1bbe1205a6e07307a10cc2c0`.
All use `x86_64-unknown-linux-gnu`, Cargo profile `water-oracle` and the exact
W0B Rust flags. The bounded JSON files remain outside Git.

| Report | Schema and result | SHA-256 |
| --- | --- | --- |
| Original-profile diagnosis | `nextengine.continuum-water.hydro-calibration-diagnostic.v1`; `EXACT_MATCH / REPORT_ONLY` | `6e8a6c29772083fbba3b8176ac3b3fb2a00528b3883093714b6e66f2b462e784` |
| `ghost-cell-shell-v1` evaluation | `nextengine.continuum-water.hydro-boundary-candidate.v1`; `EXACT_MATCH / CANDIDATE_REJECTED / NOT_SELECTED` | `f3cb823fb8d44018188d08963b45ba4bb50a13f4bf1071673dbddc8994c131aa` |
| `zero-velocity-settle-v1` evaluation | `nextengine.continuum-water.hydro-initialization-candidate.v1`; `EXACT_MATCH / CANDIDATE_REJECTED / NOT_SELECTED` | `842f1a92fe04ac578bd99e9351c9b20b46fecbbe30213d583db2175c570ce1ab` |

The shared fluid-input root is
`bd18fe6e6a305ccc875ba12014e90f0cbca74a05068c7bd573ef95d3f3f51dbc`.
The original boundary-input root is
`301246ba4e6efdf1b2f13e60605a2394aab46b22826f2083f67c6cce1d8df0a7`;
the candidate boundary-input root is
`64726ee4cebe64a393eb2476f932bb63c9c6aa601dc970d4211d3be0bcb21168`.

Exact invocation shapes:

```text
CARGO_ENCODED_RUSTFLAGS=<exact W0B flags> \
cargo run --locked --profile water-oracle \
  --target x86_64-unknown-linux-gnu -p xtask -- \
  continuum water diagnose-hydro \
  --output <absolute-path-outside-Git>

CARGO_ENCODED_RUSTFLAGS=<exact W0B flags> \
cargo run --locked --profile water-oracle \
  --target x86_64-unknown-linux-gnu -p xtask -- \
  continuum water evaluate-hydro-candidate \
  --candidate ghost-cell-shell-v1 \
  --output <absolute-path-outside-Git>

CARGO_ENCODED_RUSTFLAGS=<exact W0B flags> \
cargo run --locked --profile water-oracle \
  --target x86_64-unknown-linux-gnu -p xtask -- \
  continuum water evaluate-hydro-initialization \
  --candidate zero-velocity-settle-v1 \
  --output <absolute-path-outside-Git>
```

## Primary-source review

- The original [DFSPH paper](https://dankoschier.github.io/resources/papers/BK17.pdf)
  provides the pressure/divergence projection context, but does not justify
  changing this profile's no-warm-start authority or accepting an unstable
  local trajectory.
- Pinned SPlisHSPlasH commit
  [`eccce861`](https://github.com/InteractiveComputerGraphics/SPlisHSPlasH/tree/eccce86155776f6ac52d5080b1f720a52bf29450)
  initializes DFSPH pressure solves with 100 maximum iterations and `0.01%`
  maximum density error in
  [`TimeStepDFSPH.cpp`](https://github.com/InteractiveComputerGraphics/SPlisHSPlasH/blob/eccce86155776f6ac52d5080b1f720a52bf29450/SPlisHSPlasH/DFSPH/TimeStepDFSPH.cpp).
  This motivates the bounded `max100` counterfactual; it does not make that
  setting a Next Engine selection.
- The pinned
  [scene format](https://github.com/InteractiveComputerGraphics/SPlisHSPlasH/blob/eccce86155776f6ac52d5080b1f720a52bf29450/doc/file_format.md)
  identifies volume maps as the current default boundary method. This does not
  validate a Next Engine formula, but makes a non-particle boundary
  representation the next distinct hypothesis after two particle-shell
  failures.
- The same commit's
  [`BoundaryModel_Akinci2012.cpp`](https://github.com/InteractiveComputerGraphics/SPlisHSPlasH/blob/eccce86155776f6ac52d5080b1f720a52bf29450/SPlisHSPlasH/BoundaryModel_Akinci2012.cpp)
  computes a boundary pseudo-volume as the reciprocal of self-kernel plus
  neighbouring boundary kernels. This confirms the provenance of the W0B
  volume rule, not the correctness of its selected sampling topology.
- The primary [Ghost SPH paper](https://www.cs.ubc.ca/~rbridson/docs/schechter-siggraph2012-ghostsph.pdf)
  motivates a narrow ghost-particle layer around solids. It supports testing
  this candidate family, but supplies no evidence for the exact lattice or
  dynamic stability used here.
- A primary SPH
  [hydrostatic-settling study](https://link.springer.com/article/10.1007/s00158-017-1729-x)
  describes hydrostatic initialization as marching an unsteady calculation to
  a damped steady state. This supports testing an explicitly bounded settling
  generator; it does not excuse a generator whose iteration demand and wall
  penetration grow.

## Original W0B diagnosis

Production and an independent brute-force calculator agree exactly for every
recorded contribution, all 320 residuals and all prospective checkpoints.

| Row | Self + fluid (ppb) | Boundary (ppb) | Partition error (ppb) | Required uniform boundary scale (ppb) |
| --- | ---: | ---: | ---: | ---: |
| Corner | 606,560,836 | 1,192,211,013 | +798,771,849 | 330,007,993 |
| Edge | 719,660,965 | 1,014,602,254 | +734,263,219 | 276,304,368 |
| Face | 850,287,875 | 694,381,858 | +544,669,732 | 215,604,892 |
| Interior | 999,972,466 | 0 | -27,534 | n/a |

The boundary-adjacent excess is topology-dependent: the scale required at a
corner, edge and face is materially different. A single fitted boundary
multiplier is therefore rejected.

The unchanged solver never reaches the existing `100,000 ppb` density
threshold by iteration 320:

| Iteration | Residual (ppb) | Prospective max speed (µm/s) | Minimum outer clearance (µm) | Outer escape |
| ---: | ---: | ---: | ---: | --- |
| 20 | 74,482,699 | 45,154,403 | -83,638 | yes |
| 40 | 51,452,219 | 56,058,476 | -110,060 | yes |
| 80 | 24,624,457 | 67,856,939 | -138,816 | yes |
| 160 | 5,741,077 | 76,138,917 | -159,135 | yes |
| 320 | 382,257 | 78,577,441 | -165,233 | yes |

An extended ceiling would still fail the residual gate and already proposes
escape/penetration at iteration 20. It cannot be promoted as a repair.

## `ghost-cell-shell-v1`

The candidate places one `REST_VOLUME` sample at each exterior cell centre in
the one-cell shell around the analytical box. Its origin is
`bounds.min + PARTICLE_RADIUS`; it has 2,648 samples and no fitted multiplier.
The independent implementation hard-codes its own generator and reconstruction
and matches production exactly for boundary positions, volume bits,
contribution rows and the complete 320-iteration trace.

All four selected rows reconstruct to `999,972,466 ppb`, the same
`-27,534 ppb` lattice partition error as the interior. The initial density solve
reaches `95,755 ppb` at iteration 2, keeps `24,829 µm` minimum outer clearance
and does not escape. The normal first hydro step therefore passes.

The dynamic discriminator rejects the candidate:

| Density ceiling | Requested steps | Completed steps | Terminal residual | Maximum iterations in accepted steps | Maximum accepted penetration |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 20 (unchanged production) | 24 | 1 | step 2: `192,429 ppb` at iteration 20 | 2 | 171 µm |
| 100 (source-derived counterfactual) | 24 | 3 | step 4: `108,150 ppb` at iteration 100 | 63 | 1,149 µm |
| 160 (diagnostic) | 24 | 4 | step 5: `126,814 ppb` at iteration 160 | 108 | 1,935 µm |

The 97-frame free-fall control remains exact with final root
`cbe47b57dbb819e44eabe53049a1b9cb44c6a94626d560421c73deb6b1db4011`.
That successful control shows the candidate path is isolated, but cannot
override the interacting hydro failure.

## `zero-velocity-settle-v1`

The initialization candidate starts from the regular hydro lattice and
`ghost-cell-shell-v1`. Before every relaxation pass it sets all canonical
velocities to zero, runs one serial DFSPH step with diagnostic ceiling 160,
publishes canonical positions and discards the accepted velocity. It allows at
most 24 passes. Success requires two consecutive passes within the production
20-iteration ceiling and no more than `1 µm` maximum displacement. A fail-closed
guard rejects three consecutive transitions with increasing iteration demand
and increasing wall penetration.

Production and the separate brute-force generator agree exactly on every pass
and all 6,000 final diagnostic samples:

| Pass | Density iterations | Residual (ppb) | Maximum displacement | Penetration | Centre-of-mass y |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 2 | 95,755 | 171 µm | 171 µm | 374,840 µm |
| 2 | 33 | 98,983 | 231 µm | 340 µm | 374,723 µm |
| 3 | 40 | 99,961 | 201 µm | 521 µm | 374,610 µm |
| 4 | 47 | 98,517 | 189 µm | 710 µm | 374,506 µm |

The generator terminates at pass 4 with `SETTLING_ADVERSE_TREND`; it produces
no selectable initial state. The last accepted diagnostic state has exact
production/independent root
`373af3e7270c46741078b08335629821a50853b7e3bb1fca3c8c462748af1df9`,
but is explicitly `NOT_A_GENERATOR_OUTPUT / NO_CORPUS_CREDIT`. When probed with
the unchanged ceiling, it completes zero hydro steps and reaches
`162,015 ppb` at iteration 20. The free-fall control remains exact across all
97 frames.

## Decision and next discriminator

- Reject `ghost-cell-shell-v1`; do not issue successor roots and do not run the
  full corpus or external aggregate comparison for it.
- Reject `zero-velocity-settle-v1`; its full damping does not contract the
  hydro transient, and its last diagnostic state fails the first production
  step.
- Reject further ceiling-only variants. The `20 → 100 → 160` sequence moves
  the failure while accepted-step iteration demand and penetration grow.
- Preserve the original W0B and W1 production profiles unchanged.
- Apply the repository's persistent-problem escalation rule. The subsequent
  bounded discriminator moved to an analytical non-particle volume map while
  keeping canonical position/velocity authority and the unchanged production
  ceiling.
- The linked 2026-08-18 report shows that it fails local partition and the
  first hydro step despite exact independent equality. The predeclared stop
  rule is therefore satisfied: W0C is `RESEARCH_ONLY`, not open for another
  settling, boundary-scale or iteration variant.

## Verification

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | `PASS` |
| Strict all-target Clippy for `next_continuum_water` | `PASS` |
| `next_continuum_water` all-target tests | `PASS` — 39/39 |
| `xtask` tests | `PASS` — 99/99 library and 44/44 binary |
| Strict all-target Clippy for `xtask` | `PASS` |
| `cargo run --locked -p xtask -- boundary-scan` | `PASS` — all six checks |
| Clean exact-profile diagnostic reports | `PASS` as report generation and independent equality; both candidate dispositions remain rejected |
| `CONTINUUM-WATER-REF-P1` | `NOT_RUN` |

The broad workspace `host-check` was not rerun for this localized diagnostic
change. No production, performance, persistence, coupling or activation claim
is made.
