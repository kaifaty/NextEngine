# NSR3-B4E2D7R20R34 v4 preflight contract

Status: `FROZEN / INPUT-ONLY OPERATOR PREFLIGHT AUTHORIZED`.

## Parent

- R33 implementation `d788afbc`, semantic
  `33ff1cbf1a8ae753fb84a607d728e0e82ef487d44875ee7615e1d4aa19fcee63`;
- five exact source roots and counts `72/96/72/80/84`;
- zero prior operator/projection/solver observation for v4.

## Frozen preflight

Materialize exactly the five R33 sources in manifest order with the existing
binary64 sparse problem builder. Require:

- exact slot, entry and incidence ownership with `source_positive=0`;
- finite joint box-ball projected target and rigorous row bounds;
- at least one row satisfying `projected_raw > row_bound` per source;
- the existing R10 scale controls at rows `0`, `rows/2`, `rows-1` for both
  frozen low/high factors;
- five pairwise-distinct complete problem roots and exact workspace lifecycle.

Hash target, constraint, domain, operator, problem and per-case diagnostic
roots. Routes distinguish parent, operator, excitation and PASS boundaries.

No solver iteration, inverse audit, KKT trajectory, fixture edit, tolerance,
timing, runtime/GPU, generalization or production authority. Commit the
preflight result before authorizing a v4 solver run.
