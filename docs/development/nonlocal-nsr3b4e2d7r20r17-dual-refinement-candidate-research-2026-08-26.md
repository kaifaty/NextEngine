# NSR3-B4E2D7R20R17 dual-refinement candidate research

Status: `RESEARCH COMPLETE / BOUNDED CANDIDATE REPLAY SELECTED`.

## Candidate

Preserve the R8 NNQP algorithm and default mode. Only when the original ratio
test returns `RATIO_ORDER_AMBIGUOUS`:

1. require direct, corresponding current-solve provenance;
2. require verified inverse audits for current and candidate factors;
3. require the dual-refined ratio test to be exact;
4. replace only the two scalar error enclosures and ratio result with their
   certified refined values;
5. continue the unchanged interpolation/removal logic.

Any failed prerequisite returns the original rejection. No central-value
tiebreak, tolerance, regularization or iteration-cap change is introduced.

## Hypotheses

| ID | hypothesis | discriminator |
|---|---|---|
| C1 | on-demand dual refinement repairs the two observed counterexamples without disturbing earlier cases | all 8 development and 4 v3 cases reach complete KKT within the unchanged cap; default R8 semantic stays exact |
| C2 | it passes the first collision but exposes another missing certified branch later | at least one v3 path reaches a different named rejection |
| C3 | consuming the refined enclosure breaks cone, monotone globalization or final KKT | any downstream invariant or certificate rejects |

## Claim ceiling

All twelve cases are now development data. Even complete success is bounded
candidate evidence, not blind generalization. A new source-only v4 family must
be frozen after the candidate is closed. The two inverse matrices also add an
on-demand dense cost that must later be eliminated, amortized or shown rare
before production work.

