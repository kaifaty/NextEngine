# NSR3-B4E2D7R19R65 proportioning/filter discriminator evidence

Date: `2026-08-25`

Status: `PASS / H2 SUPPORTED_BOUNDED / FILTER-MERIT CONFLICT CANDIDATE`.

Implementation commit: `97471ca3`.

## Strongest honest result

The frozen one-fixture discriminator supports the hypothesis that the R65
active face is already proportional when face-PCG is invoked, while the
fixed-density dual direction conflicts with the strict composed-inertia gate.
Every one of the 15 outers without an accepted composed-descent line is
free-dominant; none is chopped-dominant or stationary.

This is finite binary64 numerical and correspondence evidence. It selects the
next experiment; it does not prove convergence of a filter controller, justify
a production constant, authorize a solver step or generalize beyond the
frozen 6,000-row fixture.

Harness route: `FILTER_MERIT_CONFLICT_CANDIDATE`.

## Decisive evidence

At every post-Hildreth state the diagnostic uses

```text
g = -raw
free_i    = g_i         if lambda_i > 0, else 0
chopped_i = min(g_i, 0) if lambda_i = 0, else 0
```

and independently checks

```text
||g_P||^2 = ||free||^2 + ||chopped||^2.
```

| outer | free norm | chopped norm | chopped/free | line | class |
|---:|---:|---:|---:|:---:|:---|
| 1 | `4.3210298607e-6` | `0` | `0` | accepted | free |
| 2 | `3.2968199892e-6` | `2.2996954915e-6` | `0.6975496081` | blocked | free |
| 3 | `3.1561187950e-6` | `4.0914664705e-7` | `0.1296360098` | blocked | free |
| 4 | `2.3682797809e-6` | `4.2733693864e-7` | `0.1804419149` | blocked | free |
| 5 | `1.7648009264e-6` | `1.6985419200e-7` | `0.0962455252` | blocked | free |
| 6 | `1.0389567858e-6` | `1.0505869493e-7` | `0.1011194078` | blocked | free |
| 7 | `6.9623563899e-7` | `5.0278993292e-8` | `0.0722154835` | blocked | free |
| 8 | `4.3163623026e-7` | `2.7214964855e-8` | `0.0630506963` | blocked | free |
| 9 | `3.7619801506e-7` | `3.0748265126e-9` | `0.0081734257` | blocked | free |
| 10 | `3.0950528487e-7` | `5.0728885588e-10` | `0.0016390313` | blocked | free |
| 11 | `2.6927251637e-7` | `0` | `0` | blocked | free |
| 12 | `2.3682589567e-7` | `0` | `0` | blocked | free |
| 13 | `2.0765975997e-7` | `1.1154605747e-9` | `0.0053715779` | blocked | free |
| 14 | `1.8575627746e-7` | `0` | `0` | blocked | free |
| 15 | `1.6650901630e-7` | `0` | `0` | blocked | free |
| 16 | `1.5057028814e-7` | `1.8723927602e-10` | `0.0012435340` | blocked | free |

The largest blocked chopped/free ratio is still below one, and the ratio
falls close to zero in the later phase. This is the opposite of the frozen H1
prediction that face changes dominate the KKT violation.

## Integrity and controls

- v6 solver semantic is unchanged:
  `bd568e0f367d34ef75f5ebeeca085f6cd6c36bb9fd56cb6966965a629ae8f0e2`;
- diagnostic semantic:
  `9203252f9f330c4ce92bc2bcbe6ed091c361aadb915e28daf6a7ade26620ea03`;
- all 16 squared decompositions have zero recorded gap and satisfy the frozen
  `gamma(64*N+256)` bound;
- the positive scalar control returns `4 + 9 = 13`;
- the wrong-sign lower-bound control returns `20`, differs from `13` and is
  rejected;
- v6 operator/projection ledger is unchanged: 528 `A^T`, 272 `A`, 16 baseline
  and 239 candidate joint projections, 468,968,743 structural terms;
- candidate/commit model agreement, reprojection and rollback remain exact;
- no timing, nonlinear trial, runtime mutation or production path executed.

## Hypothesis ledger

| Hypothesis | Result | Reason | Ceiling |
|---|---|---|---|
| H1: face-PCG is premature/non-proportional | `REFUTED_BOUNDED` | all 15 blocked states have `chopped<=free` | one fixture and frozen ratio split |
| H2: proportional face but density/inertia conflict | `SUPPORTED_BOUNDED` | all blocked states are free-dominant while v6 has dual-decreasing candidates rejected by composed inertia | diagnostic correlation, not a causal theorem |
| H3: recurrence/projection artifact | `FALSIFIED_BOUNDED` | inherited zero candidate/commit gap and exact reprojection | accepted candidates on this fixture |
| H4: mechanism changes by phase | `REFUTED_BOUNDED` | no mixed blocked classification | one 16-outer trajectory |

Evidence classes: `NUMERICAL`, `CORRESPONDENCE`. There is no independent
continuum oracle in this discriminator.

## Consequence

Do not add a proportioning phase, increase PCG depth, tune the chopped/free
split or search another alpha ladder. The next bounded experiment must keep
the physical objective and linearized density violation as separate measured
coordinates for the already generated projected candidates. It must determine
whether strict composed-inertia rejection is hiding robust feasibility
progress or whether the PCG direction lacks a common local path.

Primary filter-SQP results justify only the bicriteria discriminator, not
their convergence theorem for this inner Dykstra composition. Dykstra/Hildreth
duality likewise explains why a block dual improvement need not be certified
by a monotone intermediate primal-inertia check; it does not authorize removal
of the current safety gates without measurement.
