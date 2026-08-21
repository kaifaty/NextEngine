# B4C3TR fixed canonical reference research

Status: `COMPLETE / CONTRACT FROZEN / IMPLEMENTATION NEXT`

Date: `2026-08-21`

## Question

B4C3TAR2 proves that the adaptive canonical controller can finish the tiny P1
and P2 horizons. It does not prove that the canonical fixed `48/96/192`
trajectories used as independent temporal references are themselves valid.
B4C3TR therefore asks only:

```text
Does each fixed lane preserve canonical transaction/physics invariants,
and do the three levels show either first-order temporal convergence or an
honestly bounded canonical-representation floor?
```

Adaptive-versus-fixed accuracy remains a separate B4C3TC question.

## Reference ownership

Each `(case, substeps-per-frame)` lane starts independently from the immutable
fixture. Every accepted physical substep is immediately published with the
B4C3Q balanced representation and admitted through the B4C3L KKT-scale ledger
policy. The next substep consumes the decoded canonical state. A lane owns:

- one contiguous global canonical step sequence;
- one canonical trajectory root;
- one legacy publication-ledger root;
- one KKT-policy ledger root;
- exact nonlinear and neighborhood work counters.

No frame, state or ledger entry is shared across levels. A failed macro-frame
stage remains private and cannot extend any committed root.

## Convergence without hiding quantization

The old binary64 reference criterion remains necessary: successive final-state
RMS differences must have ratio `[1.25,2.75]` or overlap their computed
binary64 floors. Canonical lanes add a second, distinct criterion.

For a lane with `S` committed substeps through elapsed time `T`, reuse the
already pre-frozen B4C3T representation envelope:

```text
E_x(S,T) = min(0.05*dx, 8*S*q*(1+T))
E_v(S)   = min(0.001*c, 32*S*q)
q        = 1e-6
```

Canonical successive differences pass when they either retain the same
`[1.25,2.75]` ratio or both differences overlap the independently computed
pair floors `E(a)+E(b)+binary64_floor`. The report must say which branch
passed and expose floor utilization. A representation-floor result is not
reported as an observed convergence order.

Additionally, every canonical lane is compared at every macro boundary with
the corresponding binary64 lane at the same step count and must remain inside
its own `E_x/E_v` tube. This prevents the pair floor from accepting an
unbounded canonical trajectory.

## Physics and ledger gates

Every level retains B4C3TAR2's non-adaptive physical gates: finite state,
capacity, pressure/contact activation, KKT/support closure, P1
center/density/speed, P2 canonical free-flight and terminal contact
properties. The solved KKT state retains exact contact feasibility; decoded
published geometry uses the already frozen one-quantum `1e-6 m` allowance.
Publication pressure and mechanical absolute-delta budgets remain `1%` of the
independent case energy scale. Strict max-scale residual stays a finite
diagnostic; compensated KKT-scale residual remains gated at `1e-9`.

Terminal contacts must equal the corresponding binary lane. P2 first-contact
time may differ by no more than one fixed substep plus binary64 allowance.
Mechanical-energy creation retains the canonical accounting rule: `1%` of the
independent scale plus the explicitly measured cumulative absolute publication
mechanical delta.

## Work and scheduling

The six lanes are mathematically independent and have no shared mutable
state. The harness may execute them concurrently with six fixed jobs, while
serializing report order as P1 `48/96/192`, then P2 `48/96/192`. This is a
harness resource-utilization change, not solver authority. Each result still
reports exact per-lane work; thread scheduling is excluded from hashes.

A forced failure after two private substeps following one committed fixed
macro frame must leave state, global count and canonical/legacy/policy roots
exact. Two full reports remain the determinism gate.

## Decision

Freeze B4C3TR under identity
`52bcd5bc908ea9b2afb36e15248bfe5a623e2b5b00f90e60e3fdddb2ec624b13`.
A PASS can authorize only B4C3TC adaptive-versus-fixed comparison design. It
cannot authorize the nominal corpus, runtime, CUDA or production integration.
