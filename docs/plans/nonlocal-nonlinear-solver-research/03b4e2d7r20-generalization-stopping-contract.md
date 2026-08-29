# NSR3-B4E2D7R20 generalization/stopping contract -- revision 2

Status: `FROZEN / EXECUTION AUTHORIZED / REPORT ONLY`.

## Immutable parent

| Item | Value |
|---|---|
| manifest schema | `nextengine.nonlocal.nsr3b4e2d7r20_corpus_manifest.v2` |
| manifest semantic | `420031730236445859426960bc5540ae8b8f9522d4b9263fe5e86cf5f13f85b4` |
| manifest stdout | `cf9af006da05434233436458fca7eebbd6dc67831d41d64f5752a143d57543c7` |
| preflight semantic | `16a24ef3040411ef0bd29294f0887b29a1cce7690c7d8a8bee511ea58880b1f6` |
| preflight stdout | `a7afb864fc6d437af0dba59e76b50a819fc798adf7af24589a3dffc2340bf613` |
| implementation | `6cbf10ad` |
| cases | four transfer, two kinematic negative controls, two blind holdouts; stable listed order |
| TRQP | R64 dimensionless density rows, source contact box and global ball `Delta=0.0625` |
| target | `(semi_implicit_predictor - source_position) / SPACING` |

Revision 1 is withdrawn before any solver/oracle iteration because its input
preflight found both intended holdouts solver-trivial. Every v2 source/problem
root in the dated preflight evidence is exact. A mismatch rejects
before operator construction.

## Hypotheses

| ID | Frozen hypothesis | Decisive observation |
|---|---|---|
| H1 | composed-dual transactions generalize beyond the dam state | both blind holdouts reach the KKT tuple and oracle envelope |
| H2 | zero-step detection and active solving coexist without a fixture-specific mode | all quiet controls stop at outer zero while all excited certified cases solve |
| H3 | fixed 20 is unnecessary as a stop rule | at least two excited certified cases stop at different outer counts |
| H4 | dual/objective stagnation alone is safe | deliberately stalled control would have to satisfy KKT; otherwise H4 is refuted |

H1--H3 are not all-or-nothing algorithm truth. Failures must publish the first
case/regime and exact KKT component. H4 is expected to be refuted and exists as
a corruption control.

## Execution order

1. Recreate all sources and require v2 manifest identity.
2. Materialize `c`, sparse R64 `A`, target and contact/trust projection data;
   require exact preflight semantic and problem roots.
3. Preserve the input admission: both negative controls have zero positive
   projected rows; both blind holdouts have positive projected rows. Run
   structural/operator, row-scaling and positive-row-rescaling controls.
4. Run the binary128 offline Dykstra lane first. Stop the whole case on
   `ORACLE_UNRESOLVED`; never inspect/tune the candidate for that case.
5. Audit `s0=projection_D(t), lambda0=0`. If the complete KKT tuple passes,
   publish the exact outer-zero route with no solver transaction. Otherwise,
   from the immutable source run the unchanged R65 composed-dual policy with
   the R20 checkpoint stop and cap 32.
6. Run the frozen FISTA reference under separately reported structural work.
7. Publish every case, including failures; no best-of-case aggregation.

## Candidate stop

At an outer boundary, recompute fresh all-row action, transpose, composed dual
in direct and completed-square forms, contact projection and rollback roots.
Stop only if:

```text
q_primal          <= 2^-20
q_dual            <= 2^-20
q_complementarity <= 2^-20
q_stationarity    <= derived binary64 forward bound
dual monotonicity violation <= derived binary64 forward bound
outer             <= 32
```

Row scale is exactly

```text
sigma_i = max(|c_i|, Delta*sqrt(||a_i||^2), row_forward_bound_i).
```

Projected dual mapping is exactly

```text
G_i = ||a_i||^2 *
      (lambda_i - max(0, lambda_i + (c_i+a_i*s)/||a_i||^2)).
```

Zero/nonfinite diagonals reject. Rescaling any selected row by the positive
binary factors `{2^-8, 2^8}` must preserve scaled KKT decisions within analytic
roundoff bounds.

Small dual change without the full KKT tuple routes to
`GENERALIZATION_STAGNATION_UNRESOLVED`, not success.

## Oracle

- offline Linux x86-64 GCC `__float128`/libquadmath only;
- primal Dykstra set order: stable density halfspaces, contact box, trust ball;
- independent state/update order from the candidate;
- checkpoints `2^8, 2^10, ..., 2^18`, no tolerance-selected cycle;
- require finite primal/dual values, primal feasibility, projection
  stationarity, complementarity and a nonnegative certified primal-dual gap;
- binary128 KKT and gap normalized by the same analytic input scales must be
  `<=2^-70`;
- first certified checkpoint owns the reference; otherwise
  `ORACLE_UNRESOLVED`.

The oracle cannot certify infeasibility merely by exhaustion. Such a case is
research-unresolved and blocks corpus promotion.

## Accuracy and controls

- candidate scaled KKT tuple must pass;
- candidate objective lower/upper envelope relative to oracle must be
  `<=2^-20` of `E_scale`;
- candidate step difference from oracle must be `<=2^-20` after division by
  `max(Delta, ||s_oracle||_2, binary128_floor)`;
- direct/completed dual and physical/dimensionless model identities use
  gamma-style predeclared forward bounds;
- source, operator, case order, row order, checkpoint state and rollback roots
  are exact;
- controls: source bit, role swap, row scale, diagonal zero, lambda sign,
  contact face, trust radius, oracle precision profile, stalled iterate, outer
  cap, abort-before-commit and duplicate replay.

## Routes

Precedence:

```text
parent/source/operator/scaling/oracle/candidate/KKT/correspondence/work/
rollback/generalization
```

Terminal routes:

```text
GENERALIZATION_PARENT_REJECTED
GENERALIZATION_SOURCE_REJECTED
GENERALIZATION_OPERATOR_REJECTED
GENERALIZATION_SCALING_REJECTED
ORACLE_UNRESOLVED
GENERALIZATION_TRANSACTION_REJECTED
GENERALIZATION_STAGNATION_UNRESOLVED
GENERALIZATION_KKT_REJECTED
GENERALIZATION_CORRESPONDENCE_REJECTED
GENERALIZATION_WORK_REJECTED
GENERALIZATION_REGIME_FAILURE
GENERALIZATION_STOPPING_CANDIDATE
```

## Work and authority

No timing. Report exact sparse terms, projections, row visits, row updates,
`A/A^T`, oracle set projections and binary128 operations separately. Candidate
cap is 32 outers; oracle cap is `2^18` cycles. No extension is allowed after
observing a result.

This contract authorizes one rollback-only corpus execution. It grants no
nonlinear integration, GPU work, runtime state mutation, public schema,
performance claim or production authority.
