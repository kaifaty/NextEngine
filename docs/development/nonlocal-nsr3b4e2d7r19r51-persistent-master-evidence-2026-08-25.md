# NSR3-B4E2D7R19R51 persistent-master evidence

Date: `2026-08-25`

Status: `PASS / PERSISTENT_MASTER_CLOSURE_CANDIDATE / ROLLBACK ONLY`.

Implementation commit: `9474b422`.

Frozen v3 identity SHA-256:
`9785a2168fdcb436cc06ff7c20277284c9e8fd5d27288374469f936f95ef5fcf`.

V1 is an invalid workspace-lifecycle contract and v2 is an invalid
identity-encoding run. Neither receives solver credit.

## Result

R51 reproduces exact R50 and solves one stable ascending persistent master over
the union of every captured cache row and every terminal directed-positive row:

```text
R50 cached rows                 492
R50 terminal positive rows      488
persistent master rows          494 / 512
new rows                          2
Hildreth sweeps                   8
selected dyadic                   0
alpha                             1
predicted maximum                 2.4062191742798642e-14
terminal psi                      1.3460782774625938e-25
terminal h                        5.1885995749577626e-13
terminal maximum row upper        8.9325740894562227e-14
terminal directed-positive      481
positive outside master           0
route                             PERSISTENT_MASTER_CLOSURE_CANDIDATE
```

Only two of the 488 R50 terminal-positive rows were absent from the 492-row
cache. The exact 494-row union fits the unchanged capacity. The full projected
correction is accepted at `alpha=1`, and every remaining directed-positive row
belongs to the persistent master. This closes the external row-migration
hypothesis for this one-outer discriminator: constraints are no longer escaping
because an earlier inactive row was dropped.

The all-row certificate remains open. The predicted master maximum and the
fresh nonlinear directed maximum are positive, and 481 rows remain positive
under the authoritative enclosure. Master membership is not compatibility; no
infeasibility or restoration-exit claim is introduced.

## Comparison with R50

R51 improves the R50 terminal point by:

```text
psi                             33.1960967334x
h                                5.7616053955x
directed maximum upper          11.5285608485x
```

The witness norm changes only from `5.0436317754490575e-7` to
`5.0437202913522273e-7`, still negligible relative to the `0.03125` normal
radius. Contact/trust geometry remains inactive. The key discriminator is
structural: `outside_master_positive=0`, while the eight-sweep predicted
maximum remains positive. Research should therefore target convergence of the
fixed persistent master, not capacity growth or another current-only outer.

## Work, controls and rollback

Exact new operator work is two row VJPs, two Gram-column JVPs and one fresh
directed audit JVP: five pair passes. One moved workspace is built and released;
there are no live workspaces, lifecycle underflow or candidate-support builds.
The captured 492-row basis/Gram prefix is reused bit-for-bit.

Union, source-cache and extended-cache roots are:

```text
union          38d7a09afa7278d492e6c7981e7dd7359b482309d4d9df0efb01c84324e12e28
source cache   31a6713866e951e716b8fd77e40ae532c3d9d281d3b40b0d16cd8039c2c5112e
extended cache f4e9368971d4b6feeb13874d0279dc73fd54ebbb523c75c5c2dd303acfc58cd5
Gram           7a0358af3fda7c90c89a5cfb9abe125fadf9142fb5ce0215e94ee2e70a94dc48
solve          7b781e49af0fe471abd928aede62fb952af65ff14430910d480a6a2a7da35ee9
witness        72c570b9bb981625ee2036b5c88ff19e5a88bca662efb3474bd62ce1130a6e34
```

Sixteen precedence routes close at
`832615132c8b7cb6897babe31fe28119df78bf7fa59237d7d4c6013e5c780aac`.
Semantic result SHA-256 is
`65df360418fba870089415b922b50e96c3bba55e904b2f9eec4fadd1e096361a`.

No witness, R43 state, filter entry, trust state or following outer is
committed. Timing, runtime and production authority remain false.

## Clean Release reproducibility

```text
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r51-a.CATLZU
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r51-b.OyCH8D
binary SHA-256 d8ce5824945109d5d994f510eab8de7b667abdbc417540fa2f3f22bc6cc66726
size           7981136
ELF build-id   da9e7b334098e92fd0ffe1d9482875608b794c91
stdout bytes   1880
stdout SHA-256 c0fc53e3c1637a84c0a9cc0494a8b9fc7bc6e60b627afe032337a37c14cbf960
```

Both independently built binaries and both concurrently executed stdout
payloads are byte-exact. Each process reproduces exact R50 parent stdout SHA
`1ee396be61be90534573b8c476256d80b46ee803584617a6ab34900b8727b56d`.
Concurrent wall time is not admitted as performance evidence.

## Consequence

Preserving accumulated halfspaces is the correct structural direction for this
reference: it produces strict progress with only two new basis rows and no new
positive row outside the master. It still does not finish restoration.

Research R52 as a fixed-master convergence discriminator. Keep the exact 494-row
union, basis/Gram, ball-box geometry and all-row directed certificate fixed.
Compare a frozen higher-sweep zero-dual continuation against a principled
multiplier continuation or stronger exact QP reference before selecting one.
Do not widen capacity, add a second nonlinear outer, fit a positivity tolerance,
apply the witness or authorize restoration exit first.
