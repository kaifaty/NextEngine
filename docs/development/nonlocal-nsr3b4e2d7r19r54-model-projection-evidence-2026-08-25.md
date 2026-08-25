# NSR3-B4E2D7R19R54 model-to-projection evidence

Date: `2026-08-25`

Status: `PASS / BALL_BOX_PROJECTION_MODEL_REQUIRED / ROLLBACK ONLY`.

Implementation commit: `fc4bfdb2`.

Frozen identity SHA-256:
`14a4a9436b9e02babcadc9869c80749d77b1bf02fa5f02f1aefe52c091da91ca`.

## Result

R54 reproduces exact R53 and localizes the R52 64-sweep residual floor:

```text
stage                                  positive rows      maximum
recursive master predictor                       465   1.870744784984205e-20
direct binary64 u-G lambda                       464   1.870744785090228e-20
compensated binary128 u-G lambda                 465   1.870744782216179e-20
fresh upper(anchor)+A(correction)                464   1.870744776812350e-20
raw(anchor)+A(correction)                          2   1.548948870017089e-20
fresh raw(unprojected target)                      2   1.548916376520858e-20
captured raw(projected witness)                  219   7.753998207299572e-14
```

The Hildreth recurrence, direct Gram fold and fresh correction response agree
near `1.9e-20`. The unprojected target has only two positive rows, also around
`1.55e-20`. The post-solve ball/box projection is the first transition that
recreates the observed `~7.754e-14` floor and 219 positive rows.

The frozen largest-exact-gap route is
`BALL_BOX_PROJECTION_MODEL_REQUIRED`.

## Stage gaps

```text
recursive vs compensated direct Gram   2.071433183648797e-27 row 1780
binary64 vs binary128 direct Gram       5.170773032061458e-28 row 1646
direct Gram vs correction response      9.077477551658487e-28 row 1102
anchor linearity vs fresh target        5.551115123125783e-17 row 619
unprojected target vs projected witness 2.907867455557039e-13 row 428
```

The projection gap is `5238x` larger than the next stage gap. More Hildreth
sweeps, active-face CG, residual replacement or higher-precision Gram work
cannot remove a violation introduced after the solve.

At final worst row 848 the trace is:

```text
recursive predictor         3.253482420980897e-22
direct binary64             3.253485675198262e-22
direct binary128            3.253484386977567e-22
correction upper response   3.253475428684735e-22
anchor raw + correction    -3.641027973456574e-21
fresh target raw           -3.640835466152151e-21
projected witness raw       7.753998207299572e-14
anchor gamma bound          3.966375516325048e-21
```

The row is safely negative before projection and becomes the worst positive
row only after contact clamping.

## Geometry

The unprojected target violates 1125 active coordinate faces. Component-wise
clamping changes exactly the same 1125 components and reproduces the stored
projected witness bit-for-bit:

```text
target norm                 5.043728301070977e-7
clamped/projected norm      5.043728300410330e-7
projection displacement     8.163479043327088e-12
maximum component change    4.817265926852687e-13
maximum component index     26
ball outside                false
analytic clamp exact        true
```

The normal ball is inactive by more than four orders of magnitude. The entire
observed projection is the contact box clamp. Tiny coordinate changes matter
because the density halfspaces had already been balanced to `~1e-20`.

Roots:

```text
target        7bf5b35fa9cedb0aca615e994e1252b61c3b8a0d69db1d168a07d04f5b584203
projected     ccc1084988fb56a829649f60c292210f24723a4932874923640d5bdbf99e8729
displacement  bdb3ec8e28529c4ddfe02f5b6c6297e75e7361ac72428fed1af3f324263cce45
comparison    204e8289a677039ddb76aa69d1418fdc37e17a8d2ce34fd7fa151dd6d8703076
dense         1a75851ff17665fd09756446bcdc17e4ad651bbcebb01f3e849512c05c605295
routes        596eb293709fd126b0a90a5c5c84f353ddbf7e1566f333cfae9986f6180f1694
semantic      9d0b1b8cc165ae84463386751414fe5614f9a62af7117609cd575f4573f14e74
```

## Architectural consequence

The current sequence solves density halfspaces in unconstrained correction
space and only afterward projects the result into the contact box. These two
operators do not commute at the required accuracy. Contact feasibility must be
part of the minimum-norm correction problem itself:

```text
minimize 0.5 ||p||^2
subject to density upper + A p <= 0
           lower - anchor <= p <= upper - anchor
```

The normal ball can remain an external safety gate in the first experiment
because it is decisively inactive here. A production design must eventually
handle it coherently as well.

Research the smallest exact contact-constrained reference next. Compare an
implicit primal-dual Hildreth/Dykstra row-action method against extending the
dense density Gram with sparse coordinate-face constraints. Do not merely
repeat solve-then-clamp, fit a tolerance, or promote the current witness.

## Work, rollback and reproducibility

New work is exactly two pair-once JVPs, one compensated 494-row Gram fold and
one analytic geometry fold. No new row/Gram construction, solve, sweep,
projection call, audit, HVP, nonlinear trial or outer is executed. One moved
workspace is built and released. All parent state and certificate bytes roll
back exactly.

```text
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r54-final-a.dbRM3x
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r54-final-b.Rqh5ss
binary SHA-256 a47b34d15084d512ecb06f0c5167ded35d1219655d7ce88d71b0d3c7f95f8b98
size           8108680
ELF build-id   9ad381c9e178bd498d17c7b29bc5c43a25932262
stdout bytes   3831
stdout SHA-256 b2f32c0aa9e8dd689e3924d8a88db5c4c971404fbc3835bd313cf7512994edff
```

Both independent Release binaries and both concurrent outputs are byte-exact.
Wall time is not admitted as performance evidence. No witness, restoration,
filter, trust or runtime state is committed; production authority remains
false.
