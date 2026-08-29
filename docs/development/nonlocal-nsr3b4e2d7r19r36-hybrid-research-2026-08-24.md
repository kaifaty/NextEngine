# NSR3-B4E2D7R19R36 equal-work curvature-polish hybrid research

Date: `2026-08-24`

Status: `RESEARCH COMPLETE / CONTRACT FROZEN / IMPLEMENTATION NEXT`

## Rationale

R35 at equal work improves R34 objective and violation but regresses projected
stationarity. Its first three curvature outers already reach objective
`9.7460913443413455e-17`; the fourth outer consumes exactly 12 pair passes.
The validated first-order recurrence consumes two passes per step and improves
stationarity under changing active sets.

R36 therefore replaces only R35 outer four with six projected exact-line
polishing steps. No parameter is fitted:

```text
prefix validation                    1
3 curvature outers x 12             36
6 first-order polish steps x 2      12
fresh terminal JVP/VJP               2
total                               51 pair passes
```

The start remains exact R33 `v8`, the trust ball remains radius `0.25`, and
the first three curvature records must reproduce R35 exactly.

## Decision rule

Fresh terminal objective, violation norm and projected-mapping norm are
compared with R34. Hybrid is selected only under strict three-metric
dominance. Otherwise R34 first order remains the reference. Projected
stationarity is separate.

The experiment does not relax R35's negative result, add work, change CG
depth, introduce damping/preconditioning or use wall time.

## Controls and authority

Dense controls cover curvature-prefix reproduction, stationarity polishing
and active switching. Exact parent/source/workspace/prefix, three CG blocks,
six polish line KKT checks, terminal operators, 51-pass/15-HVP ledger and
rollback are mandatory.

R36 is report-only. It cannot apply an iterate, evaluate nonlinear moved
state, update duals or claim floor, timing, runtime or production authority.
