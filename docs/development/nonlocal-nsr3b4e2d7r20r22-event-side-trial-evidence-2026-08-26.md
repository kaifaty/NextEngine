# NSR3-B4E2D7R20R22 event-side trial evidence

Status: `PASS / NEXT_REPRESENTABLE_FACE_REJECTED`.

Implementation `17dde13c` reproduces R21 semantic exactly and emits:

```text
3c39b724b35e097d2208d5f318d7680755360fff6d29923ce3d8c758a8229ad6
```

The analytic root retains a valid old/touching face in every state and has a
strictly positive Armijo margin. The immediate next binary128 alpha also has a
positive margin in every state. However, it changes the predicted projector
mask only at step 32. In steps 25--27 and 29--31 it produces zero mask changes;
there are no wrong-scalar or ball changes.

The alpha ULP ranges from about `1.18e-38` to `9.63e-35`. That perturbation is
smaller than the end-to-end rounding displacement introduced when the solver
reconstructs the multiplier and recomputes `target-A^T lambda`. Consequently,
`nextafter` of the analytic alpha is not a reliable new-face selector even
though the mathematical event and dual acceptance are sound.

This is a finite-arithmetic representation failure, not an Armijo/model
failure and not a refutation of R21. A fixed power-of-two ULP ladder can now
measure the first alpha perturbation visible to the actual projector graph.
No result-dependent epsilon or state update is authorized.

