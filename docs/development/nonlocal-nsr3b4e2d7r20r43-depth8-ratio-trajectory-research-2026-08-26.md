# NSR3-B4E2D7R20R43 depth-eight ratio trajectory research

Status: `RESEARCH COMPLETE / PREDECLARED ARITHMETIC-FLOOR DEPTH SELECTED`.

## Question

Does the already certified R41 depth-eight center resolve R42's boundary-ratio
ordering and expose the next unchanged torsion trajectory boundary?

## Selection rationale

Depth eight was frozen and exact-oracle checked before R42. It is the first
checkpoint at the arithmetic input-bound floor: depth 16/32 do not materially
improve the radius. Selecting it therefore does not fit a new iteration count
to the two observed ratios.

R42's input-bound formula shows why depth eight is a sharp discriminator. The
candidate term falls from `0.14214` to about `4.6e-14`, leaving the existing
current error `1.71568e-4` as the dominant uncertainty. The expected ratio
bound is then around `2e-22`, nearly three orders below the nominal row-61/
row-13 gap.

## Hypotheses

| ID | hypothesis | discriminator |
|---|---|---|
| Q1 | candidate error alone caused R42 ambiguity | depth eight resolves the ratio and NNQP performs another transition |
| Q2 | current-direction error still blocks ordering | ratio remains ambiguous with a current-error-dominated budget |
| Q3 | a later independent boundary exists | ratio resolves, then another exact solver route stops torsion |
| Q4 | integration diverges from R41 | practical depth-eight roots/values/signs reject before replacement |

R43 remains target-bound and default-off. Even Q1 plus torsion certification
does not authorize generalization or a production default; the separate
counterflow certificate boundary remains untouched.
