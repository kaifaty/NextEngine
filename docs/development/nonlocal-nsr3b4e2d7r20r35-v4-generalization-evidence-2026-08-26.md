# NSR3-B4E2D7R20R35 v4 generalization evidence

Status: `PASS / V4_GENERALIZED_POLICY_REJECTED / 3 OF 5 CERTIFIED`.

Implementation `1f69b8bb` emits reproducible semantic:

```text
59c32c19863321eeee98838ef907ee02630c37ca032bb2846d655ae6165037b3
```

All five R34 problem roots reproduce and every case observation is exact. The
blind result is:

| v4 case | result | accepted steps | case root |
|---|---|---:|---|
| torsion | active-set rejected | 7 | `a543bc06...1d6d` |
| corner | ordinary certified | 10 | `204b49d5...e0e3` |
| counterflow | direction rejected | 9 | `7e2735e1...c5d3` |
| helical compression | ordinary certified | 14 | `11d800fd...0f05` |
| alternating layer | ordinary certified | 11 | `98b9eafd...09fe5` |

No case reaches exhausted-line recovery or terminal selection; there are zero
event replacements and 81 total inherited line evaluations. Thus the two
failures occur before the new globalization mechanisms can participate.

Torsion iteration 8 has an exact 68-row natural face and 65-row current support,
but NNQP stops after 93 transitions/principal solves with
`VERIFIED_INVERSE_AUDIT_REJECTED`. Counterflow iteration 10 has a resolved
66-row face and exact KKT-certified NNQP, yet its positive slope
`1.8493164062e-19` is dominated by slope bound `2.0123909084e-14`; the recorded
direction error bound is `8.8529495396e-3`.

The first executable report returned `V4_SOLVER_HARNESS_REJECTED` at semantic
`bdf7a514...31aa` because the R35 wrapper incorrectly treated the intentional
NNQP fail-closed flag as an apparatus failure. The observer-only repair uses
the established structural-failure predicate. All five case roots before and
after the repair are identical; the invalid top-level semantic receives no
scientific credit.

R32 and R34 remain exact at `e7b9acaf...0f73` and `811ac231...02ab`.
The result refutes 5/5 blind generalization but supports ordinary convergence
on 3/5. It does not authorize source edits, retries, tuning, performance work
or production promotion.
