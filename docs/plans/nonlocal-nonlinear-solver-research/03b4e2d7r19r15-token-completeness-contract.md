# NSR3-B4E2D7R19R15 -- continuation-token completeness contract

Status: `FROZEN / PASS V1 RESOURCE LEDGER COLLISION / V1 NOT RESUMABLE`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r19r15-token-completeness|v1|parent=57b14538:16b357b173d5c36f123dbbad31cca0c777229ff38772d3847a85357c9a0b4625:34fdc84cb5bddcc89337f7b1cc961cbc18f5d5eda7cfe008ac6ae0af89d08751|legacy=r13-stdout0f248c4506303055cbf7a967dd1328a36d8628341a04b6861aec210af17f57f9|target=r14-token-c06dbfeeac346d4413d114082d18b4e4725edfac81896ed72cbabfacf3f188b5;schema1;outer6;completed523|baseline=limits-16,16,34,512soft,288,64;used-outer6,inner2,last-trial25,slice523,cumulative523,workspaces33,precision21,accepted21,rejected0;epoch0|bound-controls=schema,next-outer,position,dual,previous-primal,admissibility,provisional,predicted,theta,static,epoch,slice,cumulative,soft,trial-allowance,outer-state|unbound-controls=outer-inner-workspace-precision-limits;outer-inner-workspace-precision-used;accepted-rejected-history;formula-solver-completion-policy|twins=resource-used-minus-one-and-outer-cap-plus-one;completion-policy-alternate;both-locally-valid;v1-root-equal|routes=token-completeness-parent-invalid;token-completeness-state-binding-invalid;token-v1-resource-ledger-collision;token-v1-policy-identity-collision;token-v1-complete|precedence=parent,state,resource,policy,complete|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-substeps1;new-workspaces0;new-hvp0;new-model0;new-trial0;new-precision0;new-outer0;hash-projections-only|resume=none;outer6=none;second-solve=none;second-substep=none;macro=none;trajectory=none;timing=none;public-schema=none;public-commit=none;physics-mutation=none;live-budget-code-change=none;production-policy-change=none|credit=one-v1-token-projection-completeness-classification-only
```

Identity SHA-256:
`4431b8854b5b1734fce5340587fee30bfa082d1952352bdad7db146e14e2e00f`.

## Required command

Add `--nonlocal-al-token-completeness`. It must:

1. reproduce exact R14 stdout SHA
   `16b357b173d5c36f123dbbad31cca0c777229ff38772d3847a85357c9a0b4625`
   and semantic result
   `34fdc84cb5bddcc89337f7b1cc961cbc18f5d5eda7cfe008ac6ae0af89d08751`;
2. retain exact R13 and transitive parent roots;
3. reconstruct the exact v1 token projection/root from typed R14 capture;
4. prove one-field sensitivity for every frozen v1-bound field group;
5. derive the exact projected resource ledger frozen above;
6. construct the fixed locally-valid resource twin and prove it differs in
   required resume semantics while retaining the exact v1 token root;
7. construct the fixed alternate completion-policy twin and prove the same;
8. report resource and policy collisions separately, then select exactly one
   route under the frozen precedence;
9. prove parent/capture rollback and zero new physical/solver work;
10. execute no token admission/resume, outer 6, solve/trial, budget mutation,
    state commit, substep, macro, trajectory or timing;
11. run one fresh process from each of two clean Release builds.

## Hard failures

Identity/parent bytes, transitive retention, typed token reconstruction,
bound-field sensitivity, ledger derivation, twin local validity, collision
classification, rollback, zero-work scope or build/process repeat mismatch is
hard FAIL.

## Routes and precedence

1. `TOKEN_COMPLETENESS_PARENT_INVALID`;
2. `TOKEN_COMPLETENESS_STATE_BINDING_INVALID`;
3. `TOKEN_V1_RESOURCE_LEDGER_COLLISION`;
4. `TOKEN_V1_POLICY_IDENTITY_COLLISION`;
5. `TOKEN_V1_COMPLETE`.

Resource and policy collision observations are not mutually exclusive. The
route selects the first inadequacy under the precedence above.

## Authority boundary

This contract grants one report-only v1 projection-completeness
classification. A PASS on the discriminator does not make v1 resumable. It
does not authorize v2 validation, resume, outer 6, budget-code mutation,
public serialization, timing, runtime policy or production authority.

Closed by the
[D7R19R15 token-completeness evidence](../../development/nonlocal-nsr3b4e2d7r19r15-token-completeness-evidence-2026-08-24.md).
