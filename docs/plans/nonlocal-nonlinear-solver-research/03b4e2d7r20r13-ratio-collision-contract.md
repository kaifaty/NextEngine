# NSR3-B4E2D7R20R13 ratio-collision contract

Status: `FROZEN / OBSERVATION-ONLY INSTRUMENTATION AUTHORIZED`.

## Parent

- R12 implementation `83c693b4`, semantic
  `99b0b72c964044c6b17e31439fcad4b6001db467d1e8d6aeb27fe972d4647f57`;
- exact failed case roots `cd3ee19c...a0ec4`, `7929dffc...29c5d`;
- final-step roots `b030666a...6a8b`, `4cabd7c3...7a50`;
- both named failures `BOUNDARY_RATIO_UNRESOLVED`.

## Frozen observer

Add report-only fields to the ratio result and copy them to the NNQP diagnostic
only when the existing ratio function returns unresolved. Preserve every
existing arithmetic operation, comparison, control-flow result and hash input.

Classify exactly:

```text
INVALID_RATIO_INPUT
NONPOSITIVE_DENOMINATOR
RATIO_ORDER_AMBIGUOUS
ALPHA_OUT_OF_RANGE
RATIO_SUBPREDICATE_UNRESOLVED
```

For the first failure report leaving source rows, current/candidate direction,
denominator, best/competitor central ratios and both rigorous bounds. Replay
only failed v3 ordinals 1 and 2. Require exact R12 case and final-step roots,
zero successful-case solves and unchanged solver decisions.

Route `V3_RATIO_SUBPREDICATE_IDENTIFIED` only when both failures reproduce and
receive a named subpredicate. Otherwise route parent, reproduction or observer
rejection. No ratio selection, bound change, retry, timing, runtime/GPU or
production authority.

