# NSR3-B4E2D7R3 globalization-policy research

Date: `2026-08-22`

Status: `RESEARCHED / CONTRACT_READY / REPLAY_ONLY`

## Exact failure mechanism

D7R2R proves that the first failed step is an interior Newton proposal:

```text
initial radius = 1.25e-2 m
step norm      = 2.4793640113756743e-10 m
radius / step  ~= 5.04e7
```

The rejected ratio shrinks the radius by `0.25`, but the next truncated-CG
solve returns the same unconstrained step until the thirteenth shrink. The
ninth-reject cap therefore terminates before the trust constraint can change
the proposal. This is a globalization-policy defect, not a formula, active-set
or horizon-topology defect.

## Method basis

The common trust-region framework makes an unsuccessful next radius depend on
`min(||s_k||, Delta_k)`, not only on the previous radius. Hsia, Zhu and Lin
show the interval explicitly and study step-size update pathologies; their
rejected-step rule also uses a quadratic interpolation estimate along the
computed direction ([PMLR paper](https://proceedings.mlr.press/v77/hsia17a.html),
[PDF equations 19--21](https://proceedings.mlr.press/v77/hsia17a/hsia17a.pdf)).

PETSc's current Newton trust-region interface documents the simpler
`Delta <- t1*Delta` update and separately exposes Newton, Cauchy and dogleg
fallbacks ([PETSc radius update](https://petsc.org/main/manualpages/SNES/SNESNewtonTRSetUpdateParameters/),
[PETSc Newton-TR](https://petsc.org/main/manualpages/SNES/SNESNEWTONTR/)). This
confirms that the current NextEngine rule is recognizable, but does not make
its scale-independent repetition appropriate for our microscopic interior
step.

A hybrid alternative is to reuse a rejected Newton direction in an inexact
line search rather than solve another linear system; a published hybrid trust
method proves global and superlinear convergence under its assumptions and
reports this exact design motivation
([DOI 10.1016/j.apnum.2011.03.002](https://doi.org/10.1016/j.apnum.2011.03.002)).

These sources motivate comparing policies. They do not prove which one is
correct for the Nonlocal AL objective.

## Frozen candidates

All lanes replay the same current state, gradient, Hessian and rejected full
step. None accepts or publishes state.

### A. Backtrack reuse

Evaluate `alpha=2^-e`, `e=0..6`, along the already computed direction. Select
the first moved row with positive predicted/direct reduction and direct ratio
`>=0.1`. This spends no new HVP but assumes the direction remains useful.

### B. Step-norm-aware trust recompute

From the failed full step compute the quadratic interpolation estimate using
the independently factorized actual reduction:

```text
alpha_hat = -gTs / (2 * ((f(x+s)-f(x)) - gTs))
```

Then apply the rejected-step rule:

```text
Delta_new = min(max(alpha_hat, 0.25) * ||s||, 0.5 * Delta_old).
```

Re-run the unchanged Steihaug solver once at `Delta_new`. This retains the
existing negative-curvature/boundary mechanism while making the radius respond
to the rejected step scale.

### C. Legacy first binding

Continue the unchanged quarter-radius sequence without accepting anything
until its first binding radius, then recompute once. This measures what merely
raising the reject cap would eventually test and exposes its repeated HVP and
objective work. It is a comparator, not a recommended repair.

## Selection

Prefer step-norm-aware trust only if its recomputed step is boundary-limited,
active/topology stable and admitted by positive model/direct reduction with
ratio `>=0.1`. It retains Steihaug's curvature safeguards and uses the standard
trust subproblem rather than assuming every future Newton direction is safe to
backtrack.

If that lane fails but backtrack reuse passes, select hybrid backtracking for
further research. If only the legacy first-bind lane passes, select a reject-
budget/radius initialization study; do not simply raise the cap. If none pass,
route to Cauchy/dogleg research.

The result authorizes only one policy implementation contract over the tiny
inner replay. Full AL convergence, commit stability and trajectories remain
separate later gates.
