# NSR3-B4E2D7R20R54 counterflow legacy-slope contract

Status: `FROZEN / REPORT-ONLY MINIMAL LEGACY AUDIT AUTHORIZED`.

## Parent

- R53 implementation `d6354b2e`, semantic `5418141a...40a4`;
- exact direct matrix/factor/RHS/solution roots `3b0b5b6b...a98a` /
  `23f1bbb2...10b7` / `698106e6...6d3a` / `820c3d59...3ab7`;
- legacy audit root `e49c0000...3fc6`, 66 columns, refined error
  `1.0931517281e-24`;
- R36 slope root `57b1b93b...f02f`.

## Frozen audit

Replay the exact R51 counterflow once with the root-agnostic R53 direct observer
and require the final 66/66 tuple correspondence. Run the existing
`al_r20_verified_inverse` once on that captured factor, RHS and unchanged direct
solution. Require exactly 66 columns, classical pass, exact R53 audit root and
zero factorization work.

Run `al_r20_r52_slope_audit` with the verified `refined_error`. This preserves
the old direct center, global direction, current residual, natural face and
binary128 term ordering. Require nominal slope and direction roots equal R36;
only the error-dependent bound may change.

## Routes in precedence

1. `COUNTERFLOW_LEGACY_PARENT_REJECTED`;
2. `COUNTERFLOW_LEGACY_CAPTURE_REJECTED`;
3. `COUNTERFLOW_LEGACY_INVERSE_REJECTED`;
4. `COUNTERFLOW_LEGACY_SLOPE_APPARATUS_REJECTED`;
5. `COUNTERFLOW_LEGACY_SLOPE_STILL_UNRESOLVED`;
6. `COUNTERFLOW_LEGACY_SLOPE_CANDIDATE`.

Regress R53/R52/R51/R50. R54 adds one report-only 66-column existing audit and
no factorization, compensated dot, correction, new direction, trial, tolerance,
cap, state, timing, runtime or production change.
