# NSR3-B4E2D7R20 phase-I feasibility research

Status: `RESEARCH COMPLETE / DISCRIMINATOR SELECTED`.

## Question exposed by the oracle

For every R20 materialized problem define the compact convex contact/trust set
`D` and density residual `r(s)=c+A*s`. The oracle attempts the projection QP

```text
minimize  0.5 * ||s-t||^2
subject to s in D and r(s) <= 0.
```

Exhaustion of cyclic Dykstra cannot distinguish a difficult feasible
intersection from an empty one. The v2 input preflight only proved that the
projected target is density-excited; it did not prove feasibility.

## Phase-I problem and certificate

Use the nonnegative hinge feasibility objective

```text
rho* = min_{s in D} 0.5 * ||max(c+A*s, 0)||^2.
```

The scalar identity

```text
0.5 * max(r,0)^2 = max_{lambda>=0} lambda*r - 0.5*lambda^2
```

and convex minimax over compact `D` give the dual lower bound

```text
L(lambda) = lambda^T*c
            - 0.5*||lambda||^2
            - sigma_D(-A^T*lambda),       lambda >= 0.
```

Therefore:

- a primal point whose outward-rounded all-row upper is nonpositive proves
  feasibility;
- any nonnegative `lambda` whose outward-rounded `L(lambda)` lower bound is
  strictly positive proves `rho*>0` and hence infeasibility;
- neither condition means `UNRESOLVED`; a small iterate change is never a
  certificate.

The support function over the intersection of the per-component box and
global Euclidean ball is evaluated by the same one-dimensional KKT multiplier
form used by the independently validated R48 support audit. The phase-I
generator and the final certificate arithmetic remain separate: binary64 may
generate a witness/multiplier, but binary128 recomputes the final sparse action,
transpose, support value and outward envelope from frozen binary64 inputs.

## Hypotheses

| ID | Hypothesis | Decisive result |
|---|---|---|
| P1 | both filled blind TRQPs are contact/trust infeasible | positive binary128 dual lower bound for both |
| P2 | they are feasible but cyclic Dykstra is too slow | binary128 forward-feasible witness and no valid positive dual lower bound |
| P3 | the materialized operator/certificate plumbing is inconsistent | weak-duality, dense-control, support-KKT or exact-root rejection |

The supported transfer case is included as a feasible positive control. The
four cycle-zero cases remain zero-work feasibility controls.

## Selected method

Use a bounded primal-dual phase-I generator over the frozen R64 sparse
operator. It alternates exact `D` projection with the nonnegative quadratic
dual proximal map, uses a conservative operator-norm bound and fixed
power-of-two checkpoints. It does not share candidate state or Dykstra
corrections. At every checkpoint, independently audit both certificate routes.

No geometric or numerical parameter may be changed after observing a blind
case. If the generator does not expose either certificate within its frozen
cap, retain `PHASE1_UNRESOLVED`; do not extend the prior oracle or execute the
candidate.

## Consequence for corpus design

Future corpus admission needs two distinct labels:

1. `excited`: `P_D(t)` has positive density rows;
2. `feasible`: the full density/contact/trust intersection has an independent
   certificate.

Only cases satisfying both may be candidate generalization trials. An
infeasible blind case remains valuable as a negative geometry/control fixture,
but cannot serve as a convergence holdout.
