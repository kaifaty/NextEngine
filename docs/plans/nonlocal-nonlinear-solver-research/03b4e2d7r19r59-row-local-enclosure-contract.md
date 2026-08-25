# NSR3-B4E2D7R19R59 -- row-local enclosure contract

Date: `2026-08-25`

Status: `FROZEN V2 / PRE-EXECUTION ROUTE REACHABILITY CORRECTED /
IMPLEMENTATION AUTHORIZED / ROLLBACK ONLY`.

Parent: `ccd2461e`, R58 stdout SHA-256
`8078c06230d6436253df966eb617bee202237143a1cf5def0e1dac49028ec257`,
semantic `f7070521395450cc3b54bfe85c9c4c2648abfa87ff37cfaa419da317628d74c3`.

## Frozen identity

```text
nextengine.nonlocal.nsr3b4e2d7r19r59-row-local-enclosure|v2|parent=ccd2461e:8078c06230d6436253df966eb617bee202237143a1cf5def0e1dac49028ec257:f7070521395450cc3b54bfe85c9c4c2648abfa87ff37cfaa419da317628d74c3:FIXED_POINT_CONTRACTION_CANDIDATE|source=r58-state-a4343378e4055fdc06ccb134dd44f3322473c93138960ff31965f27e8fd22af2;cycle64-record-86ad77c25f07d0337b0c2208ac208271c4d163d657c15711d5e78cfcd6c38e95;witness-040cc9f0dc57f543c8c3f9e1324b2e68dcc827e120782f9813a7aa0652d884bd;correction-7f5a7c242443b35c54616e4604531bca77c0dd3e2e7e21e850b105559a30fb1b;boxdual-da085eec472077b9ce00e8ce4881cfb3e9de52910cd5c8a60373fbf4fe3bde1b;comparison-a16fc80e265f227eb101a175b7427dce7f9bc3f2db62ae6ae6dffd6ca13247c2;active234;raw0;maximum-upper1.3678206846699582e-24;full128-positive0;negative6000;unresolved0;maximum-raw-5.1987499104498105e-22|scan=exact-r58-selected-cycle64;rows6000;degree=flat-offset-difference-including-radius-skips|current=gamma(16*maximum-degree+66)*(abs-c+directed-absolute-sum);raw=c+directed-value;upper=raw+bound|local=gamma(16*row-degree+66)*(abs-c+directed-absolute-sum);local-upper=raw+local-bound;same-gamma-function;same-inputs;same-binary64-order|gates=current-recomputed-bits-equal-captured-all-rows-and-maximum;degree<=maximum;local-upper<=current-upper-all-rows;finite;active=current-upper>0;local-active=local-upper>0|observations=degree-min-max-distinct;rows-below-max;current-positive;local-positive;current-positive-at-max-degree;local-positive-at-max-degree;strict-improved-rows;closed-current-positive;current-and-local-maxima-row-degree-bits;degree-histogram-root;row-comparison-root|selection=local-positive0-row-local-certificate;else-strict-and-local-positive-at-max-degree>0-eft-accumulation-research;else-strict-partial-staged-enclosure;else-no-improvement|controls=r58-parent-bytes;source;workspace;topology;master;witness;audit;degree;global-reproduction;local-monotonicity;comparison;work;rollback;route-precedence|routes=row-local-parent-rejected;row-local-source-rejected;row-local-workspace-rejected;row-local-topology-rejected;row-local-master-rejected;row-local-witness-rejected;row-local-audit-rejected;row-local-degree-rejected;row-local-global-reproduction-rejected;row-local-monotonicity-rejected;row-local-comparison-rejected;row-local-work-rejected;row-local-enclosure-certificate-candidate;eft-accumulation-research-required;staged-enclosure-research-required;row-local-enclosure-no-improvement|precedence=parent,source,workspace,topology,master,witness,audit,degree,global,local,comparison,work,certified,max-degree,partial,no-improvement|work=parent-r58-replays1;moved-workspace1;workspace-release1;row-scans1;new-pairpasses0;new-quad-row-passes0;new-density-sweeps0;new-box-blocks0;new-jvp0;new-vjp0;new-basis0;new-gram0;new-projection0;new-support-audits0;new-hvp0;new-model0;new-nonlinear-trial0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-r41-r42-r43-r44-r45-r46-r47-r48-r49-r50-r51-r52-r53-r54-r55-r56-r57-r58=unchanged;certificate-candidate=private-only;binary128=retained-offline-no-new-pass;gamma-definition=unchanged;operator-arithmetic=unchanged;normal-witness-apply=none;r43-restoration-commit=none;filter-runtime-commit=none;restoration-exit=none;switching=none;trust-update=none;following-outer=none;tolerance=none;capacity-change=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-row-local-enclosure-classification-only
```

SHA-256 (exact runtime literal, no terminal newline):
`bab527b0018f4265c8fedce046a07615dbd4fe95687eb6883487952bea14fb9e`.

## Hard gates

1. Reproduce exact R58 stdout, semantic, route, selected roots, counts and
   maxima before new work. Passive capture must not change R58 bytes.
2. Build/release one exact moved workspace and retain the stable topology and
   494-row master. Execute no new operator or binary128 traversal.
3. Derive each row degree only from its exact flat adjacency offsets. Count
   radius-skipped slots conservatively.
4. Recompute the current global-degree upper in the same binary64 order and
   require bit equality for every row and aggregate maximum.
5. Apply the unchanged `gamma_factor` to `16*row_degree+66`. Require every
   local upper no greater than current; publish the exact strict-improvement
   count for scientific route selection.
6. Publish the frozen degree/count/maxima/root observations and select the
   first scientific route by frozen precedence.
7. Preserve exact work and rollback. No solver iteration, third fixed-point
   outer, arithmetic change, witness application or timing.

Require two clean Release builds and byte-exact outputs. A complete local
closure is one private certificate candidate only; independent validation and
runtime integration remain separate work.

V2 correction occurred before any R59 execution. V1 made the declared
no-improvement route unreachable by simultaneously requiring strict
improvement as a hard gate; it has no execution or scientific credit.

Rationale:
[R59 research](../../development/nonlocal-nsr3b4e2d7r19r59-row-local-enclosure-research-2026-08-25.md).
