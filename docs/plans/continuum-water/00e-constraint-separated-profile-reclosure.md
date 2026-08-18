# W0E — Constraint-separated water-profile reclosure

Status: `CLOSED / LOCAL_PROFILE_DISCRIMINATOR_SURVIVED / RESEARCH_ONLY`.

## Outcome

The candidate survived every bounded W0E gate. Production and separately
written calculators match exactly for the support-complete boundary, pressure
operator, first projected-PCG solve and analytical contact operator. The
candidate passes both the 24-step local gate and a 1200-step equilibrium soak
with zero particle-radius penetration; the 97-frame free-fall control remains
byte-identical.

This result resolves the local second-step failure, but does not select a
product profile. The full W1 corpus, internal aperture semantics, dynamic
rigid contact, successor roots and 50k cost remain open. The disposition is
therefore exactly
`LOCAL_PROFILE_DISCRIMINATOR_SURVIVED / NO_CORPUS_CREDIT / NOT_SELECTED` and
`CONTINUUM-WATER-REF-P1` remains `NOT_RUN`.

Implementation commit:
`7ee1651b6c1bfcef575af1bd6952ac36f9fca661`.
The clean exact-profile report has SHA-256
`PENDING_CLEAN_CLOSURE_REPORT`;
its JSON stays outside Git. The bounded results are recorded in the
[W0E evidence report](../../development/continuum-water-w0e-constraint-separated-redesign-2026-08-18.md).

## Decision being tested

W0E replaces the rejected assumption that one boundary representation must
simultaneously provide density completion, visible containment and numerical
stabilization. The successor candidate separates those responsibilities:

1. a support-complete discrete lattice complement supplies only the static
   boundary contribution to density and its gradient;
2. an analytical unilateral constraint keeps particle centres inside the
   admissible outer box;
3. a projected pressure solve enforces the one-sided incompressibility
   constraint from the current canonical state;
4. the authored regular lattice and zero velocity are the explicit cold-state
   initializer; no hidden pressure, warm-start or settling state is assumed;
5. no viscosity, XSPH, surface tension or positional repair is present in the
   first candidate.

The candidate ID is `constraint-separated-support-pcg-v1`. It is a new
research profile, not a correction under the W0B roots and not a selected
runtime profile.

## Why the architecture changed

W0D established that the consistent DFSPH boundary equations match the
primary formulation and the pinned upstream scalar implementation. It also
established that completing the exterior particle support does not prevent
the dynamic failure. A direct implementation of the published density map is
not compatible with the authored `25 mm` wall offset: its basis field
overfills the initial face, edge and corner rows by different amounts.

The first W0E counterfactual then separated containment from density. A
terminal predictive contact projection reduced first-step penetration from
`171 µm` to zero, but the unchanged 20-sweep relaxed-Jacobi solve still failed
on step 2 at `205,138 ppb`, compared with `192,430 ppb` without the contact
constraint. Geometric penetration was therefore a consequence and an
independent missing constraint, not the cause of density non-convergence.

A diagnostic 160-sweep Jacobi solve combined with the same contact constraint
passed all 24 local hydro steps with zero penetration; without contact, the
160-sweep W0D path failed on step 5. This discriminates two independent
defects in the rejected composition:

- the relaxed-Jacobi pressure solve is too weak for the cold-start profile;
- the density boundary alone is not a geometric non-penetration contract.

The successor tests an algorithmic pressure change rather than treating 160
Jacobi sweeps as the solution. Its projected diagonally preconditioned
conjugate-gradient prototype passes the 24-step and 1200-step gates with at
most 48 iterations. Full-corpus stability and product-scale performance
remain unproven.

## Frozen candidate operations

### Density closure

- Keep `REST_VOLUME`, the `50,000 µm` fluid lattice and cubic kernel unchanged.
- Use exactly the two exterior lattice layers from W0D. They are the complete
  discrete complement for every state admitted by the old clearance gate.
- Keep the already audited consistent boundary terms in density, central
  gradient, pressure acceleration and matrix action.
- Do not add a fitted boundary multiplier, third layer or density map.

### Pressure solve

For the advected normalized density, define

```text
b_i = rho_adv_i - 1
B(k) = -dt^2 D(P(k))
```

where `P` is the existing linear pressure-acceleration operator and `D` is the
existing density-rate operator. The unilateral pressure problem is

```text
k >= 0
w = B(k) - b >= 0
k_i * w_i = 0
```

The candidate uses a deterministic projected preconditioned conjugate-gradient
iteration. The existing DFSPH factor divided by `dt²` is the diagonal inverse
preconditioner. The active set contains rows with positive multiplier or
positive compression residual. Projection or an active-set change restarts
the conjugate direction. All reductions retain stable sample-ID order.

The local candidate ceiling is 50 iterations, the minimum remains two and the
mean positive compression threshold remains `100,000 ppb`. This is a new
algorithm/profile budget. It cannot earn W0B credit merely because the number
50 is below the rejected 160-sweep diagnostic; a PCG iteration performs more
work than one Jacobi sweep and must later pass the product cost gate.

### Geometric contact

After pressure convergence and before position integration, each outer-box
velocity component is projected onto

```text
(min + particle_radius - position) / dt
    <= velocity <=
(max - particle_radius - position) / dt.
```

This is a velocity-level unilateral constraint in the fixed substep schedule,
not a post-integration clamp. The integrated position is never repaired after
validation. The exact fluid impulse `mass * (v_after - v_before)` is reduced
in sample-ID/component order and must be included with the pressure-boundary
impulse in momentum accounting. The opposite impulse belongs to the static
boundary. W0E initially covers only the analytical outer box; internal
apertures remain unsupported and fail explicitly.

### Initial state and stabilization

The initializer remains the deterministic regular lattice with zero velocity.
This choice is now explicit and is accepted only if the stronger pressure
solve carries the cold transient without hidden continuation state. A
hydrostatic pre-solve, retained pressure, XSPH or authored settling output is
not part of this candidate.

The density-map paper reports XSPH in its experiments, but neither the paper
nor the pinned upstream default supplies one universal coefficient for this
profile; pinned SPlisHSPlasH initializes both XSPH coefficients to zero and
scene configuration selects non-pressure behavior. Adding an arbitrary
coefficient would introduce a calibration parameter before evidence requires
it. Stabilization therefore remains explicitly `NONE` for this discriminator.

## Required independent checks

The candidate may survive W0E only if all of these pass:

1. The already independent support-complete boundary generator and selected
   initial density rows remain exact.
2. A separately written pressure-operator calculator confirms diagonal,
   symmetry/sign and selected first-step residual/iteration observations.
3. A separately written analytical contact calculator exactly reproduces the
   projected velocity components and contact impulse for frozen face, edge,
   corner, separating and already-violating inputs.
4. The contact-only/Jacobi-20 negative control fails as recorded, while the
   Jacobi-160 diagnostic control passes 24 steps. Neither can select a profile.
5. `constraint-separated-support-pcg-v1` passes 24 hydro steps at PCG-50,
   then a longer bounded equilibrium soak, with the unchanged density
   threshold, no retry and zero radius penetration.
6. Pressure plus contact reaction closes the canonical momentum equation
   within the existing `1%` threshold.
7. The 97-frame free-fall control remains byte-identical and activates no
   contact or pressure multiplier.

All seven checks pass. The two causal controls also separate the failure:
contact plus Jacobi-20 still fails on step 2 at `205,138 ppb`, while contact
plus Jacobi-160 passes all 24 steps with zero penetration. The new architecture
is needed in both dimensions; contact alone and a larger Jacobi ceiling are
not selected alternatives.

## Stop rules

- If PCG-50 fails the 24-step gate, reject the solver candidate. Do not raise
  its ceiling before comparing a deterministic continuation-state design.
- If 24 steps pass but the longer soak fails or contact impulse grows without
  bound, do not call the cold lattice an equilibrium initializer. The next
  decision must choose a versioned pressure-continuation state or a genuinely
  generated equilibrium configuration.
- If pressure/contact pass but the independent operator or reaction check
  fails, reject the implementation even when positions look stable.
- If the local gates pass, record only
  `LOCAL_PROFILE_DISCRIMINATOR_SURVIVED / NO_CORPUS_CREDIT / NOT_SELECTED`.
  Freeze successor document/profile/scenario roots and run W1 before W2 or any
  public/runtime work.
- A direct density map is reconsidered only if a scenario demonstrates a
  boundary-density or geometry-fidelity failure that this separated profile
  cannot represent. XSPH is reconsidered only with a named physical viscosity
  target and calibration corpus.

`CONTINUUM-WATER-REF-P1` remains `NOT_RUN` throughout W0E.

## Required successor closure

W0F must turn the surviving local candidate into a rootable W1 profile before
the serial corpus can resume:

1. Define one geometry abstraction shared by density support and unilateral
   contact. W0E deliberately supports only an outer axis-aligned box; the
   `CW-ORIFICE-001` internal aperture must fail closed until internal solid and
   opening semantics have an independent calculator and scenarios.
2. Reclose capacity. A two-layer complement around the selected
   `4 × 2 × 1 m` product extent needs
   `84 × 44 × 24 - 80 × 40 × 20 = 24,704` static samples, already above the
   rejected profile's `16,384` boundary capacity. Choose and root either a
   higher admitted capacity or an equivalent implicit complement; do not
   inherit the old number silently.
3. Freeze the successor document, float, execution, corpus and scenario roots.
   The W0B roots remain immutable rejected-profile evidence.
4. Extend reaction ownership to dynamic rigid geometry and test the one-pass
   pressure/contact split. W0E's outer wall is static, so it cannot prove the
   W3 crate exchange or added-mass behavior.
5. Run the full W1 corpus and exact cross-target gate before performance work.
   A PCG iteration is more expensive than a Jacobi sweep; the 50k `4/6 ms`
   stop target remains completely unmeasured.

No W0E evidence requires XSPH, viscosity, a density map, warm pressure state,
settling output or retry. Those operations remain absent unless a later named
scenario falsifies the separated profile and supplies a calibration target.
