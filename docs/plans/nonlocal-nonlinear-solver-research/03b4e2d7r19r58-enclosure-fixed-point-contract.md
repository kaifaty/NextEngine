# NSR3-B4E2D7R19R58 -- directed-enclosure fixed-point contract

Date: `2026-08-25`

Status: `CLOSED / PASS / FIXED_POINT_CONTRACTION_CANDIDATE / ROLLBACK ONLY`.

Parent: `5aaae13e`, R57 stdout SHA-256
`a1930bc92665cb08f5f46ff53e9b53da48d09438d3bc52c25ec848a2db0a6b5c`,
semantic `666e9f62ab3e0ac027bd855893f90c08dc2d61433f913170e736c6d9e50a39b6`.

## Frozen identity

```text
nextengine.nonlocal.nsr3b4e2d7r19r58-directed-enclosure-fixed-point|v1|parent=5aaae13e:a1930bc92665cb08f5f46ff53e9b53da48d09438d3bc52c25ec848a2db0a6b5c:666e9f62ab3e0ac027bd855893f90c08dc2d61433f913170e736c6d9e50a39b6:CERTIFICATE_REFINEMENT_ENCLOSURE_FIXED_POINT_REQUIRED|source=r57-state-952d2541ed707bc36bec78e10f32ae6c60bd062c081f3811a611fe68ead27d63;cycle64-record-856dc573d7b1320659a554d0b33522a74b03eb34dc1747518e7692a684158231;witness-e4c5aa2d5cdddaaeb55c620d37f43856d5ed797f199fad2253175bef2ddd341d;correction-18bc74476b5887a4b832e226c8f09984a91e91d84d8636d4ceffd4a9c824d53b;boxdual-e58553b7864502442b95d8b20e20e16a255afe0b59f64d43bf1f3e6fc7ca7d5f;upper=exact-transitive-capture;active245;raw0;maximum-upper6.3619020626182659e-24;full128-positive0;negative6000;unresolved0;maximum-raw-5.1986625146610136e-22|map=T-r57;evaluate-T-of-r57-selected-once|problem=d0;min0.5*norm2delta;r57-directed-upper-plus-A-delta<=0-on-master494;box=contact-lower-minus-r57-witness<=delta<=contact-upper-minus-r57-witness;normal-ball-audit-only|solver=reuse-r57-zero-initialized-grouped-box-dykstra;density-lambda0;box-correction0;delta0;stable-master-order;fresh-residual-after-box|depth=64;checkpoints8,16,32,64;no-tolerance-stop;first-certified-else64;no-third-outer|oracle=unchanged-directed-audit-each-checkpoint;selected-fresh-pair-once-binary64;selected-compensated-binary128-directed-traversal;frozen-coefficient-binary128-recompute;current-gamma-unchanged|contraction=selected-active<245-and-selected-maximum-upper<6.3619020626182659e-24;stall=remaining-positive-and-not-contraction|controls=r57-parent-bytes;source;workspace;topology;master;geometry;core-reuse;dykstra;box;ball;primal;binary128;selection;work;rollback;route-precedence|routes=fixed-point-parent-rejected;fixed-point-source-rejected;fixed-point-workspace-rejected;fixed-point-topology-rejected;fixed-point-master-rejected;fixed-point-geometry-rejected;fixed-point-dykstra-rejected;fixed-point-box-rejected;fixed-point-ball-rejected;fixed-point-primal-audit-rejected;fixed-point-binary128-rejected;fixed-point-selection-rejected;fixed-point-work-rejected;fixed-point-master-expansion-required;fixed-point-ball-integration-required;restoration-next-trqp-compatible-candidate;fixed-point-raw-regression;fixed-point-sign-unresolved;fixed-point-stalled;fixed-point-contraction-candidate;fixed-point-reference-retained|precedence=parent,source,workspace,topology,master,geometry,dykstra,box,ball,primal,binary128,selection,work,outside-master,ball-integration,certified,raw-positive,unresolved,stall,contraction,reference|work=parent-r57-replays1;moved-workspace1;workspace-release1;density-sweeps64;box-blocks64;fresh-residual-jvp64;checkpoint-directed-jvp4;selected-fresh-pair-jvp1;binary128-row-traversal1;new-pairpasses69;quad-row-passes1;new-row-vjp0;new-gram-jvp0;new-basis0;new-projection0;new-support-audits0;new-hvp0;new-model0;new-nonlinear-trial0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-r41-r42-r43-r44-r45-r46-r47-r48-r49-r50-r51-r52-r53-r54-r55-r56-r57=unchanged;certificate=current-directed-unchanged;binary128=offline-only;normal-witness-apply=none;r43-restoration-commit=none;filter-runtime-commit=none;restoration-exit=none;switching=none;trust-update=none;following-outer=none;tolerance=none;gamma-change=none;capacity-change=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-directed-enclosure-fixed-point-classification-only
```

SHA-256 (exact runtime literal, no terminal newline):
`a0327b38e7d2f7552bfc223efdb99a796e968ec8cfa8b70560ac32493206542f`.

## Hard gates

1. Reproduce exact R57 stdout, semantic, route, state and selected roots before
   new work. Internal capture must not change R57/R56/R55 bytes.
2. Retain the exact R57 selected witness and terminal directed upper vector;
   reconstruction from rounded report values is forbidden.
3. Build/release one moved workspace with unchanged stable topology and exact
   494-row master.
4. Call the existing R57 refinement core exactly once with the R57 source,
   zero `delta`, zero density multipliers and zero box correction.
5. Execute exactly 64 cycles, checkpoints 8/16/32/64, first-certified else 64,
   69 new pair passes and one quad row traversal.
6. Use unchanged all-row binary64 directed certificate and offline binary128
   sign oracle. No gamma or arithmetic-policy change.
7. Define contraction exactly as selected active count `<245` and selected
   maximum upper `<6.3619020626182659e-24`. No ratio/tolerance is admitted.
8. Apply frozen route precedence, exact work and rollback. No third fixed-point
   outer, state commit, restoration exit, switching, trust update or timing.

Require two clean Release builds and byte-exact outputs. Any PASS is one
private fixed-point classification only, not a completed restoration
transaction, runtime permission or production evidence.

Rationale:
[R58 research](../../development/nonlocal-nsr3b4e2d7r19r58-enclosure-fixed-point-research-2026-08-25.md).

Closure evidence:
[R58 evidence](../../development/nonlocal-nsr3b4e2d7r19r58-enclosure-fixed-point-evidence-2026-08-25.md).
