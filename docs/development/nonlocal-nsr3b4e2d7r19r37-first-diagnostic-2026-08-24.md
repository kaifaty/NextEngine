# NSR3-B4E2D7R19R37 first diagnostic -- 2026-08-24

Status: `EXPECTED-CONTROL REPAIRED / NOMINAL WORK NOT STARTED`.

The first strict-f64 diagnostic exits fail-closed at
`CONTINUATION_DENSE_REJECTED`. R36 parent stdout and R37 identity are exact,
but no nominal workspace or pair pass is admitted.

The failed expectation required the conditioned dense fixture to become
stationary with objective `<=1e-28`. That fixture has opposing simultaneously
positive inequalities and therefore a positive exact minimum; zero objective
is mathematically impossible. This was an invalid test expectation introduced
with R37, not a failure of the frozen polish recurrence.

The repair preserves the identity, nominal source, algorithm, checkpoints and
work budget. It compares the same conditioned fixture after 6 and 30 unchanged
steps, requiring both executions to pass, exact accepted counts `6/30` and
strictly smaller objective at 30. The independent active-switch and exact
stationary controls remain unchanged.

Failed diagnostic stdout-with-LF SHA-256 is
`5203983aa6ff3baa84fe96057dde321f711090bf2a4ea3bed043b91994d56aa0`;
semantic result SHA-256 is
`5a2a279f10b6a663f71289354f217deda0ddfb84511782838a5134924e07d83f`.
It records zero new pair passes and cannot provide method evidence.

## Reclosed diagnostic

With the corrected expectation, the same strict-f64 build passes every hard
gate and selects `HYBRID_POLISH_CONTINUATION_CANDIDATE`. All 24 steps are
accepted; pair work closes exactly at 51 and fresh terminal response defect is
`6.2930931293254465e-16`.

Relative to the exact R36 endpoint, terminal objective, violation and
projected mapping are `0.1288668112x`, `0.3589802378x` and `0.3536743759x`.
The nonzero terminal mapping is `2.6422690805514893e-10`; exact stationarity is
not claimed. Reclosed diagnostic stdout-with-LF SHA-256 is
`f475c20302c444be1ca355a1c9f0ec39309bbe7cd33f4db73fad676bef1c3571`;
semantic result SHA-256 is
`c6a254243c8d549f7f20322ed61321eb94ff3d551e75c850d9b1a549a2d14118`.
Clean proof runs remain pending.
