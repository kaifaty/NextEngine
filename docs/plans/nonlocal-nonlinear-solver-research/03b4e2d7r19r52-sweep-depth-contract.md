# NSR3-B4E2D7R19R52 -- fixed-master sweep-depth contract

Date: `2026-08-25`

Status: `FROZEN V1 / IMPLEMENTATION NEXT / ROLLBACK ONLY`.

Parent: `8bf5863f`, R51 stdout SHA-256
`c0fc53e3c1637a84c0a9cc0494a8b9fc7bc6e60b627afe032337a37c14cbf960`,
semantic `65df360418fba870089415b922b50e96c3bba55e904b2f9eec4fadd1e096361a`.

## Frozen identity

```text
nextengine.nonlocal.nsr3b4e2d7r19r52-persistent-master-sweep-depth|v1|parent=8bf5863f:c0fc53e3c1637a84c0a9cc0494a8b9fc7bc6e60b627afe032337a37c14cbf960:65df360418fba870089415b922b50e96c3bba55e904b2f9eec4fadd1e096361a:PERSISTENT_MASTER_CLOSURE_CANDIDATE|source=r50-terminal-witness-e2ccfb4281b2b598b0a9864a8c9bb92f5c2512e018501147ca82b39b14b67592;r51-witness-72c570b9bb981625ee2036b5c88ff19e5a88bca662efb3474bd62ce1130a6e34;union-root-38d7a09afa7278d492e6c7981e7dd7359b482309d4d9df0efb01c84324e12e28;cache-root-f4e9368971d4b6feeb13874d0279dc73fd54ebbb523c75c5c2dd303acfc58cd5;gram-root-7a0358af3fda7c90c89a5cfb9abe125fadf9142fb5ce0215e94ee2e70a94dc48;master494;r51-psi1.3460782774625938e-25;r51-h5.1885995749577626e-13;r51-maximum-upper8.9325740894562227e-14|problem=unchanged-r51-fixed-master-same-r50-terminal-anchor-upper-and-ball-box;independent-zero-dual-depth-probes;no-new-row|solver=stable-master-order;exact-sweeps16,32,64;independent-zero-dual-each;checkpoints1,2,4,8-plus-terminal-state;no-tolerance-stop;dense-gram-reused|globalization=each-depth-target=r50-terminal+correction;unchanged-r48-ball-box-projection;fresh-r48-all-row-directed-jvp;evaluate-all-three|selection=smallest-sweep-certified-first;else-smallest-sweep-strict-simultaneous-r51-psi-h-maximum-upper-dominance;else-r51-reference-retained|diagnostic=predicted-maximum-at8,16,32,64;strict-monotone-depth-contraction;cg-polish-required-only-if-no-selected-candidate-and-contraction-valid|controls=r51-parent-bytes;source;workspace;master-union;basis-reuse;gram;depth-sweeps;projection;directed-audit;selection;work;rollback;route-precedence|routes=sweep-parent-rejected;sweep-source-rejected;sweep-workspace-rejected;sweep-master-rejected;sweep-basis-rejected;sweep-gram-rejected;sweep-depth-rejected;sweep-projection-rejected;sweep-primal-audit-rejected;sweep-selection-rejected;sweep-work-rejected;restoration-next-trqp-compatible-candidate;persistent-master-sweep-depth-candidate;persistent-master-cg-polish-required;persistent-master-r51-reference-retained|precedence=parent,source,workspace,master,basis,gram,depth,projection,primal-audit,selection,work,certificate,sweep-candidate,cg-polish,reference|work=parent-r51-replays1;moved-workspace1;workspace-release1;captured-cache494-reused;new-row-vjp0;new-gram-jvp0;candidate-directed-jvp3;pairpasses3;independent-sweeps112;projection-calls3;new-support-audits0;new-hvp0;new-model0;new-nonlinear-trial0;new-outer0|runs=2-clean-release-builds;1-process-each;byte-exact|r40-r41-r42-r43-r44-r45-r46-r47-r48-r49-r50-r51=unchanged;normal-witness-apply=none;r43-restoration-commit=none;filter-runtime-commit=none;restoration-exit=none;switching=none;trust-update=none;following-outer=none;tolerance=none;capacity-change=none;penalty-change=none;support-expansion=none;state-mutation=none;timing=none;runtime=none;production=none|credit=one-private-fixed-master-sweep-depth-classification-only
```

SHA-256 (exact runtime literal, no terminal newline):
`fde05c9b766f5bf81283adcfa9374cc60b490bd75c850e1fba599de70707aafd`.

## Hard gates

1. Reproduce exact R51 bytes, semantic, route, R50/R51 witnesses, 494-row union,
   extended cache and Gram roots before new solve work.
2. Rebuild and release exactly one moved workspace. Reuse all captured basis and
   Gram bytes; no new row VJP or Gram JVP is admitted.
3. Run independent zero-dual solves at exactly 16, 32 and 64 sweeps in stable
   master order. No tolerance stop, warm start, row drop or reordering.
4. Project every checkpoint from the exact R50 terminal anchor through the
   unchanged R48 ball-box and perform exactly three fresh directed audits.
5. Evaluate every checkpoint. Select the smallest-sweep certificate; otherwise
   the smallest-sweep strict simultaneous R51 dominance; otherwise retain R51.
6. Predicted master contraction is diagnostic only and cannot certify
   compatibility or admit a nonlinear candidate by itself.
7. Exact new work: 112 dense sweeps, three projections, three audit JVP/pair
   passes, zero row VJPs and zero Gram JVPs.
8. Exact rollback. No witness/state/filter commit, restoration exit, trust
   update, following outer, multiplier-transfer policy or timing.

Require two clean Release builds and byte-exact outputs. Any PASS is one private
fixed-master depth classification only, not a completed restoration transaction,
runtime permission or production evidence.

Rationale:
[R52 research](../../development/nonlocal-nsr3b4e2d7r19r52-sweep-depth-research-2026-08-25.md).
