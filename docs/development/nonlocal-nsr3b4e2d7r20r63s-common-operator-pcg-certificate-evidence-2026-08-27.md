# NSR3-B4E2D7R20R63S common-operator PCG certificate evidence

Status: `PASS / RETAINED_WIDE_COMMON_OPERATOR_PCG_REJECTED`.

Claim status: `SUPPORTED_EXACT` for the rejection of both frozen direct-PCG
lanes as common-operator solutions under the eight-update budget.
Evidence classes: `EXACT_CERTIFICATE`, `CORRESPONDENCE`, `NUMERICAL`.

## Reproducible result

Implementation commit: `ec247656`.

Command:

```text
/tmp/nextengine-r20r4-build/nonlocal-formula-reclosure \
  --nonlocal-al-generalization-v5-common-operator-pcg
```

Two independent executions are byte-identical:

```text
stdout sha256   15568ae0d50ea3285da6d12cc07ba80d650994bd95b833daaae71868c6404dd1
semantic sha256 97d992f18912ed9a223a17080820b72d4da54635534d96dba3dc9e5d71454e14
route           RETAINED_WIDE_COMMON_OPERATOR_PCG_REJECTED
controls root   48915ddf2007b39fe2450faaab14bafd366f25d687d22086165f473d21c71350
exact RHS root  9db272a7000883d5f61884f05a5c86c1465d272b585ce6e9f359f91b99c5898e
```

Platform, controls, exact R63R parent reconstruction, retained/exported
preconditioners and starts, all exact common residuals, fixed work and
lifecycle gates pass. The report status is `PASS` because the frozen
experiment executed exactly; its scientific route is the negative route
above.

## Exact common residual result

Both lanes remain finite for all eight updates. Neither obtains a complete
componentwise sign certificate:

```text
lane       best residual inf   best error bound   final signs   first pass
retained   5.2160533e-19       5.4778384e14       12+/24-/66?   none
exported   4.7123587e-19       4.9488643e14       12+/24-/66?   none
```

The final certificates are:

```text
retained residual  5.4079562458204771e-19
retained error     5.6793726876980030e14
exported residual  4.8831084574893430e-19
exported error     5.1281836508876086e14
```

Both final sign roots are
`4dc6f1bc8ba00c9bcfa22589b4b586324aa71cc9c811af04cd49d0b70f04150c`.
Thus the two finite candidates agree on every resolved component, but exact
intervals still cross zero for 66 components. Those components are not
reclassified from R60 signs and are not treated as structural zeros.

Iterations 6, 7 and 8 are byte-identical within each lane, including exact
residual, error and sign certificate roots. Increasing only the PCG iteration
budget therefore has no evidence-based path through this boundary.

Lane roots are:

```text
retained  2ec70f637f37e8927a86c071b24029e6d0bf4724968ba40609a46dbc23195376
exported  5dcc65f74b8769d5a53cf5db2937f4c2013dc4b92f0333bb42a791006bc3c255
```

## Exact work and independence

The complete stage performs 18 direct rectangular products, 16 factor
preconditioner solves and 187,272 exact common residual products. Each of the
18 certificates converts all 102 candidate components and derives signs by
strict exact comparison against the R63R inverse bound.

Candidate recurrence uses no dense matrix product. Certification uses no
stored dense `H`, dense `X`, R60 reference solution/signs or transported R63O
defect. No sparse construction, timing sample, runtime state or production
path is present.

## Meaning and next discriminator

R63S separates recurrence convergence from common-operator correctness. The
binary128 direct tangent recurrence reaches its own arithmetic fixed point,
but an order-`5e-19` exact common residual amplified by the verified
order-`1e33` inverse bound is insufficient in the weak direction. This is not
a PCG breakdown and cannot be repaired by accepting the recurrence residual
as its own oracle.

The next research must first establish mathematical recoverability under
common semantics: apply a separately bounded common-residual correction from
the R63R verifier to the frozen final candidates and recertify at fixed depths.
That verifier-driven experiment may diagnose the required weak-mode
correction but cannot authorize dense inverse use in runtime. Only after
recoverability is shown should a factor-based correction or wider finite
operator representation be frozen.

R63B--R63R stdout files remain byte-identical to all frozen hashes. Build
passed. No CPU/wall performance comparison, runtime state, GPU or production
inference occurred.
