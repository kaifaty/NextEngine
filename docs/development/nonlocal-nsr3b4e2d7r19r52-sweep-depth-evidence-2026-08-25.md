# NSR3-B4E2D7R19R52 fixed-master sweep-depth evidence

Date: `2026-08-25`

Status: `PASS / PERSISTENT_MASTER_SWEEP_DEPTH_CANDIDATE / ROLLBACK ONLY`.

Implementation commits: `ab3ad82e`, membership-audit fix `42a3307f`.

Frozen identity SHA-256:
`fde05c9b766f5bf81283adcfa9374cc60b490bd75c850e1fba599de70707aafd`.

## Result

R52 reproduces exact R51, reuses its 494-row cache/Gram and independently solves
the same R50-terminal anchored master at three frozen depths:

```text
sweeps                    16                   32                   64
predicted maximum         2.1583529353e-15     3.7729144151e-17     1.8707447850e-20
psi                       8.2797947288e-26     7.9348302019e-26     7.9325157689e-26
h                         4.0693475469e-13     3.9836742342e-13     3.9830932123e-13
maximum directed upper    7.8576367100e-14     7.7547306509e-14     7.7539986039e-14
raw active              480                  479                  219
directed positive       481                  480                  340
raw outside master        0                    0                    0
directed outside master   0                    0                    0
```

All three checkpoints strictly dominate R51 and none certifies compatibility.
The frozen certificate-first/smallest-strict rule selects 16 sweeps. The route
is `PERSISTENT_MASTER_SWEEP_DEPTH_CANDIDATE`.

The selected checkpoint improves R51 by:

```text
psi                       1.62573870677x
h                         1.27504459011x
maximum directed upper    1.13680161340x
```

The 64-sweep checkpoint is diagnostic only. It improves those metrics by
`1.69691219871x`, `1.30265582512x` and `1.15199583411x`, while reducing the
eight-sweep predicted master maximum by `1,286,235.94x`. This extreme mismatch
shows that coordinate depth is no longer the dominant certificate boundary.

## Closed alternatives

No checkpoint activates any raw or directed-positive row outside the exact
494-row master. Capacity expansion and external constraint generation are not
the next observed need.

At 64 sweeps, 219 rows remain raw-positive and 340 are positive under the
directed enclosure. The gap is therefore not solely the conservative gamma
bound. However, the nearly closed pair-once Gram residual and much larger fresh
per-row directed residual use different binary64 reduction orders. Their
operator-consistency error is now comparable to, or larger than, the physical
residual. More cyclic sweeps or a conjugate-gradient polish would solve the same
pair-once model more accurately without resolving that cross-fold discrepancy.

## Work, controls and rollback

All 494 basis and Gram entries are reused bit-for-bit. New operator work is
exactly three directed audit JVPs/pair passes. The reference executes 112 dense
coordinate sweeps and three ball-box projections. One moved workspace is built
and released with no lifecycle underflow or live owner.

Record, dense and route roots are:

```text
records a82f1f50ce5ebca0561befbf0e3bc9def0b1cfbea1f3516e59fa9ec498f9fb3b
dense   4bb913b1e75fae9b9b5950982a53344b14973859ce2119563abed9615fb54475
routes  db4f70efc55fc1478062f6183402cf94bf1fb890d2f04ee0f2bc6d6986e1309e
```

Selected witness root is
`5517c49b257b355386e657be2b9ea1a28d7cfb113018742a337ec228e8d3afe8`.
Semantic result SHA-256 is
`ad1efbe449d809039ad41404dc94736f8fcb536c48d724f38598f70d99cc744b`.

No witness, R43/filter/trust state or following outer is committed. Restoration
exit, timing, runtime and production authority remain false.

## Clean Release reproducibility

```text
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r52-final-a.tFGfrw
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r52-final-b.fud65x
binary SHA-256 a99eef319abd98d7c89acf465d751f0f33924dddfdd2c95775b49623aab98e3f
size           8024000
ELF build-id   aae7f808e9827cdf42561922a247758b66bcc007
stdout bytes   2753
stdout SHA-256 b1d439950f174b66b837041d077ed24fd19524bf2880ded37fc6392f9c65cf4e
```

Both independently built binaries and both concurrently executed stdout
payloads are byte-exact. Each reproduces exact R51 parent stdout SHA
`c0fc53e3c1637a84c0a9cc0494a8b9fc7bc6e60b627afe032337a37c14cbf960`.
Concurrent wall time is not admitted as performance evidence.

## Consequence

Retain 16 sweeps as the private fixed-master depth candidate, but do not grant a
runtime budget or apply its uncertified witness. Stop sweep extension and defer
active-face CG.

Research R53 as an operator-consistency/certificate decomposition at the exact
64-sweep diagnostic witness. Compare pair-once and directed row images, raw
values, summation envelopes and a higher-precision or compensated reference;
identify worst rows and proven sign margins. Do not weaken gamma, fit a zero
tolerance, widen the master or execute another nonlinear outer first.
