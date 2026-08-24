# NSR3-B4E2D7R19R38 -- direction-memory replay contract

Date: `2026-08-24`

Status: `FROZEN / REPLAY ONLY`.

Parent: `c30febef`, R37 stdout SHA-256
`f475c20302c444be1ca355a1c9f0ec39309bbe7cd33f4db73fad676bef1c3571`,
semantic `c6a254243c8d549f7f20322ed61321eb94ff3d551e75c850d9b1a549a2d14118`.

## Frozen identity

```text
nextengine.nonlocal.nsr3b4e2d7r19r38-direction-memory-replay|v1|parent=c30febef:f475c20302c444be1ca355a1c9f0ec39309bbe7cd33f4db73fad676bef1c3571:c6a254243c8d549f7f20322ed61321eb94ff3d551e75c850d9b1a549a2d14118|source=r37-checkpoints6,12,24;e404d5997578df09bab907e16f37e2344f4c754cb7951fad2aa01a0c43f31f4e;terminal-response-ff2ba63971c215a505b6b2c8b26b941410e631fa33110227e39560e67f3056de;gradient-a7cd95da69de32c3a50a6c1b856a7eda5248598ef835d460d38815c99819e675;mapping-64f004ed0238c57b1be68bb6b741673944a5a38a304da9e588e1fcc4d9549f85|lanes=steepest;prp-plus;dy-hs-plus;hager-zhang-raw|direction=raw-minus-gradient-plus-beta-previous-reference-direction;normalize;project-global-l2-ball;normalize-chord|restart=invalid-denominator-or-beta;nonfinite;raw-nondescent;projected-nondescent;fallback-steepest|line=exact-r32-piecewise-all-row-hinge;direct-kkt;no-update|selection=all-three-valid-no-restart-strict-objective-dominance-over-steepest;precedence-hz,dy-hs-plus,prp-plus,steepest|work=states3*(source-jvp1+lanes4*jvp1);pair-pass-cap15;vjp0;hvp0|controls=dense-spd-exact-line-equivalence;dense-active-switch-restart;parent;source;workspace;state;direction;line;work;rollback|routes=memory-parent-rejected;memory-source-rejected;memory-dense-rejected;memory-workspace-rejected;memory-state-rejected;memory-direction-rejected;memory-line-rejected;memory-work-rejected;hager-zhang-direction-candidate;dy-hs-plus-direction-candidate;prp-plus-direction-candidate;steepest-reference-retained|runs=2-clean-release-builds;1-process-each;byte-exact|recurrence=none;accepted-step=none;correction=none;nonlinear-evaluation=none;tolerance=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-direction-memory-replay-classification-only
```

SHA-256: `8ebdf87b7d9635b15d276f0b05e11cb5d81e5909ea96c12f80458c64cc1f6aa1`.

## Hard gates

1. Exact R37 parent, checkpoint root and captured states `6/12/24`, including
   current/previous gradients and previous accepted reference directions.
2. Independent dense SPD exact-line equivalence and active-switch/restart
   controls for every formula and route.
3. One exact nominal workspace. Fresh JVP at every state must reproduce the
   captured response within `1e-12` and state objective/violation/checkpoint
   roots must remain exact.
4. Exact parameter formulas:
   `PRP+=max(0,gTy/||g_prev||2)`,
   `HS=gTy/(d_prevTy)`, `DY=||g||2/(d_prevTy)`,
   `DY_HS_PLUS=max(0,min(HS,DY))`, and
   `HZ=((y-2*d_prev*||y||2/(d_prevTy))Tg)/(d_prevTy)`.
5. Invalid/nonfinite denominator or beta, nonnegative raw descent, or
   nonnegative projected-chord descent restarts that lane to the exact
   steepest direction before its JVP. Restart reason and pre-restart facts
   remain observable.
6. All 12 lanes require finite projected chords, exact line success, direct
   KKT, strict objective reduction and trust feasibility. No lane updates its
   source state.
7. Exactly 15 new pair passes, zero VJP/HVP/model/trial/outer work and one
   workspace lifecycle.
8. Exact rollback and every route case.

A memory lane is selected only under all-three-state, no-restart strict
objective dominance over steepest, in frozen precedence HZ, DY-HS+, PRP+.
Otherwise retain `STEEPEST_REFERENCE_RETAINED`. Selection authorizes only a
separately frozen bounded recurrence experiment.

Require two clean Release builds and byte-exact fresh outputs. Do not time,
accept or apply a step, evaluate nonlinear moved state, select a tolerance or
claim runtime/production.

Rationale:
[R38 research](../../development/nonlocal-nsr3b4e2d7r19r38-direction-memory-research-2026-08-24.md).
