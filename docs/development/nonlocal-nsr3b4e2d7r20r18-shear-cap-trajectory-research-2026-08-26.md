# NSR3-B4E2D7R20R18 shear cap-trajectory research

Status: `RESEARCH COMPLETE / TRAJECTORY AUDIT SELECTED`.

## Observation

R17's shear candidate has no active-set, direction or globalization rejection.
It consumes one certified dual ratio refinement, accepts all 32 outer steps and
stops only at the frozen cap with primal/dual mapping `1.745e-7`.

## Competing hypotheses

| ID | hypothesis | prediction |
|---|---|---|
| T1 | the solver retains useful contraction and only lacks budget | late residual ratios remain bounded materially below one, with stable faces/masks and mostly full steps |
| T2 | it reaches a numerical/model plateau | late ratios approach one while masks and accepted powers stop changing |
| T3 | projector active-set chatter forces linearized re-solving | late steps keep changing many projector masks or natural faces |
| T4 | Armijo globalization, not the local solve, throttles progress | late accepted line powers remain positive despite stable masks/faces |

## Selected discriminator

Replay only the now-public shear candidate with the exact R17 method and cap.
Require its complete case root. Report every accepted step's KKT tuple,
natural/passive support, active-set work, line power, mask changes, ball change,
dual increase and consecutive primal/dual-mapping ratios.

No extended-cap solve or parameter change is allowed. The late trajectory will
select the smallest next counterfactual: bounded cap extension for T1, local
precision/model audit for T2, active-set stabilization for T3, or globalization
research for T4.

Before replay, the late-window labels are fixed: every ratio `<=0.9` means
useful contraction; every ratio `>=0.99` means plateau; any mask or natural-face
change means chatter; at least six positive line powers means globalization
throttling. Route priority is chatter, globalization, plateau, contraction,
then unresolved.
