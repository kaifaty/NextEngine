# NSR3-B4E2D7R20R10 v3 operator preflight contract

Status: `FROZEN / EXECUTION AUTHORIZED / INPUT ONLY`.

## Parent

- R20R9 implementation `49ea40b8`, semantic
  `4125a52d71a2002d1188b948b71a5d2c0173dc8e25dab30ace59fbd9ba43e9c3`;
- four immutable source roots and exactly 277 fluid samples;
- operator/preflight/solver/timing counters zero at freeze.

## Frozen checks

Materialize each v3 problem with the unchanged R20 operator/preflight code and
trust radius `0.0625`. Require workspace and R64 slot/entry/incidence audits,
finite positive diagonals/scales, zero positive source constraints and at least
one projected-positive row whose lower value exceeds its outward bound.

Run the same selected-row scale controls under factors `2^-8` and `2^8`.
Freeze target, constraint, domain, operator and complete problem roots. Release
all four workspaces exactly once.

Routes:

```text
V3_PREFLIGHT_PARENT_REJECTED
V3_PREFLIGHT_OPERATOR_REJECTED
V3_PREFLIGHT_EXCITATION_REJECTED
V3_PREFLIGHT_PASS
```

Candidate/solver/Newton/inverse iterations, KKT decisions and timing are zero.
No source edits, threshold fitting, runtime/GPU integration, production or
generalization authority. Commit the preflight before authorizing the one-shot
R8 generalization run.
