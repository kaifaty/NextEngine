# NSR3-B4E2D7R19R63 -- cached-normal projection TRQP contract

Date: `2026-08-25`

Status: `CLOSED / PASS / TANGENTIAL_MASTER_EXPANSION_REQUIRED /
ROLLBACK ONLY`.

Parent: `e2e19892`, R62 stdout SHA-256
`acbfa4befbc91a1c59e0ab75937665e7747112fd118a66eea6753f1206195fae`,
semantic `61de28c308a1f9199c954a4c785fe88e767cabd96e71223d3503b0111145da43`
and route `RESTORATION_EXIT_TRANSACTION_CANDIDATE`.

## Frozen identity

```text
nextengine.nonlocal.nsr3b4e2d7r19r63-cached-normal-projection-trqp|v1|parent=e2e19892:acbfa4befbc91a1c59e0ab75937665e7747112fd118a66eea6753f1206195fae:61de28c308a1f9199c954a4c785fe88e767cabd96e71223d3503b0111145da43:RESTORATION_EXIT_TRANSACTION_CANDIDATE|source=payload-5995a2cca7c24b99b328d4c66af7f9c7a710cfe4e684d0af305f81f1e162b41c;position-0fb11d7feb63a38f285798bd1eb2aad49131fbe5ff72df48e2230b31b8ffcc50;normal-040cc9f0dc57f543c8c3f9e1324b2e68dcc827e120782f9813a7aa0652d884bd;filter-19c1311bf84aaa236a8ce4ec82359ffadb6db404a721ea804f66daabef472c19;radius0.0625;certificate-78472fcb347565bdcc4be443a3cbf220cc5459fd5959a1bcc1c8bd1d428e55bf;active-82086595b4ed35abd2bf7f7cf6107359eaa9fe92be80df7aad1174dbf043bdb3;master-38d7a09afa7278d492e6c7981e7dd7359b482309d4d9df0efb01c84324e12e28;cache-f4e9368971d4b6feeb13874d0279dc73fd54ebbb523c75c5c2dd303acfc58cd5|model=f=0.5*norm2x-minus-predicted;dimensionless-s;target=predicted-minus-r43-divided-by-spacing;hessian=spacing-squared-identity;trqp=min0.5*norm2s-minus-target;density=c-r43-plus-A-s<=0;contact-box-at-r43;global-l2-radius0.0625;composite-s=normal+tangential|consumption=r62-payload-consumed-once;normal-used-as-feasible-anchor-and-model-reference;normal-not-applied-alone;tangential=composite-minus-normal|solver=cyclic-hildreth-density-plus-dykstra-ball-box;initial-composite=target;density-lambda0;box-dual0;stable-r51-master494;captured-basis-and-gram;fresh-pair-once-residual-after-each-box;cycles64;checkpoints8,16,32,64;no-tolerance-stop|certificate=unchanged-r61-topology-owned-row-local-audit-each-checkpoint;binary64;candidate-positive0-only;raw-and-candidate-outside-master-counted;first-certified-model-reducing-checkpoint;else-terminal-classification|geometry=all-r43-contact-intervals;projection-ball-intersection-box-each-cycle;selected-or-terminal-composite-endpoint-contact-tests36000;trust-and-component-direct-audit|model-audit=normal-endpoint-inertia;checkpoint-composite-inertia;strict-reduction-no-fitted-tolerance;target-identity;no-phr-objective;no-hvp|controls=parent;source;capture;workspace;topology;master;model;geometry;dykstra;projection;certificate;contact;consumption;work;rollback;route-precedence|routes=cached-trqp-parent-rejected;cached-trqp-source-rejected;cached-trqp-capture-rejected;cached-trqp-workspace-rejected;cached-trqp-topology-rejected;cached-trqp-master-rejected;cached-trqp-model-rejected;cached-trqp-geometry-rejected;cached-trqp-dykstra-rejected;cached-trqp-projection-rejected;cached-trqp-certificate-audit-rejected;cached-trqp-contact-rejected;cached-trqp-consumption-rejected;cached-trqp-work-rejected;tangential-master-expansion-required;tangential-model-reduction-rejected;cached-normal-projected-trqp-candidate;tangential-projection-depth-required;tangential-certificate-refinement-required;cached-trqp-reference-retained|precedence=parent,source,capture,workspace,topology,master,model,geometry,dykstra,projection,audit,contact,consumption,work,outside-master,certified-model,certified,raw-positive,bound-only,reference|work=parent-r62-replays1;new-static-index1;new-r43-workspace1;workspace-release1;captured-row-vjp0;captured-gram-jvp0;density-cycles64;ball-box-projections64;fresh-pair-once-jvp64;checkpoint-directed-jvp4;new-pairpasses68;selected-contact-tests36000;new-hvp0;new-model-hvp0;new-nonlinear-density-trial0;new-switching0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-r41-r42-r43-r44-r45-r46-r47-r48-r49-r50-r51-r52-r53-r54-r55-r56-r57-r58-r59-r60-r61-r62=unchanged;r62-public-output=byte-exact;private-trqp-only;runtime-position-commit=none;normal-apply=none;tangential-apply=none;filter-runtime-commit=none;switching=none;trust-update=none;multiplier-update=none;hessian-update=none;following-outer=none;tolerance=none;capacity-change=none;binary128=none;timing=none;runtime=none;production=none|credit=one-private-cached-normal-projection-trqp-classification-only
```

SHA-256: `7cc6eb487c099cce6c08c4ef52c7474f304fdf4e8a127cc49c2926a22b4c38d8`.

## Hard gates

1. Exact R62 public bytes/semantic/route. Add only a passive capture; require
   unchanged R62 public output.
2. Exact private payload, R43 position, cached normal, filter, radius,
   certificate, R51 master and cache roots. Consume the payload exactly once.
3. Rebuild one exact R43 workspace/topology. Derive
   `target=(predicted-R43)/SPACING` and prove the pure-inertia projection
   identity against direct endpoint objective evaluations.
4. Rebuild R43 contact intervals and prove the cached normal is their certified
   feasible anchor inside radius `0.0625`.
5. Starting from target, execute exactly 64 stable-order Hildreth density
   cycles over the captured 494-row basis/Gram cache. Follow every cycle by one
   exact ball-intersection-box Dykstra block and one fresh pair-once residual.
   No new row VJP, Gram JVP or tolerance stop is permitted.
6. At cycles 8, 16, 32 and 64 run the unchanged topology-owned R61 audit.
   Record objective reduction from the normal, trust/box facts, raw/candidate
   positive counts and positives outside master. Select the first zero-positive
   checkpoint with strict model reduction; otherwise retain cycle 64.
7. Materialize only the selected/terminal composite endpoint and perform all
   36,000 R43-to-endpoint contact tests. Root composite and
   `tangential=composite-normal`; do not publish either to runtime state.
8. Route outside-master positives before candidate/depth/refinement. A
   certified candidate requires strict inertial reduction. Remaining raw
   positives select depth; zero raw with bound-only positives select
   certificate refinement. No tolerance or binary128 route is allowed.
9. Exact work, lifecycle, payload consumption, rollback and route precedence.

Require two clean Release builds and byte-exact outputs. PASS is one private
linearized TRQP classification only. It does not execute a nonlinear density
trial, switching/filter globalization, trust response, multiplier/Hessian
update, following outer, timing or production promotion.

Rationale:
[R63 research](../../development/nonlocal-nsr3b4e2d7r19r63-cached-normal-trqp-research-2026-08-25.md).

Closure evidence:
[R63 evidence](../../development/nonlocal-nsr3b4e2d7r19r63-cached-normal-trqp-evidence-2026-08-25.md).
