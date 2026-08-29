# NSR3-B4E2D7R19R29 -- linearized-feasibility operator contract

Status: `FROZEN / IMPLEMENTATION NEXT / SHADOW ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r19r29-linearized-feasibility-operator|v1|parent=ca652ba9:9c953a118d6babb192be6be23e94374576f8a48619754567d341576c33a37237:006620643aae830c4bfe5d07e3d0a0610d34ac0fb2ada57d452597ddaa4f12d0|source=state-851b4eb8d1387a7347bb4dd8adba3be016d8a39dbd04133dcaf306fb46b690d7;receipt-b145ce1af6d3ffca167f2264b55618b0bfccc55c340ff1eee17206569a239af6;history-fd838a046b2629f5d5e4d5aa0ab87160078b2e4b748b98101ac2da00bfa3a37d;position-e0f7ba79637171012b11750b752ab862b7b24174f4d79e2ec115e06bc8b93828;dual-cd0100c30eb55ad7f91f1b94c00ee4281ff3c75e90ca287be038e87ab7a5215d;epoch1;next-outer12;slice333;cumulative856;used-12,2,26,1,333,856,58,34,824,32,2,34,0;primal0x3e53c2bf74000000;stationarity0x3d2e9d0a6841b669;particles6000;theta0x3fc5cccccccccccd;static-a2d97ab6f26383d826366eba2a3d4392f5ef9610dda87e89509e93bd9daf61e8|operator=A=SPACING*Jc;fluid-dof3N;support-fixed;binary64-owned-pairs;pair-once-jvp-vjp;directed-row-reference|rows=all;violated-c>0;phr-active-u+c>0;boundary-coupled;interior;exact-membership-roots|probes=deterministic-fluid-rms1;deterministic-row-rms1;frozen-topology-centered-difference-epsilon2^-16;constant-translation|bounds=pair-directed-componentwise-gamma(16*max-degree+64);finite-difference-relative-l2<=1e-6;adjoint-relative<=1e-12;translation-interior-exact-zero;pair-passes<=8|result=operator-and-partition-roots-derived-after-execution;no-range-solve;no-floor-classification|controls=parent;source;workspace;topology;row-partitions;jvp-reference;finite-difference;adjoint;translation;work;rollback|routes=linearized-operator-parent-rejected;linearized-operator-source-rejected;linearized-operator-workspace-rejected;linearized-operator-topology-rejected;linearized-operator-partition-rejected;linearized-operator-jvp-rejected;linearized-operator-finite-difference-rejected;linearized-operator-adjoint-rejected;linearized-operator-translation-rejected;linearized-operator-work-rejected;linearized-feasibility-operator-candidate|precedence=parent,source,workspace,topology,partition,jvp,finite-difference,adjoint,translation,work,candidate|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-substeps1;diagnostic-workspaces1;pair-passes<=8;new-hvp0;new-model0;new-trial0;new-precision0;new-outer0|lsqr=none;correction=none;state-mutation=none;following-outer=none;substep=none;macro=none;trajectory=none;timing=none;public-schema=none;runtime=none;production=none|credit=one-private-linearized-feasibility-operator-candidate-only
```

Identity SHA-256:
`e1d54356d0f0c6e65434b5670f63bf92780612eeb19791b388702831e85224f2`.

## Required command

Add `--nonlocal-al-linearized-feasibility-operator`. Reproduce exact R28 and
the frozen source roots; build one read-only workspace; close row partitions,
JVP, VJP, finite-difference, adjoint and translation controls. Emit exact
operator/membership/work roots and reproduce them across two clean builds.

## Hard failures

Identity/parent/source/workspace/topology, any row partition, derivative,
adjoint, translation, work bound, rollback or two-build/process mismatch is
hard FAIL. No range solve or floor classification is allowed in R29.

## Authority boundary

Read-only private diagnostic only. No correction, state/public/world commit,
outer, solver-policy change, timing, runtime or production authority.

Research basis:
[D7R19R29 research](../../development/nonlocal-nsr3b4e2d7r19r29-linearized-feasibility-operator-research-2026-08-24.md).
