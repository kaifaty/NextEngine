# NSR3-B4E2D7R20R63W finite residual-image evidence

Status: `PASS / RETAINED_WIDE_FINITE_IMAGE_REJECTED`.

Claim status: `REFUTED` for the frozen componentwise residual-interval
mechanism on both state-2 candidates. Evidence classes:
`EXACT_CERTIFICATE`, `NUMERICAL`, `CORRESPONDENCE`.

## Reproducible result

Implementation commit: `d7a29758`.

Command:

```text
/tmp/nextengine-r20r4-build/nonlocal-formula-reclosure \
  --nonlocal-al-generalization-v5-finite-residual-image
```

Two fresh executions are byte-identical:

```text
stdout sha256   1e85677e07e22d91fc8b3ca7f9a4b7f9bdfbafcb7a404b4797854c1fc531c2fe
semantic sha256 35a74726a47d377ec8f97aac77dc52b22fb2463de1b33d77e6ea1fa63b48d2c3
route           RETAINED_WIDE_FINITE_IMAGE_REJECTED
controls root   b4500c02f25ac9d967f98c8dcc71c8264b3561f9adb8c4a7fbbc2612c8d18612
profile root    917f70e014602255e23036cce0037ba8dd47fa1a196a3656f37e614464806744
```

The first two apparatus executions used a magnitude-only dyadic comparator
for signed interval endpoints. They stopped before solver work and have no
scientific credit. The final apparatus defines an explicit signed comparison,
passes identity, propagation, cancellation, orientation, dropped-radius and
invalid-input controls, and reconstructs R63V exactly.

## Containment succeeds

The finite profile encloses all 10,404 common-matrix entries and the exact left
defect with:

```text
rho_upper             9.061521879470768e-3
maximum matrix radius 4.235258916316070e-36
matrix center root    85ee9d3845db489975f886fe2585846a59c873ad5255f82eea16d148124e9d71
matrix radius root    88849e7f29ec11bc2f6baf5fbfb61966ed05f972af8d6cbffb1a10f47c116027
```

Across both lanes and states `0..2`, all 612 exact residual entries and all
612 exact residual-image entries are contained. Thus the finite arithmetic,
outward rounding and exact correspondence apparatus are sound for the frozen
corpus.

## The selected mechanism fails

Despite containment, state 2 remains at `12 positive / 24 negative / 66
unresolved` in both lanes:

```text
lane       finite state-2 error       exact R63V error
retained   5.795446506391547e14       4.438606313181393e-4
exported   5.795446506391547e14       1.010447802166550e-3
```

The finite radius is roughly 18 orders too large. The failure is not ordinary
Dot2 rounding and not missing matrix containment. R63W first encloses each
residual component independently and then propagates
`sum_j |Z_ij| residual_radius_j`. That destroys the correlated sign/cancellation
pattern which makes the exact `Zr` small, so the `1e33` inverse sensitivity
reappears.

This refutes componentwise residual intervals as the selected finite image
mechanism. Raising precision alone is not the evidence-backed next action:
the dependency loss is structural.

## Work and ceiling

The final run performs 612 finite residual Dot2 operations, 612 finite image
Dot2 operations and exact containment audits only. It performs zero exact-
oracle classification uses, candidate updates, sparse constructions or timing
samples. R63B--R63V stdout hashes remain exact.

The next discriminator should preserve correlation by evaluating the affine
image `Zb-(ZH*)x` directly in one compensated dot per output row, with finite
center/radius data independently built from the exact common model. Even a
success there is a dense offline arithmetic result, not runtime/GPU/production
authority.
