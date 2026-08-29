# NSR3-B4E2D7R19R65 equal-work composed-dual completion evidence

Date: `2026-08-25`

Status: `PASS / EQUAL_WORK_COMPOSED_DUAL_ACCELERATION_CANDIDATE`.

Implementation commit: `eddd71ad`.

## Strongest honest result

The frozen 20-outer composed-dual transaction strictly dominates the FISTA
reference in both terminal KKT residuals while using fewer sparse structural
terms. The complete v10 outer-16 prefix is exact, all 20 lines are accepted,
and every dual/model/recurrence/projection/rollback gate passes.

This is the first bounded R65 acceleration candidate. It is one 6,000-row
linearized binary64 fixture, not a production solver, runtime throughput claim
or proof of convergence on the nonlinear Nonlocal system.

Route: `EQUAL_WORK_COMPOSED_DUAL_ACCELERATION_CANDIDATE`.

## Decisive comparison

| metric | v11 outer 20 | FISTA | improvement |
|---|---:|---:|---:|
| maximum raw | `3.6408446365e-10` | `1.1823169687e-9` | `3.247x` lower |
| projected-gradient norm | `6.7513573313e-9` | `9.5382120868e-9` | `1.413x` lower |
| sparse structural terms | `584,699,855` | `596,971,680` | `12,271,825` fewer (`2.056%`) |

Actual work is below the predeclared worst-case ceiling `586,379,282` and the
immutable FISTA budget. No checkpoint or outer count was selected after
observing residuals.

## Transaction integrity

- v9 semantic exact: `77cbed07...83af`;
- v10 prefix exact: `f497ab0a...7d32b`;
- v11 semantic:
  `8994703f102ef4124994c8ee0c8f2b1b255541bdce39b74fe3b1e2e66585ae9f`;
- 20/20 accepted lines, 300 face-PCG products, 320 dyadic trials;
- 303 candidate projections: 17 fixed-density rejects, one normal reject,
  zero composed-dual rejects and 302 admissible candidates;
- 660 `A^T`, 340 `A`, 21 committed all-row audits;
- 1,932,000 completed-dual local vector terms, separately accounted;
- outer 20 selects `alpha=1`, applies 16 zeros and retains positive
  normal-reference model reduction `1.795e-14`;
- selected/committed dual and physical model gaps are zero at outer 20;
- outer-20 stationarity `7.49e-24`, complementarity maximum `1.27e-14`;
- curvature, line, dual/model prediction, recurrence, reprojection, dense
  controls and rollback are exact;
- no timing, nonlinear trial, runtime/public state or production authority.

## Claim ledger

| Claim | Status | Evidence | Ceiling |
|---|---|---|---|
| v11 dominates frozen FISTA terminal KKT pair at lower sparse work | `SUPPORTED_BOUNDED` | strict two-metric/work gates | one 6,000-row fixture |
| completed dual is a viable inner merit | `SUPPORTED_BOUNDED` | 20 safe applied transactions and exact direct commits | fixed convex TRQP only |
| outer count 20 is a production stopping rule | `NOT_SUPPORTED` | count derives only from comparison budget | requires independent scale-free stop research |
| R65 is production ready | `NOT_SUPPORTED` | no independent corpus, nonlinear integration, GPU or measured runtime | R66+ required |

Evidence classes: `ANALYTIC_WORK_BOUND`, `NUMERICAL`, `CORRESPONDENCE`.

## Next boundary

Stop depth experiments. Research an independent convex-TRQP holdout corpus and
a dimensionless KKT/dual stopping contract. The corpus must be frozen before
execution, use states not selected from the winning trajectory, and compare
against a higher-accuracy offline reference rather than only FISTA. Fixed 20
may remain a budgeted reference, never a runtime constant.
