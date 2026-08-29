# NSR3-B4E2D7R20R24 zero-bound mask-flutter research

Status: `RESEARCH COMPLETE / EXISTING-LADDER RUN AUDIT SELECTED`.

## Question

Is R23's step-32 later event a genuine second projector event, or an old/new
mask flutter caused by rounding the zero-bound preprojection scalar?

## Hypotheses

| ID | hypothesis | prediction |
|---|---|---|
| F1 | the same scalar flutters around zero in the evaluation graph | the 65-point run contains only old and predicted-new masks, ball stays active, actual `z_i` changes sign more than once while analytic affine `z_i` is nonnegative and monotone |
| F2 | there is a second geometric event | a different scalar or ball mode changes, or the analytic affine input itself crosses another boundary |
| F3 | R23's later-event detector is over-sensitive | the full mask code stays on the predicted new face after its first crossing |

## Selected discriminator

Replay the exact R23 65-point ladder without adding samples. For every power,
record a three-state code: old mask, exactly predicted one-scalar new mask, or
other. Also record the actual projector input component recomputed through
`lambda -> A^T lambda` and the independently affine value from the R21 event
polynomial. Emit run-length encoding, sign-transition counts, maximum actual/
affine difference and the first power of the final all-new suffix.

Classify old/new returns with no other mask/ball event and a monotone analytic
input as `ZERO_BOUND_ROUNDING_FLUTTER`; an other-mask/ball/analytic event as
`GENUINE_SECOND_EVENT`; an all-new suffix beginning at the first crossing as
`LADDER_EVENT_STABLE`; otherwise unresolved.

This audit may identify the mathematical requirement for a forward-error-owned
crossing, but it cannot choose a suffix length, force a mask, alter projector
comparisons, generate/apply a solver trial or authorize runtime/production.

