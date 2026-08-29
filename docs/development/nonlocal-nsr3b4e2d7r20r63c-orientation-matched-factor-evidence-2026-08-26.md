# NSR3-B4E2D7R20R63C orientation-matched factor evidence

Status: `REFUTED_BOUNDED / BINARY64_LOWER_LEFT_NONCONTRACTIVE`.

Claim status: `REFUTED` for the immutable factor/profile.

Evidence classes: `EXACT_CERTIFICATE`, `NUMERICAL`, `CORRESPONDENCE`.

## Strongest result

Directly constructing separate orientation-matched inverse factors in
binary64 does not close both triangular systems. The upper/transpose factor
admits a strict left-inverse certificate, but the required lower-factor left
defect is genuinely noncontractive:

| Audit | Role | Exact projected norm | `Dot2Err` bound | Result |
|---|---|---:|---:|---|
| `I-X_L L` infinity norm | required lower left | `1.747387716959624` | `1.7473877169597274` | reject |
| `I-L X_L` infinity norm | report-only opposite side | `1.860858937681735e-13` | `1.865197201937785e-13` | contracts |
| `I-Z_R^T L^T` infinity norm, equivalently `I-LZ_R` one norm | required upper left | `1.045106551222288e-14` | `1.085915549519051e-14` | contracts |
| `I-Z_R L` infinity norm | report-only opposite side | `16.50323111130780` | `16.50323111130808` | reject |

Every interval contains its exact-dyadic value. The first frozen boundary is
therefore not an enclosure artifact and cannot be repaired by tightening the
same arithmetic.

## Frozen execution

```text
command:
  /tmp/nextengine-r20r4-build/nonlocal-formula-reclosure \
    --nonlocal-al-generalization-v5-orientation-matched-factor

status: FAIL
route: BINARY64_LOWER_LEFT_NONCONTRACTIVE
semantic SHA-256:
  39ca735ea4c51a0ecf21cf13a1c5a3653a94361724da9ecc57e8a7d4fd854aa1
stdout byte SHA-256, both executions:
  0ace54f4baa595a931db4a7da14464fca22b1cdfe68d4869212b50c39b56581b
```

The credited executions are byte-identical. Their nonzero exit is the expected
frozen negative route, not an apparatus crash.

## Construction and controls

The exact R63B factor is captured once at dimension 102 and projected root
`cc4c5ff0...67b6`. The two candidates are not projections of the old
binary128 inverse:

```text
column/forward candidate root  376874e5...09f9
row/transposed candidate root  9481e31f...2d3c
```

Each direct binary64 construction executes exactly 102 systems, 176,851 FMA
terms and 5,253 divisions. All operations are finite. Each of the four audits
executes 10,404 candidate dots and 10,404 independent exact-dyadic dots, with
`10404/10404` contained entries.

The literal well-conditioned factor passes both required orientations. A
diagonal that projects to zero is rejected, a deliberately shrunk dot interval
does not contain its exact value, and a post-binding factor mutation changes
the root. Hook lifecycle and work ledgers close; no new NNQP RHS, center,
replacement, state update or timing is admitted.

## Interpretation

R63C rules out a simple explanation that R63B failed only because it projected
one binary128 inverse. Direct binary64 row/back construction still produces a
large `X_L L` residual even though its opposite product is near machine
precision. The factor has a real orientation-sensitive numerical boundary at
this profile.

This does not prove that the physical constraints are inconsistent and does
not prove that every binary64 algorithm must fail. The matrix is a Gram form of
active projector rows, and the condition lower bound remains order `1e32`.
The next question is therefore structural: whether the 102 active rows contain
a canonically identifiable numerical or exact dependence that should be
represented as rank rather than inverted through normal equations.

Ordinary binary64 inverse-factor work stops here. A subsequent experiment must
capture the underlying active-row operator, declare exact rank semantics and
compare rank-revealing QR/pivoted Cholesky against a no-drop control before it
may solve any RHS. Runtime binary128 remains only a later rare fallback option.

## Parent regression and ceiling

R63B remains byte-identical at `320d6ea1...24216`, semantic
`28bfb856...160fc`, route `BINARY64_FACTOR_LEFT_NONCONTRACTIVE`.

R63C certifies only one factor/profile boundary. It authorizes no constraint
drop, rank threshold, scaling, NNQP RHS, center, trajectory, runtime/GPU,
performance or production change. R64 and R65 remain blocked.
