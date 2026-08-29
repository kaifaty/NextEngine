# NSR3-B4E2D7R19R30 -- linearized range-projection contract

Status: `FROZEN / IMPLEMENTATION NEXT / SHADOW ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r19r30-linearized-range-projection|v1|parent=f3185b23:596c81d4979bed80181d65429e5d253a6464d61f39ec76d8330fe0ad464dfe7b:f6736b14927c8daedf751649a32cdf3f5ac527116b3037c9367ed921c369c835|source=state-851b4eb8d1387a7347bb4dd8adba3be016d8a39dbd04133dcaf306fb46b690d7;position-e0f7ba79637171012b11750b752ab862b7b24174f4d79e2ec115e06bc8b93828;dual-cd0100c30eb55ad7f91f1b94c00ee4281ff3c75e90ca287be038e87ab7a5215d;topology-070ab5209d8b32c942a244d9d5ae5442baba6f281c9010f65a5f0d1cf184c2e5;violated-bd06f7b7c5e1f1943e5c269bbfb192229504091381b8f20f0f4988eecb021c70;active-b84e502736ee7eed3239fe4dfcd86ec92c54dbffb0ce78854087318275d57029;boundary-25ed05cd3b2703de3f2434c825dc4d8894fcab76e824298421f0416e36df32fc;interior-72515617c229e4db8ef84aaa96455c3ba8a4f7cdd2060315e1aff5d406d94f2d;particles6000;pairs340340;max-degree113;violated1420|problem=B=restrict-violated(SPACING*Jc);b=-restrict-violated(c);min||Bv-b||2;support-fixed;no-row-scaling|scaling=exact-scalar-column-l2;D=reciprocal-nonzero;zero-columns-derived;C=BD;range-preserved|solver=LSMR;golub-kahan;lambda0;y0zero;max-iterations512;atol1e-10;btol1e-10;conlim1e12;reorth-none|direct-stop=compatible-or-least-squares;fresh-Cy;fresh-Ct-residual;orthogonality<=1e-10;pythagorean-relative<=1e-10|controls=dense-identity;dense-underdetermined-compatible;dense-overdetermined-inconsistent;dense-rank-deficient-inconsistent;parent;source;operator;partition;column-scaling;lsmr;direct-residual;orthogonality;work;rollback|observations=rhs-root;column-root;iterate-root;projection-root;residual-root;normal-root;residual-ratio;normal-ratio;condition-estimate;boundary-interior-decomposition;full-row-response;new-positive-count;dimensionless-preimage-rms-max;no-physical-threshold|routes=range-projection-parent-rejected;range-projection-source-rejected;range-projection-operator-rejected;range-projection-partition-rejected;range-projection-dense-control-rejected;range-projection-scaling-rejected;range-projection-not-converged;range-projection-residual-rejected;range-projection-orthogonality-rejected;range-projection-work-rejected;linearized-range-projection-candidate|precedence=parent,source,operator,partition,dense-control,scaling,convergence,residual,orthogonality,work,candidate|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-substeps1;diagnostic-workspaces1;column-passes1;initial-adjoint-passes1;iteration-passes<=1024;final-passes2;total-pair-passes<=1028;new-hvp0;new-model0;new-trial0;new-precision0;new-outer0|correction-application=none;nonlinear-evaluation=none;floor-classification=none;state-mutation=none;following-outer=none;substep=none;macro=none;trajectory=none;timing=none;public-schema=none;runtime=none;production=none|credit=one-private-linearized-range-projection-candidate-only
```

Identity SHA-256:
`c1e548cc5dba9f4467ee932fa4596e787a11b43d8ba6836c2ab6101ee967a0eb`.

## Required command

Add `--nonlocal-al-linearized-range-projection`. Reproduce exact R29 and its
R28 source; rebuild the same read-only workspace and row roots; pass the four
dense LSMR controls; assemble exact scalar column scaling; solve only the
frozen violated-row problem; independently recompute the final residual,
normal residual and orthogonality; emit all frozen roots and observations.

## Hard failures

Identity/parent/source/operator/partition, any dense control, nonfinite or
invalid column scaling, unconverged direct stopping rules, final residual,
orthogonality/Pythagorean, work bound, rollback or two-build/process mismatch
is hard FAIL. The residual magnitude, boundary/interior share, new-positive
count and preimage magnitude are report-only and cannot be fitted into a pass
threshold after observation.

## Authority boundary

Read-only private diagnostic only. The LSMR iterate is not a correction and
must not update particle positions or any AL/public/world state. No nonlinear
evaluation, floor classification, following outer, policy change, timing,
runtime or production authority.

## Implementation clarification

`orthogonality` is the dimensionless cross-energy
`|p^T q| / max(||b||^2, tiny)`. The raw angle cosine is report-only: it is
undefined at an exact compatible projection and numerically unstable once
`||q||` reaches roundoff. This clarification does not change the frozen
`1e-10` gate, LSMR policy, iteration cap or any physical acceptance threshold.

Research basis:
[D7R19R30 research](../../development/nonlocal-nsr3b4e2d7r19r30-linearized-range-projection-research-2026-08-24.md).
