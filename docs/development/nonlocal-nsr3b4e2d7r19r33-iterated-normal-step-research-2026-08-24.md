# NSR3-B4E2D7R19R33 iterated all-inequality normal-step research

Date: `2026-08-24`

Status: `RESEARCH COMPLETE / CONTRACT FROZEN / IMPLEMENTATION NEXT`

## Question left by R32

R32 proves that one all-row Cauchy step is finite, line-exact and reduces the
violation norm by about `9.94%`. It does not show whether this cheap
first-order recurrence continues to make useful progress, stalls because of
conditioning, or needs generalized-Hessian curvature immediately.

The next experiment must answer that question before a more complex
semismooth/Newton-CG implementation or any nonlinear moved-state evaluation.

## Primary-method review

The objective

```text
phi(v) = 0.5 * ||[c + A v]+||^2
```

has the same squared-hinge structure studied for L2-loss linear SVM. Lin,
Weng and Keerthi show that this loss is continuously differentiable but not
twice differentiable, and use an active-set generalized Hessian with
matrix-free Hessian-vector products inside a trust-region Newton method:

- https://www.jmlr.org/papers/volume9/lin08b/lin08b.pdf

Ho and Lin make the same engineering point for large-scale L2-loss SVR: the
generalized Hessian need not be stored, and each inner iteration can use only
Hessian-vector products:

- https://jmlr.org/papers/volume13/ho12a/ho12a.pdf

That makes generalized-Hessian TRON a credible later candidate, not the
correct first discriminator. It adds a nested Krylov solve and substantially
more pair passes before we know whether the R32 recurrence already suffices.

Boyd and Vandenberghe give exact-line steepest descent as a standard convex
baseline and emphasize that its convergence rate exposes conditioning. Beck
and Vaisbourd independently show that projected first-order families can
converge on Euclidean-ball trust-region subproblems:

- https://stanford.edu/~boyd/cvxbook/bv_cvxbook.pdf
- https://doi.org/10.1137/16M1150281

R33 does not claim algorithmic equivalence to either source. They justify the
method-family ordering: run the cheapest deterministic projected first-order
pilot first; admit generalized-Hessian work only if that pilot saturates.

## Selected R33 discriminator

Keep the exact R32 objective, frozen support and global-L2 ball
`||v|| <= 0.25`. Starting at `v=0`, repeat at most eight times:

```text
r = c + A v
g = A^T [r]+
z = project_ball(v - g / ||g||, 0.25)
h = z - v
d = h / ||h||
q = A d
alpha = exact_piecewise_hinge_minimum([0, ||h||])
v = v + alpha d
r = r + alpha q
```

The projection makes the entire search chord feasible, including at the
trust boundary. At `v=0`, it reproduces R32 exactly: `z=-0.25*g/||g||`, so
the first direction and line interval are unchanged.

Eight iterations are fixed by the doubling ladder `1,2,4,8`. R32 already
supplies checkpoint one; R33 adds three predeclared checkpoints without
choosing a stop after seeing nominal progress. Each iteration costs one VJP
and one JVP. A fresh terminal JVP/VJP closes maintained-response and gradient
checks, so the new hard cap is 18 pair passes.

## Direct evidence and classification

Every accepted iteration must have a finite feasible chord, exact R32
piecewise line result, strict direct objective reduction and the inherited
line KKT tolerance `1e-10`. The first iteration must reproduce the exact R32
gradient, direction, response and line roots.

At termination, recompute `A v` and `A^T[c+A v]+` from the operator. The
maintained/direct response relative defect must be at most `1e-12`. Report the
projected-gradient mapping, objective, violation norm, active count, trust
use and checkpoint roots.

Hard-gate PASS has three report-only classifications:

- projected stationarity;
- no additional progress beyond the exact R32 first step;
- iterated all-inequality normal-step candidate.

The last classification requires at least two accepted iterations and a
terminal objective strictly below the exact R32 first-step objective. This is
a dominance test, not a fitted production tolerance.

## Independent controls

1. A conditioned two-axis diagonal problem needs two accepted exact-line
   steps before reaching zero residual.
2. A two-row scalar problem switches an inactive row into the active set and
   stops at the joint hinge minimum.
3. A one-row underpowered direction reaches the trust boundary and then
   satisfies projected stationarity with positive residual.
4. An initially feasible one-row problem is stationary without operator
   progress.

These controls are analytic dense problems and share no nominal pair data.

## Authority boundary

R33 is one private linearized normal-step classification. It cannot apply
`v`, move particles, update dual state, evaluate the nonlinear objective,
classify a nonlinear floor, alter penalty/trust policy, execute another outer
or claim timing, runtime or production authority.

If the first-order pilot saturates, research R34 generalized-Hessian
TRON/semismooth Newton with an explicit Krylov forcing and pair-pass cap. If
it continues to reduce the residual, first extend the bounded first-order
certificate before adding Newton complexity.
