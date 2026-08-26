# NSR3-B4E2D7R20R63T common-residual recoverability evidence

Status: `PASS / RETAINED_WIDE_COMMON_RESIDUAL_REFINEMENT_REJECTED`.

Claim status: `SUPPORTED_EXACT` for the rejection of both frozen binary128
correction lanes under the 16-update budget.
Evidence classes: `EXACT_CERTIFICATE`, `CORRESPONDENCE`, `NUMERICAL`.

## Reproducible result

Implementation commit: `0a566d57`.

Command:

```text
/tmp/nextengine-r20r4-build/nonlocal-formula-reclosure \
  --nonlocal-al-generalization-v5-common-residual-recoverability
```

Two valid independent executions are byte-identical:

```text
stdout sha256   4b71247bbf51f89e59ccbe1e671641381b31647a22fdef92e7a643f511cc73f5
semantic sha256 8aa1e245058243cf1d33fd00c7d6688082fe7c708c532fb5f1fb9fd0454fff2c
route           RETAINED_WIDE_COMMON_RESIDUAL_REFINEMENT_REJECTED
controls root   97b1e3f6d12f707a13be480a8ed528efe947efa95ce3b73a94039eb35ad19f69
```

The first apparatus execution incorrectly interpreted the physical tangent
width `matrix.columns=315` as the square common dimension 102 and stopped
before correction work. It has no scientific credit. The valid implementation
uses `rows=102` plus 10,404 oracle numerators, matching the R63S common
certificate. The invalid premise and repair are recorded in task state.

Platform, controls, exact R63S replay, residual projection, all correction and
update Dot2 reductions, all independent certificates, fixed work and lifecycle
gates pass.

## Correction result

Neither lane obtains a complete sign certificate. Their first transactions
are nevertheless small and informative:

```text
lane       initial exact residual   ||computed Zr||inf   projection error
retained   5.4079562458e-19         4.4739959720e-4      6.1177404e-53
exported   4.8831084575e-19         1.0928340419e-3      6.8842315e-53
```

The independent exact common residual falls to approximately `2.617e-19`
after the first transaction. By updates 4--5 both lanes reach the same exact
center/certificate. Updates through 16 retain:

```text
residual             2.6168222052687032e-19
coarse error bound   2.7481562138470910e14
signs                12 positive / 24 negative / 66 unresolved
correction magnitude 1.3593502283902575e-17
projection error     4.9020939814341823e-53
```

The final certificate root is
`5880c053b186083462d166189b1b9503b3428c61269cf6c66e360622314faa40` for
both lanes. Later corrections are below the effective update spacing of the
large center and no longer change it. Lane roots are:

```text
retained  add6c05732d27cce2aa319603786bd875c711c0b96a2da005f60f537a8af4162
exported  83bab5732888bdb650b02c8609a629333d9b04f3eee38a3d820ed53dae2e22fd
```

## Why this changes the next question

R63S/R63T certify error using

```text
||e||inf <= ||Z||inf ||r||inf / (1-rho).
```

That is safe but replaces the actual residual image by its worst-case norm
product. R63T observes a computed `Zr` near `1e-3`, while the product-norm
radius is near `1e14`. The gap is about 17--18 orders. The failed correction
therefore does not establish that the current candidates are far from the
exact common solution; it establishes that binary128 center updates cannot
reduce the already small residual further under this representation.

The sharper exact identity is

```text
e = Zr + (I-ZH*)e,
||e||inf <= ||Zr||inf/(1-rho).
```

R63T's computed correction is not itself this certificate: its residual was
projected to binary128 and its matrix product was rounded. The next gate must
compute `Zr` exactly from the dyadic verifier and exact residual numerators,
then derive every sign without changing the candidate.

## Exact work and ceiling

The new stage performs 32 correction residual records, 32 dense verifier
applications, 332,928 correction Dot2 terms and 665,856 new exact common
products. Dense `Z` is an offline diagnostic. Stored dense `H/X`, R60 signs,
factor correction solves, sparse work, state changes and timing are absent.

R63B--R63S stdout files remain byte-identical to all frozen hashes. Build
passed. No runtime, GPU or production inference occurred.
