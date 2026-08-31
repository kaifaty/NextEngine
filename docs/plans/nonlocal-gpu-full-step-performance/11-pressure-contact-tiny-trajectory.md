# NCGP13 — Nonlocal pressure/contact composition and tiny trajectory

Status: `FROZEN_REVISION_2 / CPU_ONLY / PRE_IMPLEMENTATION`

Date: `2026-08-31`

## Decision under test

NCGP12 independently established two bounded facts on the corrected Nonlocal
density operator: a nonnegative pressure multiplier closes its derivative,
KKT, nonlinear-density and stationarity gates, and that pressure plus ghost
density support still crosses the exact basin inset by `0.178781 mm`.

NCGP13 asks the next smallest question:

> Does an explicit Nonlocal pressure projection, composed with a separate
> swept analytic box contact and relinearized after every accepted update,
> produce one admitted step and a stable one-second tiny hydrostatic hold?

This remains the Nonlocal model. It does not call the Rust DFSPH solver and it
does not restore the old finite `kappa` pressure penalty. Surface and viscosity
remain disabled so that a result cannot hide a pressure/contact defect behind
an unrelated force. No CUDA or timing participates.

## Frozen hypotheses

| ID | Hypothesis | Distinguishing prediction |
| --- | --- | --- |
| H13A | Explicit pressure plus separate analytic contact is a viable incompressible support step | the 512-sample step and 128-sample 240-step hold close density, contact, balance and topology gates |
| H13B | Pressure is viable, but the proposed alternating composition is insufficient | the one-step route exhausts its fixed projection/QP budget or leaves density/contact error |
| H13C | One step is admissible but the pressure/contact state is not dynamically stable | phase A passes and the tiny trajectory later violates a frozen physical gate |
| H13D | NCGP12 input/operator lineage was not preserved | the exact retained input, pressure-only witness, derivative or permutation identities fail |

No route in this stage distinguishes the old surface term as correct or
incorrect. Surface re-entry is a separately frozen successor only after H13A.

## Frozen profile and fixtures

Both phases use `nonlocal_water_corrected_profile()` with:

- `dt=1/240 s`, `spacing=0.05 m`, `horizon=0.15 m`;
- `mass=0.125 kg`, `rho0=1000 kg/m^3`;
- corrected `kernel_scale=7.985668078772472`;
- gravity `(0,0,-9.81) m/s^2`;
- retain corrected `kappa=1226.25` in the admitted profile and input root, but
  do not evaluate the penalty energy, gradient or HVP anywhere; the sealed
  penalty-work count is exactly zero;
- set `profile.lambda=mu=gamma=0`; here `profile.lambda` is the disabled
  normal-viscosity coefficient, not a pressure multiplier;
- exact particle-centre inset
  `[spacing/2, basin_extent-spacing/2]` on every axis;
- the same canonical binary32 dynamic and three-layer ghost bytes used by
  NCGP12, widened to `long double` only after input-root admission.

Phase A is the exact canonical `8 x 8 x 8 = 512` NCGP12 lattice. It runs one
step from zero velocity and must reproduce the retained NCGP12 input root
`82cf83cbfc5839026c8dff77c55b81be2e5597982c31493a7016a9f5fca2a8fa`.

Phase B is a canonical `4 x 4 x 8 = 128` lattice with the same generator,
stable IDs and zero initial velocity. It runs exactly `240` accepted steps.
Its frozen input root is
`f9dbf235d5e176efe29468eba26551a1958106a5eb9d6cffbcb74ee57b4b4d1e`.
The permuted route changes only input storage order; all owner work and output
serialization remain in ascending sample-ID order. The permuted bytes remain
physically permuted through admission and are copied into a separate canonical
working order only afterwards. Ghost positions are fixed.

## Frozen pressure/contact step

Let `x` and `v` be the accepted state at the start of a step. The free
predictor is

```text
d = dt v + dt^2 g
y = swept_contact(x, x + d)
```

`swept_contact(a,b)` is the frictionless Euclidean projection of endpoint `b`
onto the axis-aligned inset box. It tests all six stationary planes. For every
violated endpoint coordinate it computes the time of impact
`t_f=(plane-a_j)/(b_j-a_j)` as swept evidence, but clamps only that outward
normal coordinate; all nonviolated/tangential endpoint coordinates remain
exactly those of `b`. The earliest valid `t_f` is reported as `t_first`.

The contact mask contains every clamped face, not merely the earliest face:
bit `0=x-low`, `1=x-high`, `2=y-low`, `3=y-high`, `4=z-low`, `5=z-high`.
The separate first-hit mask contains all exact ties at `t_first`. A segment
inside the box and ending inside is unchanged. A nonfinite segment, starting
outside, an endpoint violation without a valid `t_f` in `[0,1]`, or mutually
contradictory low/high mask is an apparatus failure. Every call performs and
accounts for exactly six plane tests.
The contact displacement correction
`delta_c=swept_contact(a,b)-b`, masks, `t_first` and impulse
`mass*delta_c/dt` are accumulated in the work/result receipt. Because
`delta_c` is normal-only, analytic contact may remove normal kinetic energy but
must not manufacture frictional damping or positive kinetic energy.

After the predictor contact, perform at most `8` pressure/contact projection
rounds. Round `k` independently assembles corrected Nonlocal density and the
sparse dynamic Jacobian at the current admitted `y_k`:

```text
c_k = rho(y_k) / rho0 - 1
A_k = (dt^2 / mass) J_k J_k^T

minimize    0.5 lambda_k^T A_k lambda_k - c_k^T lambda_k
subject to  lambda_k >= 0

delta_p = -(dt^2 / mass) J_k^T lambda_k
y_(k+1) = swept_contact(y_k, y_k + delta_p)
```

Each QP starts from `lambda_k=0` and uses deterministic cyclic projected
coordinate descent in ascending sample-ID order, exactly as NCGP12:

- at most `4096` complete sweeps per projection round;
- complete-gradient update after each changed coordinate;
- `A_ii>0`, finite arithmetic and NCGP12 primal/projected-KKT/
  complementarity limits;
- no warm start, relaxation, active-set epsilon or tolerance fitting.

The graph is rebuilt from the current accepted `y_k` before each assembly and
is fixed during that assembly/QP. Dynamic and ghost support is inclusive
`r<=horizon`. Ghosts contribute to density and the owner derivative but have
no degree of freedom, inertia, viscosity or surface dynamics.

After every pressure/contact round, recompute exact nonlinear density and
componentwise inset admission through independent checkers. Stop at the first
completed round satisfying the frozen density/contact gates, but Phase A
executes at least two rounds so that post-contact relinearization is an
observed and sealed path. An already feasible second round may legitimately
return zero multipliers. Exhausting either budget rejects only this frozen
solver/profile/budget; it does not refute every possible Nonlocal
pressure/contact formulation and is not permission to change the budget.

On acceptance:

```text
x_new = y
v_new = (y - x) / dt
```

No persistent multiplier, fitted damping or hidden state crosses a step.
Reference position and ghosts are immutable. Failure is transactional: the
complete accepted `(x,v)` state and its root remain unchanged.

## Frozen arithmetic and independent checks

- All NCGP13 diagnostic state, density, Jacobian, QP and trajectory arithmetic
  is `long double` after binary32 input canonicalization.
- Kernel/density/Jacobian equations are implemented in the new NCGP13 source;
  it may reuse only the DTOs, immutable corrected profile/lattice/ghost fixture
  constructors and SHA-256. It locally implements admission, binary32 checks,
  input serialization, graph, density, Jacobian, matrix and QP and may not call
  `validate_nonlocal_input`, either canonicalizer directly,
  `build_reference_graph`, the NCGP12 candidate solve, Rust DFSPH or either
  CUDA/full-step evaluator. The immutable fixture constructors' internal one-
  time binary32 canonicalization is part of their frozen output and is allowed;
  NCGP13 performs no second canonicalization.
- The retained NCGP12 `Jv` check first runs at original `x`. The new round-path
  checks then run at predictor-contact `y_0` and every subsequent Phase-A
  assembled state. No centred-difference `Jv` is required in Phase B; its
  independent exact density and inset checks still run after every step.
- Both lineage and round-path directions first widen `SampleId` to signed
  `int64_t`, then evaluate `(id%17)-8`, `(id%13)-6`, `(id%11)-5`; unsigned
  modulo subtraction and wraparound are forbidden.
- Phase A repeats the NCGP12 stable-ID directional `Jv` centred-difference
  check with relative L2 `<=2e-7` and matrix symmetry `<=2e-12` at the
  predictor-contact state and every subsequent projected state that is
  assembled. Each round seals
  `assembly_input_root=state_root(y_k)` together with graph, density, Jacobian
  and matrix roots; a mismatch rejects the assembly before the QP.
- Every final accepted phase-A constraint is independently recomputed by a
  density-only direct all-pairs path that shares no Jacobian/QP helper.
- Every accepted Phase-B state is likewise checked by the independent
  density-only path and by a separate componentwise inset oracle that does not
  call the candidate contact helper.
- The algebraic step-balance residual includes gravity, every pressure
  correction and every analytic-contact impulse:

```text
R = (mass/dt^2)(x_new-x-d)
    + sum_k J_k^T lambda_k
    - (mass/dt^2) sum_k delta_c,k
```

  where the contact sum includes the predictor contact and every post-pressure
  contact. It is normalized by the maximum L2 norm of its three nonzero terms
  and `1e-30 N`. This is an implementation identity, not a substitute for the
  trajectory gates.
- Every pressure round separately closes

```text
||(mass/dt^2) delta_p + J_k^T lambda_k||_2 /
max(||J_k^T lambda_k||_2, mass*|g|*sqrt(N), 1e-30 N) <= 1e-8.
```

## Frozen gates

### Phase A — one 512-sample step

- retained NCGP12 input identity, derivative and symmetry checks pass;
- every QP reaches primal `<=1e-8`, projected KKT `<=1e-8` and
  complementarity `<=1e-10 J`;
- final maximum/RMS positive density strain are `<=1e-3 / <=2.5e-4`;
- exact inset penetration is zero on every axis;
- normalized algebraic balance residual is `<=1e-8`;
- at least one pressure multiplier and at least one lower-wall contact are
  positive over the complete step;
- bottom logical layer mean multiplier-equivalent pressure proxy is strictly
  greater than the top logical layer mean, with
  `p_proxy=(rho0/mass)*sum_k(lambda_k)` by stable ID. Because the Jacobian is
  relinearized, this is a pressure-impulse diagnostic, not a claim that the
  sum is one simultaneous physical pressure field;
- count, stable IDs and total mass are exact;
- candidate/permuted accepted state, velocity, pressure, contact, work and
  result roots are identical.

### Phase B — 128 samples for 240 accepted steps

Every step must pass the phase-A QP, density, independent-density,
componentwise-contact, balance, count, ID and mass gates. Define particle RMS
as `sqrt(sum_i ||value_i||^2/N)` and particle maximum as
`max_i ||value_i||_2`. Across the trajectory:

- maximum-over-steps position RMSE from the initial hydrostatic lattice
  `<=2.5 mm` and maximum-over-particles-and-steps displacement `<=5 mm`;
- maximum-over-steps velocity RMS `<=0.05 m/s` and particle speed
  `<=0.10 m/s`;
- with the lower geometric basin plane as the potential-energy zero,
  `E_n=sum_i(0.5*mass*||v_i||^2-mass*g dot x_i)`. The maximum positive
  no-explosion observable
  `max_n(max(E_n-E_0,0)) /
   max(abs(E_0),total_mass*|g|*spacing,1e-30 J)` is `<=1%`;
- define global three-vector momentum `P_n=sum_i mass*v_i`. Through step `N`,

```text
R_P = P_N - P_0 - sum_n(
        total_mass*g*dt
        - dt*sum_k sum_dynamic_dofs(J_nk^T lambda_nk)
        + sum_contacts mass*delta_c/dt)
```

  using ascending step/round/stable-ID accumulation. Normalize `||R_P||_2`
  by the maximum norm of accumulated momentum change, gravity impulse,
  pressure impulse, contact impulse and `1e-30 kg*m/s`; require `<=1%`;
- the inclusive `r<=horizon` dynamic graph has one connected component and no
  stable-ID satellite at every accepted step;
- at least one pressure multiplier and one lower-wall contact occur during the
  trajectory;
- candidate/permuted per-step state, work and final trajectory roots are exact.

The energy gate is deliberately only `NO_ENERGY_EXPLOSION`; unilateral
contact and projection may dissipate energy, so NCGP13 makes no reversible-
drift claim. These gates test a quiet pressure/contact hydrostatic hold. They do not test
surface relaxation, viscosity decay, free-fall invariance, dam-break, orifice,
4k/16k/50k capacity or any performance budget.

## Mandatory controls

Controls have their own work/result roots and never mutate the accepted
baseline state.

Unconditional apparatus controls run before physical classification:

1. **Retained NCGP12 pressure-only witness:** independently rebuild the exact
   one-round NCGP12 predictor/QP without analytic contact and reproduce its
   input root, legacy multiplier root
   `e3a62beac5ce4ebb2d39e9baf8ce8a28d95f43ab9291c11f415ba20cb6975b6a`,
   legacy trial root
   `00f5d0ff3bc77716e4a3da4a209bef8ab1382de6c317710058495e9ecd27b0c5`,
   `726` sweeps, `43,612` updates, `60` positive multipliers, scalar metrics
   within `2e-12` relative of Jacobian `2.8023154802122483e-11`, symmetry
   `2.943682118562611e-20`, primal `9.902735128437468e-9`, projected KKT
   `1.559010213940065e-11`, complementarity `5.949392072812528e-12 J`,
   linearized error `1.9226987759652108e-19`, maximum/RMS positive strain
   `3.574240623292345e-6 / 5.801039442456803e-7` and stationarity
   `2.0784084315481946e-20`, and positive
   `1.7878107032179183e-4 m` inset crossing. This owns the no-contact and H13D
   lineage claims without assuming the new eight-round route must penetrate.
   Here scalar correspondence divides absolute error by
   `max(abs(candidate),abs(retained),1e-30)`.
2. **Contact oracle:** manufactured cases cover interior no-op; all six single
   faces; starting on each plane moving outward (`t_first=0`) and inward;
   simultaneous low and high corners; exact tie masks; retained tangential
   endpoint motion; inward impulse signs; non-expansiveness; and starting-
   outside rejection. A separately written componentwise clamp/inset checker
   validates candidate output.
3. **Exact-radius graph witness:** this is an isolated graph-unit fixture, not
   Phase-A/B state. It uses the exactly binary32-representable control horizon
   `h_control=0.125 m` and two canonical dynamic positions with IDs
   `9000001/9000018` at `(0,0,0)` and `(0.125,0,0)`. Its separately rooted
   control profile/input produce exactly two more directed cross-pairs under
   `<=` than `<`; inclusive and strict graph roots differ. No physical-state
   difference is required because `W(h_control)=W'(h_control)=0`. This control
   must not substitute `0.125` into either baseline profile.
4. **Ghost derivative:** on the phase-A state, uniform downward `Jv` is checked
   against finite-difference density. Omitting ghost derivatives must exceed
   the normal `2e-7` relative gate; dynamic pair terms cancel under translation.
5. **Nonfinite input and duplicate ID:** each fails admission before density,
   QP or contact work and preserves the prior state root.
6. **Transactional injection:** a forced failure after a private
   pressure/contact round preserves the complete prior accepted `(x,v)` root.

If and only if Phase A admits, run these success-conditional controls:

7. **No pressure:** contact-only phase A violates either maximum or RMS
   positive-density gate.
8. **Negated pressure:** the wrong-sign update violates a density or inset
   gate.
9. **Stale relinearization:** deliberately present the round-zero assembly at
   a later contacted state. The `assembly_input_root != state_root(y_k)` check
   returns typed `STALE_ASSEMBLY_REJECTED` before matrix/QP work.
10. **Permutation identity loss:** serializing in input order rather than
    stable-ID order is rejected by the root comparator.
11. **Typed work/root mutation:** a pressure/position binary64 value changes by
    one `nextafter` ULP; a contact mask flips fixed bit `4`; a projection count
    increments by one within range. Each mutation changes its leaf and final
    result roots.

Controls skipped because an earlier physical phase rejected publish the sealed
status `NOT_RUN_BY_PRECEDENCE`; they do not convert an otherwise valid
`STEP_REFUTED` into `APPARATUS_INCONCLUSIVE`.

Execution precedence is exact: identity/admission and unconditional controls;
Phase A; success-conditional controls; Phase B. A reachable apparatus failure
stops immediately. A valid Phase-A physical rejection seals all later nodes as
`NOT_RUN_BY_PRECEDENCE`. Phase B starts only after admitted Phase A and all
success-conditional controls.

## Work and identity closure

The versioned JSON report publishes and the final result root binds:

- contract/source commit/source tree/source aggregate/binary/input roots and
  compiler flags;
- phase/route, complete profile and both fixture roots;
- for every projection and step: graph candidates/accepted pairs, density and
  derivative pairs, matrix products, QP sweeps/updates/gradient recomputations,
  `Jv`/independent-density work, six-plane tests/hits, contact projections,
  projection rounds and root/hash derivations;
- QP, density, penetration, balance, pressure-layer, position, velocity,
  energy-no-explosion, momentum and topology observables;
- canonical graph, multiplier, contact-mask/impulse, state, velocity, per-step
  work/result and trajectory roots;
- every control outcome and its work/result root.

All new vector roots bind `SampleId` beside each value; they never infer
identity from storage order. The retained NCGP12 input/multiplier/trial roots
repeat the exact legacy serialization only for the explicit lineage control.
Every new root uses an unambiguous length-prefixed domain and payload. Baseline
and controls have separate receipts; mutation-root derivations never increment
baseline work.

Binary identity is read fail-closed from `/proc/self/exe`. Values entering a
portable root are finite and explicitly narrowed to IEEE binary64, checked
again for finiteness after narrowing, then hashed as little-endian bits; raw
`long double` bytes are forbidden. Final evidence is built only from a clean
committed snapshot and fresh CMake configure. Two clean Release builds/runs
must be byte-identical before review.

## Routes and claim ceiling

- `PRESSURE_CONTACT_TINY_SUPPORTED` — both phases and every control pass. This
  authorizes only a separately frozen surface/viscosity reintroduction on the
  tiny CPU trajectory.
- `PRESSURE_CONTACT_STEP_REFUTED` — unconditional apparatus controls are valid
  but phase A exhausts its fixed work budget or violates a physical gate;
  success-conditional controls and phase B are `NOT_RUN_BY_PRECEDENCE`.
- `PRESSURE_CONTACT_TRAJECTORY_REFUTED` — phase A passes but phase B violates a
  physical gate; later steps and all larger/GPU work are `NOT_RUN`.
- `APPARATUS_INCONCLUSIVE` — an executable identity/admission/oracle/
  derivative/transaction/work/root check fails. It outranks a physical route
  only for checks reachable under the frozen precedence above.

Even `PRESSURE_CONTACT_TINY_SUPPORTED` is not correct-water, GPU feasibility,
game performance or R8 completion. CPU DFSPH remains the product fallback;
SPEC-38 and ADR-076 remain Proposed. No public Rust/engine API, renderer,
PhysX, runtime integration or roadmap status changes in this stage.

One independent read-only review follows the exact numerical result. One
batched repair and one re-review are allowed for apparatus defects; a remaining
load-bearing defect closes the stage `INCONCLUSIVE`.

## Revision 2 correction

Revision 1 was drafted but not committed, built or executed. Three independent
pre-code audits found that it could not reproduce the retained NCGP12 input
root with `kappa=0`, its full-segment stop introduced hidden sticking, and its
baseline had no exact-radius pair for the strict-radius control. They also
found ambiguous route precedence, trajectory norms/momentum, weak stale-
assembly detection and insufficient independent contact/trajectory checks.
Revision 2 applies the complete batch above before the first implementation or
numerical result. No observed NCGP13 value informed these changes.
