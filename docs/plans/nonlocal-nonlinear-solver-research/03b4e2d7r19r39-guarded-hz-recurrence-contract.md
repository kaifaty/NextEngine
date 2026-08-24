# NSR3-B4E2D7R19R39 -- guarded Hager--Zhang recurrence contract

Date: `2026-08-24`

Status: `FROZEN / IMPLEMENTATION NEXT / ROLLBACK ONLY`.

Parent: `b36e0c33`, R38 stdout SHA-256
`7f574289496e70fb76886381c2b1460227f15a3f6d07214b0772dabf7247abb1`,
semantic `cc030c6559643952faf70dd3545f7f62a01133527ad5fdf8427159ed994ab7d3`.

## Frozen identity

```text
nextengine.nonlocal.nsr3b4e2d7r19r39-guarded-hz-recurrence|v1|parent=b36e0c33:7f574289496e70fb76886381c2b1460227f15a3f6d07214b0772dabf7247abb1:cc030c6559643952faf70dd3545f7f62a01133527ad5fdf8427159ed994ab7d3|source=r37-step24;checkpoint-463fc296b6adf7eb690c7ad4bbf05e07238424abe83ab0243cb6f7c28ac84be0;response-ff2ba63971c215a505b6b2c8b26b941410e631fa33110227e39560e67f3056de;gradient-a7cd95da69de32c3a50a6c1b856a7eda5248598ef835d460d38815c99819e675;mapping-64f004ed0238c57b1be68bb6b741673944a5a38a304da9e588e1fcc4d9549f85;r38-hz-record-185635ed433b785f5df56533c467a0ff2a13ab61c89ce9e8cddca9632777e18b|lanes=steepest;guarded-hager-zhang|history=previous-gradient-and-accepted-feasible-normalized-direction;restart-history-becomes-accepted-steepest|direction=r38-raw-hz;normalize;project-global-l2-ball;normalize-chord;restart-invalid-beta-or-nonfinite-or-raw-nondescent-or-projected-nondescent|line=exact-r32-piecewise-all-row-hinge;direct-kkt;accepted-linearized-lane-state-only|trajectory=max8;checkpoints1,2,4,8;exact-zero-only|selection=all-checkpoints-strict-objective-violation-mapping-dominance;memory-streak-at-least2;precedence=projected-stationary,guarded-hz-recurrence,hz-one-step-benefit-only,steepest-reference-retained|work=lanes2*(source-jvp1+line-jvp8+intermediate-vjp7+terminal-jvp1+terminal-vjp1);pair-pass-cap36;vjp16;jvp20;hvp0|controls=r38-dense-root;dense-spd-two-step-hz;parent;source;workspace;first-step;trajectory;checkpoint;direct;work;rollback|routes=recurrence-parent-rejected;recurrence-source-rejected;recurrence-dense-rejected;recurrence-workspace-rejected;recurrence-first-step-rejected;recurrence-trajectory-rejected;recurrence-checkpoint-rejected;recurrence-direct-rejected;recurrence-work-rejected;hz-projected-stationary;guarded-hz-recurrence-candidate;hz-one-step-benefit-only;steepest-reference-retained|runs=2-clean-release-builds;1-process-each;byte-exact|correction=none;nonlinear-evaluation=none;tolerance=none;public-state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-guarded-direction-recurrence-classification-only
```

SHA-256:
`acff94da15e985dad6695c64883893cdf3de6d571e9332e48f90b37b812b5fc2`.

## Hard gates

1. Exact R38 parent output/semantic result and exact captured R37 step-24
   checkpoint, response, gradient and projected-mapping roots.
2. Exact R38 dense-control root plus a separate two-dimensional SPD control
   where raw Hager--Zhang under exact line reproduces the two-step conjugate-
   gradient solution within a predeclared `1e-13` norm defect.
3. One exact nominal workspace. Each lane spends its own source JVP and must
   reproduce the captured response within `1e-12`.
4. Lanes are exactly `STEEPEST` and `GUARDED_HAGER_ZHANG`. HZ uses the R38 raw
   formula, normalized projected chord and restart precedence. Invalid beta,
   non-finite state, raw non-descent or projected non-descent restarts before
   the line JVP.
5. History is exactly `(gradient_used, accepted_feasible_normalized_chord)`.
   A restarted accepted steepest chord replaces history; a rejected or failed
   direction never does.
6. At most eight accepted linearized steps, checkpoints `1/2/4/8`, and exact
   zero projected mapping as the only early stationarity condition. Every step
   requires finite state, strict objective reduction, direct exact-line KKT
   and trust feasibility.
7. HZ checkpoint 1 must reproduce R38 state-24 HZ direction/line record,
   objective `1.3684753550918572e-20`, violation
   `1.6543732076480548e-10` and active count `260` exactly.
8. Fresh terminal JVP/VJP for both lanes; incremental/direct response defect
   `<=1e-12`; finite objective, violation, gradient and projected mapping.
9. Nonstationary full-horizon work is exactly `18` pair passes per lane and
   `36` total: `20` JVP, `16` VJP, zero HVP/model/trial/outer work. Early exact
   stationarity may only reduce its own lane work.
10. Exact parent/source/workspace/history/checkpoint/terminal roots, exact
    rollback and every route case.

Select `GUARDED_HZ_RECURRENCE_CANDIDATE` only when HZ has a non-restarted
memory streak of at least two, strictly dominates steepest objective,
violation and mapping at every common checkpoint, and uses no more work.
Exact HZ stationarity has precedence. Terminal benefit without the memory
streak is `HZ_ONE_STEP_BENEFIT_ONLY`; otherwise return
`STEEPEST_REFERENCE_RETAINED`.

Require two clean Release builds and byte-exact fresh outputs. Do not time,
apply either lane, evaluate a nonlinear moved state, select a tolerance,
mutate public state or claim runtime/production authority.

Rationale:
[R39 research](../../development/nonlocal-nsr3b4e2d7r19r39-guarded-hz-recurrence-research-2026-08-24.md).
