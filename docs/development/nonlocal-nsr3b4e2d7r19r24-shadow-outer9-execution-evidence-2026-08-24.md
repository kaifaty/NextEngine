# NSR3-B4E2D7R19R24 shadow outer-9 execution evidence

Date: `2026-08-24`

Status: `PASS / SHADOW_OUTER9_EXECUTION_CANDIDATE / NOT ADMISSIBLE`

## Outcome

The slice candidate and independent unsliced oracle execute outer 9 with
complete bit/work equivalence. Candidate remains within epoch-1 capacity.
The result is finite and accepted but still non-admissible.

```text
update root       e056907da2364c033f12a7a44c235b5274aa675cb548c9a84248178a2efb06b6
work root         b28255cad5e7aa3a86176ff53642493972ae47ac6208ae75cd29e7f653499212
trials            2 accepted / 0 rejected
HVP/workspace/precision delta   49 / 4 / 2
candidate/oracle exact          true
```

## Physical and mechanism result

```text
primal          1.9451631638744971e-08   bits 0x3e54e2d2e4000000
stationarity    9.5866405588047984e-11   bits 0x3dda59ffa2498bd8
admissible      false
position root   b59fdcd410ab2612246324deb5417aa4cec908c644e7ba0d970c348ca088861a
dual root       69c062bb1ce89250d978dce462b8f733bc88bfd48c1fe880e57290be0e07550a
```

Primal improves about 4.454% from outer 8, close to outer 8's 4.9% and much
slower than outer 7's roughly 14.9%. The two accepted trials use only 49 HVP,
and the last uses 24 against cap 34. R22's 32-HVP last trial therefore was not
evidence of monotonically increasing cap pressure.

Stationarity rises from `1.924e-14` to `9.587e-11`. It remains finite and
inside the frozen validity path, and candidate/oracle exactness proves that it
is not caused by the slice offset. Preserve it as an explicit R25 input and
observe the next outer; this one sample does not authorize a policy change.

## Canonical successor

```text
history         d39b98c23d63217b0145229b593154fa3b49e2ea2b701ba708c51f5f8eb0b28d
consumed owner  3f4b2fb5d84b3f27260e53e344d5141530b8105f577483aca7a435ec85372522
receipt         9c618bd10b6aeaed9ae9e5fddd9f7287e4df6d7adf3e76e467543c70e8674e28
state           749f0805dc2c4b80e2207ea456071271d9c16521515f0554a13dd7203da32d98
used            10,2,24,1,231,754,50,30,726,28,2,30,0
```

Epoch remains `1`; slice/cumulative HVP become `231/754`, leaving 281 slice
HVP. Recurrence/direct accounting closes at `726 + 28 = 754`.

All `13/13` routes pass at corpus root
`5e3c4fcae5933a450b1b98ac99b327892696e2c9086aff7582985165a42ea1d0`.
Rollback and duplicate replay are exact. No following outer, substep, macro,
trajectory, timing or public/world commit occurs.

## Reproducibility

Contract/research commit: `d853f585`.

Implementation commit: `29c5a4c4`.

Two clean Release builds:

- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r24-a.xGgqLP`;
- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r24-b.Gakq8H`.

Both binaries are `6,681,928` bytes, have SHA-256
`ec25998c8050948e53325d7e42d16742fd9b31031ba3170e8d2bbce69c97167b`
and GNU build ID `5a72481664cff9cc45007156f3e7808c50d480fa`.

Fresh sequential one-process outputs:

- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r24-a.IjBELq`;
- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r24-b.rJjgpL`.

Both exit `0`, emit empty stderr and reproduce `1,885` stdout bytes:

```text
stdout SHA-256  e401056652ba56aad8c9f7836ac963a57721ef4d3c498ae29cb1736e03562eed
semantic        5b623755c2fd1dc136e9e9281c32b387955b1daea4e435460e9f764892b88bbc
route           SHADOW_OUTER9_EXECUTION_CANDIDATE
```

## Next action

Research/freeze a zero-work one-use outer-10 grant at exact state/receipt/
history and slice/cumulative `231/754`. Preserve the stationarity observation,
keep all caps unchanged and do not execute outer 10 before the grant passes.
