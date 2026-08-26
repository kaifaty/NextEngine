# NSR3-B4E2D7R20R63 binary64 Dot2Err evidence

Status: `REFUTED_BOUNDED / BINARY64_RIGHT_NONCONTRACTIVE`.

## Outcome

R63 is a valid negative result. The strict binary64 arithmetic apparatus passes
all profile, EFT, underflow, cancellation and fail-closed controls. Every one
of the 20,808 candidate intervals contains the independent exact-dyadic result.
However, independently projecting the already constructed binary128 matrix and
inverse to binary64 destroys the inverse relation itself.

The first frozen failure is the right defect:

```text
||I - A64 X64||inf exact outward = 1.8214455886679097e15
||I - A64 X64||inf candidate      = 1.8214455886679992e15
```

The report-only left audit reaches the same classification even more strongly:

```text
||I - X64 A64||inf exact outward = 3.1513632037477018e15
||I - X64 A64||inf candidate      = 3.1513632037478830e15
```

Both values are far above the required strict bound `<1`. This is not a false
rejection caused by the compensated enclosure: the exact projected defects are
already noncontractive.

## Frozen execution

```text
command:
  /tmp/nextengine-r20r4-build/nonlocal-formula-reclosure \
    --nonlocal-al-generalization-v5-binary64-dot2err

status: FAIL
route: BINARY64_RIGHT_NONCONTRACTIVE
semantic SHA-256:
  70f298d4e5931bda4a2e3d6d6ff6123f2739f992a084cf204653a01265411496
stdout byte SHA-256, both executions:
  b54cd01e7318d54adcec50c43ddf1f113c7e4b0a078c28baaab0b9966359d472
```

The two complete executions are byte-identical. Their nonzero process exit is
the expected representation of the frozen negative route, not an apparatus
crash.

## Profile and controls

The admitted profile reports Linux x86_64 GCC, radix 2, 53 binary64 mantissa
bits, `FLT_EVAL_METHOD=0`, round-to-nearest, IEC-559 and gradual underflow.

All literal controls pass:

- minimum subnormal preservation;
- exact reconstruction by `TwoSum`;
- exact reconstruction by explicit-FMA `TwoProduct`;
- cancellation dot containment;
- underflow dot containment;
- contractive and noncontractive 1x1 controls;
- rejection of a deliberately shrunk interval.

Control root:
`7b127b6bb50bd220920cff02def47751d1f398d11c05865ca69a515d31fb52f5`.

## Immutable capture and projection

The R60 parent is reproduced at the exact case/material roots, dimension 102,
8 accepted iterations and 1,092 principal solves/transitions. The capture-only
refiner applies no replacement.

```text
projected matrix root:
  83c5bf929d36fc0c578e14c18f71b30305f7455348b98cc09939c78e41e63838
projected inverse root:
  9b5f37e184b15295b4cedf04dc145eae8c022aeb83f795724e1da0c442a233f5
```

Right and left audits each execute exactly 10,404 candidate dots and 10,404
exact-oracle dots. Containment is `10404/10404` on both sides. The maximum
reported dot bound is `3.931862450255724`; maximum exact dot error is about
`5.50023e-2`, and the maximum exact-error/bound ratio is about `0.07497`.
Thus the intervals are conservative but valid; tightening them cannot turn an
exact norm of order `1e15` into a contraction.

## Parent regression

After R63, all required parents remain byte- and semantic-exact:

| Parent | Stdout SHA-256 | Semantic SHA-256 | Route |
|---|---|---|---|
| R60 | `4b87849b...f83ef` | `c8f11806...1ae2a` | `DIMENSION_GENERIC_CENTER_CANDIDATE` |
| R59 | `461c4962...44ac` | `a0881eaa...faa9` | frozen 5/6 blind boundary |
| R50 | `cdb2128b...6e61` | `190ac441...d86e` | `GENERIC_VERIFIER_TORSION_CANDIDATE` |

## Interpretation and ceiling

R63 refutes only the direct projection route
`(A128,X128) -> (round64(A128),round64(X128))`. It does not refute
`Dot2Err`, because exact containment passes, and it does not yet prove that no
scaled or directly constructed binary64 preconditioner can contract.

The next research must first decompose the projected defect into matrix- and
inverse-projection contributions and inspect conditioning/scaling. It may not
proceed to the planned R64 centered/sign certificate, refine the inverse inside
R63, run a trajectory, restore runtime binary128 or claim performance or
production readiness.

