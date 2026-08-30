# NCGA5 revision 2 — independent-reference apparatus repair

| Field | Value |
| --- | --- |
| Research ID | `NCGA5` revision 2 |
| Status | `FROZEN / SINGLE_APPARATUS_REPAIR / IMPLEMENTATION_AUTHORIZED / REPORT_ONLY` |
| Parent checkpoint | commit `2feaa9e571a9bb88ad587572c6d4b8ef0889bad8`, tree `4c394e1caf0d2c297842bc45f5b897bf82cae7c3` |
| Parent contract | NCGA5 revision 1, SHA-256 `6771ad7769010785f3473c2f9f68753d2985f77c25dc39ddf5845c0c52ab84cb` |
| Frozen candidate source | SHA-256 `90c001456c2df5f8df03d7aeda450ca40e355b49511e887f2bdfe0f675a478ee` |
| Revision-1 raw report | SHA-256 `87b7e43431a467afe304141c7152c0a1deb207c971bbde645731e877b34ea2ef` |
| Pre-existing scale-aware stop contract | SHA-256 `198979ed1ff82bbf3dffd7340a40786d870356b325e5d07d7a297c53d3b7582b` |

## Why revision 1 is inconclusive

The compressed-pair independent `long double` dense-assembly reference exactly
reproduced the retained NSR1 shape (`4` outer, `5` evaluations, `8` Hessian
products). The combined reference reached a much smaller physical residual but
did not reproduce the historical matrix-free binary64 terminal route: after
`12` accepted states it had gradient `4.8570228456397774e-8`, scaled
displacement residual `9.5411881975632536e-11`, then zero-resolution actual
reduction and minimum-radius termination.

The historical NSR1 report explicitly says its final near-stationary ratios
are dominated by binary64 noise. Requiring an independently ordered,
higher-precision dense Hessian assembly to reproduce that exact terminal ratio
is therefore an invalid apparatus gate, not evidence against either physics or
CUDA. Revision 1 remains immutable `INCONCLUSIVE`.

## The only authorized change

All fixtures, CUDA arithmetic, controller policy, candidate thresholds,
controls, execution order and claim ceiling from revision 1 remain exact.
Only `host_shape_valid` changes:

- compressed pair still must reproduce `4/5/8`, raw gradient success, no
  rejects/radius changes/active changes/negative curvature, and a
  multi-iteration `RESIDUAL` stop;
- combined tetrahedron is a valid independent reference when it is finite,
  monotone, has no active-set or negative-curvature change, exercises a
  multi-iteration `RESIDUAL` path and reaches either the raw `1e-10` gradient
  stop or the already frozen NSR2-A1 scale-aware displacement criterion
  `R_x<=1e-8`;
- if the scale-aware branch is used, its only allowed terminal failure is
  `MINIMUM_TRUST_RADIUS` or `OUTER_TRIAL_LIMIT` after a recorded
  zero/non-positive actual-reduction floor. That failure is reclassified as
  reference convergence for apparatus purposes only.

The host's historical `13/14/33` NSR1 counts remain reported but are no longer
an equality gate for the independently ordered dense assembly. The strict-f32
candidate is **not** granted this repair: its revision-1 raw success,
`5 um`, `1e-6`, `R_x<=1e-7` and numerical-floor `R_x<=1e-5` gates remain
byte-for-byte unchanged.

## Ordered rerun and stopping rule

Change only the host apparatus predicate and rerun the exact revision-1
executable sequence. Two fresh Release builds/runs, sanitizers and retained
regressions occur only if the repaired apparatus produces a non-inconclusive
classification. No tolerance, arithmetic, state rounding, fixture or second
apparatus repair is authorized.

The revision-1 classifications and claim ceiling remain unchanged. In
particular, a candidate outside the frozen state/objective/residual bands is
`F32_STATIC_SOLVE_MATERIAL` even if the absolute discrepancy appears small for
a game. That result authorizes causal arithmetic investigation, not a physical
trajectory or timing claim.
