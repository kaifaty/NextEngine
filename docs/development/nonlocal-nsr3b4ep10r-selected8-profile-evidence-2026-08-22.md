# NSR3-B4EP10R selected-8 profile evidence -- 2026-08-22

Status: `FAIL / PROFILE_RELIABILITY_WARNING / NO_ROUTING_CREDIT`

## Result

The target process itself is exact:

- exit zero and empty collector stderr;
- extracted target stdout SHA-256
  `c47e9393b88208ade76274cff1ee66af00d7da6bef7a461df7b3575b6f4c2ee3`;
- common B4EP10I correspondence, observed team `8:8` and zero executor
  mismatch.

The profile is nevertheless inadmissible. Its header records:

```text
Collector Warning: Collection interval timer period was changed (1000 -> 0);
profile data may be unreliable
```

The frozen contract requires no reliability warning. B4EP10R therefore fails
before category aggregation or routing.

## Preserved diagnostic, not evidence

gprofng recorded 32.302 sampled CPU seconds over 5.884314823 seconds and a
1,900,544-byte PC profile. Its native synchronization trace is empty. The raw
function view assigns 60.91% exclusive sampled CPU to an anonymous libgomp
symbol and exposes separate HVP/evaluation/topology outlined functions, but
the reliability warning prevents using any of those percentages to select an
optimization.

This is a profiler-method failure, not a solver, exactness or B4EP10S failure.
The exact 8-worker scaling evidence remains valid.

## Reproducibility

- contract identity:
  `6a3b44a4012d23e9ff218e33ebfc896d4e61d566bc34affc42949f3ef68a4358`;
- implementation:
  `abb7a06bdc16c3a906ce3b0e5d85f19a250dd9bd`;
- experiment manifest SHA-256:
  `219e7b375baa4e19f844ce7556e151986dfc86ebe48395525ce34f78253b9da4`;
- report manifest SHA-256:
  `598771ad1068d23a10bc13f04f46ae9ab964b9e3a97dbe58b3161228ee5e6165`;
- header SHA-256:
  `3bfc069b1ff6740ac44b076cbba4109f8b9a37819566cffe1e7b0bde966bd2c1`;
- overview SHA-256:
  `f990b952dbf475625d867928c78458adc8b96f6e88ffc2fdf17c47050831f36a`;
- function view SHA-256:
  `b2820665f53ddb4be53ddfd555e1e53333c91dda0e5b72df2c8102ab4cdc3aa6`.

External artifacts remain outside Git under
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4ep10r.7ITGik`.

## Evidence attestation

Exact projection, without final LF:

```text
nextengine.nonlocal.nsr3b4ep10r-evidence|v1|identity=6a3b44a4012d23e9ff218e33ebfc896d4e61d566bc34affc42949f3ef68a4358|implementation=abb7a06bdc16c3a906ce3b0e5d85f19a250dd9bd|target=c47e9393b88208ade76274cff1ee66af00d7da6bef7a461df7b3575b6f4c2ee3|experiment-manifest=219e7b375baa4e19f844ce7556e151986dfc86ebe48395525ce34f78253b9da4|report-manifest=598771ad1068d23a10bc13f04f46ae9ab964b9e3a97dbe58b3161228ee5e6165|header=3bfc069b1ff6740ac44b076cbba4109f8b9a37819566cffe1e7b0bde966bd2c1|overview=f990b952dbf475625d867928c78458adc8b96f6e88ffc2fdf17c47050831f36a|functions=b2820665f53ddb4be53ddfd555e1e53333c91dda0e5b72df2c8102ab4cdc3aa6|duration=5.884314823;total-cpu=32.302;sync=0;profile-bytes=1900544|warning=collection-interval-timer-period-changed-1000-to-0;profile-data-may-be-unreliable|admission=target-exact;collector-stderr-empty;profile-reliability-fail|decision=reject-profile;internal-parallel-phase-timing-research
```

SHA-256:
`1ceaa1eda36280916a2fad0a17a65532c6db29e3ab91ebacbb07e2ba574d89aa`.

## Decision

Reject gprofng clock attribution for this target. Preserve B4EP10S and
research opt-in internal parallel phase timing whose durations are excluded
from semantic evidence. No optimization is selected from the rejected
samples.
