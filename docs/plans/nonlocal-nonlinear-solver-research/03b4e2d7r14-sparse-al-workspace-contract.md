# NSR3-B4E2D7R14 -- sparse AL workspace equivalence contract

Status: `CLOSED / PASS / SPARSE_AL_WORKSPACE_CANDIDATE`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r14-sparse-al-workspace-equivalence|v2|parent=bb5d6f5d9371636e71accefb6af70b0918597622:37c0828f6f3621f87560aa447aa0ad5f4a7391f9f6134bad46b6f13858b3ecd8:514ea1925a85d398a948a2dcbc319689116114a02335a599e51d6703202c18de|nominal-parent=343f6c424b43bccba57eb59e192fd545c968ab382e6abd539ce748c686e86a3d:4805530ed85d6a6b1f05ca855c8661fe036b16e8201fcbf745864de9e78d1be1|tiny=active-root:4d6b8c58c2db672bea5fcae0a4d5fb077cc8587398540128bcb09b6a75400ef8;inactive-root:6df37c6ae522de96d3a300abefcbf035af55e04d9f6d7bf77b6cedbea5f1ab1e|representation=joint-neighborhood;canonical-sorted-pairs;csr;al-tape=radius,wprime,wsecond,constraint,active-coefficient|divided=sorted-current-trial-pair-union;two-pointer;current-member-bit;trial-member-bit;canonical-order|oracle=tiny-dense-evaluation,gradient,hvp,divided,full-transaction;binary64-bit-exact|nominal=reference-frame-report:5a9d2f67550b73169c7405c1c46fb6ee5eeebeb539915d296620e1c407922c07;decoded-frame-zero:0d567ba5512ba237a48e5e0b828a670a398f1bf23a35ac269729cad535f374d7;pair-root:fb2b8f8b4c0227cf5d8a7a43ce518ed72b2e5d5cda31fb8e8c17a5727c24ba13;pairs342502;directed611520;degree120;active0|work=dense-candidate-checks116301000;sparse-pair-visits342502;all-pair-candidate-calls0;workspace-live<=2|controls=d7r13-complete-bytes;nominal-topology-complete-bytes;tiny-active-inactive;pair-union-crossing;workspace-lifecycle;forced-rollback|routes=al-sparse-evaluation-mismatch;al-sparse-hvp-mismatch;al-sparse-divided-mismatch;sparse-al-workspace-candidate|precedence=evaluation,hvp,divided,candidate|runs=2-release-builds;2-processes;byte-exact;timing=none|nominal-solve=none;trajectory=none;public-commit=none;physics-mutation=none|credit=one-sparse-al-structural-backend-candidate-only
```

Identity SHA-256:
`dab6f6033218d09b94bb07531f9039aa48c2cfce2178303237e947806d6d27c1`.

## Required command

Add `--nonlocal-al-sparse-workspace-equivalence`. It must:

1. reproduce complete D7R13 stdout bytes and the selected decoded nominal
   topology parent bytes;
2. implement a separate AL joint workspace/tape over canonical sorted pairs
   and flat CSR, without changing the existing penalty workspace;
3. compare sparse/dense AL evaluation, gradient and HVP bit-for-bit on frozen
   active and inactive tiny controls;
4. merge sorted current/trial pair sets with explicit membership bits and
   compare every D7R13 divided reduction bit-for-bit, including pair crossings;
5. reproduce the complete active-repeat and inactive D7R13 transaction roots
   using only the sparse candidate path;
6. reproduce D1 reference-frame report SHA `5a9d2f67...c07`, then build only
   the decoded nominal frame-zero workspace and reproduce its raw-bit root
   `0d567ba5...4d7`, pair root `fb2b8f8b...ba13`, pair/directed/degree counts
   `342502/611520/120` and zero active pressure centres;
7. prove zero all-pair candidate calls, at most two live workspaces, exact
   lifecycle accounting and forced rollback;
8. execute twice in each of two clean Release builds and emit exactly one
   route under the frozen precedence.

## Routes

1. `AL_SPARSE_EVALUATION_MISMATCH`.
2. `AL_SPARSE_HVP_MISMATCH`.
3. `AL_SPARSE_DIVIDED_MISMATCH`.
4. `SPARSE_AL_WORKSPACE_CANDIDATE`.

Identity, parent bytes, nominal topology, nonfinite, pair ordering/membership,
work/lifecycle ledger, rollback, process/build repeat or route-precedence
mismatch is hard FAIL. Evaluation/HVP/divided mismatch routes are admissible
research PASS classifications only when all hard controls pass.

Candidate PASS grants one sparse AL structural backend only. It grants no
nominal solve, trajectory, wall timing, GPU/runtime, public state, physics
mutation or production authority.

Closure evidence:
[NSR3-B4E2D7R14 sparse AL workspace evidence](../../development/nonlocal-nsr3b4e2d7r14-sparse-al-workspace-evidence-2026-08-22.md).
