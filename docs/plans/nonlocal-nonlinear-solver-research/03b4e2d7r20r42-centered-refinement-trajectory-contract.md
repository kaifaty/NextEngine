# NSR3-B4E2D7R20R42 centered-refinement trajectory contract

Status: `FROZEN / DEFAULT-OFF TARGET-BOUND TRAJECTORY AUTHORIZED`.

## Parent

- R41 implementation `e92a7b8c`, semantic
  `0ff6f9cc597166ca962e3fa3c97b29ed6350f62b22c6bb2768fa2a95daedf3cb`;
- left/z/generation/checkpoint roots `45237f7a...013b` /
  `4db7e3e6...1a6b` / `ee9fd91d...8e83` / `64160584...e81e`;
- depth-four root `3352aeb1...843f`, radius `0.1421399732`, signs
  `24/41/0`, minimum separation `1054.2439` and zero conflicts;
- R40/R39 semantics `a999b65a...f7e` / `83e1f738...1bac`.

## Frozen implementation

Add one default-null, thread-local verified-solution refiner at the existing
verified-inverse fallback. With no hook, every audit root, solution and control
must remain byte-exact. The target-bound R42 hook may return a replacement only
for the exact R41 matrix/inverse/RHS/solution roots.

The practical hook uses binary128 Dot2/upward arithmetic with explicit
underflow rejection. It constructs left defect, residual and `z` intervals,
requires `rho_left<1`, executes exactly four center recurrences, then
independently certifies the fixed-point residual and compensated `x+y` center.
Require all entries structurally produced, finite and no-underflow; require all
65 final intervals strictly signed. Hash the practical certificate and require
the frozen R41 depth-four numerical values and sign counts. Exact big integers
remain oracle-only and cannot enter the hook.

Replay the torsion R35 case once with the hook installed. Require exactly one
target match and at most one replacement. Report the consumed solution/error,
negative-set transition, subsequent NNQP trajectory, final case/step route and
all work counters. Always clear the hook before workspace release and regress
R41/R40/R39 with it null.

Routes in precedence:

1. `CENTERED_REFINEMENT_PARENT_REJECTED`;
2. `CENTERED_REFINEMENT_HOOK_REJECTED`;
3. `CENTERED_REFINEMENT_TARGET_CARDINALITY_REJECTED`;
4. `CENTERED_REFINEMENT_NOT_CONSUMED`;
5. `CENTERED_REFINEMENT_LATER_BOUNDARY`;
6. `CENTERED_REFINEMENT_TORSION_CANDIDATE` if torsion certifies.

R42 is target-bound and default-off. It reuses the existing inverse columns and
adds no factorization or principal solve beyond the unchanged torsion
trajectory. It changes no tolerance, cap, ratio, globalization, event,
counterflow or production state. It authorizes one local passive-vector
replacement only; any subsequent failure is preserved, not retried.
