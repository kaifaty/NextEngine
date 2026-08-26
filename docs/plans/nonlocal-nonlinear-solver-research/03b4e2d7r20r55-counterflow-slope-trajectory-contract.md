# NSR3-B4E2D7R20R55 counterflow slope-refinement trajectory contract

Status: `FROZEN / DEFAULT-OFF ONE-REFINEMENT TRAJECTORY AUTHORIZED`.

## Parent

- R54 implementation `32005478`, semantic `cce59025...389e`;
- exact R51 counterflow prefix: nine accepted iterations, 701 transitions,
  final case/step roots `7e2735e1...c5d3` / `c7a33746...1590`;
- direct tuple and legacy audit roots frozen by R53/R54;
- old direction root `f09a8af6...821e`, nominal slope
  `1.8493164062e-19`, cheap/refined errors `8.8529495396e-3` /
  `1.0931517281e-24`.

## Frozen mechanism

Add one default-null `ALR20SlopeErrorRefiner` in the research semismooth path.
When non-null, privately observe the last successful direct principal solution
during that NNQP call. Invoke the refiner only if:

1. natural face and NNQP are exact;
2. NNQP terminated and is KKT-certified with failure `NONE`;
3. the direct capture corresponds to final direction/support/error;
4. initial slope is not certified;
5. no line trial has yet executed.

The returned refinement must be exact, passed, finite, nonnegative and no wider
than the initial error. Recompute the slope bound in the identical loop/order,
preserve nominal slope and direction byte-for-byte, and proceed only if its
lower bound is positive. Include refinement provenance in the step root only
when attempted, preserving all default hashes.

The R55 callback is root-agnostic and permits one successful existing
verified-inverse audit total. Later calls fail closed before inverse work. Run
only immutable counterflow once. Preserve the exact first nine accepted steps
and original rejected-step pre-refinement observation.

## Classification

1. `COUNTERFLOW_SLOPE_TRAJECTORY_PARENT_REJECTED`;
2. `COUNTERFLOW_SLOPE_TRAJECTORY_APPARATUS_REJECTED`;
3. `COUNTERFLOW_SLOPE_REFINER_UNUSED`;
4. `COUNTERFLOW_SLOPE_REFINER_REJECTED`;
5. `COUNTERFLOW_SLOPE_LATER_BOUNDARY`;
6. `COUNTERFLOW_SLOPE_TRAJECTORY_CANDIDATE`.

R55 permits at most 66 new inverse columns, no new factorization, compensated
dot, solution correction, line-search formula/trial/cap, tolerance, event,
runtime or production change. State beyond the old boundary is private
trajectory evidence only. Regress R54/R53/R52/R51/R50 exactly.
