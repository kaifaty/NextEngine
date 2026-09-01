# NCGP15 — Unified constrained surface/viscosity step discriminator

Status: `FROZEN_REVISION_1 / CPU_LONG_DOUBLE / IMPLEMENTATION_AUTHORIZED`

Date: `2026-09-01`

## Decision under test

NCGP14 independently established one bounded fact: the corrected density
operator and frictionless analytical box contact can support the exact
side/bottom-confined 128-particle state for two steps when pressure is an
explicit unilateral constraint rather than a finite compression penalty.
That result deliberately set the corrected viscosity and surface coefficients
to zero.

NCGP15 asks the next smallest question:

> Does one unified position-space minimization of inertia, corrected Nonlocal
> viscosity and corrected Nonlocal surface energy, subject to the same
> unilateral density and analytical box constraints, pass the term oracles,
> one-step correspondence and a fixed 16-step confined trajectory?

This is not an operator-split surface kick followed by the NCGP14 pressure
projector. The published Nonlocal method selects a single variational objective
precisely to avoid pressure/viscosity/surface splitting artefacts. NCGP15 keeps
that unified objective while replacing only the structurally inadequate finite
pressure penalty by an explicit multiplier constraint. A conventional
non-pressure-predictor/pressure-projection ordering remains useful prior art,
but is not allowed to masquerade as the corrected unified candidate.

Primary-source basis:

- [A Nonlocal Unified Variational Framework for Free Surface Flows](https://doi.org/10.1145/3799902.3811196)
  defines the coupled position-space variational model and reports operator
  splitting as the limitation the unified solve is intended to remove;
- [Divergence-Free SPH for Incompressible and Viscous Fluids](https://discovery.ucl.ac.uk/id/eprint/10056699/1/BK17.pdf)
  is a comparator for the conventional ordering in which non-pressure forces
  predict velocity before density/pressure correction. It does not override
  the selected Nonlocal objective.

## Frozen parent identity

The immutable parent result is NCGP14:

- implementation/review commit
  `d1cfe76c4d2a342e99dc0954ba7e322709af1784`;
- tree `bbdf9f83918bd21698396e069907b6c3aa198255`;
- contract file SHA-256
  `4573690e22e79c999b2cdcd609747d0cb061ee59df475dd9040450c6678fd880`;
- contract root
  `e6d9cdce3a67818024c68ad2da7f4d2405613b7b27953678530b4a415609779f`;
- source root
  `a3f980e56dff5ce0e599f74b2048092ebac4d4b2d875f8d1c2a4d918a40ccd16`;
- Release binary
  `f924209d34ce75ed679bda043156d0d4f79ec13f266ea7ce4bdd6cb5141cab14`;
- stdout
  `15a92dff1e927f4137c43a607a9b31e30200a76e1ebd49a99127543d7a98bd37`;
- result root
  `54c89f7a0bd4fd13920db325a2b401591cd8cfca3690fc694440d28b42f54221`;
- TIGHT-128 fixture root
  `6dbaddf563b825e28e37aee58606f25cf50b17479cc3260e919103be79f7c2b2`;
- TIGHT-128-CAP16384 lane root
  `8d13d0eefb7e27acafb0588c052ed252a1246ec8e3c5cd5ebe4c748b59796ba6`.

Before NCGP15 evidence is admitted, a fresh detached NCGP14 rebuild/run must
reproduce the parent semantic result and every physical observable. A
different build-path salt may change only the binary-transitive roots already
allowed by the NCGP14 evidence procedure. The NCGP15 executable also imports
the exact TIGHT fixture bytes and independently reconstructs its frozen
NCGP14 fixture root before any new solve.

## Frozen hypotheses

| ID | Causal hypothesis | Prediction | Falsifier |
| --- | --- | --- | --- |
| H15A | The corrected terms and explicit density pressure admit one stable unified constrained solve | term oracles, one-step masks and the 16-step full lane pass | a converged, apparatus-valid full lane violates a physical or KKT gate |
| H15B | The corrected surface term is the first failing physical component | pressure+viscosity passes; pressure+surface and full masks fail the same first surface/energy gate; zero-gamma removes it | pressure+surface passes or the failure remains with gamma zero |
| H15C | The corrected normal-viscosity term or its reference-graph convention is the first failing component | pressure+surface passes; pressure+viscosity and full masks fail the same viscosity/dissipation gate; zero-lambda removes it | analytic normal-pair decay and pressure+viscosity both pass |
| H15D | The formulation is viable but the frozen deterministic optimizer ceiling is insufficient | residuals remain finite and decreasing until a typed work cap, with no failed oracle/physical predicate | a converged solve violates KKT/physics, or a term oracle fails before work exhaustion |

The hypotheses are intentionally not exhaustive. A result contradicting all
four is `UNEXPECTED_RESULT_INCONCLUSIVE`; it does not authorize tolerance,
coefficient, solver-order or work-cap changes.

## Frozen arithmetic and material profile

All fixture inputs first round once through IEEE binary32 exactly as NCGP14.
Every subsequent candidate and independent-oracle calculation uses `long
double`; hash publication narrows finite scalar observables once to checked
IEEE binary64. Raw `long double` object bytes and padding are forbidden.

The integrated corrected profile is immutable:

```text
dt           = 1/240 s
spacing      = 0.05 m
horizon      = 0.15 m
surface r0   = spacing
mass         = 0.125 kg
rho0         = 1000 kg/m^3
kernel_scale = 7.985668078772472
beta         = 1226.25 J
lambda_v     = 1.4138231728735551e-5 kg m/s
mu_v         = 0 kg m/s
gamma        = 0.010664424039285813 m/(kg s^2)
gravity      = (0,0,-9.81) m/s^2
basin        = (0.2,0.2,0.6) m
ghost_layers = 3
maximum_dynamic_samples = 50000
maximum_neighbors       = 256
```

`beta` reuses the frozen numerical value formerly named `kappa`, but only as
the Powell--Hestenes--Rockafellar augmentation parameter. The quadratic
penalty energy `beta/2*max(c,0)^2` is not added separately. `lambda_v` is the
normal viscosity coefficient; pressure multipliers are written `pi_i` to
avoid a name collision. Ghosts participate only in density and its Jacobian.
They receive no inertia, viscosity or surface dynamics.

Every `P/PV/PS`, zero-coefficient and mutation route has its own input root
that binds the exact term mask or mutation. It may reuse the immutable primary
profile root only as a parent; it may not print the full-profile root while
silently changing a coefficient. Manufactured Phase-A routes also bind their
gravity mode and boundary mode. `UNBOUNDED_MANUFACTURED` means that no box
projection, plane test, contact reaction or basin gate executes; it is not an
implicit enlargement of the primary tank.

## Unified constrained problem

For one accepted state `(x,v)`, define the unconstrained inertial prediction

```text
y_star_i = x_i + dt*(v_i + dt*g).
```

The selected non-pressure objective is exactly the corrected FCR0 objective
without its finite pressure penalty:

```text
E_np(y;x,y_star) = m/(2*dt^2) * sum_i |y_i-y_star_i|^2
                 + Psi_dt(y;x)
                 + Upsilon(y).
```

Viscosity uses the accepted-state graph and normals frozen from `x` within
`horizon`, unique dynamic/dynamic pairs in ascending stable-ID order, and

```text
delta_ij = (y_i-y_j) - (x_i-x_j)
omega(r) = -dW/dr
n_ij     = (x_i-x_j)/|x_i-x_j|
P_n      = n_ij*n_ij^T
P_t      = I-P_n

Psi_dt = m/(rho0*dt) * sum_unique_pairs
  [mu_v*|P_t delta_ij|^2 + lambda_v/2*|P_n delta_ij|^2] * omega(r_ij).
```

The reference membership predicate is `0<r<=horizon`; a coincident distinct
dynamic pair is invalid rather than assigned an arbitrary normal. The pair is
owned by the smaller stable ID and accumulated in ascending `(owner,neighbor)`
order. `omega`, normals and projectors are frozen from the accepted `x` for
the whole step, while `delta_ij` is evaluated from the current trial `y`.

Surface pairs are rebuilt from the current candidate `y`, contain each
unordered dynamic/dynamic pair once for `0<r<3*r0`, are owned by the smaller
stable ID and are accumulated in ascending `(owner,neighbor)` order. A
coincident distinct pair is invalid rather than silently skipped. The exact
zero at `r=3*r0` is excluded from both the pair count and energy. The term uses

```text
c_s(q) = q^2-1                 for 0<=q<=1
       = 1-(q-2)^2             for 1<q<3
       = 0                     for q>=3

C(r) = r0*(q^3/3-q-2/3)                  for 0<=q<=1
     = r0*(q-(q-2)^3/3-8/3)              for 1<q<3
     = 0                                  for q>=3

Upsilon = 2*gamma*m^2 * sum_unique_pairs C(r).
```

The factor two is the exact unique-pair form of the FCR0 directed energy.
The stated constants make `C` continuous and zero at support; no per-run
energy offset is allowed.
The resulting endpoint gradient is
`2*gamma*m^2*c_s(r/r0)*normalized(y_i-y_j)` and the physical force is its
negative.

For every dynamic owner, corrected density includes self, dynamic neighbors
and fixed ghosts:

```text
rho_i(y) = sum_j m*W(|y_i-y_j|)
c_i(y)   = rho_i(y)/rho0 - 1 <= 0.
```

`kernel_scale` is applied inside the following exact parent kernel and nowhere
else. With `q=2*r/horizon` and
`alpha=kernel_scale*3/(2*pi*horizon^3)`:

```text
W(r) = alpha*(2/3-q^2+q^3/2)  for 0<=q<1
     = alpha*(2-q)^3/6         for 1<=q<=2
     = 0                       for q>2

dW/dr = alpha*(-2q+3q^2/2)*(2/horizon)  for 0<=q<1
      = -alpha*(2-q)^2/2*(2/horizon)     for 1<=q<=2
      = 0                                      for q>2.
```

Accordingly `omega(r)=-dW/dr`. The duplicate `kernel_scale*W` spelling is
forbidden in implementation. Density graph
membership is the inclusive raw-long-double predicate `r<=horizon`; dynamic
self is present exactly once, zero-distance distinct records are invalid, and
all accepted non-self contributions are accumulated in ascending stable-ID
order. The density Jacobian differentiates both the dynamic-owner and dynamic-
neighbor endpoints and treats every ghost endpoint as fixed.

The full step solves

```text
minimize_y E_np(y;x,y_star)
subject to c_i(y) <= 0 for every dynamic owner
           y in the exact NCGP14 particle-centre box.
```

For each enabled tank axis the box is exactly
`[0.5*spacing, basin_axis-0.5*spacing]`, evaluated from widened binary64
profile values in `long double`. `P_box` is the componentwise orthogonal clamp;
all tangential coordinates are retained. Manufactured unbounded lanes replace
this set by all of `R^(3N)` and execute no contact work.

Its KKT multiplier `pi_i>=0` has units joules and maps to physical pressure
`p_i=(rho0/mass)*pi_i` pascals. At a solution:

```text
0 in gradient(E_np) + J^T*pi + N_box(y)
c <= 0,  pi >= 0,  pi_i*c_i = 0.
```

This is the only primary candidate. The old finite penalty and a sequential
surface/viscosity kick followed by pressure projection are named comparators
and can never select PASS.

## Deterministic CPU reference optimizer

The candidate uses a box-projected PHR augmented-Lagrangian method. Every
simulation step cold-starts `pi=0`; pressure is published as an observable but
is not hidden persistent state or a cross-step warm start.

For outer iteration `l`, with fixed `beta`:

```text
L_beta(y;pi_l) = E_np(y)
  + sum_i (max(0,pi_l_i+beta*c_i(y))^2-pi_l_i^2)/(2*beta).
```

The inner minimization starts from the previous outer iterate; the first outer
iterate starts at `P_box(y_star)`. At every inner iteration:

```text
g = gradient(L_beta)
a = dt^2/mass
d = P_box(y-a*g) - y
alpha = 1
while L_beta(y+alpha*d) > L_beta(y) + 1e-4*alpha*dot(g,d):
    alpha = alpha/2
y = y + alpha*d
```

The graph is rebuilt for every accepted or line-search trial position and is
fixed for one complete objective/gradient evaluation. A line-search trial
may not reuse density or surface data from another position. The reference
viscosity graph remains frozen from `x` for the entire step.

The inner solve stops only when both projected corrections satisfy

```text
||d||_2 / max(spacing*sqrt(3*N),1e-30 m) <= 1e-9
max_abs_component(d)/spacing              <= 1e-8.
```

Then update every multiplier without short-circuit:

```text
pi_(l+1) = max(0,pi_l + beta*c(y)).
```

The projected-direction convergence test occurs before a line search and does
not increment `accepted_inner_iterations`. Otherwise `alpha=1` is trial zero;
at most `40` failed halvings and therefore at most `41` evaluated trial points
are allowed before one accepted update. The accepted point increments the
inner counter once. After the multiplier update, recompute the complete final
KKT payload using `pi_(l+1)`; success on the 64th outer update or the 8192nd
accepted inner iteration is admitted before the cap is classified. A cap
blocks only the next required operation.

The outer solve accepts only when all final KKT gates below pass. It has fixed
caps of `64` outer updates, `8192` accepted inner iterations total per step and
`40` line-search backtracks per accepted inner iteration. Requiring work beyond
a cap while all reached values remain finite is
`SOLVER_WORK_CEILING_INCONCLUSIVE`. Non-descent (`dot(g,d)>=0` when the
projected-direction gates have not passed), nonfinite, graph/capacity or
line-search exhaustion after trial 40 is apparatus failure, not physical
refutation.

Candidate code uses a canonical CSR implementation. The independent oracle is
separately written, enumerates all dynamic/dynamic and dynamic/ghost pairs
directly in ascending stable-ID order and does not call candidate graph,
energy, gradient, projection, optimizer or root helpers. It implements the
same frozen optimizer schedule so state correspondence is meaningful, and
also checks the analytical gradient with central finite differences on the
tiny corpus. The finite-difference displacement is exactly
`epsilon=spacing*2^-18` on one coordinate at a time; both displaced states
must retain the same density/surface membership and stay away from the box.
The candidate gradient versus the independent central difference must have
relative L2 error `<=1e-8`, with denominator `max(||g_fd||_2,1 N)`, and maximum
absolute component error `<=1e-7 N`. A membership change makes the control
invalid rather than silently selecting a one-sided derivative.

## Solver and physical gates

For final `y,pi`, recompute all values from scratch. Let

```text
g_kkt = gradient(E_np) + J^T*pi
d_kkt = P_box(y-(dt^2/mass)*g_kkt) - y.
```

The solver gate is:

- finite objective, gradient, multipliers, state and observables;
- exact `pi_i>=0`;
- maximum/RMS positive density strain `<=1e-3 / 2.5e-4`;
- RMS/max `d_kkt` magnitude `<=1 um / 5 um`;
- `max_i |pi_i*c_i| / max(1 J,max_i pi_i) <=1e-8`;
- multiplier fixed-point relative L2
  `||max(0,pi+beta*c)-pi||_2/max(||pi||_2,1 J) <=1e-8`;
- independent candidate/oracle density relative L2 `<=2e-12`;
- exact candidate/oracle active multiplier ID signature;
- exact box containment and no capacity overflow.

The accepted state is published only after all gates:

```text
x_new = y
v_new = (y-x)/dt.
```

The final box reaction is derived from the KKT stationarity vector, not by
summing line-search projection artefacts. On a lower-bound coordinate it is
`dt*max(g_kkt,0)`, on an upper-bound coordinate
`dt*min(g_kkt,0)`, and it is zero in the strict interior. Tangential reaction
is exactly zero. The momentum receipt separately reduces dynamic/dynamic
pressure, viscosity and surface contributions, dynamic/ghost pressure and the
box impulse; internal dynamic/dynamic terms must close to normalized `1e-12`.

Common trajectory gates retain NCGP14 limits:

- exact sample count, stable IDs and total mass;
- position RMSE/max from the initial state `<=2.5 mm / 5 mm`;
- velocity RMS/max `<=0.05 m/s / 0.10 m/s`;
- normalized momentum residual `<=1%`;
- positive complete mechanical-energy excess `<=1%`;
- one connected dynamic component and zero satellites;
- exact zero inset penetration, zero top contact and nonempty support on all
  four lateral faces plus the bottom.

Complete mechanical energy is kinetic plus gravity potential plus corrected
surface potential. Viscosity contributes dissipation and no stored energy.
Pressure and ideal frictionless contact contribute no positive internal energy.

For a trajectory beginning at accepted state `0`, define

```text
P_k        = sum_i m*v_i,k
M          = N*m
F_ghost,k  = -sum_i pi_i *
               gradient_dynamic[c_i]_terms_involving_fixed_ghosts
I_box,k    = sum_i final_KKT_box_impulse_i,k

R_P(k) = norm(P_k-P_0-sum_(s=1..k)[dt*(M*g+F_ghost,s)+I_box,s])
         / max(norm(P_0), k*dt*M*norm(g), spacing*M/dt)

E_k = sum_i m*|v_i,k|^2/2 + sum_i m*(-g dot x_i,k) + Upsilon(x_k)
R_E(k) = max(E_k-E_0,0)/max(abs(E_0),1 J).
```

The momentum gate is `max_k R_P(k)<=0.01`; the energy gate is
`max_k R_E(k)<=0.01`. Internal dynamic/dynamic pressure, viscosity and surface
forces are reduced separately and must each close to normalized `1e-12`; they
are not inserted as external corrections. `F_ghost` is the pressure force on
dynamic particles from only dynamic/fixed-ghost density pairs; equivalently it
is the opposite of the virtual force on those fixed ghosts. It uses the
accepted final multiplier. `I_box` uses the final KKT reaction defined above.
Every sum is evaluated in ascending stable-ID order with a fixed pairwise
reduction tree.

## Frozen corpus and execution order

### Phase A — term and representation controls

All manufactured cases use stable IDs in the listed order, `reference=x`, no
ghosts, zero gravity and `UNBOUNDED_MANUFACTURED`. Listed decimal coordinates
and velocities round once to binary32 and then widen, except for the explicitly
defined translation operation below. Density constraints remain enabled; each
manufactured case must remain under-dense and publish an exactly empty active
multiplier signature. The exact base fixtures are:

```text
normal pair:
  id0 x=(0.10,0.10,0.10), v=(-1,0,0)
  id1 x=(0.20,0.10,0.10), v=(+1,0,0)

tangential pair:
  id0 x=(0.10,0.10,0.10), v=(0,-1,0)
  id1 x=(0.20,0.10,0.10), v=(0,+1,0)

surface repulsive pair:
  id0 x=(0.0800,0.10,0.10), v=(0,0,0)
  id1 x=(0.1200,0.10,0.10), v=(0,0,0)

surface attractive pair:
  id0 x=(0.0575,0.10,0.10), v=(0,0,0)
  id1 x=(0.1425,0.10,0.10), v=(0,0,0)

combined tetrahedron:
  id0 x=(0.10,0.10,0.10), v=(-0.15,-0.15,-0.15)
  id1 x=(0.15,0.10,0.10), v=(+0.15,0,0)
  id2 x=(0.10,0.15,0.10), v=(0,+0.15,0)
  id3 x=(0.10,0.10,0.15), v=(0,0,+0.15)
```

The normal/tangential cases enable only their named viscosity coefficient.
The surface cases enable only `gamma`. The tetrahedron enables `lambda_v` and
`gamma` together; it is a term-composition/gradient control, not an active-
pressure witness. Active pressure coupling is tested only by the exact
confined Phase-B/C lanes. Every analytical oracle evaluates the actual widened
binary32 separation, not the decimal label or an idealized `0.1/0.8/1.7`
ratio.

1. Reproduce the retained FCR0/FCR1 kernel, pair-counting, derivative and
   momentum controls and the exact NCGP14 parent result.
2. `normal-viscosity-pair`: two interior particles at distance `0.1 m`,
   center-of-mass velocity zero and relative normal speed `2 m/s`, with
   gravity, pressure, surface and contact inactive. Require the analytical
   implicit decay
   `v_rel_new=v_rel_old/(1+2*lambda_v*omega*dt/rho0)` within relative `1e-10`.
3. `tangential-mu-zero-pair`: the same pair with tangential counterflow.
   Position/velocity change caused by viscosity is `<=1e-12` absolute.
4. `surface-repulsive-pair` at `0.8*r0` and
   `surface-attractive-pair` at `1.7*r0`, zero gravity/pressure/contact.
   Compare the converged scalar separation with a separately written
   bracketed one-dimensional long-double root to `<=5 um`; center of mass is
   invariant to `<=1e-12` normalized.
5. Repeat both surface pairs after translations `(0.75,0.75,0.75) m` and
   `(100000,100000,100000) m`. The base pair is first canonicalized through
   binary32, widened to `long double`, and only then translated; the translated
   control is not narrowed back to one binary32 position. This deliberately
   tests both-part relative arithmetic rather than binary32's large-origin ULP.
   Pair-distance and one-step displacement differ by at most `5 um`;
   candidate/permuted semantic roots remain exact.
6. `combined-tetrahedron`: retain the inactive density constraint and activate
   normal viscosity plus surface without contact. Candidate CSR and independent
   all-pairs state differ by at most `5 um`, the empty pressure signature is
   exact and the KKT gates pass.
7. Run 32 steps of the retained pressure-inactive surface tetrahedron at zero
   gravity. Corrected surface energy plus kinetic energy may not increase by
   more than relative `1e-8`; positive excess above `1e-10 J` fails. A separate
   inviscid reversible-energy control runs 16 forward steps, keeps the final
   positions, negates every final velocity and runs 16 more steps from that
   time-reversed state. Its complete-energy drift, normalized by the initial
   energy with a `1 J` floor, is
   `abs(E_reverse_final-E_initial)/max(abs(E_initial),1 J)` and must be `<=1%`;
   return-position error is diagnostic because the selected implicit
   integrator is dissipative. Every forward and reverse accepted state is
   separately rooted; a reverse failure cannot reuse the last forward energy.

Phase A mutations are mandatory: omitted kernel derivative factor `2/h`, half
normal-viscosity coefficient, wrong surface sign, omitted surface factor two,
current/reference viscosity-graph swap, finite pressure-penalty substitution,
malformed/nonfinite state, permutation identity loss and capacity overflow.
Each mutation has a typed expected rejection and changes the appropriate input,
work or result root. `gamma=0` and `lambda_v=0` are exact comparator masks, not
mutations; they must remove only their named term and change the state root on
the corresponding active control.

### Phase B — one-step confined mask discriminator

From the exact NCGP14 TIGHT-128 initial bytes, run independently in this order:

1. `P`: pressure/contact only (`lambda_v=0,gamma=0`);
2. `PV`: pressure/contact plus corrected normal viscosity (`gamma=0`);
3. `PS`: pressure/contact plus corrected surface (`lambda_v=mu_v=0`);
4. `PVS`: all corrected terms.

Each mask performs one complete candidate CSR step, one independently written
all-pairs step and physically permuted repetitions. State position RMSE/max
between candidate and oracle are `<=1 um / 5 um`; active multiplier IDs,
contact masks, accepted/rejected category, outer-update count, accepted-inner
count and line-search-backtrack sequence are exact. Candidate CSR and oracle
all-pairs pair-enumeration counters are expected to differ and are verified
against their own independently constructed work receipts; only the frozen cap
and reached optimizer schedule must agree. Every mask must pass its applicable
KKT, containment, density, momentum, energy and topology gates. The old NCGP14
pressure-only state difference is reported but does not gate: NCGP14 used a
different linearized pressure algorithm.

The first failing mask in `P,PV,PS,PVS` order selects the smallest causal term
set only if every preceding oracle/apparatus control passes. A work cap produces
H15D/`INCONCLUSIVE`, never a later physical mask result.

### Phase C — fixed short trajectory

Only after every Phase A/B gate passes, run exactly `16` sequential accepted
PVS steps from the immutable TIGHT-128 state. Candidate, independent all-pairs
oracle and physically permuted candidate all start from identical binary32
bytes. Every step cold-starts pressure multipliers and publishes all state,
pressure, contact, graph, objective, KKT, work and physical observables in
ascending stable-ID order.

The first rejected private trial ends the trajectory and preserves the last
accepted state. No retry, tolerance change, alternative start, warm start or
work-cap increase is allowed. The published correspondence and physical
metrics are maxima over all 16 accepted states.

## Work and transaction closure

`Ncgp15WorkV1` contains these ordered `u64` fields:

```text
fixture_records, ghost_records, graph_builds, graph_candidates,
accepted_pairs, density_pairs, viscosity_pairs, surface_pair_tests,
surface_active_pairs, objective_evaluations, gradient_evaluations,
jacobian_products, projected_directions, line_search_trials,
accepted_inner_iterations, outer_multiplier_updates, box_plane_tests,
contact_reaction_values, oracle_pair_tests, finite_difference_evaluations,
topology_edges, scalar_predicates, portable_fields, hash_derivations
```

Every operation increments at its execution site. Each top-level phase, mask,
step, mutation and skip has raw work, an independently constructed expected
work value, work root and result root. Expected work is derived from immutable
input sizes, reached-stage trace lengths and independently enumerated pair
counts; it may not copy candidate counters or accepted-pair booleans.
Receipt self-seals are unmetered; every semantic/data-root serializer and root
derivation is counted once by its owner. A typed not-run receipt has 24 zeroes.

All candidate/oracle/permuted state is private until its complete step gate and
work verifier pass. On any failure, restore positions, velocities, pressure
observables and all future-affecting scratch to the previous accepted state;
publish the rejected trial and work separately. A forced failure immediately
after the private step/observable seal and before commit must prove identical
prior roots and empty scratch. A failure discovered by the final work verifier
also rolls back before the next trial can begin.

No full vector, graph, Hessian or state transfer to CUDA exists in NCGP15.
Timing, wall-clock and allocation counts are diagnostic and excluded from all
physics/result roots.

## Root and report identity

The executable is tool-only and uses one new target:

```text
nonlocal-corrected-cpu-unified-constrained \
  --unified-constrained-surface-viscosity
```

The versioned JSON schema is `nextengine.nonlocal.ncgp15.result.v1`.
The source aggregate hashes, in path order, the target CMake file, the new
translation units, the exact NCGP14 wrapper/parent source leaves reused only
for fixture and regression identity, the corrected reference header/source and
the SHA implementation. Configure binds the clean source commit/tree, complete
contract SHA, compiler family/version, strict C++17 flags and generated compile
command. `/proc/self/exe` is read fail-closed.

New roots use length-prefixed domains and little-endian fields:

- `nextengine.nonlocal.ncgp15.profile.v1`: profile ID followed by all material,
  geometry, capacity, optimizer and tolerance fields above;
- `nextengine.nonlocal.ncgp15.input.v1`: phase/lane name, profile root, parent
  TIGHT fixture root, coordinate-representation tag, exact ordered base
  dynamic/ghost binary32 bytes, optional checked-binary64 translation vector,
  gravity/boundary modes and term mask; translated manufactured positions are
  reconstructed by widening the base bytes and adding the bound translation
  in `long double`, never by narrowing the translated result;
- `nextengine.nonlocal.ncgp15.state.v1`: input root, one-based step, ordered
  stable-ID reference/position/velocity binary64 records;
- `nextengine.nonlocal.ncgp15.pressure.v1`: state root and ordered
  `(stable_ID,pi,pressure_pa,active)` records;
- `nextengine.nonlocal.ncgp15.contact.v1`: state root and ordered final KKT box
  reaction records;
- `nextengine.nonlocal.ncgp15.term-oracle.v1`: case/variant, exact input root,
  analytical/bracketed/candidate observables and typed outcome;
- `nextengine.nonlocal.ncgp15.work.v1`: the 24 fields above;
- `nextengine.nonlocal.ncgp15.step.v1`: input/previous/state/pressure/contact
  roots, ordered graph/objective/KKT/physical observables, expected/actual work
  roots and typed outcome;
- `nextengine.nonlocal.ncgp15.trajectory.v1`: ordered accepted and rejected step
  roots, final accepted state root, maxima, expected/actual aggregate work and
  typed outcome;
- `nextengine.nonlocal.ncgp15.result.v1`: identity roots, exact NCGP14 replay,
  hypotheses, ordered Phase A/B/C receipts, first failure, selectors, total
  work and primary status.

Every vector root binds stable IDs. Every finite scalar hashes checked binary64
bits. The report publishes enough raw fields to independently reconstruct all
roots and work receipts; a digest without its preimage fields is invalid.
There are no CPU vector round-trips hidden behind a scalar result and no
unversioned success output.

Exit `0` means a valid scientific classification, exit `2` means
`APPARATUS_INCONCLUSIVE`, exit `3` is an executable-read or unexpected
fail-closed exception, and exit `64` is usage error. Stderr is empty on every
versioned JSON route.

## Classification precedence

1. `APPARATUS_INCONCLUSIVE`: identity, parent replay, formula/oracle,
   permutation, transaction, nonfinite, capacity, work or root closure fails.
2. `SOLVER_WORK_CEILING_INCONCLUSIVE`: an otherwise finite valid primary mask
   or trajectory step reaches the frozen optimizer cap before KKT closure.
3. `UNIFIED_CONSTRAINED_TERM_REFUTED`: a converged Phase-A term or Phase-B mask
   violates its frozen formula, KKT or physical gate. The first mask names the
   supported H15B/H15C boundary when applicable.
4. `UNIFIED_CONSTRAINED_SHORT_TRAJECTORY_REFUTED`: Phase A/B pass, but a
   converged apparatus-valid Phase-C trial violates a physical gate.
5. `UNIFIED_CONSTRAINED_SHORT_SUPPORTED`: all 16 PVS steps and every control
   pass.
6. `UNEXPECTED_RESULT_INCONCLUSIVE`: the exact observation matches none of the
   frozen causal routes.

No classification authorizes coefficient, tolerance, solver-order or cap
tuning. A work result may motivate a separately frozen solver discriminator;
a physical result may motivate only the first-specific term/formulation
research.

## Verification and claim ceiling

Before evidence is admitted:

1. run the detached NCGP14 replay;
2. perform two fresh strict Release builds/runs and require byte-identical
   normalized JSON;
3. run ASan+UBSan;
4. run retained FCR0--FCR2 and NCGP12--NCGP14 controls;
5. obtain one independent read-only review before reading the author conclusion;
6. allow at most one coherent apparatus repair and one re-review.

Even `UNIFIED_CONSTRAINED_SHORT_SUPPORTED` proves only a 128-particle,
16-step CPU long-double constrained-variational correctness corpus. It does
not prove macroscopic water calibration, a 4k/16k/50k trajectory, CUDA
correspondence, GPU performance, integrated frame time, runtime/PhysX/renderer
integration or R8 completion. Its sole successor authority is a separately
frozen CUDA correspondence port of the same Phase-A/B/C bytes and receipts,
followed—only after correspondence passes—by longer 4k/16k/50k correctness and
timing. CPU DFSPH remains the product fallback; SPEC-38 and ADR-076 remain
Proposed; public Rust/engine APIs and `docs/roadmap.md` do not change.
