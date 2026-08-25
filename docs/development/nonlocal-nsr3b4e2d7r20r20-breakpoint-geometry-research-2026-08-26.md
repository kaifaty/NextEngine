# NSR3-B4E2D7R20R20 projector-breakpoint geometry research

Status: `RESEARCH COMPLETE / FIXED SUBDIVISION AUDIT SELECTED`.

## Question

Does each late shear line collapse track one simple projector-mask breakpoint,
or do the dyadic endpoints hide a same-face Armijo layer or multiple projector
events?

## Hypotheses

| ID | hypothesis | prediction |
|---|---|---|
| B1 | one simple mask event is the local frontier | every one of the seven R19 coincidence brackets contains one mask transition and one Armijo sign transition, with no rejected stable sample |
| B2 | the same-face Newton model still needs globalization | at least one sampled point already has the current mask but retains a negative certified Armijo margin |
| B3 | endpoint mask comparison hides a piecewise path | a bracket contains multiple mask transitions, a return to an earlier mask, or more than one changing scalar |
| B4 | mask and Armijo frontiers are separated | both transitions are simple but occur in distinct subdivision cells |

## Selected discriminator

Replay the exact R19 candidate once. For each of its seven frontier-coincidence
steps, reconstruct the affine multiplier path from the pre-step multiplier and
the accepted update. The frozen bracket is

```text
[alpha_accept, 2 alpha_accept].
```

Evaluate its 65 exactly representable uniform points
`alpha_k=alpha_accept*(1+k/64)`, including both existing endpoints. At every
point evaluate the unchanged joint projector, dual objective and rigorous R19
Armijo margin. Record the projector-mask root, changed scalar indices, ball
activity and sign-certified margin.

The existing endpoint roots and decisions must reproduce exactly. Classify in
this priority:

1. any mask-stable negative-margin sample: `SAME_FACE_ARMIJO_LAYER`;
2. multiple mask transitions, mask return or more than one changing scalar in
   any bracket: `MULTI_EVENT_BREAKPOINT_PATH`;
3. exactly one mask and one Armijo transition in the same subdivision cell for
   every bracket: `SINGLE_BREAKPOINT_ALIGNED`;
4. otherwise: `SIMPLE_BREAKPOINT_OFFSET` or `BREAKPOINT_GEOMETRY_UNRESOLVED`.

This is a report-only geometry audit. It may select the design of a separate
breakpoint-aware candidate, but it cannot add a line trial to the solver, move
state to a breakpoint, increase the cap, claim convergence or authorize
runtime/GPU/production work.

