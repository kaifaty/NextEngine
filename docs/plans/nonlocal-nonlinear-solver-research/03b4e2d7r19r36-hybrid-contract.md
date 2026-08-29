# NSR3-B4E2D7R19R36 -- equal-work curvature-polish hybrid contract

Date: `2026-08-24`

Status: `FROZEN / REPORT ONLY`

Parent: `a1b0f1e8`, R35 stdout SHA-256
`8f4161e6d4cf9ac24d1ee828dec7019ae0179bd5979c97ea8fec3f2910446fe9`,
semantic `bacfbc3d6a3a620d83bddac51cde4133169100e7300dd9850ae1851d6997c368`.

## Frozen identity

```text
nextengine.nonlocal.nsr3b4e2d7r19r36-equal-work-curvature-polish-hybrid|v1|parent=a1b0f1e8:8f4161e6d4cf9ac24d1ee828dec7019ae0179bd5979c97ea8fec3f2910446fe9:bacfbc3d6a3a620d83bddac51cde4133169100e7300dd9850ae1851d6997c368|source=v8-from-r33;r35-outer-prefix-b35a0e69d29316e6a53836e5c5d689d1ad6b915ea46dda968149459f587b1fa2,6457aeb488feff2b4526a1d227ecd13a6f0c37f7b4045e70102ce07eae189391,9520017434f95d8c1cabd9217dcfa88a1ac589efb03526420313bce9ff708c53;baseline-r34-phi-1.6477454905319591e-16;violation-1.8153487216135412e-8;mapping-1.4988794937300478e-9|model=all-row-squared-hinge;trust-global-l2-delta0.25|hybrid=curvature-outers3;cg-max5;undamped;then-projected-exact-line-polish6|work=prefix1+curvature36+polish12+terminal2;pair-pass-cap51;hvp15|comparison=strict-terminal-phi-violation-mapping-dominance-over-r34|controls=dense-curvature-prefix;dense-polish-stationarity;dense-active-switch;parent;source;workspace;prefix;curvature;polish;direct;work;rollback|routes=hybrid-parent-rejected;hybrid-source-rejected;hybrid-dense-rejected;hybrid-workspace-rejected;hybrid-prefix-rejected;hybrid-curvature-rejected;hybrid-polish-rejected;hybrid-direct-rejected;hybrid-work-rejected;all-inequality-projected-stationary;first-order-reference-retained;equal-work-curvature-polish-hybrid-candidate|runs=2-clean-release-builds;1-process-each;byte-exact|correction=none;nonlinear-evaluation=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-equal-work-hybrid-classification-only
```

SHA-256:
`2e5cd1cb1fc8722c7d75fdf81c59e0d999809d8a9c00886b5260de62b80e3054`.

## Hard gates

1. Exact R35 parent, R34 baseline and R33 `v8` prefix.
2. Independent dense curvature-prefix, polish and active-switch controls.
3. One exact nominal workspace and fresh `v8` prefix JVP.
4. Reproduce exact R35 outer records 1--3 with 15 generalized HVPs.
5. Execute exactly six unchanged R33 projected exact-line steps; every step
   has strict reduction, trust feasibility and direct line KKT.
6. Fresh terminal JVP/VJP and response defect `<=1e-12`.
7. At most 51 new pair passes, exactly 15 HVPs, one workspace lifecycle and
   zero nonlinear models/trials/outers.
8. Exact rollback and all route cases.

PASS selects projected stationarity, retains R34, or selects the hybrid. Hybrid
requires strictly smaller objective, violation and projected mapping than R34.

Require two clean Release builds and byte-exact fresh outputs. Do not time,
apply state, evaluate nonlinear moved state or claim runtime/production.

Rationale: [R36 research](../../development/nonlocal-nsr3b4e2d7r19r36-hybrid-research-2026-08-24.md).
