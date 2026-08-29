# NSR3-B4E2D7R20R63X composed affine-image evidence

Status: `PASS / TWO_LANE_AFFINE_IMAGE_CANDIDATE`.

Claim status: `SUPPORTED` for the frozen dense, offline, correlation-preserving
binary128 affine-image enclosure on both state-2 candidates. Evidence classes:
`EXACT_CERTIFICATE`, `NUMERICAL`, `CORRESPONDENCE`.

## Reproducible result

Implementation commit: `b61cde9b`.

Command:

```text
/tmp/nextengine-r20r4-build/nonlocal-formula-reclosure \
  --nonlocal-al-generalization-v5-composed-affine-image
```

Two fresh executions are byte-identical:

```text
stdout sha256   9e218dbd248fc079fd058a44753d5d09b4dbaa8ed5ac9695eca2be68ccf25bf7
semantic sha256 904614d78c1948f4af836167936bdc528a0c69a0ded7e85cfe9ae264be395a65
route           TWO_LANE_AFFINE_IMAGE_CANDIDATE
controls root   05aecfd9a92bde5b782f6f1a97e815634b48f672bc39dc1801ddca9c0594fed9
profile root    0505cdb98db31a7f087cbc23290b5e0adc964dce05f4e73a4429083d860e97d0
retained root   a52826e657f81c401306fe19451d9dccd99fd4a8d66b1c300bfb9e88bed61cb9
exported root   907c3780ed8e41ae64a74fc60df856a1368348ee96dccb6fe12bb97f73c3a1d1
```

The apparatus reconstructs the R63W parent semantic exactly. Identity,
high-cancellation, nonzero/dropped profile-radius, orientation, subnormal,
nonfinite, negative-radius, noncontractive-defect, oracle-independence, root-
mutation and classifier-precedence controls pass. R63B--R63W all retain their
frozen stdout hashes.

## Correlation-preserving finite certificate succeeds

The exact profile builder forms

```text
g       = Z b
M       = Z H*
image_i = g_i - M_i x
```

and projects `g` and `M` to independently contained binary128 center/radius
data. Each finite certificate then evaluates one compensated length-103
affine dot per output row. It never constructs an independently widened
residual vector.

The profile closes with:

```text
rho_upper             9.061521879470768e-3
maximum vector radius 2.772940427276558e-17
maximum matrix radius 2.751031236622535e-34
exact Zb products     10,404
exact ZH* products    1,061,208
```

All 612 finite image intervals across two lanes and states `0..2` contain the
independent exact R63V images. Exact values and signs are not available to the
finite classifier.

## Frozen state discriminator

```text
lane       state 0 error   state 1 error   state 2 error   state-2 signs
retained   4.11438e15      1.68289e15      4.43861e-4      24+/78-/0?
exported   4.08658e15      3.46642e15      1.01045e-3      24+/78-/0?
```

States 0 and 1 are rejected with 66 unresolved signs. State 2 passes in both
lanes, and both finite state-2 sign roots equal the independently frozen R63V
root `89b2908b...6094`.

The finite state-2 bounds reproduce the exact-image scale to the displayed
digits. Relative to R63W's `5.80e14` bound, the improvement is about 18
orders of magnitude without changing precision, candidates, theorem or sign
threshold. This supports the causal conclusion that R63W failed by destroying
cross-component correlation before multiplication by `Z`, not because the
Nonlocal common operator or binary128 arithmetic was intrinsically invalid.

## Work and ceiling

The final run performs six finite certificates, 612 compensated affine dots
and 61,824 outward matrix-radius terms. It performs zero intermediate residual
interval constructions, exact-oracle classification uses, candidate updates,
sparse constructions or timing samples.

This is deliberately a dense offline mechanism discriminator. Its exact
profile build is cubic and its finite profile stores a dense `102 x 102`
matrix. Success authorizes research of a factorized or sparse
correlation-preserving image enclosure only. It does not authorize a runtime
binary128 path, state-2 production stop, GPU implementation, corpus
generalization, performance claim or production promotion.
