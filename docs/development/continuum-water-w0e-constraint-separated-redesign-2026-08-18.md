# Continuum water W0E constraint-separated redesign evidence — 2026-08-18

Status: `REPORT_ONLY / LOCAL_PROFILE_DISCRIMINATOR_SURVIVED / NOT_SELECTED`.

## Scope and result

This report records the bounded W0E candidate
`constraint-separated-support-pcg-v1`. It redesigns the failed hydro profile
around separate constraints instead of asking one boundary discretization to
provide density completion, geometric containment and numerical damping.

The redesign resolves the previously repeatable local failure. It passes a
24-step gate and a 1200-step equilibrium soak with the unchanged
`100,000 ppb` density threshold, a 50-iteration projected-PCG ceiling, no
retry and zero particle-radius penetration. Production and separately written
calculators agree exactly for the initial boundary, pressure operator,
first-step projected PCG result and contact operator. The 97-frame free-fall
control remains byte-identical.

This is not a W1 or product PASS. Internal apertures, dynamic rigid contact,
successor roots, the full corpus, cross-target roots and 50k cost are not
closed. The only admissible disposition is
`LOCAL_PROFILE_DISCRIMINATOR_SURVIVED / NO_CORPUS_CREDIT / NOT_SELECTED`.
`CONTINUUM-WATER-REF-P1` remains `NOT_RUN`.

## Architectural decision

| Responsibility | W0E operation | Authority consequence |
| --- | --- | --- |
| Density support | Static two-layer exterior lattice complement | Rebuilt profile data; not canonical state |
| Incompressibility | Deterministic projected diagonally preconditioned PCG | Multiplier is transient and not persisted |
| Non-penetration | Analytical velocity-level unilateral outer-box constraint | Contact impulse is included in reaction accounting |
| Initialization | Authored regular lattice and zero velocity | No hidden equilibrium or continuation state |
| Stabilization | `NONE` | No XSPH, viscosity, surface tension or damping parameter |
| Published state | Stable ID plus canonical integer position/velocity | Existing fixed-point authority remains unchanged |

The static density complement and geometric constraint intentionally share
the same admitted particle radius but do different jobs. Pressure is applied
first, contact projects the resulting velocity before integration, and no
position is repaired afterward. This sequential split is sufficient for the
bounded static-box evidence; it is not yet proof that a monolithic coupled LCP
is unnecessary for moving rigid bodies.

## Causal discrimination

The earlier W0D two-layer complement passed step 1 and failed step 2 at
`192,430 ppb`; its first accepted state penetrated the wall by `171 µm`.
W0E evaluated containment and pressure convergence independently:

| Profile | Accepted steps | Terminal density result | Penetration | Meaning |
| --- | ---: | ---: | ---: | --- |
| Two layers + Jacobi-20, no contact | 1/24 | step 2: `192,430 ppb` | `171 µm` | W0D rejected baseline |
| Contact + Jacobi-20 | 1/24 | step 2: `205,138 ppb` | `0 µm` | Contact fixes geometry, not convergence |
| Contact + Jacobi-160 | 24/24 | maximum accepted `99,941 ppb` | `0 µm` | Stronger pressure solve and contact are independently needed |
| Contact + projected PCG-50 | 24/24 | maximum accepted `99,723 ppb` | `0 µm` | Surviving algorithmic candidate |

Without contact, the W0D Jacobi-160 path failed on step 5. With contact it
passes 24 steps, while contact with Jacobi-20 still fails on step 2. The old
failure was therefore not one bad formula and not one missing shell layer. It
was the composition of a weak cold-start pressure solver and a density-only
boundary with no geometric non-penetration contract.

## Frozen mathematical profile

For advected normalized density, W0E defines

```text
b_i = rho_adv_i - 1
B(k) = -dt^2 D(P(k))
```

and solves the unilateral complementarity problem

```text
k >= 0
w = B(k) - b >= 0
k_i * w_i = 0.
```

`P` is the previously audited linear pressure-acceleration operator and `D`
is the density-rate operator. The existing DFSPH factor divided by `dt²` is
the inverse diagonal preconditioner. A row is active when its multiplier is
positive or its compression residual is positive. Projection or an active-set
change restarts the conjugate direction; every reduction retains stable
sample-ID order.

After pressure convergence and before position integration, each velocity
component is projected onto

```text
(min + particle_radius - position) / dt
    <= velocity <=
(max - particle_radius - position) / dt.
```

The exact fluid impulse is `mass * (v_after - v_before)`. Its opposite belongs
to the static boundary, and the fluid impulse is included alongside the
pressure-boundary impulse in momentum accounting.

## Independent implementation checks

The independent code does not call the production pressure action, PCG loop,
contact calculator or boundary generator.

- The two-layer boundary and selected initial density reconstruction match
  exactly. The boundary-input root is
  `16b70d3db32f099630101021e542aaa6d7d95d9be6c931207f6521df8056251a`.
- Independently assembled full pressure actions have roots
  `0307a94a6c175b7a143c650c038006fa31cecab8cc4db1ad0d85bea8a7fc0026`
  and
  `07aa7a500fd90d2eaeb57b62cd38c777cb360b6bf3bc51cc14fd61643e4a6fb3`.
  Selected actions match exactly, both quadratic products are positive, the
  symmetry difference quantizes to `0 ppb`, and selected action diagonals
  differ from factor-derived values by at most one binary64 ULP (`0 ppb`).
- The separately written first-step projected-PCG loop matches at 2
  iterations, `99,723 ppb`, and maximum multiplier bits
  `0x3ff20b06e99b10cf`.
- The separately written contact calculator matches every velocity component,
  impulse component and active count for eight cases: lower/upper faces,
  separating motion, edge, corner, an admitted shallow position, an interior
  no-hit and an interior predicted hit.

These checks reject a shared implementation mistake as the explanation for
the surviving result. They do not replace independent external corpus roots.

## Bounded soak and accounting

| Observation | 24 steps | 1200 steps |
| --- | ---: | ---: |
| Completed | `24/24` | `1200/1200` |
| Maximum density iterations | `48` | `48` |
| Maximum accepted density error | `99,723 ppb` | `99,998 ppb` |
| Maximum divergence iterations/error | `1 / 428,407 ppb` | `1 / 428,407 ppb` |
| Minimum outer clearance | `25,000 µm` | `25,000 µm` |
| Maximum penetration | `0 µm` | `0 µm` |
| Maximum contact rows/components | `1,464 / 1,600` | `1,512 / 1,652` |
| Maximum contact delta velocity | `394,901 µm/s` | `702,836 µm/s` |
| Maximum momentum residual | `214 ppb` | `214 ppb` |
| Maximum energy residual | `2,503,297 ppb` | `3,426,068 ppb` |
| Maximum vertical COM drift | `1,049 µm` | `1,708 µm` |

The existing accounting gates are 1%; the largest momentum residual is
`0.0000214%` and the largest energy/work residual is approximately `0.343%`.
The centre of mass moves from `(500000, 375000, 500000) µm` to
`(500000, 373338, 500000) µm` after 1200 steps and remains bounded around its
settled height. The final roots are
`399705af374393503581dd9f3d03bc0bba8e44af732544c5ea3c19d7dbd90b4f`
at step 24 and
`026d26585edbda74aff93ef126810b0ced0a7c9f5d623b4dbf60260b48554b18`
at step 1200.

Free-fall matches all 97 frames, activates zero contact components, converges
with density error zero and ends at the unchanged root
`cbe47b57dbb819e44eabe53049a1b9cb44c6a94626d560421c73deb6b1db4011`.

## Source audit and stabilization decision

The retained pressure/boundary formulation remains consistent with
[Consistent SPH Rigid-Fluid Coupling](https://animation.rwth-aachen.de/media/papers/84/2023-VMV-SPH_ConsistentBoundaryHandling.pdf),
the DFSPH formulation in
[Divergence-Free SPH for Incompressible and Viscous Fluids](https://animation.rwth-aachen.de/media/papers/2017-TVCG-ViscousDFSPH.pdf),
and the scalar path of pinned
[SPlisHSPlasH commit `eccce861`](https://github.com/InteractiveComputerGraphics/SPlisHSPlasH/tree/eccce86155776f6ac52d5080b1f720a52bf29450).

[Density Maps for Improved SPH Boundary Handling](https://animation.rwth-aachen.de/media/papers/kb17.pdf)
reports XSPH in its experiments, but does not define a universal coefficient
for this lattice and wall offset. The pinned SPlisHSPlasH implementation
initializes both XSPH coefficients to zero and leaves their selection to scene
configuration. W0E therefore does not invent a damping coefficient. The
1200-step result supplies positive evidence that XSPH, a density map, warm
pressure state and a settling generator are unnecessary for this local gate.

## Decision and remaining architecture gaps

The proposed separation is correct for the bounded static outer-box problem:
it resolves both independently demonstrated causes, preserves fixed-point
state authority, accounts for constraint impulses and survives independent
operator checks. It is not yet a complete product architecture.

1. W0E rejects internal apertures explicitly. `CW-ORIFICE-001` requires a
   shared solid/opening geometry representation for density support and
   contact, plus an independent implementation.
2. The selected `4 × 2 × 1 m` extent needs `24,704` two-layer exterior samples
   (`84 × 44 × 24 - 80 × 40 × 20`), exceeding the old `16,384` boundary
   capacity before the crate is represented. W0F must root a larger capacity
   or an equivalent implicit complement.
3. Contact is a pressure-then-contact split against a static box. Dynamic
   crate reaction, fast impact, added-mass behavior and the need for a coupled
   or iterated solve remain W3 questions.
4. PCG-50 is an iteration cap, not a cost result. A PCG iteration performs
   extra operator actions and reductions; 50k performance remains unmeasured.
5. The successor document/float/execution/corpus/scenario roots are not issued.
   The rejected W0B roots must not be relabelled or edited.

The next bounded package is W0F: close geometry, capacity and successor roots,
then run the complete W1 serial corpus and cross-target exactness. W2, GPU,
PhysX coupling, runtime/public contracts and promotion remain blocked.

## Clean report

Implementation commit:
`7ee1651b6c1bfcef575af1bd6952ac36f9fca661`.

The clean exact-profile JSON uses schema
`nextengine.continuum-water.constraint-separated-redesign.v1`, records a
`CLEAN` tool tree at checkpoint
`b1f7c43925ae6231556e856cc9cddd2b174c2e18` and has SHA-256
`24a1de90b8e02dc3146f6c01e868e8f6ab5a76b0aaa3679aa6f4adbb7cbec8a3`.
It remains outside Git. Its command was:

```bash
CARGO_ENCODED_RUSTFLAGS=$'-Ctarget-cpu=x86-64\x1f-Ctarget-feature=-sse3,-ssse3,-sse4.1,-sse4.2,-avx,-avx2,-fma\x1f-Cllvm-args=-fp-contract=off' \
cargo run --locked --profile water-oracle \
  --target x86_64-unknown-linux-gnu -p xtask -- \
  continuum water evaluate-hydro-redesign \
  --candidate constraint-separated-support-pcg-v1 \
  --output /tmp/nextengine-w0e-clean-b1f7c43.json
```

## Verification

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | `PASS` |
| Strict all-target Clippy for `next_continuum_water` | `PASS` |
| `next_continuum_water` tests | `PASS` — 49/49 |
| `xtask` tests | `PASS` — 99/99 library and 44/44 binary |
| Strict all-target Clippy for `xtask` | `PASS` |
| `cargo run --locked -p xtask -- boundary-scan` | `PASS` — all six checks |
| Clean exact-profile redesign report | `EXACT_MATCH / LOCAL_PROFILE_DISCRIMINATOR_SURVIVED / NOT_SELECTED` |
| `CONTINUUM-WATER-REF-P1` | `NOT_RUN` |

The broad workspace `host-check`, full W1 corpus and performance/coupling
checks are intentionally not claimed by this local research discriminator.
