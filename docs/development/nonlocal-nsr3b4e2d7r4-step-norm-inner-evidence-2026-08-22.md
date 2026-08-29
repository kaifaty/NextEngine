# NSR3-B4E2D7R4 step-norm trust inner evidence

Date: `2026-08-22`

Status: `PASS / STEP_NORM_TRUST_INNER_CANDIDATE / PRIVATE_INNER_ONLY`

## Reproducibility

Two clean Release builds produce byte-identical 4,810,344-byte executables at
SHA `13f1a71143dfdd9dfb640e6ccf847b94a9340487d2fe7b4260eabfa758aa159c`
and Build ID `9acb36ee560d68101fddd6163d25b61fba0f1165`.

Both fresh processes exit zero with empty stderr and byte-identical 2,207-byte
stdout reports at SHA
`975da3f5adc13bba6c5fec3cfe08f0bc395f8fac58882b841ba5026774d6e886`.
The semantic result is
`77f7afc7e9a936ba19e183da07175b64e20beb8f3db939f868d0a578b3ffa289`.
Raw evidence is under
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4e2d7r4.jsJ0qV`.

All D7 through D7R3 command reports and exit states remain exact.

## Inner result

The separate candidate solves the exact post-outer-9 failed inner state in two
trials and four HVPs:

```text
trial 0  full interior step    rejected
         radius owner          STEP_NORM
         radius after          1.0585364232894624e-10 m

trial 1  boundary step         accepted
         raw ratio             0.73114360457587191
         direct ratio          0.7304410053344077
         topology exact        true

final scaled stationarity      6.0063645568914374e-9
accepted / rejected            1 / 1
step-norm / quarter updates    1 / 0
```

The accepted step passes the unchanged raw objective gate. The direct
difference is used only for interpolation. The exact D7R3 first event and
proposal reproduce, and a nonpositive-denominator control selects quarter
fallback at exactly `0.25*old_radius`.

## Scope boundary

The candidate position root is
`c2783c2021f54e1a475cdfc1a31dca41b49cf1a9e97ea27c6ca7303a631a091d`.
It remains private: multiplier is not updated, outer updates and commit count
are zero, and the public state rolls back exactly.

This proves the local inner failure is repaired by radius ownership. It does
not prove that the nested AL sequence converges or that two consecutive
pressure-state confirmations can commit.

## Decision

Select `STEP_NORM_TRUST_INNER_CANDIDATE`. Research/freeze a full private D7R5
outer integration under a new solver identity. It must replace only the inner
call in D7R's 14-update/two-confirmation transaction, retain every primal/dual/
pressure/position gate, and expose the exact first new failing boundary or one
confirmed private state. Publication and nominal Dam/Hydro remain blocked.
