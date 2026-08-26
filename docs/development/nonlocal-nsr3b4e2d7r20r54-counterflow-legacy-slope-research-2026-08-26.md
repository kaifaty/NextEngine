# NSR3-B4E2D7R20R54 counterflow legacy-slope research

Status: `RESEARCH COMPLETE / MINIMAL VERIFIER-PLACEMENT AUDIT SELECTED`.

## Question

Can the existing verified-inverse certificate close counterflow's global slope
without moving the direct solution or invoking compensated centered refinement?

## Evidence motivating the discriminator

R53 explicitly invoked the dormant verifier on the final 66-row direct factor.
It passed with `rho=2.72e-25` and error `1.09e-24`, whereas the ordinary cheap
error is `8.85e-3`. Because that legacy error is centered on the unchanged
direct solution, it may be substituted into the existing slope enclosure
without changing the nominal direction.

## Bounded discriminator

Replay the exact counterflow case with the R53 direct-solution observer. Invoke
the existing verified-inverse exactly once on the final captured factor and
require its classical audit to pass. Recompute the R36 slope using the original
line representative and only the verified error in place of the cheap error.

No Dot2, centered correction, solution replacement, direction, line trial or
state update is admitted.

## Hypotheses

| ID | hypothesis | discriminator |
|---|---|---|
| L1 | verifier placement is the sole remaining blocker | unchanged-center slope lower bound becomes positive |
| L2 | legacy audit improves but is insufficient | audit passes, but recomposed bound still covers nominal slope |
| L3 | R53 audit cannot be independently reproduced | direct tuple correspondence or classical verified-inverse fails |

L1 authorizes a separately frozen default-off trajectory that invokes verified
inverse only after an otherwise exact/KKT direction fails solely its slope
certificate. It does not authorize unconditional inverse work or production.
