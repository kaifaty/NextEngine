# NSR3-B4E2D7R19R65 best composed-dual path evidence

Date: `2026-08-25`

Status: `PASS / COMPOSED_DUAL_PATH_REFERENCE_RETAINED / 16-OUTER DOMINANCE REFUTED`.

Implementation commit: `baceb525`.

## Strongest honest result

The completed-square dual is a correct executable inner merit on the frozen
fixture. The new trajectory accepts a strict safe dual-ascent line in all 16
outers, preserves direct/commit dual correspondence, positive cached-normal
reduction, recurrence, projection, KKT decomposition and rollback, and uses
fewer sparse terms than FISTA.

It does not strictly dominate FISTA after 16 outers. Therefore v10 is retained
as the correct reference transaction, not promoted as an acceleration result.

Route: `COMPOSED_DUAL_PATH_REFERENCE_RETAINED`.

## Terminal comparison

| metric | v10 | FISTA | ratio v10/FISTA |
|---|---:|---:|---:|
| maximum raw | `2.0835414728e-9` | `1.1823169687e-9` | `1.762x` |
| projected-gradient norm | `1.4052069981e-8` | `9.5382120868e-9` | `1.473x` |
| sparse structural terms | `467,826,594` | `596,971,680` | `0.784x` |

The solver is close but does not satisfy either strict terminal dominance
gate. This is a bounded negative result for 16 outers, not a rejection of the
composed dual or active-face PCG.

## Transaction evidence

- parent v9 semantic exact: `77cbed07...83af`;
- 16/16 accepted lines, 240 face-PCG products and 256 dyadic trials;
- 241 candidate projections: 15 fixed-density rejects, one normal-reference
  reject, zero composed-dual rejects and 240 admissible candidates;
- 528 `A^T`, 272 `A`, 17 committed all-row audits;
- 1,536,000 local completed-dual vector terms, reported separately;
- candidate/commit dual gaps are zero or at most `5.17e-26`, far below bounds;
- physical model prediction gaps are zero at all frozen checkpoints;
- final stationarity is `6.06e-24`, complementarity maximum `4.84e-14`;
- curvature, line, dual/model prediction, recurrence, reprojection, controls
  and rollback all pass;
- no v8 all-candidate fresh-JVP audit is used by selection;
- no timing, nonlinear trial, runtime mutation or production state exists.

Semantic result:
`f497ab0a61e846d95ec5d86cc4de0328a01b99b8d622fadf17a431aa4af7d32b`.

## Claim ledger

| Claim | Status | Evidence | Ceiling |
|---|---|---|---|
| completed-square dual can own inner selection | `SUPPORTED_BOUNDED` | 16 applied lines and exact committed direct dual | one linearized fixture |
| v10 strictly dominates FISTA at lower sparse work | `REFUTED_BOUNDED` | both terminal residuals remain higher | 16 outers only |
| strict primal-inertia gate should return | `REFUTED_BOUNDED` | exact safe dual trajectory remains coherent without it | inner convex layer only |
| current remaining work can be spent without fitting depth | `SUPPORTED_ANALYTIC` | four-outer worst-case bound remains below FISTA | structural terms, not wall time |

## Equal-work consequence

One additional outer has the deterministic sparse upper bound

```text
33 * 535,588                    transposed entry terms
+ 18 * 605,144                  action/audit slots
+ 2 * 535,588                   worst row residual/update terms
= 29,638,172.
```

Therefore four additional outers are guaranteed to end at most at
`586,379,282` sparse terms, below FISTA by `10,592,398`. Five additional
outers have no such guarantee. Freeze exactly 20 outers as the unique
equal-work completion test; do not search a depth grid.
