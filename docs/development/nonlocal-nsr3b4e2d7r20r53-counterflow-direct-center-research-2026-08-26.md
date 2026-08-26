# NSR3-B4E2D7R20R53 counterflow direct-center research

Status: `RESEARCH COMPLETE / FINAL DIRECT-SOLVE AUDIT SELECTED`.

## Question

Does counterflow reject a valid positive full-support Newton direction only
because its ordinary direct-solve enclosure is too wide for the global slope
predicate?

## Correction after R52

R52 proved that counterflow executes no verified-inverse audit. Its final NNQP
support is nevertheless 66/66 and the final direction is an ordinary direct
principal solution. Therefore the relevant tuple is the last direct
factor/matrix/RHS/solution, not a nonexistent successful-audit record.

Also, a centered certificate bounds the exact solution around its newly
computed center. It is not sound to attach that smaller radius to the old
binary128 center. The shadow slope must be recomputed from the certified center
itself while leaving solver state untouched.

## Bounded discriminator

Replay the exact R51 counterflow case once with a default-null observer on
successful direct principal solutions. Require the last observed tuple to equal
the final 66-row NNQP direction, RHS, factor and cheap error.

Using the already captured factor, construct exactly one 66-column approximate
inverse report-only. Evaluate the existing exact/Dot2 practical centered
certificate at fixed depth 16, with target-specific checks disabled. Require
exact containment, no underflow, left contraction and 66 strictly positive
certified components.

Rebuild the final global direction from the certified center and the unchanged
accepted-prefix lambda. Recompute nominal slope and every residual/direction/
rounding bound in the original binary128 order. Do not apply the center or
execute line search.

## Hypotheses

| ID | hypothesis | discriminator |
|---|---|---|
| D1 | cheap direct-solve enclosure causes the rejection | centered full-support certificate passes and shadow slope lower bound is positive |
| D2 | centered solve is valid but slope remains unresolved | certificate passes, yet the shadow bound covers the shadow nominal slope |
| D3 | represented inverse is unsuitable | compensated left defect is noncontractive or underflow containment fails |
| D4 | direct center changes the active face | any certified component is nonpositive or unresolved |
| D5 | observer does not identify the final solve | support/direction/error/factor correspondence fails |

D1 authorizes only a separately frozen default-off trajectory that replaces the
final direct center before the unchanged slope and globalization predicates. It
does not authorize production, runtime or performance work.
