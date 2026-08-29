# NSR3-B4E2D7R20R29 exhausted-line recovery evidence

Status: `PASS / V3_EXHAUSTED_RECOVERY_SOLVER_REJECTED`.

Implementation `685e940c` emits twice:

```text
3c4f5d5149d1510880f2d65b4c122e51fa441a6102d43f492a5c4e25930bba7b
```

The R26 19-step prefix, rejected step root and R28 recovery correspondence are
exact. The sole recovery re-derives scalar 168, 12 contributing entries,
`alpha_root=1.8908164557797987e-7`, `Bz=9.879651368079126e-33` and exact new-
side trial root `d5536891...53c1`. It applies at iteration 20 with one mask
change and no ball change.

Iteration 21 then accepts the unchanged full Newton step `alpha=1`. The KKT
tuple changes from roughly `4.343e-9` primal/dual mapping to:

```text
primal            0
dual mapping      1.7809093060570164e-18
complementarity   2.6090874500579383e-20
stationarity      0
gap scaled        7.1935228178421094e-20
```

This is a roughly 2.4-billion-fold dual-mapping reduction, but the frozen
certificate tolerance is `2^-70 = 8.470329472543003e-22`. Dual mapping,
complementarity and scaled gap remain approximately `2102.5x`, `30.8x` and
`84.9x` above it. The solver therefore correctly does not certify and
iteration 22 returns `GLOBALIZATION_REJECTED`. The candidate shear root is:

```text
3f4798c8882eee7226a67381414fc7bba9fc47a0ffa06a7f5a3b90e1bf83e473
```

Eleven of twelve cases remain certified and no historical path regresses.
R28 and R26 retain semantics `a95370b4...e52d` and `4203c7ce...a8d2`.

The one-shot crossing is strongly useful but insufficient for 12/12. This
result does not justify lowering `2^-70`, adding another recovery or extending
the cap. The next report-only audit must classify all iteration-22 margins and
mask events before any second candidate.
