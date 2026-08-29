# NSR3-B4E2D7R dimensionally stable AL commit evidence

Date: `2026-08-22`

Status: `FAIL / INNER_ACCURACY_FLOOR / NO_COMMIT / NO_TRAJECTORY`

## Reproducibility and stop boundary

Two clean Release builds produce byte-identical 4,675,568-byte executables at
SHA `3b57eedafb42c6e026308318356a8a23dba33dedd3d0f2f6e93115a6bc5e0854`
and Build ID `2ac4a5549e6ba2850463cf69c7e52b52872641a1`.

Fresh process A exits one with empty stderr and a 6,784-byte stdout report at
SHA `4b0272df2d46bb53ec09175ea7095b0e8c699ac9e0e4e093c6e20dfd00240801`.
Its semantic result is
`b384964ddb70aede09d6dc3994d851b90ca3af7153ef340d9ed45423c5c53a63`.
Process B is `NOT_RUN(FIRST_PROCESS_HARD_FAIL)`. Raw evidence is under
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4e2d7r.Zq80f6`.

The clean executable also reproduces the preserved D7 failure report exactly:

```text
stdout SHA = e0542abc4e0c7ff0b38acc0fe38095e270dec030617ad5183665414ce9b3db11
result SHA = 0453794038d8788e4ae8e6a77a691619a6e0769aa910b82291031c7d042cbab4
```

## Passing controls

- identity, fixture and all D7 derivative/dense/reaction gates pass;
- the first eight outer records reproduce exact outer-array root
  `9bffc61a...82c2` and state root `04a9c033...d95e`;
- primal violation stays monotonically nonincreasing;
- the prior public state remains exact, forced confirmation rejection rolls
  back exactly and `commit_count=0`;
- the old D7 command bytes remain unchanged.

## First failing boundary

Outer 8 behaves as the earlier D7 warm control:

```text
primal                       = 2.8522317840895539e-10
absolute multiplier update  = 3.4975492257949270e-7 J
equivalent pressure update   = 2.7980393806359416e-3 Pa
position update              = 5.1231115737842871e-9 dx
stationarity                 = 1.0654481140911209e-14
```

It is KKT-small but correctly fails the new absolute pressure-state gate.
Outer 9 then exposes the nested-accuracy mismatch:

```text
inner trials / accepts / HVP = 1 / 0 / 0
position update              = 0
stationarity                 = 5.2405990388081412e-9
primal                       = 2.8522317840895539e-10
absolute multiplier update  = 3.4975492257949270e-7 J
```

The fixed inner criterion regards `5.24e-9 <= 1e-8` as converged, so the
primal state does not respond, while the outer PHR update changes pressure
again. On the next outer call the inner solver reaches `REJECT_LIMIT`. No
provisional or confirmation state exists and no route is selected.

## Decision

An outer cap increase cannot fix an inner solver that alternates between
premature zero-work admission and energy-floor rejection. Lowering the inner
stationarity tolerance alone is also not authorized: the following rejection
suggests that raw objective subtraction may already lack enough resolution.

Preserve D7R FAIL. Freeze one replay-only observability discriminator over the
exact post-outer-9 state. It must expose initial gradient/stationarity and all
failed trust trials, including step/model/actual reductions, total-energy ULP
scale, topology/active-set stability and a direct per-term difference
counterfactual. It changes no solver state, tolerance, beta or route.
