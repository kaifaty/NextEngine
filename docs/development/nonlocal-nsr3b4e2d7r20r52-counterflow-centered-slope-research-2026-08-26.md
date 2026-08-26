# NSR3-B4E2D7R20R52 counterflow centered-slope research

Status: `CLOSED / CAPTURE PREMISE REFUTED / NO SOLVER CREDIT`.

## Question

Is counterflow's global slope rejection caused by the same zero-centered
solution error loss now removed from torsion, even though its legacy inverse
audit technically passes?

## Bounded discriminator

Replay only the exact R51 counterflow case with no generic replacement hook.
Capture the already executed final successful inverse audit and its matrix,
inverse, RHS and solution. Run the practical depth-16 centered certificate
report-only, without changing the solve.

Reconstruct the R36 natural face/current/global direction exactly. Recompute
the global slope bound once with the centered uniform solution error in place
of the inherited `8.85295e-3` scalar. Preserve residual and arithmetic terms in
the same binary128 order. Add no direction, line trial or state update.

## Hypotheses

| ID | hypothesis | discriminator |
|---|---|---|
| S1 | zero-centered solution error dominates | centered certificate passes and refined slope lower bound is positive |
| S2 | centered error improves but remains insufficient | refined error is tighter, yet bound still covers nominal slope |
| S3 | successful inverse orientation is unsuitable | left defect is noncontractive or centered signs remain unresolved |
| S4 | capture does not correspond to the final direction | audit root, RHS/solution or R36 face/direction parity fails |

S1 authorizes only a separately frozen shadow trajectory using a generic
successful-audit refinement point. It does not change the production policy.

## Outcome

The frozen replay reproduced the exact R51 counterflow case and R36 slope, but
the observer recorded zero verified-inverse calls. The final NNQP solve also
contains zero inverse-audit records. Counterflow never reaches that predicate:
its final 66-row support is already sign-resolved by the cheap direct-solve
bound, then the global slope certificate rejects the direction.

Therefore the assumed "already executed final successful inverse audit" does
not exist. R52 is an invalid-apparatus result with no certificate or solver
credit. A successor must explicitly capture the final direct principal solve
and budget any new report-only inverse work; it must not reinterpret the empty
R52 capture as a physical or convergence failure.
