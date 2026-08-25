# NSR3-B4E2D7R20R27 post-event globalization research

Status: `RESEARCH COMPLETE / REJECTED-STEP AUDIT SELECTED`.

## Question

Why does the R26 shear trajectory reject all line trials immediately after
three successful certified face crossings?

## Hypotheses

| ID | hypothesis | prediction |
|---|---|---|
| P1 | the new Newton representative crosses another projector event before any Armijo-valid dyadic point | rejected trials change masks/ball and the first stable trial lies beyond the 21-point envelope |
| P2 | the same-face local model is no longer an ascent model | at least one mask-stable trial exists but its rigorous Armijo margin is negative |
| P3 | the rejection is a finite-precision sign boundary | exact margins approach/overlap zero while KKT and face remain stable |
| P4 | the event replacement leaves a face/natural-set inconsistency | the next natural face, projector mask or NNQP support shows an immediate nonlocal jump inconsistent with the committed one-scalar event |

## Selected discriminator

Replay only the exact R26 shear case and require candidate root
`1c9a0bf...cf91`. Expose all 20 steps, with special focus on the rejected final
step: natural face/root, passive support, NNQP work, direction slope/bound, and
every existing dyadic trial's exact Armijo margin, mask/ball delta, dual
increase and KKT tuple. Record the three replacement steps and the committed
mask/support transitions leading to the failure.

Classify stable rejection, all-crossing envelope exhaustion, precision margin,
face inconsistency or unresolved under a frozen priority. This is report-only:
no extra alpha, event, fallback, cap, tolerance or state update is admitted.

