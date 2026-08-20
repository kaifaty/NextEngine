# NSR3-B3D4 displacement-owned gradient research -- 2026-08-21

Status: `RESEARCH_COMPLETE / CONTRACT_REQUIRED / B3_RETRY_BLOCKED`

D3 shows two later floor regimes: coarse/mid trials reduce the reaction
residual by about four orders but need another correction; fine trials increase
it. All retain support topology and have no negative-curvature exit. Before
adding iteration or line search, the numerical first-order identity must be
reclosed.

## Identity

For the owned state,

```text
F(delta) = Phi(x+delta) + M/(2h^2)||delta-delta*||^2
g_owned  = grad Phi + M/h^2*(delta-delta*)
R        = M/h*(delta-delta*) + h*grad Phi
         = h*g_owned.
```

The inherited evaluator instead computes

```text
g_legacy = grad Phi + M/h^2*((y-y*)).
```

Although `y=x+delta` and `y*=x+delta*` algebraically, each materialization
rounds before the subtraction. The difference is multiplied by `M/h^2`, so it
can dominate a tiny stationarity residual as `h` shrinks. D1 removed this
cancellation from velocity/reaction but did not remove it from the gradient
that drives the trust step.

The Hessian is unchanged: the inertia contribution remains `M/h^2 I`.
Therefore a diagnostic can change only the gradient input to the already
verified trust step while retaining the same pressure gradient, HVP, radius
and topology.

## Hypotheses

1. If `h*sum(g_owned)` matches measured `R`, legacy does not, and one
   owned-gradient trust step reaches the existing reaction gate in all six D3
   failure states, select `OWNED_INERTIA_GRADIENT_CANDIDATE`.
2. If the identity is repaired and every new step strictly improves residual
   but some need more work, select `BOUNDED_OWNED_RESIDUAL_ITERATION_REQUIRED`.
3. If the repaired step still overshoots, test a separately frozen residual
   line search; do not infer it here.
4. If owned identity itself cannot be certified, stop and investigate local
   geometry/accumulation.

## Decision

Execute the frozen
[D4 contract](../plans/nonlocal-nonlinear-solver-research/03b3d4-owned-gradient-contract.md)
on the six exact D3 first-failure states. No trajectory continuation is
authorized by the diagnostic.
