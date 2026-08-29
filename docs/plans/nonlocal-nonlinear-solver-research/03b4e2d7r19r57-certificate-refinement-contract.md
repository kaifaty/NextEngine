# NSR3-B4E2D7R19R57 -- certificate-aware refinement contract

Date: `2026-08-25`

Status: `CLOSED / PASS / CERTIFICATE_REFINEMENT_ENCLOSURE_FIXED_POINT_REQUIRED / ROLLBACK ONLY`.

Parent: `6b346ff6`, R56 stdout SHA-256
`e0ececa2d57448248e86d623d4c1e15e69abe68576c503e1e802ef990b7c46aa`,
semantic `625f8db0a1be011dd36737743487ad9ee54e629bfb8926c0fb130120c5b9f7ca`.

## Frozen identity

```text
nextengine.nonlocal.nsr3b4e2d7r19r57-certificate-aware-refinement|v1|parent=6b346ff6:e0ececa2d57448248e86d623d4c1e15e69abe68576c503e1e802ef990b7c46aa:625f8db0a1be011dd36737743487ad9ee54e629bfb8926c0fb130120c5b9f7ca:JOINT_WITNESS_HIGH_PRECISION_RAW_RESIDUAL_CONFIRMED|source=r55-state-0edef9f324518fd77716b8fbc420a1c9b1ccd4187fef28af222a0d2ea840c1b8;cycle64-record-dd2db6e5ffe417ac6fc2b5a6e1eae884607cb8400aacd4d9f3e2959b5945bc6d;witness-19da2a335377f195bc0ce110977d19f6fa30223054d02d485a0b790123270406;correction-8ae05f144d760cfcfd3232dc0113365dff7e3a680769442cbdef1e0c6977b387;boxdual-512c77152fb3f4e8d55603cb765f540dd36cc30e18bb2465e910112a11aacb7d;master-38d7a09afa7278d492e6c7981e7dd7359b482309d4d9df0efb01c84324e12e28;upper-active476;raw-positive308;full128-positive308;negative5692;unresolved0;maximum-upper4.1425104588214889e-20|problem=d0;min0.5*norm2delta;cycle64-directed-upper-plus-A-delta<=0-on-master494;box=contact-lower-minus-cycle64-witness<=delta<=contact-upper-minus-cycle64-witness;normal-ball-audit-only|solver=zero-initialized-grouped-box-dykstra;density-lambda0;box-correction0;delta0;stable-master-order;fresh-residual-after-box|depth=64;checkpoints8,16,32,64;no-tolerance-stop;first-certified-else64|oracle=unchanged-directed-audit-each-checkpoint;selected-fresh-pair-once-binary64;selected-compensated-binary128-directed-traversal;frozen-coefficient-binary128-recompute;current-gamma-unchanged|rows=checkpoint-psi-h-maximum-upper-raw-directed-outside-box-ball-roots;selected-full128-positive-negative-unresolved;bound-only;maxima;roots|controls=r56-parent-bytes;source;workspace;topology;master;geometry;dense-refinement;dykstra;box;ball;primal;binary128;selection;work;rollback;route-precedence|routes=refinement-parent-rejected;refinement-source-rejected;refinement-workspace-rejected;refinement-topology-rejected;refinement-master-rejected;refinement-geometry-rejected;refinement-dykstra-rejected;refinement-box-rejected;refinement-ball-rejected;refinement-primal-audit-rejected;refinement-binary128-rejected;refinement-selection-rejected;refinement-work-rejected;certificate-refinement-master-expansion-required;certificate-refinement-ball-integration-required;restoration-next-trqp-compatible-candidate;certificate-refinement-solver-polish-required;certificate-refinement-sign-unresolved;certificate-refinement-enclosure-fixed-point-required;certificate-refinement-reference-retained|precedence=parent,source,workspace,topology,master,geometry,dykstra,box,ball,primal,binary128,selection,work,outside-master,ball-integration,certified,raw-positive,unresolved,enclosure,reference|work=parent-r56-replays1;moved-workspace1;workspace-release1;density-sweeps64;box-blocks64;fresh-residual-jvp64;checkpoint-directed-jvp4;selected-fresh-pair-jvp1;binary128-row-traversal1;new-pairpasses69;quad-row-passes1;new-row-vjp0;new-gram-jvp0;new-basis0;new-projection0;new-support-audits0;new-hvp0;new-model0;new-nonlinear-trial0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-r41-r42-r43-r44-r45-r46-r47-r48-r49-r50-r51-r52-r53-r54-r55-r56=unchanged;certificate=current-directed-unchanged;binary128=offline-only;normal-witness-apply=none;r43-restoration-commit=none;filter-runtime-commit=none;restoration-exit=none;switching=none;trust-update=none;following-outer=none;tolerance=none;gamma-change=none;capacity-change=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-certificate-aware-refinement-classification-only
```

SHA-256 (exact runtime literal, no terminal newline):
`2a41c16b73c553a966c2e73f2a435a70f99a6d1eb766f359e78e247da69b5d9e`.

## Hard gates

1. Reproduce exact R56 stdout, semantic, route and transitive R55 cycle-64
   roots before new work. Internal capture must not change R56 or R55 bytes.
2. Rebuild/release exactly one moved workspace with unchanged stable topology
   and exact 494-row master identity.
3. Use the retained cycle-64 directed upper vector as the shifted halfspace
   source. Rounded report values, fitted margin and reconstructed witness are
   forbidden.
4. Start refinement `delta`, density multipliers and grouped-box correction at
   exact zero. Derive relative box bounds from the exact cycle-64 witness.
5. Execute exactly 64 grouped-box Dykstra cycles in stable master order, with
   one fresh pair-once residual refresh after every box block.
6. Audit the unchanged all-row directed certificate at cycles 8, 16, 32 and
   64. Select the first certified checkpoint; otherwise select cycle 64.
7. On the selected witness execute exactly one fresh pair-once JVP and one
   compensated/full binary128 row traversal. Binary128 is an offline sign
   oracle only.
8. Apply the frozen route precedence. No residual tolerance, gamma change,
   active-row truncation or post-terminal projection is admitted.
9. Exact work accounting and rollback. No witness/state/filter commit,
   restoration exit, trust update, following outer, arithmetic/runtime policy
   or timing.

Require two clean Release builds and byte-exact outputs. Any PASS is one
private certificate-aware refinement classification only, not a completed
restoration transaction, runtime permission or production evidence.

Rationale:
[R57 research](../../development/nonlocal-nsr3b4e2d7r19r57-certificate-refinement-research-2026-08-25.md).

Result:
[R57 evidence](../../development/nonlocal-nsr3b4e2d7r19r57-certificate-refinement-evidence-2026-08-25.md).
