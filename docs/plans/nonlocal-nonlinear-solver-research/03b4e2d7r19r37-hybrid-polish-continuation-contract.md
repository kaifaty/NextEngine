# NSR3-B4E2D7R19R37 -- hybrid polish continuation contract

Date: `2026-08-24`

Status: `FROZEN / REPORT ONLY`.

Parent: `eb50f25c`, R36 stdout SHA-256
`13566a2dc1ed03336e906ba68a6a2c80af84cb98a38981e61e22595b50c07550`,
semantic `6efda6f07993f4e4cf067c894bd2ffee7b0bf3f10585ad043aba6054008bae99`.

## Frozen identity

```text
nextengine.nonlocal.nsr3b4e2d7r19r37-hybrid-polish-continuation|v1|parent=eb50f25c:13566a2dc1ed03336e906ba68a6a2c80af84cb98a38981e61e22595b50c07550:6efda6f07993f4e4cf067c894bd2ffee7b0bf3f10585ad043aba6054008bae99|source=r36-endpoint;response-3bcaf5f90958f743eda5a4dccad83cc1b844a3c12083ac15292cb8f79a08e84b;gradient-f4bf70ad53ee6410a21d756e80558fe01d1eb47423fbec8dcd277d138d0b0558;mapping-54d472d43715c3592cc1e01283ddc79709e810ed23994f8f0f26e87dbf450e78;phi-3.5527997925597766e-17;violation-8.4294718607511539e-9;mapping-norm-7.4709090069224081e-10|model=all-row-squared-hinge;trust-global-l2-delta0.25|continuation=unchanged-r33-projected-normalized-steepest;exact-r32-piecewise-line;additional-max24;checkpoints6,12,24;exact-zero-projected-stationarity|terminal=fresh-jvp;fresh-vjp;response-relative<=1e-12;checkpoint-objective-violation-mapping-block-contractions;no-fitted-tolerance|work=prefix1+polish48+terminal2;pair-pass-cap51;hvp0|controls=dense-conditioned-continuation;dense-active-switch;dense-stationary;parent;source;workspace;prefix;iteration;checkpoint;direct;work;rollback|routes=continuation-parent-rejected;continuation-source-rejected;continuation-dense-rejected;continuation-workspace-rejected;continuation-prefix-rejected;continuation-iteration-rejected;continuation-checkpoint-rejected;continuation-direct-rejected;continuation-work-rejected;hybrid-polish-projected-stationary;hybrid-polish-mapping-regressed;hybrid-polish-continuation-candidate|runs=2-clean-release-builds;1-process-each;byte-exact|correction=none;nonlinear-evaluation=none;tolerance-selection=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-hybrid-polish-termination-curve-only
```

SHA-256: `746d699d1f724f584dcd156838ea1500118e81000ee19a9d49d85de1aa3a574d`.

## Hard gates

1. Exact R36 parent and captured private endpoint, including response,
   gradient and projected-mapping roots and frozen terminal scalars.
2. Independent dense conditioned-continuation, active-switch and exact
   stationary controls.
3. One exact nominal workspace and fresh endpoint JVP with maintained/direct
   response defect `<=1e-12`.
4. Up to 24 unchanged R33 polish iterations. Every nonstationary iteration
   must strictly reduce the hinge objective, satisfy direct line KKT and stay
   inside the global `0.25` L2 trust ball.
5. Checkpoints after `6`, `12` and `24` additional steps. Mapping at 6/12 is
   formed from the next iteration's already-owned gradient; checkpoint 24 is
   the fresh terminal mapping. Exact stationarity may stop early only when the
   projected mapping is binary64 zero.
6. Fresh terminal JVP/VJP, response defect `<=1e-12` and finite direct
   objective, violation, gradient and mapping.
7. At most 51 new pair passes, zero HVPs, one workspace lifecycle and zero
   nonlinear models/trials/outers.
8. Exact rollback and all route cases.

If all 24 steps complete, terminal mapping strictly below the R36 parent
selects `HYBRID_POLISH_CONTINUATION_CANDIDATE`; otherwise a finite valid
nonzero result selects `HYBRID_POLISH_MAPPING_REGRESSED`. Exact zero selects
`HYBRID_POLISH_PROJECTED_STATIONARY`. These classifications do not choose a
production tolerance.

Require two clean Release builds and byte-exact fresh outputs. Do not time,
apply state, evaluate nonlinear moved state or claim runtime/production.

Rationale:
[R37 research](../../development/nonlocal-nsr3b4e2d7r19r37-hybrid-polish-continuation-research-2026-08-24.md).
