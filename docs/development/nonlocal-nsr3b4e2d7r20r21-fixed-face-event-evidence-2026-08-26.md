# NSR3-B4E2D7R20R21 fixed-face projector-event evidence

Status: `PASS / FIXED_FACE_EVENT_PREDICTOR_CANDIDATE`.

Implementation `0c261dd4` reproduces R20 semantic exactly and emits:

```text
3ff49117f8ad70f55c345e4a9f9459bd11bf994125267baf04891c89ec857bca
```

All seven fixed-face audits find exactly one admissible root inside the frozen
R20 transition cell. Ambiguous, unbracketed and wrong-scalar counts are zero.
Predicted scalars equal 168 in steps 25--27 and 189 in steps 29--32.

All seven current projectors have an active trust ball, with 93/117 free/
clamped components before the step-28 transition and 94/116 after it. Although
the general active-ball event equation is quadratic, every selected physical
event is a release from a zero lower bound. Its unsquared event equation is
therefore linear and the relation residual is zero in six cases and
`-3.76e-37` in one.

Each state enumerates 420 bound polynomials. It produces 346 algebraic roots
before the transition and 338 after it; all but one lie outside the already
frozen local cell. Thus the successful result is not a claim that every global
quadratic root is a valid projector event. Uniqueness follows only after the
positive-cell, unsquared sign, face-side and endpoint-mask predicates.

The predictor is validated as report-only mathematics. It has not generated a
solver trial, selected an offset, changed a multiplier or established broad
corpus generalization.

