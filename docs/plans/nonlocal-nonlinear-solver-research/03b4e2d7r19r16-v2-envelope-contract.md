# NSR3-B4E2D7R19R16 -- canonical v2 continuation-envelope contract

Status: `FROZEN / PASS V2 VALIDATION CANDIDATE / READ ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r19r16-v2-envelope|v1|parent=108a14bf:257244d44f3ec4460889968221cf95f0f4ca72090ec6406c7b7349a0710b81b4:ed209bfa1c6a52411aef21b9f5036fabe46baf68bd2acf95e3b46a424782f0bf|legacy=r14-stdout16b357b173d5c36f123dbbad31cca0c777229ff38772d3847a85357c9a0b4625|target=v2-root069f8bdddfaeed4d09156f4577f9922d4f606f0c2aaee3e27ff5a66c2915b8f9;magic-NEALCTN2;schema2;body524;total540;flags13;particles6000;outer6;provisional-1;primal0x3e5c40ff44000000;theta0x3fc5cccccccccccd|roots=source-c06dbfeeac346d4413d114082d18b4e4725edfac81896ed72cbabfacf3f188b5;position-58dadefd7426e9f80188b694557a02efcaf005dd67abfff5e337b90f214be5a8;dual-f4279bde29358f65b17b15ad7456e8c5743b44ec99fd3612d78cb5f905b00aca;outer-5dd9a07d60cbfe851bdc4c383944a1eb8042f178a821f4a546da33101dd0a19a;predicted-36112dde1e0b274c5b9216f4818b82977a0c80dc257478c111b0f0a9390d2d7e;static-a2d97ab6f26383d826366eba2a3d4392f5ef9610dda87e89509e93bd9daf61e8;formula-676900335dcf77e2f8536e69e92597d608baf1bb7c4e514cfe071070e6c88895;solver-193016ce6c55a4b3052d3c980a38364de72cc9b168653bcb7e3a5b9790347697;completion-c6bb447392499aad19c49b69857d43b061c73c9468fe64d097a98ce5ecf99034;history-9754a0bb714011fe4eb637060c6c1eee930e7eb4a1c7db991f664ae40011af7b|limits=16,16,34,512,288,64,8704|used=6,2,25,0,523,523,33,21,504,19,2,21,0|encoding=fixed-order;little-endian;binary64-bits;raw-digests;no-padding;exact-flags|owner=expected-root;expected-epoch;consumed;read-only|controls=valid;bad-magic;bad-schema;bad-length;truncated;bad-flags;token-root;owner-root;duplicate;stale;position;dual;particle-count;outer-context;predicted;theta;static;formula;solver;completion;limit;used;cumulative-invariant;history|routes=token-v2-canonical-rejected;token-v2-integrity-rejected;token-v2-owner-rejected;token-v2-duplicate-rejected;token-v2-stale-epoch-rejected;token-v2-payload-rejected;token-v2-context-rejected;token-v2-policy-rejected;token-v2-resource-rejected;token-v2-history-rejected;token-v2-validation-candidate|precedence=canonical,integrity,owner,duplicate,stale,payload,context,policy,resource,history,candidate|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-substeps1;new-workspaces0;new-hvp0;new-model0;new-trial0;new-precision0;new-outer0;encoding-and-hash-only|owner-consume=none;epoch-transition=none;resume=none;outer6=none;second-solve=none;second-substep=none;macro=none;trajectory=none;timing=none;public-schema=none;public-commit=none;physics-mutation=none;live-budget-code-change=none;production-policy-change=none|credit=one-private-v2-envelope-validation-candidate-only
```

Identity SHA-256:
`700629514111fa5630d4463abadaedaa55709704fb588cb6db9d748e91badec2`.

## Required command

Add `--nonlocal-al-v2-envelope-validation`. It must:

1. reproduce exact R15 stdout SHA
   `257244d44f3ec4460889968221cf95f0f4ca72090ec6406c7b7349a0710b81b4`
   and semantic result
   `ed209bfa1c6a52411aef21b9f5036fabe46baf68bd2acf95e3b46a424782f0bf`;
2. retain exact R14 and transitive parents;
3. independently derive every payload, policy, history, limit and used field;
4. encode exactly `524/540` body/envelope bytes and reproduce v2 root
   `069f8bdddfaeed4d09156f4577f9922d4f606f0c2aaee3e27ff5a66c2915b8f9`;
5. decode into a fresh value and require exact field equality and re-encoding;
6. validate one exact baseline against a read-only trusted owner;
7. run every fixed negative control from the identity projection, recomputing
   category-control roots where required;
8. require each negative's exact first route under the frozen precedence;
9. prove parent/envelope/payload/owner rollback and zero new solver work;
10. execute no owner consume, epoch transition, resume, outer 6, solve/trial,
    budget mutation, state commit, substep, macro, trajectory or timing;
11. run one fresh process from each of two clean Release builds.

## Hard failures

Identity/parent bytes, transitive retention, field derivation, canonical byte
size/order/endianness, target root, decode/re-encode, baseline admission,
negative route, rollback, zero-work scope or build/process repeat mismatch is
hard FAIL.

## Route precedence

1. `TOKEN_V2_CANONICAL_REJECTED`;
2. `TOKEN_V2_INTEGRITY_REJECTED`;
3. `TOKEN_V2_OWNER_REJECTED`;
4. `TOKEN_V2_DUPLICATE_REJECTED`;
5. `TOKEN_V2_STALE_EPOCH_REJECTED`;
6. `TOKEN_V2_PAYLOAD_REJECTED`;
7. `TOKEN_V2_CONTEXT_REJECTED`;
8. `TOKEN_V2_POLICY_REJECTED`;
9. `TOKEN_V2_RESOURCE_REJECTED`;
10. `TOKEN_V2_HISTORY_REJECTED`;
11. `TOKEN_V2_VALIDATION_CANDIDATE`.

## Authority boundary

This contract grants one private, read-only v2 validation corpus. A PASS does
not consume ownership or authorize resume. It does not define a public schema,
change budget code, execute outer 6, run timing or grant runtime/production
authority.

Closed by the
[D7R19R16 v2 envelope evidence](../../development/nonlocal-nsr3b4e2d7r19r16-v2-envelope-evidence-2026-08-24.md).
