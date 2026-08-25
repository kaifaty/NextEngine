# NSR3-B4E2D7R20R23 event-side ULP-ladder evidence

Status: `PASS / ULP_LADDER_MULTI_EVENT`.

Implementation `423d52ad` reproduces R22 semantic exactly and emits:

```text
48d97d2110b3d3d5c6ebeff24a46c5183bb58063005a7af168b30d383aa11987
```

Every event crosses inside the fixed 65-point ladder, changes exactly the
predicted scalar, retains ball activity and has a strict positive Armijo
margin. The first crossing powers for steps 25, 26, 27, 29, 30, 31 and 32 are
respectively:

```text
10, 14, 13, 4, 6, 11, 0
```

The required absolute alpha displacements range from `3.76e-37` to
`1.54e-33`; no fitted physical epsilon is indicated. Six ladders then retain
the first new-face mask throughout the remaining fixed powers. Step 32 changes
its mask again, so the predeclared no-later-event gate rejects the candidate.

R20 proves that the much wider surrounding bracket contains only the same
one-scalar event. The isolated step-32 result is therefore more consistent
with a zero-bound finite-arithmetic mask flutter than with a second geometric
breakpoint, but R23 did not record the full old/new run structure or the
preprojection scalar. That mechanism must be discriminated before selecting a
stable crossing rule.

No ladder sample was inserted or applied by the solver.

