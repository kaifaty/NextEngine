# NSR3-B4E2D7R19R64 sparse row-operator evidence

Date: `2026-08-25`

Status: `PASS / SPARSE_ROW_OPERATOR_BOUNDED_EQUIVALENCE_CANDIDATE /
ROLLBACK ONLY`.

Implementation commit: `f65ab2d8`.

Frozen identity SHA-256:
`a3cd93bb1dc785e7784ff048bc8989b19215e5bc010ffe7a859b3a098f3ffca2`.

## Result

R64 replaces the hypothetical dense all-row representation with two sparse
views of the unchanged R43 operator. Exact-order directed slots own row action;
unique particle coefficients plus particle-to-row incidence own transpose and
local overlap. No Hildreth solve, projection, nonlinear trial or dense Gram is
created.

```text
rows                                      6000
directed slots                          605144
unique row-particle entries             535588
particle incidences                     535588
degree minimum / maximum              44 / 113
distinct degrees                            53
rows below maximum                       5992
```

The R62 workspace topology remains
`cfcc7ebbeac9ce8d54093d3af109367e5b68b4aceadf11ed516dbfddf5c2d73f`.
The more detailed R29 operator-topology root is independently recorded as
`547065f0304ef2e24eba9c008d9c56888af0bf56d47a3a0f6a62f262cd04a2e8`.
The sparse operator root is
`45e2d6ced5eda523040cf04eeff7f8f38a8c9c90c47637b642aba0949e2b6445`.

## Equivalence

```text
directed action exact                 12000 / 12000
transpose exact                        1737 / 18000
transpose maximum error          9.298117831235686e-16
transpose maximum bound         5.5304595104859081e-13
transpose maximum error/bound    0.0050337436850915471

captured gradient exact             8892000 / 8892000
captured diagonal exact                  76 / 494
diagonal maximum error/bound       0.0011887955982643503

overlap exact                       2859318 / 2964000
overlap maximum error           8.3266726846886741e-17
overlap maximum bound          3.4830702100616587e-13
overlap maximum error/bound     0.00071375300296245723
```

All 494 captured R51 gradients reproduce bit-for-bit. Different legal
addition groupings make the transpose, diagonal and some Gram overlaps
non-bit-exact; every value is inside the gamma bound frozen before nominal
execution. The selected classification is therefore bounded equivalence, not
bit-exact equivalence.

Offset, entry-particle, incidence-backreference and stale-epoch mutations all
fail closed. Work owns one fresh directed JVP, one fresh pair VJP, 494 captured
gradient/diagonal checks and 494 by 6000 overlap comparisons. Dense Gram
storage, new row VJP/Gram JVP, Hildreth, projection, HVP and nonlinear work are
all zero. Rollback is exact.

## Clean reproduction

```text
build A  /home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r64-final-a.yYpxKx
build B  /home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r64-final-b.yfr0D6

binary SHA-256  b7825c4187139531a7fdd4c7fd447c23787cce6f7cc495510df7f4ef8ea70ac9
binary bytes    8586816
ELF Build ID    55de9cffac3cc46ff49df24e5aaee8caa102b7dd

stdout SHA-256  ec83c0b93af1643deec854965bde1be121d21db513c54f24a67a3d2332e4e886
stdout bytes    2543
semantic        793597c8adf1f4d9fc602f552135e2a6fa36dc85643840d994aa6af334296ff2
route root      f9faba399caf76c1420b90d34be62c62d047033ff45ded4fe0f984049e8bacd8
route           SPARSE_ROW_OPERATOR_BOUNDED_EQUIVALENCE_CANDIDATE
```

Both clean binaries and stdout files are byte-identical. Each R64 process
also reproduces exact R63 parent stdout
`38298214e352a87cfffb5c5432be90ef822c3f8ab00da5a2f7a3b1cb64565b62`.
These are reproducibility checks, not shared-host timing evidence.

## Pre-PASS failures with no scientific credit

The first nominal binary crashed after a provenance mismatch left the
diagnostic operator empty while later comparison code still indexed it.
Diagnostic construction is now conditioned only on a valid workspace;
provenance independently controls credit and routing. Future harnesses must
not use an authority gate to suppress data required for a fail-closed report.

The subsequent diagnostic FAIL selected `SPARSE_ROW_TOPOLOGY_REJECTED` because
the implementation compared the R62 `al_r42_superset_root` identity with the
different R29 operator-topology canonicalizer. Both roots are now named and
validated separately. A degree-audit defect also counted after `std::unique`
without erasing its unspecified tail; counting before `unique` restores the
independently established R59--R61 value 5992. Neither failure changed the
frozen formula, bounds or contract identity.

## Consequence

R65 may research and freeze a dynamic all-row active-set projection using this
operator and retaining R63 as the full-vector reference. R64 grants no
nonlinear step application, filter/switching decision, runtime mutation,
timing or production authority.
