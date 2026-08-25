# NSR3-B4E2D7R20R13 boundary-ratio collision research

Status: `RESEARCH COMPLETE / OBSERVER INSTRUMENTATION SELECTED`.

## Exact question

Which first condition makes the unchanged Lawson-Hanson boundary ratio test
return unresolved on each R12 counterexample?

## Hypotheses

| ID | causal hypothesis | prediction | falsifier |
|---|---|---|---|
| Q1 | a supposed leaving direction has a nonpositive computed denominator | first rejected denominator `<= 0` | all denominators positive |
| Q2 | the selected computed ratio lies outside `[0,1]` | unique best ratio, then range rejection | rejection happens before the range check |
| Q3 | two distinct leaving candidates have overlapping certified ratio intervals | best and competitor central ratios differ, but their bound-enlarged intervals overlap | intervals are disjoint |
| Q4 | the failure label hides an unrelated state/reproduction defect | R12 case or step root changes under observation-only instrumentation | exact roots remain unchanged |

Because current directions are maintained nonnegative and candidate directions
are selected as definitely negative, Q1 and Q2 should be impossible in exact
real arithmetic. Q3 is the leading finite-precision/enclosure hypothesis, but
it must be observed rather than assumed.

## Smallest discriminator

Extend only the ratio-result observer with the failing subpredicate, source-row
identities, current/candidate values, denominator, central ratios and bounds.
Do not change comparisons, operation order, returned `exact`, chosen alpha or
any hash-bearing solver state. Replay only the two failed cases and require the
R12 case and final-step roots byte-for-byte.

If Q3 is observed, the next research must derive whether the intervals are
genuinely indistinguishable at binary128 or merely inflated by the single
global direction-error bound. It must not choose a ratio by an arbitrary
tiebreak while the certified ordering is unresolved.

