# NCGP3 independent-review repair contract

| Field | Value |
| --- | --- |
| Research ID | `NCGP3` revision 5 |
| Status | `FROZEN / SINGLE_REPAIR_BATCH / REPORT_ONLY` |
| Parent | NCGP3 revisions 1–4 and independent verdict `NO-GO / INCONCLUSIVE` on candidate `1b638bfd` |
| Claim class | Unchanged: complete 4k correctness before any 50k or performance claim |

## Purpose and precedence

This revision freezes the only repair batch allowed after the initial NCGP3
review. It changes no FCR2 coefficient, physical tolerance, solver rule, HVP
ceiling, trajectory size or stop rule. It closes the reviewed apparatus gaps
before the single re-review.

Revision 4 accidentally broadened exact active-pressure identity from the
identical-state tiny/operator gate frozen by revision 1 to independently
evolved CPU/GPU trajectory states. This revision supersedes only that scope:

- CPU/GPU active-pressure IDs are exact for tiny and HVP evaluations that
  consume identical state bytes;
- corrected/permuted GPU active-pressure IDs are exact at every trajectory
  step because those routes consume identical semantic state;
- CPU/GPU active-ID differences after their accepted states have numerically
  diverged are reported with first step, ID and densities, but are not a
  trajectory rejection by themselves.

Position, density, momentum, energy and penetration trajectory tolerances are
unchanged. No active-set epsilon or hysteresis is introduced.

## Compensated multi-step state

After every accepted step, public position and velocity are rounded once to
binary32 as already frozen. The next predicted position must then be rebuilt
as a canonical `(hi, lo)` pair using exactly the upload-time fixed order:

1. `TwoSum(reference_hi, dt * velocity_hi)`;
2. canonical add of `dt * velocity_lo + dt^2 * gravity`;
3. store both `predicted_hi` and `predicted_lo` and validate the pair.

Ordinary f32 addition followed by `predicted_low = 0` is the mandatory
negative control. Prediction EFT component counts, canonical checks and roots
are sealed per accepted step.

## Complete pre-trajectory controls

The corrected FCR2 route must expose and seal these controls before 4k
trajectories:

1. 32-step analytic free fall for one interior sample;
2. 8-step translation and positive 90-degree rotation invariance for the
   frozen `4x4x4` interior block;
3. 32-step retained two-sample tangential viscosity control; with selected
   `mu=0`, kinetic energy is non-increasing within relative `1e-6` and the
   zero-decay expectation is explicit;
4. 32-step retained pressure-inactive four-sample surface relaxation with
   gravity zero and decreasing independent CPU mechanical energy;
5. corrected FCR2 HVP on identical binary32 state bytes: relative L2
   `<=1e-3`, cosine loss `<=1e-6`, exact active-pressure ID signature;
6. the retained raw/normalized profile, graph, boundary and executable
   rollback controls.

Every control has corrected/permuted identity where applicable and a sealed
input/work/result root. Failure stops before the 4k corpus.

## Trajectory observables

At every successful 4k step, outside the hot step boundary, the correctness
harness may download a diagnostic state snapshot. The hot `step()` call itself
must not allocate host vectors or transfer full position, velocity, density or
active-flag arrays. The diagnostic snapshot and its D2H work are sealed
separately and are excluded from future performance timing.

The trajectory report gates and seals:

- position and one-sided compression-density RMSE/max;
- exact particle count/mass and corrected/permuted state identity;
- normalized momentum residual using the frozen gravity, ghost-pressure and
  analytical-contact impulse definition;
- positive complete mechanical-energy excess and the reversible free-fall
  energy drift, each with denominator `max(abs(E0), 1 J)`;
- maximum inset-plane penetration and closed-basin bounds;
- HVP budget/used, outer/accepted/rejected trials, graph pairs, active counts,
  snapshot transfers, device allocation and timing fields (`NOT_RUN` where a
  prior correctness gate blocks timing).

## Independence, failure and report closure

The CPU solver owns a local input/profile validator and must not call the
shared GPU validator, formula, graph, scheduler or root helpers. Sharing the
immutable profile/state DTO bytes remains allowed.

The versioned JSON result contains contract/profile/input/source commit/tree,
source/binary/compiler/environment roots, allocated bytes, state/work/result
roots, failure-restored-state root, all observable metrics and the exact
command. `WorkBudgetExceeded` may be classified as `PHYSICS_REFUTED` only
after every preceding revision-5 control passes on the repaired predictor.

Two clean Release builds/runs, retained NCGP1/NCGP2 controls, CUDA
memcheck/initcheck/synccheck and one read-only re-review are required. Any
remaining load-bearing defect closes NCGP3 as `INCONCLUSIVE`. Performance
remains `NOT_RUN` unless the complete repaired correctness sequence passes.
