# NSR3-B4E2D7R5 private outer-AL integration evidence

Date: `2026-08-22`

Status: `PASS / OUTER_CAP_OR_NESTED_ACCURACY_RESEARCH / PRIVATE_ONLY`

## Reproducibility

Two clean Release builds produce byte-identical 4,832,600-byte executables at
SHA `e7fcd511283092cd1fbff751fc8e995184e1eb2c1f43a36128a2bbb59e283c72`
and Build ID `456f7b7aed27de83ad49c086bcf6f6b59dcd28e8`.

Both fresh processes exit zero with empty stderr and byte-identical 9,324-byte
stdout reports at SHA
`7ea5489fa90346f385ce8db08dba445f2e2389e65fdf556a13a17e837ba916d7`.
The semantic result is
`46bfccedddada3dc38479e3f510998a7cf00fa68c04f9a37703016bd4d98398e`.
Raw evidence is under
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4e2d7r5.Pjtm08`.

D7 through D7R4 reports retain their exact historical stdout hashes and exit
states. In particular, the D7R4 parent remains
`975da3f5adc13bba6c5fec3cfe08f0bc395f8fac58882b841ba5026774d6e886`.

## Outer result

The candidate preserves the exact first-eight roots:

```text
outer JSON  9bffc61a943f50cab449c052bd0a6791c40128e894d98d9a9589b0969dbb82c2
state       04a9c03308662d6102b165d8109b23c1b60a3145314603909b7dbfda8385d95e
```

All 14 inner solves pass, every value remains finite, multiplier feasibility
holds and primal violation is monotone. The selected step-norm policy first
activates at outer 10 and repairs the old failure with one rejected and one
accepted trial. The transaction reaches the unchanged cap without an
admissible pressure-state pair:

```text
outer 10  primal                     8.3349549484523777e-11
          absolute dual change       1.0220738499988613e-7 J
          pressure change             8.1765907999908904e-4 Pa
          inner stationarity          6.0063645568914374e-9

outer 13  primal                     0
          absolute dual change       4.8422284681937100e-7 J
          pressure change             3.8737827745549680e-3 Pa
          inner stationarity          7.1566237390554322e-15
```

The last three primal violations are nonincreasing. No provisional or
confirmed pressure state exists, so the warm holdout is correctly not run.

## Interpretation

The old problem was a globalization failure: repeated interior proposals
never made the trust radius bind. D7R4/D7R5 repair it. The remaining boundary
is different: fixed inner stationarity `1e-8` permits zero-work inner exits
while the next multiplier update is still above the absolute pressure-state
gate.

The absolute dual and pressure gates are the same bound in this fixture:

```text
delta lambda <= 1e-8 J
delta pressure = delta lambda * rho0 / mass <= 8e-5 Pa
positive constraint <= 1e-8 / 1226.25
                    <= 8.1549439347604491e-12
```

Outer 10 remains about `10.22x` above that dual/pressure gate. Outer 13 has
zero positive primal violation but the corrective multiplier decrease is
about `48.42x` above it. Therefore primal feasibility alone cannot establish
pressure-state stability.

## Controls and scope

Inactive, zero-multiplier reset, parent-byte and forced-rollback controls all
pass. The final private root is
`1fdbcb763dedce34937c399986691469eaab55565024277ddc70f4dbf8c86546`;
the public root remains
`3af35d5de0607dd19225135c91eb214433bbc80577a2b54ff2ab0af8889de91b`.
Public commit count, trajectory steps and physics mutations are zero.

This PASS classifies the complete private transaction. It is not a confirmed
pressure state and grants no tolerance/cap change, nominal trajectory,
performance, runtime, GPU/PhysX or production authority.

## Decision

Select `OUTER_CAP_OR_NESTED_ACCURACY_RESEARCH`. Freeze one D7R6 private
discriminator that forks after the exact D7 prefix and observes a fixed
inner-stationarity ladder through a bounded 64-update outer horizon. It must
separate cap-only, accuracy-only, coupled and formulation outcomes without
publishing state or selecting a new production parameter.
