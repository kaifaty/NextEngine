# NSR3-B4E2D7R15 -- nominal AL prerequisites contract

Status: `PASS / NOMINAL_AL_PREREQUISITES_CONFIRMED / SHARED_HOST_PERFORMANCE_STOP`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r15-nominal-al-prerequisites|v1|parent=a3ad06cda68b832ba21d6ab3b0315607089a1888:b9b37dad69dd48066eec693cb327001efe5121111aaf60b5b8888a78e0559e61:88d83b6ec6e659f405595b22e13df363bef2ae4c1f99e4838b54f06a585ad1d3|legacy=d7r14-complete-bytes;active-root4d6b8c58c2db672bea5fcae0a4d5fb077cc8587398540128bcb09b6a75400ef8;inactive-root6df37c6ae522de96d3a300abefcbf035af55e04d9f6d7bf77b6cedbea5f1ab1e|dt=explicit-positive-binary64;legacy=0x3f71111111111111;dam-step1-alignment-substeps78;substep=0x3f0c01c01c01c01c;thread=inertia,gradient,hvp,stationarity,direct,divided,long-double,binary128|support=joint-static-support-index;binding-identity;flat-csr;canonical-pairs;no-support-recanonicalization-per-workspace|audit=sparse-binary128;superset-skin0.04h;current-trial-union;binary128-membership-recomputed;runtime-state-binary64|audit-roots=3f6b74612d212d48ee40ba1bc2fffd27a23a20cc1d2c390cf2e1b59194a4a8fc,a11e56103bc1ea0b3a88b4b82354a0e66240db3e9057f98a1a0f987951cfe0aa,e06edef57dda0cf7cabc75de35365ea57432f19d4706879fc7086afb40865e51,b0db278089eb109d19879de3433fb2d69c6160f755fee572f75fdda86c584312|oracle=legacy-transaction-bit-exact;substep-dense-sparse-evaluation-gradient-hvp-divided;four-binary128-audits-exact;active-inactive;forced-dt-mutation;support-binding-mutation;lifecycle|nominal=decoded-frame-zero0d567ba5512ba237a48e5e0b828a670a398f1bf23a35ac269729cad535f374d7;pair-rootfb2b8f8b4c0227cf5d8a7a43ce518ed72b2e5d5cda31fb8e8c17a5727c24ba13;workspace-only;solve-none|work=all-pair-candidate-calls0;workspace-live<=2;static-index-builds1;timing=none|routes=explicit-dt-mismatch;sparse-binary128-mismatch;static-support-binding-mismatch;nominal-al-prerequisites-confirmed|precedence=dt,binary128,support,confirmed|runs=2-release-builds;2-processes;byte-exact|trajectory=none;nominal-solve=none;public-commit=none;physics-mutation=none;runtime-binary128=none|credit=one-nominal-al-prerequisite-set-only
```

Identity SHA-256:
`1ddc90c68e9113a8fc161a42670e0e35527f2aa2194baf71c93d97d10c113dd3`.

## Required command

Add `--nonlocal-al-nominal-prerequisites`. It must:

1. reproduce complete D7R14 stdout bytes;
2. introduce an explicit finite positive binary64 `dt` only on the sparse AL
   candidate path and thread it through inertia, gradient, HVP, stationarity,
   direct/divided reduction and both independent audit precisions;
3. reproduce the complete D7R14 active/inactive transactions at legacy `dt`
   and pass frozen explicit-substep dense/sparse derivative/reduction controls;
4. implement a sparse binary128 accepted-sign audit over the canonical
   current/trial `0.04h` superset, recomputing membership in binary128, and
   reproduce all four D7R13 audit roots;
5. construct AL workspaces from an identity-bound
   `JointStaticSupportIndex`, reproduce tiny and decoded nominal topology and
   reject a one-bit binding identity mutation before evaluation;
6. require exactly one nominal static-index build, zero all-pair candidate
   calls, maximum two live workspaces, exact releases and forced rollback;
7. perform no nominal prediction, solve, integration, publication, trajectory
   or timing measurement;
8. execute twice in each of two clean Release builds and emit exactly one
   route under the frozen precedence.

## Routes

1. `EXPLICIT_DT_MISMATCH`.
2. `SPARSE_BINARY128_MISMATCH`.
3. `STATIC_SUPPORT_BINDING_MISMATCH`.
4. `NOMINAL_AL_PREREQUISITES_CONFIRMED`.

Identity, parent bytes, nonfinite/invalid `dt`, pair/superset ordering,
membership, audit-root, lifecycle, mutation, rollback, process/build repeat or
route-precedence mismatch is hard FAIL. A classified candidate mismatch may
PASS only when its preceding controls and all hard controls pass.

Confirmation grants one nominal AL prerequisite set only. It grants no
nominal solve, macro/trajectory, wall timing, runtime binary128, parallel/GPU
path, public state, physics mutation or production authority.

Execution is closed by the
[NSR3-B4E2D7R15 evidence](../../development/nonlocal-nsr3b4e2d7r15-nominal-al-prerequisites-evidence-2026-08-22.md).
