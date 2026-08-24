# NSR3-B4E2D7R19R48 restoration-certificate research

Date: `2026-08-25`

Status: `RESEARCH COMPLETE / SPECIALIZED PRIMAL-DUAL CERTIFICATE SELECTED /
CONTRACT NEXT`.

## Question corrected by primary literature

R47 proves that the source-linearized R43 normal step is not compatible with
an ordinary filter-SQP transaction. The next action is not to append an
uncommitted correction to that old normal step. In the trust-region filter-SQP
algorithm, restoration produces a new filter-acceptable point at which the
*next* TRQP is compatible for a positive next radius. R43 is already the exact
filter-acceptable candidate. R48 must therefore ask:

> Does the freshly relinearized density system at the exact R43 moved point
> admit a contact-safe normal step with enough trust-region reserve for a
> later tangential step?

This is a rollback-only existence question. R48 neither applies that witness
nor commits R43 as a restoration output.

The distinction matters because compatibility is stronger than merely finding
some point inside the trust region: the normal step must satisfy the linearized
constraints and remain sufficiently far from the boundary to leave tangential
freedom. Fletcher et al. formalize this through their normal-step length
condition before calling a TRQP compatible. Their restoration phase seeks a
new point whose next TRQP has that property.

## Exact local convex problem

Let `y` be the immutable R43 projected trial, `c` its freshly rebuilt density
inequality vector, and

```text
A = SPACING * Jc(y)
```

the exact R42 stable-superset operator relinearized at `y`. For a dimensionless
next normal step `d`, define

```text
P(d) = 0.5 * ||max(c + A*d, 0)||_2^2.
```

The feasible geometry is the convex set

```text
C = {d : ||d||_2 <= 0.125, l_contact <= d <= u_contact}.
```

The next TRQP radius remains the inherited `0.25`; limiting the normal witness
to half of it is an outcome-independent compatibility reserve. The component
bounds are rebuilt at `y`: an already penetrating face permits only
non-worsening motion, while an interior component may move only up to the exact
first box-face crossing. Every interval contains zero.

The R43 total displacement has norm `1.0290544256239871e-6`. Even the triangle
bound with the complete `0.125` normal radius remains below the existing
source-anchored half-skin radius `0.3` in dimensionless units. The frozen R42
superset can therefore own every R48 operator evaluation without support
expansion.

## Specialized primal-dual form

For `lambda >= 0`, Fenchel duality gives the lower bound

```text
D(lambda) = dot(lambda, c)
          - 0.5 * ||lambda||_2^2
          - support_C(-A^T*lambda).
```

Weak duality alone is enough for both authoritative outcomes:

- a `d in C` whose independently bounded row upper limits all satisfy
  `c + A*d <= 0` is a primal compatibility witness;
- a `lambda >= 0` whose outward-rounded `D(lambda) > 0` proves that no zero-
  hinge point exists in `C`.

If neither closes under finite work, the only valid outcome is unresolved. A
small residual or a stalled projected mapping is not an infeasibility proof.

The support function is not approximated from below. For any multiplier
`eta >= 0`, the Lagrangian expression

```text
U(eta) = 0.5*eta*radius^2
       + sum_i max_{l_i<=z<=u_i}
           (q_i*z - 0.5*eta*z^2)
```

is an upper bound on `support_C(q)`. A monotone scalar solve minimizes it; the
reported value is rounded outward and therefore keeps `D` a lower bound. This
is the same separable structure exploited by continuous quadratic-knapsack
projection algorithms.

## Candidate generator

Select the Malitsky--Pock primal-dual algorithm with line search (PDAL), applied
to the saddle form of `P + indicator_C`:

```text
d update       projection_C(d - tau*A^T*lambda)
lambda update  max((lambda + sigma*(c + A*d_bar))/(1+sigma), 0)
```

It uses only JVP, VJP, a componentwise dual prox and the projection onto
`ball intersection box`. Its line search avoids pretending that a finite power
iteration is a rigorous upper bound on `||A||`. Each rejected line-search trial
changes only the dual prox and VJP; the expensive JVP of the extrapolated
primal point is reused.

The Euclidean projection onto `C` is a monotone one-multiplier problem:

```text
d_i(eta) = clamp(z_i/(1+eta), l_i, u_i).
```

If the box projection lies outside the ball, bracket and solve for the
nonnegative `eta` whose norm reaches the radius. A feasible inward endpoint and
direct KKT/bracket audit own the certificate; the generator is never trusted
merely because PDAL converged.

## Alternatives rejected for R48

1. **Continue Hager--Zhang hinge descent.** It is a strong primal candidate
   generator, but provides no independent lower bound or infeasibility
   certificate. Keep it as a later comparison, not R48 authority.
2. **Generic SCS/HSDE integration.** Homogeneous self-dual embedding can return
   primal/dual solutions and infeasibility certificates and has an indirect
   matrix-free path, but it introduces a general cone-program layer and linear
   system solve for a problem whose prox operators are already explicit. The
   SCS authors also position the first-order method at modest accuracy. Retain
   HSDE as a fallback if the specialized dual bound remains unresolved.
3. **Frobenius-bound fixed-step PDHG.** It is rigorous but can be extremely
   conservative. PDAL supplies a convergence-backed operator-norm-free line
   search without outcome-fitted steps.
4. **Diagonal preconditioning first.** Pock--Chambolle preconditioning is a
   credible later optimization, but changes the metric projection and obscures
   the first certificate. Establish the scalar-metric reference first.
5. **Treat stationary positive hinge as infeasible.** Restoration methods can
   converge to a stationary point of their violation measure that is not
   feasible. Only the positive dual lower bound may own an infeasibility route.

## R48 boundary

Freeze one scalar-metric PDAL reference with predeclared parameters and finite
work. Require exact R47 parent, R43 state/operator identity, source-anchored
superset coverage, contact/trust projection audits, primal rowwise forward
enclosures, outward dual support, dense feasible/infeasible controls, work and
rollback. The valid scientific routes are:

```text
RESTORATION_NEXT_TRQP_COMPATIBLE_CANDIDATE
RESTORATION_RADIUS_INFEASIBLE_CERTIFICATE
RESTORATION_CERTIFICATE_UNRESOLVED
```

None commits the witness, R43, a runtime filter, a trust update or a following
outer. A compatible candidate would permit a later complete restoration-exit
transaction study; an infeasibility certificate would apply only to the frozen
radius/contact/operator model.

## Primary sources

- Fletcher, Gould, Leyffer, Toint and Wächter,
  [Global Convergence of a Trust-Region SQP-Filter Algorithm for General Nonlinear Programming](https://www.numerical.rl.ac.uk/media/people/nick-gould/FletGoulLeyfToinWach02_siopt.pdf).
- Malitsky and Pock,
  [A First-Order Primal-Dual Algorithm with Linesearch](https://optimization-online.org/wp-content/uploads/2016/08/5609.pdf).
- O'Donoghue, Chu, Parikh and Boyd,
  [Conic Optimization via Operator Splitting and Homogeneous Self-Dual Embedding](https://web.stanford.edu/~boyd/papers/pdf/scs.pdf).
- Curtis, Nocedal and Wächter,
  [A Matrix-free Algorithm for Equality Constrained Optimization Problems with Rank-deficient Jacobians](https://optimization-online.org/2008/05/1984/).
- Pock and Chambolle,
  [Diagonal Preconditioning for First Order Primal-Dual Algorithms in Convex Optimization](https://doi.org/10.1109/ICCV.2011.6126441).
- Cominetti, Mascarenhas and Silva,
  [A Newton's Method for the Continuous Quadratic Knapsack Problem](https://optimization-online.org/2012/08/3571/).
