# NSR3-B4E2D7R20R22 event-side trial research

Status: `RESEARCH COMPLETE / NEXT-REPRESENTABLE SHADOW SELECTED`.

## Question

Is the smallest binary128 point strictly beyond each analytic projector event
already a valid Armijo step on the new face?

## Hypotheses

| ID | hypothesis | prediction |
|---|---|---|
| S1 | the event can be crossed without an empirical offset | `nextafter(root,+inf)` changes exactly the predicted scalar and passes rigorous Armijo in all seven states |
| S2 | only the five R20 offset brackets admit immediate crossing | the two grid-aligned brackets reject the next-representable point |
| S3 | finite root rounding does not reliably select the new face | next-representable evaluation remains on the old face, changes another scalar or changes ball activity |
| S4 | the event and objective frontiers are ordered adversely | the new face is selected but Armijo is negative in one or more states |

## Selected discriminator

Replay R21 exactly. For each unique predicted event evaluate two shadow points:
the emitted root and `nextafterq(root,+HUGE_VALQ)`. Require the root to retain
or touch only the predicted bound and the next-representable point to change
exactly the predicted scalar with unchanged ball activity. Evaluate the exact
dual, R19 Armijo margin and KKT tuple at both.

No fallback, midpoint, epsilon, bisection or state update is allowed. The
classification distinguishes all-seven acceptance, the pre-observed five-case
subset, new-face Armijo rejection and representable-side failure. A positive
result can authorize only a separately frozen opt-in candidate trajectory.

