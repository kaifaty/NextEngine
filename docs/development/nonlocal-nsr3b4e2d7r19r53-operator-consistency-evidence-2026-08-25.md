# NSR3-B4E2D7R19R53 operator-consistency evidence

Date: `2026-08-25`

Status: `PASS / HIGH_PRECISION_RAW_RESIDUAL_CONFIRMED / ROLLBACK ONLY`.

Implementation commit: `6facf893`.

Frozen identity SHA-256:
`3dbdcd223d53816eeb3c196e03c20606aefe6c0427f3356157d869b7401aacc6`.

## Result

R53 reproduces exact R52 and evaluates the unchanged 64-sweep witness through
four row representations:

```text
representation             positive rows       maximum raw          row
pair-once binary64                    219   7.753998207299572e-14     848
directed binary64                     219   7.753998207299572e-14     848
binary64 terms / binary128 fold       219   7.753998207390028e-14     848
full frozen-coefficient binary128     219   7.753998207401504e-14     848
```

All 6000 full-binary128 row signs are resolved outside the conservative
binary128 forward envelope: 219 positive, 5781 negative and zero unresolved.
There are zero pair-versus-full128 sign disagreements. The fresh pair-once and
captured directed binary64 image roots are identical:
`f19cc1913ac2cf0061c9f6240f0f6378fc513108c77975e02cbd0f1f45f632ca`.

Maximum rowwise differences are:

```text
pair-once binary64 vs directed binary64        0                     row 0
directed binary64 vs binary128 term fold       2.775547288243752e-17 row 959
binary128 term fold vs full binary128          8.112395088285601e-25 row 1442
```

The route is therefore `HIGH_PRECISION_RAW_RESIDUAL_CONFIRMED`. Reduction
order, directed recomputation and local binary64 products do not explain the
remaining `~7.754e-14` maximum. More precision cannot turn this exact witness
into a feasible one.

## Certificate decomposition

The unchanged directed certificate has 340 positive upper rows. Of those, 219
are raw-positive and 121 are bound-only positives. Its maximum upper remains
`7.753998603944715e-14` at row 848. The maximum gamma envelope is
`1.9270494676623053e-13` at row 5959, but that is not the worst upper row.

The current gamma certificate remains authoritative. R53 neither tightens it
nor substitutes binary128 at runtime.

## Correction to the R52 interpretation

R52's `predicted maximum = 1.8707447849842052e-20` is the recursively updated
Hildreth master predictor, not a fresh pair-once evaluation of the projected
witness. R53 proves that a fresh pair-once JVP is bit-identical to the directed
JVP and retains all 219 raw-positive rows. The earlier attribution to
pair-once-versus-directed fold order is rejected by evidence.

The next discriminator must locate the first mismatch along:

```text
recursive Hildreth prediction
  -> directly recomputed u - G lambda
  -> JVP of assembled correction
  -> unprojected anchor + correction
  -> ball/box projected witness
```

This separates recurrence drift, Gram/application nonlinearity in binary64,
correction assembly, anchor addition and projection displacement. Do not add
more sweeps, CG, a fitted zero tolerance or weaker gamma before this attribution.

## Work, controls and rollback

New work is exactly one fresh pair-once JVP/pair pass and one compensated
binary128 directed row traversal. The captured directed audit is reused. No
new row VJP, Gram JVP, audit JVP, solve, sweep, projection, support audit, HVP,
model, nonlinear trial or outer is executed. One moved workspace is built and
released with exact ownership rollback.

Stable roots:

```text
term128     bb44c8c9a47c3d8a487722d80dfaffdd0f9d4098e09e2a3404e372ef1b506327
full128     161b254a648b1a559ca609d37c6b510f237ee84f3709ca7afc2291e28bb5a14c
comparison  5e060e5ae7be812480abc026b4d82c388e57d043377560dac6d4ba6e6bfc273e
dense       4ecee1823c3f8f57325ac25f6ea2a3249a943f52f4946d37ce66826420d07e85
routes      6a0266d25704ab8fe404dc9c6354822aecf6ba59b1ba588559d176097f08ab8d
semantic    4a778c2e546636c28f38d71a72f86e434c089fbc0a441b2babab1e519d86e80c
```

No witness, R43/filter/trust state or following outer is committed.
Restoration exit, timing, runtime and production authority remain false.

## Clean Release reproducibility

```text
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r53-final-a.lCGxap
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r53-final-b.u7vJTO
binary SHA-256 556bfd262e970e85b98cdaa56e29f4df97ad5904bba83676b642f865eca08bad
size           8058080
ELF build-id   0f08574087d61c53b4be3711c39ac55873d5f0ab
stdout bytes   2281
stdout SHA-256 490669f7533c6b160a8426b029ee458cf80dc97439872f9b16bbf077a94b531a
```

Both independently built binaries and both concurrently executed stdout
payloads are byte-exact. Concurrent wall time is not admitted as performance
evidence.

## Consequence

Close operator alignment, summation order and binary64 local arithmetic as the
dominant cause at this witness. Preserve the 16-sweep private R52 selection and
the exact current certificate, but do not apply either witness. Research one
read-only model-to-projection consistency decomposition next.
