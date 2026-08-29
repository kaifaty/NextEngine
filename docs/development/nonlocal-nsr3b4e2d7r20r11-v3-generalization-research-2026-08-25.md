# NSR3-B4E2D7R20R11 v3 solver generalization research

Status: `RESEARCH COMPLETE / ONE-SHOT EXECUTION SELECTED`.

## Question

Does the complete R8 algorithm — natural-residual faces, single-pivot NNQP,
exact-dual Armijo relinearization and on-demand verified inverse — certify new
topologies that did not select any part of the method?

## Experimental firewall

R9 froze source hashes before operator observation. R10 froze exact problem
roots and confirmed all four are excited without running a solver. R11 now
calls the unchanged R8 case routine with the unchanged 32-step cap and all
unchanged arithmetic/error policies.

There is no tuning branch after observation. A rejected face, direction,
globalization or cap is the generalization result. Success requires the full
`2^-70` KKT tuple for all four cases, not merely dual ascent or reduced
residuals.

## Hypotheses

| hypothesis | discriminator |
|---|---|
| G1: R8 generalizes across the frozen v3 family | all four cases strictly certify within 32 accepted iterations |
| G2: the architecture is sound but incomplete | arithmetic and monotone globalization pass, but at least one case reaches the cap uncertified |
| G3: a structural branch is missing | any active-set, direction, inverse or globalization rejection |

Even G1 authorizes only solver-level generalization evidence. Production still
requires binary64/mixed-precision design, sparse/matrix-free implementation,
warm starts, workload scaling and runtime integration.
