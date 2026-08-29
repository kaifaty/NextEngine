# NSR3-B4E2D7R19R55 contact-constrained Dykstra evidence

Date: `2026-08-25`

Status: `PASS / CONTACT_CONSTRAINED_DYKSTRA_CANDIDATE / ROLLBACK ONLY`.

Implementation commit: `32e98ed2`.

Frozen identity SHA-256:
`4195f54a1b18cbe53222b4d3a9da3c9004f401bcb9b528162b38d20b537cbe28`.

## Result

R55 solves the exact 494 density halfspaces and contact box in one grouped-box
Dykstra continuation. Every checkpoint is contact-feasible, inside the normal
ball and has zero raw or directed-positive row outside the master:

```text
cycles             8                16               32               64
h              2.77930656e-13   3.40062622e-14   4.96407572e-16   7.33683160e-20
psi            3.86227247e-26   5.78212934e-28   1.23210239e-31   2.69145489e-39
maximum upper  3.38921419e-14   4.33298289e-15   6.86330412e-17   4.14251046e-20
raw positive       479              477              475              308
directed positive  480              479              477              476
outside master       0                0                0                0
```

Fresh after-box master maxima contract strictly at every checkpoint. The
pre-box and post-box maxima remain close, showing that persistent box feedback
has removed the R54 destructive post-solve jump.

Relative to the R52 64-sweep projected witness, the R55 64-cycle checkpoint
improves:

```text
psi             2.9472965661944e13 x
h               5,428,900.96 x
maximum upper   1,871,811.47 x
```

The frozen smallest-strict rule selects cycle 8 as a private candidate. Cycle
64 remains the high-accuracy diagnostic checkpoint. No checkpoint is yet
certified, so no witness or restoration exit is promoted.

## What changed architecturally

R52 solved density and clamped contact afterward. R55 retains both density
dual corrections and a vector box correction across cycles:

```text
density halfspaces -> grouped box -> fresh A p
          ^                              |
          +------------------------------+
```

There is no post-terminal projection. Box feasibility is true directly in the
Dykstra state, the ball remains inactive, and all positive rows stay inside the
same master. This validates the joint density/contact formulation at the exact
reference boundary.

At cycle 64, `raw_active=308` but `h=7.34e-20`; 476 rows remain positive only
under the current directed enclosure. The next question is therefore numerical
certification of this new witness, not another architectural solve change.

## Work and roots

Exact new work is 64 density sweeps, 64 grouped box blocks, 64 fresh pair-once
residual refreshes and four directed checkpoint audits: 68 pair passes. Every
one of the 494 density bases/Gram columns is reused. There is no new row/Gram
construction and no projection call.

```text
state     0edef9f324518fd77716b8fbc420a1c9b1ccd4187fef28af222a0d2ea840c1b8
dense     3a2d9b973b464e9264cf4428bd46ec153e796103bd07447ff0fa00f36a4ef98c
routes    0f5bc0c2a619fa3cafc5d59339b044019fc0ab804e5bbb5b2b625764e80347e1
semantic  fb549fd34090aae8a573715991e7b30059b7c779c1e4123a07b652509fb3c83e
```

Cycle-64 roots:

```text
witness     19da2a335377f195bc0ce110977d19f6fa30223054d02d485a0b790123270406
correction  8ae05f144d760cfcfd3232dc0113365dff7e3a680769442cbdef1e0c6977b387
box dual    512c77152fb3f4e8d55603cb765f540dd36cc30e18bb2465e910112a11aacb7d
record      dd2db6e5ffe417ac6fc2b5a6e1eae884607cb8400aacd4d9f3e2959b5945bc6d
```

## Pre-execution control repair

The first executable attempt stopped at `CONTACT_WORKSPACE_REJECTED` with zero
density sweeps, box blocks and JVPs. Its synthetic 2D control already bounded
both components to `1e-15`, but additionally required the one-sided predicate
`x+y>=1`, which failed at the final rounding side. Replacing only that redundant
predicate with symmetric `abs(x+y-1)<=1e-15` admitted the intended known point.
The frozen identity, nominal algorithm, work budget and physical inputs did not
change. The pre-execution attempt receives no solver credit.

## Clean Release reproducibility

```text
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r55-final-a.pDfrvP
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r55-final-b.C3ToXP
binary SHA-256 31cc0d24b76d6ee431b76490e63d487d21b4ef96932c28ef178ff8b750249806
size           8158896
ELF build-id   9d26ba8440e62b2553e151b84f86be4cebfae548
stdout bytes   4493
stdout SHA-256 fce9bec7ef370caed4904cedf61851f0fb87bf539960134fabefbb9dc94d6db8
```

Both independent Release binaries and concurrent outputs are byte-exact. Wall
time is not performance evidence. Parent state, certificate, witness, filter,
trust and restoration state roll back exactly; runtime and production authority
remain false.

## Consequence

Retain grouped-box Dykstra as the joint correction reference and stop work on
solve-then-clamp. Do not extend cycle depth or optimize the 68 refreshes yet.
Research R56 as a read-only binary64/binary128 raw-and-enclosure decomposition
at the exact cycle-64 witness. Determine whether remaining positives are true
raw residual, cancellation, or conservative directed bounds before selecting
polish or a proved tighter certificate.
