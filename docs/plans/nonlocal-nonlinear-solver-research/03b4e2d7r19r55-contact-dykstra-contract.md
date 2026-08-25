# NSR3-B4E2D7R19R55 -- contact-constrained Dykstra contract

Date: `2026-08-25`

Status: `CLOSED PASS / CONTACT_CONSTRAINED_DYKSTRA_CANDIDATE / ROLLBACK ONLY`.

Parent: `76e72ba9`, R54 stdout SHA-256
`b2f32c0aa9e8dd689e3924d8a88db5c4c971404fbc3835bd313cf7512994edff`,
semantic `9d0b1b8cc165ae84463386751414fe5614f9a62af7117609cd575f4573f14e74`.

## Frozen identity

```text
nextengine.nonlocal.nsr3b4e2d7r19r55-contact-constrained-dykstra|v1|parent=76e72ba9:b2f32c0aa9e8dd689e3924d8a88db5c4c971404fbc3835bd313cf7512994edff:9d0b1b8cc165ae84463386751414fe5614f9a62af7117609cd575f4573f14e74:BALL_BOX_PROJECTION_MODEL_REQUIRED|source=r54-target-7bf5b35fa9cedb0aca615e994e1252b61c3b8a0d69db1d168a07d04f5b584203;projected-ccc1084988fb56a829649f60c292210f24723a4932874923640d5bdbf99e8729;displacement-bdb3ec8e28529c4ddfe02f5b6c6297e75e7361ac72428fed1af3f324263cce45;box-violations1125;target-positive2;projected-positive219;master-38d7a09afa7278d492e6c7981e7dd7359b482309d4d9df0efb01c84324e12e28;cache-f4e9368971d4b6feeb13874d0279dc73fd54ebbb523c75c5c2dd303acfc58cd5|problem=p0;min0.5*norm2p;density-upper-r50-plus-Ap<=0;box=lower-minus-r50-anchor<=p<=upper-minus-r50-anchor;normal-ball-audit-only|solver=dykstra-cyclic;sets=494-density-halfspaces-stable-master-order-then-one-box-block;lambda0;box-correction0;primal-correction0;retain-density-and-box-duals|operator=recursive-gram-inside-density-sweep;fresh-pair-once-residual-after-every-box;captured-basis-and-gram;no-post-terminal-projection|depth=single-continuation64;checkpoints8,16,32,64;no-tolerance-stop;directed-certificate-at-checkpoints|rows=master-predicted-maximum-before-box;fresh-master-upper-after-box;all-raw-positive;directed-positive;outside-master;box-feasibility;ball-norm;correction-and-box-dual-norms;roots|selection=first-certified-checkpoint;else-smallest-checkpoint-strict-r52-64-psi-h-maximum-upper-dominance;outside-master-before-candidate;ball-before-candidate;else-reference|controls=r54-parent-bytes;source;workspace;topology;master;geometry;dense-dykstra;lambda;gram;box-block;residual-refresh;checkpoints;directed-certificate;selection;work;rollback;route-precedence|routes=contact-parent-rejected;contact-source-rejected;contact-workspace-rejected;contact-topology-rejected;contact-master-rejected;contact-geometry-rejected;contact-dykstra-rejected;contact-box-rejected;contact-ball-rejected;contact-primal-audit-rejected;contact-selection-rejected;contact-work-rejected;restoration-next-trqp-compatible-candidate;contact-constrained-master-expansion-required;contact-constrained-ball-integration-required;contact-constrained-dykstra-candidate;contact-constrained-reference-retained|precedence=parent,source,workspace,topology,master,geometry,dykstra,box,ball,primal,selection,work,certificate,outside-master,ball-integration,candidate,reference|work=parent-r54-replays1;moved-workspace1;workspace-release1;captured-cache494;density-sweeps64;box-blocks64;fresh-pair-once-jvp64;checkpoint-directed-jvp4;pairpasses68;new-row-vjp0;new-gram-jvp0;new-projection0;new-support-audits0;new-hvp0;new-model0;new-nonlinear-trial0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-r41-r42-r43-r44-r45-r46-r47-r48-r49-r50-r51-r52-r53-r54=unchanged;certificate=current-directed-unchanged;normal-witness-apply=none;r43-restoration-commit=none;filter-runtime-commit=none;restoration-exit=none;switching=none;trust-update=none;following-outer=none;tolerance=none;residual-replacement=none;post-projection=none;gamma-change=none;capacity-change=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-contact-constrained-dykstra-classification-only
```

SHA-256 (exact runtime literal, no terminal newline):
`4195f54a1b18cbe53222b4d3a9da3c9004f401bcb9b528162b38d20b537cbe28`.

## Hard gates

1. Reproduce exact R54 bytes, semantic, route and source roots before new work.
   Retain exact R50 anchor, R51 494-row master/cache and R52/R54 evidence.
2. Build/release exactly one unchanged moved workspace and prove stable
   topology. Reuse every density basis and Gram column bit-for-bit.
3. Derive correction box bounds relative to the exact R50 anchor. Zero must be
   feasible in every interval. Audit the unchanged normal ball separately.
4. Execute one 64-cycle Dykstra continuation. Stable set order is all 494
   density halfspaces followed by one grouped box set. Retain density scalar
   corrections and the full box correction across cycles.
5. Use recursive Gram updates only inside a density sweep. After every box
   block execute one fresh pair-once JVP and replace the full residual vector.
6. Capture exact checkpoints 8/16/32/64. Each receives one unchanged directed
   certificate, all-row/master-membership audit, box/ball audit and roots.
7. Exact new work: 64 density sweeps, 64 box blocks, 64 fresh pair-once JVPs and
   four directed checkpoint JVPs: 68 pair passes total. No row/Gram build or
   post-terminal projection.
8. Apply the frozen certificate/outside-master/ball/strict-progress selection.
   No fitted tolerance or early stop.
9. Exact rollback. No witness/state/filter commit, restoration exit, trust
   update, following outer, capacity/gamma change, runtime policy or timing.

Require two clean Release builds and byte-exact outputs. A PASS is one private
contact-constrained Dykstra classification only, not runtime or production
authority.

Rationale:
[R55 research](../../development/nonlocal-nsr3b4e2d7r19r55-contact-dykstra-research-2026-08-25.md).

Evidence:
[R55 closure](../../development/nonlocal-nsr3b4e2d7r19r55-contact-dykstra-evidence-2026-08-25.md).
