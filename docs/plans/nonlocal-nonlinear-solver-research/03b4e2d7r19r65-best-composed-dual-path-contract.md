# NSR3-B4E2D7R19R65 best composed-dual path contract

Date: `2026-08-25`

Status: `FROZEN / PRIVATE TRANSACTION IMPLEMENTATION AUTHORIZED`.

Parent: v9 `PASS / COMPOSED_DUAL_PATH_CANDIDATE`, semantic
`77cbed07c8cdc615110677cbae0266946b1fed777f2ba6ceba875a1ba4f583af`.

## Algorithm

For 16 outer blocks:

1. Run one ascending `omega=1` Hildreth identification sweep.
2. Run at most 15 Jacobi-PCG products on strict `lambda>0` rows.
3. Evaluate `lambda(alpha)=max(0,lambda+alpha*d)` for
   `alpha=2^-k`, `k=0..15`, in stable order.
4. For every strict fixed-density dual decrease, compute `A^T delta`, project
   `z=t-A^T lambda(alpha)` through the exact joint box-ball set and evaluate
   the v9 completed-square dual.
5. Retain candidates with strict positive cached-normal inertia reduction and
   strict positive composed-dual change over the no-PCG baseline. Select the
   largest dual change; exact equality retains the smaller exponent visited
   first.
6. Apply only that multiplier/primal candidate, freshly reconstruct, project
   and audit. The committed direct dual from `F+lambda^T raw` must match the
   selected completed-square prediction under `gamma(256*N+1024)`.
7. Refresh all rows and monotonically add all newly candidate-positive rows.

## Hard gates

- Exact v9 parent semantic and immutable R63/R64 source/operator roots.
- Nonnegative finite multipliers, positive PCG curvature and exact recurrence.
- Strict fixed-density dual decrease, positive cached-normal reduction and
  positive composed-dual change for every selected line.
- Candidate/commit dual, physical model, projection and all-row raw
  correspondence within predeclared bounds.
- Exact contact box, trust ball, stationarity, complementarity, ownership,
  reprojection, work and rollback gates at outers 1/2/4/8/16.
- Exactly 16 dyadic trials per outer and no fitted stop/tolerance/margin.
- No v8 all-candidate fresh-JVP filter audit in the transaction path.
  Completed-square selection adds zero sparse actions. The one inherited
  committed all-row audit per outer remains mandatory.
- Count baseline/candidate projections, completed-square vector terms,
  `A/A^T`, coordinate terms and sparse structural terms separately.
- Strict acceleration requires both final maximum raw and projected-gradient
  norm below the frozen FISTA values at fewer than 596,971,680 sparse terms.
- No CPU/wall timing on the shared host; no nonlinear trial or runtime state.

## Controls

1. Reuse all v9 scalar, active-box, sign and scaling controls.
2. A two-candidate dense control must select the larger positive dual change,
   not lower primal inertia.
3. Reject a candidate with positive dual ascent but nonpositive cached-normal
   reduction.
4. Reject equality at composed dual, fixed-density dual or normal gates.
5. Prediction/commit corruption, route precedence and rollback controls must
   fail closed.

The maximum result is an exploratory solver candidate on one linearized
fixture. Production, outer nonlinear acceptance, runtime integration and
performance claims remain unauthorized.

Rationale:
[best composed-dual path research](../../development/nonlocal-nsr3b4e2d7r19r65-best-composed-dual-path-research-2026-08-25.md).
