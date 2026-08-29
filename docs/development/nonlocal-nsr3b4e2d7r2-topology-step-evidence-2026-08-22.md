# NSR3-B4E2D7R2 topology/step discriminator evidence

Date: `2026-08-22`

Status: `FAIL / KERNEL_HORIZON_SIGNED_ZERO / REPLAY_ONLY`

## Reproducibility

Two clean Release builds produce byte-identical 4,751,120-byte executables at
SHA `bd6fc67ff66a5ccdfc1c0962bbf58621620ef1b27f02387d086f8e0bbd978417`
and Build ID `e1d128217ac59e5a43c1c657c6bf78cf1ec1d4f4`.

Both fresh processes exit one with empty stderr and byte-identical
104,717-byte stdout reports at SHA
`f6b18542b1a25a548a11925552f014148152c817c8de557a86f9e97f4c628bcb`.
The semantic result is
`efd1f8d003a66a4c76544a17697c99c74232257ac2fde56caee520b6115904b3`.
Raw evidence is under
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4e2d7r2.49yqB3`.

The final executable preserves all parent failure reports exactly:

```text
D7   stdout SHA = e0542abc4e0c7ff0b38acc0fe38095e270dec030617ad5183665414ce9b3db11
D7R  stdout SHA = 4b0272df2d46bb53ec09175ea7095b0e8c699ac9e0e4e093c6e20dfd00240801
D7R1 stdout SHA = 9a9582d09897f63ed7cd9953cc2fb6153b4266813fb6797ba71a1de98140cbfe
```

## First failing gate

The replay, full Newton step, set accounting, all 21 alpha rows, finite values,
directional descent and rollback are exact. The command fails only the frozen
positive-zero bit gate at the horizon:

```text
W(h)   = +0.0
W'(h)  = -0.0
W''(h) = +0.0
```

The kernel gradient expression reaches zero through a negative factor and
therefore retains the IEEE sign bit. Numerically `-0.0 == 0.0`, but its bits
are not the positive-zero pattern literally required by the frozen contract.
The executable correctly returns `first_failure=KERNEL_HORIZON` and emits no
authorized route. Changing the diagnostic or canonicalizing the physical
kernel after execution is not permitted.

## Non-authoritative observations for reclosure

The complete report nevertheless narrows the subsequent contract:

- all eight PHR centres remain active at every moved alpha;
- all 28 fluid pairs remain unchanged;
- the full step adds 72 and removes 120 boundary pairs;
- the current-branch and live objective differences are exactly equal at the
  full step, both `-4.528143341123291e-16 J`, so the horizon branch does not
  explain the ascent;
- the half step has no active, fluid or boundary membership change and gives:

```text
predicted reduction       = 9.9226915852551228e-16 J
raw actual reduction      = 1.3183898417423734e-15 J
direct actual reduction   = 1.3181955852256048e-15 J
direct/model ratio        = 1.3284657432912772
```

The original direction has negative gradient slope and needs 13 quarter-radius
shrinks before the radius binds, four more than the frozen nine attempts.
These observations do not authorize a trust-policy change because the D7R2
contract failed before route selection.

## Decision

Preserve D7R2 FAIL. Freeze a narrow D7R2R signed-zero reclosure under a new
identity. It must accept either IEEE sign only when all three horizon values
compare numerically equal to zero, publish their exact bits, reproduce the
entire D7R2 trace internally and preserve D7R2 stdout bytes. It may then emit
the already frozen route classification; it may not change the kernel or any
solver state.
