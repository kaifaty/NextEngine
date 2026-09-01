# NCGP16 — Pressure-QP-preconditioned unified step discriminator

Status: `FROZEN_REVISION_1 / CPU_LONG_DOUBLE / IMPLEMENTATION_AUTHORIZED`

Date: `2026-09-01`

## Decision under test

NCGP15 Revision 5 admits every corrected term and mutation control, then stops
the unchanged physical `P` mask at the deterministic PHR outer-work ceiling.
Its final primal density and projected-position residuals are already small;
the dual complementarity/fixed-point gates remain open. Increasing that
observed cap or dropping those gates is not authorized.

The published Nonlocal method uses a semi-implicit successive-substitution
position update rather than the nested PHR optimizer. A bounded exact-fixture
probe also shows that copying the upstream finite-penalty SISSM unchanged does
not satisfy this task's density gates. NCGP14, however, independently proves
that the corrected linearized nonnegative-pressure QP closes the exact
TIGHT-128 density/contact state.

NCGP16 therefore asks one narrow question:

> Does a deterministic SQP/fixed-point map that evaluates the unchanged
> NCGP15 non-pressure objective and uses the independently reviewed NCGP14
> pressure/contact QP as its constraint projection close `P/PV/PS/PVS` and a
> 16-step confined trajectory without changing physical coefficients or
> gates?

This is not a one-shot force kick followed by pressure. The full private map
is iterated to a frozen fixed-point tolerance. A state may commit only after
the pressure QP, contact, fixed-point, candidate/oracle correspondence,
physical and work gates all pass.

Primary sources and bounded diagnosis:

- [A Nonlocal Unified Variational Framework for Free Surface Flows](https://doi.org/10.1145/3799902.3811196),
  Equation 26 and Algorithm 1;
- [official PeriDyno SIUnifiedFluid source](https://github.com/peridyno/peridyno/tree/5d5a081b1c30d09499c100ffaaf6745be632f546/src/Dynamics/Cuda/ParticleSystem/SIUnifiedFluid);
- `docs/development/nonlocal-gpu-unified-solver-diagnosis-2026-09-01.md`.

## Immutable lineage

NCGP14 pressure/contact parent:

- source/review commit
  `d1cfe76c4d2a342e99dc0954ba7e322709af1784`;
- tree `bbdf9f83918bd21698396e069907b6c3aa198255`;
- contract file SHA-256
  `4573690e22e79c999b2cdcd609747d0cb061ee59df475dd9040450c6678fd880`;
- contract root
  `e6d9cdce3a67818024c68ad2da7f4d2405613b7b27953678530b4a415609779f`;
- source root
  `a3f980e56dff5ce0e599f74b2048092ebac4d4b2d875f8d1c2a4d918a40ccd16`;
- binary/stdout/result
  `f924209d34ce75ed679bda043156d0d4f79ec13f266ea7ce4bdd6cb5141cab14` /
  `15a92dff1e927f4137c43a607a9b31e30200a76e1ebd49a99127543d7a98bd37` /
  `54c89f7a0bd4fd13920db325a2b401591cd8cfca3690fc694440d28b42f54221`;
- TIGHT fixture/lane roots
  `6dbaddf563b825e28e37aee58606f25cf50b17479cc3260e919103be79f7c2b2` /
  `8d13d0eefb7e27acafb0588c052ed252a1246ec8e3c5cd5ebe4c748b59796ba6`;
- first accepted TIGHT trial state/velocity roots
  `e8b9d51befb1444cf16021e12275585d0fca5e8a398bdce42c82f4a6b0920199` /
  `ca8d7ab81a427256f734821ddfddcfc4ec684718eff30890488b6b7b2495e41c`;
- first trial pressure rounds/sweeps:
  `5 / [6887,7663,6759,6521,6042]`.

NCGP15 admitted term/apparatus parent:

- source commit `ff0ea1fe0711016874fe4b62a2501d33223480d7`;
- tree `9527930aa28e8e6c42f7a0367506eeaa08a442fa`;
- contract SHA/root
  `3c1b486094828efb6d6887a7d75d50131de6e4e844426725916bf414f4454f5b`;
- source root
  `bd5382cbbabaf59e4c7f3e9d4f7397b1af4c59eaab55ab583cc1ddccb79f333a`;
- binary/stdout/result
  `6bf468537bcd91dc2f8bbf650452785d1d3c98fa351ba6aec780f8ae7ad66e3d` /
  `151394d8046b34b1bb8b435c290e657af8bce2be09165e52cf623c665864e440` /
  `ff9070f7bdd065fa9106633a7e104922f041f98120f2023228f75dfdbd30f926`.

NCGP15 is lineage for formulas, term controls and the honest PHR work result;
its `SOLVER_WORK_CEILING_INCONCLUSIVE` is not converted into PASS.

## Frozen profile, state and equations

Reuse byte-for-byte from NCGP15 Revision 5:

- profile ID `nonlocal-water-50k-v1`;
- `dt=1/240 s`, spacing `0.05 m`, horizon `0.15 m`, surface
  `r0=0.05 m`, mass `0.125 kg`, rest density `1000 kg/m^3`;
- corrected kernel scale `7.985668078772472` and derivative chain `2/h`;
- normal viscosity `lambda_v=1.4138231728735551e-5`, tangential viscosity
  `mu_v=0`, surface `gamma=0.010664424039285813`;
- gravity `(0,0,-9.81) m/s^2`, basin `(0.2,0.2,0.6) m`, three ghost layers,
  maximum 50,000 dynamic samples and 256 neighbors per owner;
- exact TIGHT-128 dynamic/ghost bytes, stable IDs, reference positions,
  binary32 widening and canonical serialization;
- corrected density, Jacobian, normal-viscosity/reference-graph and surface
  force/energy formulas;
- componentwise frictionless analytical-box projection and reaction signs.

The old PHR `beta`, line-search parameters and PHR work caps are identity-only
lineage fields and are not evaluated by NCGP16. They remain published so a
profile mutation cannot be hidden. NCGP16 does not evaluate a finite pressure
penalty or an augmented-Lagrangian multiplier update.

## Exact private SQP map

Let accepted state be `x`, predicted position be

```text
y_star = x.position + dt * (x.velocity + dt * gravity)
```

and `P_box` be the frozen NCGP14 componentwise analytical-box projection.
For one term mask, define the non-pressure objective

```text
E_np(y) = m/(2 dt^2) * ||y-y_star||^2 + E_viscosity(y;x) + E_surface(y)
```

with absent terms exactly zero. Its gradient is the exact NCGP15 corrected
gradient, excluding pressure and contact.

Initialize `y_0 = P_box(y_star)`. For outer map evaluation `k=0..7`:

1. evaluate candidate and independent all-pairs `E_np(y_k)` and gradient;
2. require finite values and the frozen candidate/oracle gradient gates;
3. form the private proposal

   ```text
   z_k = P_box(y_k - dt^2/m * gradient(E_np(y_k)))
   ```

   Since the inertia gradient is included, this is equivalently
   `P_box(y_star - dt^2/m*(g_viscosity+g_surface))`;
4. pass `z_k` to `PressureProject(z_k)` below;
5. call the returned state `y_{k+1}` and compare the ordered stable-ID vector
   with `y_k` using fixed pairwise reductions;
6. accept the map fixed point when

   ```text
   sqrt(sum_i ||y_{k+1,i}-y_{k,i}||^2 / N) / spacing <= 1e-9
   max_i ||y_{k+1,i}-y_{k,i}|| / spacing <= 1e-8.
   ```

Both comparisons execute without short circuit. A maximum of eight complete
map evaluations is frozen before the run. Exhaustion is
`SOLVER_WORK_CEILING_INCONCLUSIVE`, not physics.

### `PressureProject(z)`

This is the NCGP14 TIGHT-CAP16384 pressure/contact algorithm applied to a
private proposal rather than to a new external time step:

1. set `q_0=z`;
2. for pressure round `r=0..7`, assemble at `q_r` the corrected density
   `c=rho/rho0-1`, full dynamic+ghost derivative `J`, and

   ```text
   A = dt^2/m * J J^T,
   min_{lambda>=0} 0.5 lambda^T A lambda - c^T lambda;
   ```

3. solve from cold zero multipliers by the exact deterministic projected
   Gauss--Seidel row order, maximum `16384` sweeps per round;
4. require normalized primal `<=1e-8`, projected KKT `<=1e-8`, maximum
   complementarity `<=1e-10`, symmetry `<=2e-12` and all finite/capacity
   checks;
5. compute `delta_p = -dt^2/m * J^T lambda`, then
   `q_{r+1}=P_box(q_r+delta_p)`;
6. independently recompute density and require exact candidate/oracle active
   IDs plus relative L2 `<=2e-12`;
7. return at the first complete round with maximum positive strain `<=1e-3`,
   RMS positive strain `<=2.5e-4`, exact zero penetration and pressure balance
   `<=1e-8`.

QP sweep exhaustion is `QP_SWEEP_CAP_EXHAUSTED`; eight completed pressure
rounds without density closure is `PROJECTION_ROUND_CAP_EXHAUSTED`. Both map
to `SOLVER_WORK_CEILING_INCONCLUSIVE` and preserve the accepted state.

Every pressure round is independently assembled for candidate CSR and an
all-pairs oracle. Candidate/oracle must match in graph membership, density,
Jacobian action, QP category, active IDs, position, contact masks and all
published metrics before the private round can be used.

## Frozen masks and execution order

Run one accepted-state step in this exact order, stopping the suffix on the
first non-PASS route:

1. `P`: pressure/contact only;
2. `PV`: pressure/contact + corrected normal viscosity;
3. `PS`: pressure/contact + corrected surface;
4. `PVS`: pressure/contact + corrected normal viscosity + corrected surface.

Tangential viscosity remains exactly zero and is published as an inactive
profile field. Each mask runs canonical candidate, independent all-pairs
oracle and one stable-ID input permutation from the same immutable prior.

The first `P` outer evaluation must reproduce the NCGP14 first accepted TIGHT
trial exactly: five pressure rounds, the frozen sweep vector, dynamic state
and velocity roots after reserialization in the parent domains. The second
`P` map evaluation must return an exact zero position difference. This is a
lineage gate, not a performance expectation.

The pre-freeze feasibility counts (`P` two outer evaluations, other masks
three) are root-bound diagnostics, not PASS thresholds. Only the fixed caps
and gates decide execution.

## Short trajectory

Only if all four one-step masks PASS, run exactly 16 accepted `PVS` steps.
Each step starts from the last committed state, performs the complete private
SQP map and commits once. There is no multiplier, QP, graph or contact warm
start across steps or outer map evaluations.

Retain NCGP15 physical gates and definitions:

- one-step candidate/oracle position RMS `<=1e-6 m`, maximum `<=5e-6 m`;
- trajectory position RMS `<=2.5e-3 m`, maximum `<=5e-3 m`;
- velocity RMS `<=0.05 m/s`, maximum `<=0.1 m/s`;
- normalized momentum residual `<=1e-2`;
- positive energy excess `<=1e-2`;
- pressure/viscosity/surface internal-force closure `<=1e-12`;
- exact one connected dynamic component, zero satellites, exact containment,
  no top contact and nonempty side/bottom support witness.

Observables for a failing private trial are sealed separately. The accepted
state, velocity, accumulated impulse, trajectory counters and accepted root
remain at the preceding commit.

## Mandatory controls

All controls use production helpers where stated and publish typed
result/work roots.

1. **Lineage:** reconstruct NCGP14 profile/TIGHT roots and the exact first `P`
   pressure result, round vector, state and velocity roots.
2. **Non-pressure oracle:** retain every NCGP15 Revision-5 viscosity/surface
   analytical, finite-difference, translation, zero-coefficient and energy
   control.
3. **Pressure assembly:** candidate CSR versus separately written all-pairs
   density/J/A/QP on the first pressure round; include the NCGP14 sign,
   ghost-derivative, stale-assembly and QP-cap negative controls.
4. **Map fixed point:** production map on `P` must be exactly stationary on
   its second evaluation. A mutation omitting the final map evaluation must
   alter `PV`, `PS` and `PVS` result roots even if a physical scalar remains
   inside its broad gate.
5. **Term mutations:** omitted kernel `2/h`, half normal viscosity, wrong
   surface sign, missing surface factor two, current-position viscosity graph
   and finite pressure penalty retain the NCGP15 healthy fixtures and expected
   rejection semantics.
6. **Permutation:** canonical/CSR, canonical/all-pairs and permuted routes
   must have exact categories, stable-ID state, active IDs, contact masks,
   and pressure-round/outer counts after canonicalization. The canonical and
   permuted executions of the same implementation must also have exact work
   roots. Candidate CSR work roots are not compared with all-pairs-oracle work
   roots because their admitted operation schedules are intentionally distinct.
7. **Transaction:** inject failure after a complete private pressure round,
   after sealed non-pressure observables, after fixed-point PASS and after the
   final work verifier. Every route must preserve the exact prior accepted
   state and clear all private graph/J/A/lambda/gradient/contact buffers.
8. **Invalid input:** nonfinite record, duplicate stable ID, 257th neighbor,
   malformed binary32 fixture coordinate and nonfinite derived observable
   fail closed before a publishable result root.
9. **Work/root mutations:** mutate each solver work counter, one outer root,
   one pressure-round root, final pressure state, final term force and final
   transaction flag; every mutation must change the owning and final root.

An apparatus control failure stops all later physical work and emits typed
zero-work skips. No retry-to-green route exists.

## Work receipt

Use a versioned `Ncgp16WorkV1` with at least these exact counters:

- fixture/ghost records and canonicalized records;
- graph builds, graph candidates, accepted dynamic/ghost pairs;
- density and derivative pairs;
- Jacobian products, matrix products and symmetry reductions;
- QP sweeps, QP row updates and full gradient recomputations;
- pressure rounds and outer map evaluations;
- non-pressure objective/gradient evaluations;
- viscosity pairs, surface tests and active surface pairs;
- box plane tests, clamp hits and contact reaction values;
- fixed-point, correspondence, physical and route scalar predicates;
- independent all-pairs tests;
- portable fields serialized and semantic hash derivations.

Each operation is charged at its execution site. Expected work is reconstructed
independently from immutable input and the sealed reached-stage trace; it must
not copy actual counters. Receipt self-seals and the componentwise work
verifier are nonrecursive/unmetered. Candidate/oracle work is separate before
the enclosing comparison receipt. The final aggregate is the ordered
componentwise sum of identity, controls, four masks, trajectory and
finalization.

Any expected/actual mismatch is `APPARATUS_INCONCLUSIVE`, selectors false and
exit `2`.

## Root and report closure

All new roots use SHA-256 over length-prefixed ASCII domains and fixed-width
little-endian integers/IEEE-754 binary64 values. Long-double calculations are
narrowed once to binary64 for portable roots; raw long-double bytes and
padding are forbidden.

Required new domains:

```text
nextengine.nonlocal.ncgp16.identity.v1
nextengine.nonlocal.ncgp16.input.v1
nextengine.nonlocal.ncgp16.nonpressure.v1
nextengine.nonlocal.ncgp16.pressure-round.v1
nextengine.nonlocal.ncgp16.pressure-project.v1
nextengine.nonlocal.ncgp16.outer-map.v1
nextengine.nonlocal.ncgp16.step.v1
nextengine.nonlocal.ncgp16.trajectory.v1
nextengine.nonlocal.ncgp16.control.v1
nextengine.nonlocal.ncgp16.work.v1
nextengine.nonlocal.ncgp16.result.v1
```

The final JSON publishes:

- exact contract/source/binary/compiler/command identities;
- complete profile, fixture and mask input roots;
- every candidate/oracle pressure round including graph/density/J/A,
  multiplier, correction, contact, post-density, work and result roots;
- every outer input/non-pressure/proposal/pressure/fixed-point root;
- candidate/oracle/permuted state, pressure, force, contact and work roots;
- all physical and failing-trial observables;
- all control scalar/evidence arrays;
- ordered final work/result closure.

Successful stdout is one canonical JSON object and one trailing newline.
Identity/self-read failure exits `3` without evidence; every declared failure
after identity emits versioned JSON. Success/support, physical refutation and
work-ceiling scientific routes exit `0`; apparatus failure exits `2`; bad CLI
usage exits `64`.

## Classification

Precedence is strict:

1. identity, admission, control, correspondence, nonfinite, transaction,
   work or root failure -> `APPARATUS_INCONCLUSIVE`;
2. any QP, pressure-round or outer-map cap ->
   `SOLVER_WORK_CEILING_INCONCLUSIVE`;
3. converged `P` physical failure -> `PRESSURE_BLOCK_BASELINE_REFUTED`;
4. `P/PV` pass and `PS` first fails with the same full-term gate ->
   `SURFACE_COUPLING_REFUTED`;
5. `P/PS` pass and `PV` first fails with the same full-term gate ->
   `VISCOSITY_COUPLING_REFUTED`;
6. all one-step masks pass but the 16-step lane fails ->
   `UNIFIED_TRAJECTORY_REFUTED`;
7. all controls, masks and trajectory pass ->
   `PRESSURE_QP_UNIFIED_SUPPORTED`.

If more than one causal selector fits, set `cause_not_unique=true` and do not
claim a first cause. A work ceiling never selects a physical hypothesis.

## Build and evidence order

1. Freeze this contract in its own documentation commit.
2. Implement one CPU-long-double executable without editing the frozen NCGP14
   or NCGP15 parent sources.
3. Run strict ISO C++17 Release with `-O3 -DNDEBUG -Wall -Wextra -Wpedantic
   -Werror -ffp-contract=off -fno-fast-math`.
4. Build/run twice from clean directories and require byte-identical binaries
   and stdout after the documented build-path salt normalization.
5. Recompute every root/work receipt independently; run retained parent
   checks and ASan/UBSan where supported.
6. Permit one independent review and at most one frozen apparatus repair batch.

No CUDA build, sanitizer or performance run is authorized by this contract.

## Claim ceiling and fallback

The strongest possible claim is that one exact CPU-long-double 128-particle
pressure-QP-preconditioned unified corpus is bounded-supported. It does not
prove the paper's upstream finite-penalty implementation, production water,
CUDA correspondence, scalability, stability beyond 16 steps or any timing
target.

CPU DFSPH remains the product fallback. SPEC-38 and ADR-076 remain Proposed;
ADR-081 remains the active architecture authority. CUDA correctness is the
only successor after independent GO, followed by 4k, 16k and 50k correctness
before any performance claim.
