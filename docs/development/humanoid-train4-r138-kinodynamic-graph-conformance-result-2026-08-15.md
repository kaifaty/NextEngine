# TRAIN-4 R138 kinodynamic graph conformance result — 2026-08-15

| Field | Value |
| --- | --- |
| Scope | Sole clean report-only fixed-mode graph implementation conformance |
| Status | `R138_PASS_R139_FORMULATION_ONLY` |
| Gate decision | `PERMIT_SEPARATE_REPORT_ONLY_R139_KINODYNAMIC_SOLVE_FORMULATION_ONLY` |
| Claim ceiling | Synthetic graph/controller conformance only; no real reconstruction, assembly or solve |

## Immutable result

The sole R138 process ran from clean commit `4745c0a` and passed all six frozen
validations. Canonical/file/profile SHA-256 is
`c2343b289a11e7794d4f092b261f18dc6f4776b265854893cd3403fc53daea6c` /
`cec76311db0c0c4d4cb693b806a1fae344eb3e79e24161aa4081c37a5e258d83` /
`54e6291852997450f4acd2e15f9fb6e499a16196b9aea53295ffef74a979b216`.
The canonical hash independently reproduces exactly.

R138 reads only the R137 report and 29 transition metadata rows. It does not
receive or read R120/R133 state arrays, R136 cache/forces or any candidate.
Every real state, controller, dynamics, Jacobian, integration, impulse,
kinodynamic assembly/solve, factorization, optimizer, cache/candidate, PhysX
and training counter is zero.

## Index and event conformance

The address map closes all `800` motor rows, `3200` unique physics intervals
and `3201` state nodes. Each command target owns exactly four ordered physics
steps; first address is node `0 -> 1`, and final address is `3199 -> 3200`.
The ordered address SHA-256 is
`844ed3a2e727712fedff2c0cd858c30639c8956e054f79f2012d67277535639b`.

Independent replay closes all `29` changed motor boundaries, `11` activation
boundaries with `18` activated points, and `18` zero-impulse release boundaries
with `18` deactivated points. Its ordered SHA-256 is
`9f289c31ecfe2ef18b4beed4559982c3da25f7244eb6b17c022ac6000edc322a`.

Six synthetic layouts pass: ordinary flight, flight-to-forefoot,
flight-to-flat, forefoot-to-flat, flat-to-forefoot and forefoot-to-flight. The
malformed simultaneous release/reseat case fails closed. Free effort, inactive
force and unscheduled impulse columns remain structurally absent.

## Exact-controller differential

R138 derives six synthetic schedules, `19200` controller rows in total, through
the independently tracked R130 and R133 controller implementations. Their
applied targets and efforts differ at zero scalars in all three cases:

- zero state: no controller event;
- ties-to-even: the first three targets are exactly `[0, 2, -2]` microradians;
- limiter state: one target slew, eight effort-rate clamps, six positive-work
  clamps and 15 row-addressed events; the synthetic one-microjoule work ceiling
  is reached but not exceeded.

The limiter descriptor change exists only in an in-memory synthetic copy and
has no production descriptor or acceptance authority. R138 proves controller
graph equivalence for these cases, not feasibility of the real trajectory.

## Exact stop boundary

The exact transition is `R138_PASS_R139_FORMULATION_ONLY`. It authorizes only a
separate report-only R139 solve formulation that must freeze source
reconstruction, objective hierarchy/scaling/trust region, exact-versus-smooth
candidate boundary, algorithm/tolerances, resource budget, hashes and stop
conditions. R139 may not evaluate the real controller graph, assemble or solve
the real trajectory.

A real kinodynamic solve requires a later implementation-conformance gate and
explicit roadmap authority. R136/R137 retry, contact-semantics changes,
candidate, PhysX, corpus admission, visual/exhaustive gate, learned optimizer
and training remain unauthorized.
