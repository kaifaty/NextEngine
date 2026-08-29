# NSR3-B4BK constrained-contact research -- 2026-08-21

Status: `COMPLETE / R0_EXECUTED_FAIL / FACE_SYMMETRY_REPAIR_SELECTED`

## Question

Can a static analytical wall and ghost pressure support share one stationary
incremental update, or is a new contact potential required before pressure
water can be validated?

## Source boundary

The [Nonlocal Unified paper](https://doi.org/10.1145/3799902.3811196)
minimizes an unconstrained position objective and reports ghost particles or
signed-distance fields for solid boundaries. It lists unified fluid-solid
collision handling as future work. Its
[pinned public solver](https://github.com/peridyno/peridyno/blob/1aa892bb296fe766d2f9249c881b8605af23a69b/src/Dynamics/Cuda/ParticleSystem/SIUnifiedFluid/SemiImplicitUnifiedFluidSolver.cu)
also performs the unconstrained update; a separate volume-boundary component
owns geometry.

The 2026
[semi-analytical boundary paper](https://peridynamics.com/publications/2026-Liu-SAM.pdf)
does put bulk and contact potentials in one minimization. It introduces
virtual boundary/contact particles, a finite-horizon contact potential,
adaptive contact parameters, reduced-order CCD and a line-search filter.
That is a valuable future formula lineage, but importing only its contact
term would not preserve the selected FCR2 identity.

[Incremental Potential Contact](https://ipc-sim.github.io/) likewise supports
the general architectural conclusion that collision should participate in
the incremental potential solve. Its barrier, feasibility line search and
surface CCD are designed for general deformable geometry; they are much more
than the static axis-box discriminator needed here.

For bound-constrained nonconvex optimization, projected-gradient stationarity
and an active/free trust-region subproblem are standard tools; see the
[matrix-free trust-region Newton work](https://optimization-online.org/2021/01/8231/).
This does not establish our physical signs or ledger, which are derived below.

## Constrained incremental problem

For owned displacement `delta=y-x` and predictor
`delta*=h(v+h g)`, retain the selected pressure objective exactly:

```text
F(delta) = M/(2h^2) ||delta-delta*||^2 + Phi(x+delta;q)
```

For a static box, add only exact centre-clearance bounds:

```text
l_i - x_i <= delta_i <= u_i - x_i.
```

This is a bound-constrained form of the same smooth objective, not a fitted
penalty. For each scalar degree of freedom, with total gradient
`g=dF/ddelta`, KKT requires:

```text
free:        g = 0
lower bound: g >= 0, lambda_minus =  g
upper bound: g <= 0, lambda_plus  = -g
lambda >= 0 and lambda * clearance = 0.
```

The contact force on the fluid is `g_active`: positive along a lower normal,
negative along an upper normal. Consequently:

```text
J_fluid_contact = h * sum(g_active)
J_wall_contact  = -J_fluid_contact.
```

Writing `g = g_inertia + g_pressure` and multiplying stationarity by `h`
gives the complete impulse identity:

```text
M(v_new-v*) = -h*g_pressure + J_fluid_contact.
```

Ghost support still owns density completion and its pairwise reaction;
the KKT multiplier owns only nonpenetration. The externally reported ledger
therefore remains:

```text
Delta P - M_total*h*g
  + J_wall_pressure + J_wall_contact = 0.
```

This formulation removes the B4B ambiguity: no intermediate state may enter
the wall and ask an infinitesimal pressure branch to produce a reaction that
a later sweep will replace.

## Alternatives rejected for the discriminator

1. **Accept the old floor trial.** It crosses the unilateral pressure active
   set and remains above the reaction limit; this would weaken two frozen
   gates.
2. **Ghost support only.** B2 already contains a ghost-only penetration
   counterexample. Density completion is not nonpenetration.
3. **Clamp the predictor and keep the old ledger.** This hides the projection
   impulse and cannot prove momentum conservation.
4. **Add a tuned contact penalty/barrier now.** SAM/IPC show viable general
   directions, but both change the objective, parameters and globalizer.
5. **Immediately rerun B4B.** A full trajectory would conflate KKT signs,
   active-set logic, finite-precision stopping and temporal accuracy.

## Selected minimal experiment

Freeze B4BK as a one-substep discriminator:

- replay the exact P1 initial state at fixed `48/96/192` steps per frame;
- preserve the current split results, including 96/192 failures;
- solve the box-constrained KKT problem from a feasible projected predictor;
- separately prove a detached P2 block retains exact pressure-inactive free
  flight and zero contact multiplier;
- publish per-axis active sets, multiplier signs, projected KKT residual,
  pressure/contact impulses, complementarity and complete ledger;
- do not continue either physical trajectory.

The static box makes projection and bounds exact. General triangle meshes,
moving solids, friction, IPC/SAM contact potentials and GPU kernels remain
outside this discriminator.

## Decision

Implement the separately frozen
[B4BK contract](../plans/nonlocal-nonlinear-solver-research/03b4bk-contact-kkt-discriminator-contract.md).
PASS may authorize a new B4B corpus identity with constrained contact. FAIL
preserves B4B and moves the branch to an explicit contact-potential formula
study; it cannot be repaired by changing the water coefficient or timestep.

## r0 execution correction

The constrained solver closes KKT, complementarity, objective, contact and
complete-ledger gates for all three P1 step sizes, and detached P2 remains
bit-exact. The r0 report nevertheless fails because the contract required no
x/z or upper-face multiplier.

That expectation confused "no lateral drift" with "no lateral wall
reaction". P1 fills the entire `4 x 4` horizontal cross-section: twelve
particles touch each x face and twelve touch each z face. Pressure can and
should activate equal-and-opposite lateral wall multipliers. The observed
net lateral contact impulses remain at roundoff (`<=1.1e-16 N s`).

Preserve r0 FAIL. A new r1 identity may replace only the erroneous face gate
with exact geometry counts and opposing-face impulse symmetry. It may not
change the solver, thresholds or any observed KKT result.
