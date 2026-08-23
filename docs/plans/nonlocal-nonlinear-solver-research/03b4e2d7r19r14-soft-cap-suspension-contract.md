# NSR3-B4E2D7R19R14 -- soft-cap suspension contract

Status: `FROZEN / PASS OUTER BOUNDARY SUSPENDED / SHADOW ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r19r14-soft-cap-suspension|v1|parent=dfa474f2:0f248c4506303055cbf7a967dd1328a36d8628341a04b6861aec210af17f57f9:3a60f64ee09a85b4ea892f12341ff3b8c4e7dcb63bbe4eb4cbebd99c237e5e57|legacy=r12-stdoute57ba96aca2c3114ea2c9c10fa2728d8c91dd7ac7969010f381a245219175cee;r11-stdoutbd05f7ac3ca76425dbfda1e43882f48adcb4c83addf1fe4ddeaacd37ca8a8efe;r10-stdout15719465931a21421e094abde1200d685bee44e4b112dc314d5857f863074953;r9-stdoutf1cb461d270e1642e9c0220c60fc59c64cb5bb69039f293ed8e9eade7f6ed1f0|target=outer5-complete;state-5dd9a07d60cbfe851bdc4c383944a1eb8042f178a821f4a546da33101dd0a19a;dual-f4279bde29358f65b17b15ad7456e8c5743b44ec99fd3612d78cb5f905b00aca;trial-start498;soft512;trial-allowance34;dynamic-ceiling532;actual-trial25;completed523|admission=trial-start-only-below-soft;admitted-trial-owns-existing-hard-allowance;outer6-denied-at-or-above-soft|suspension=explicit-not-converged-not-failed;next-outer6;private-position-dual;previous-primal-admissibility-provisional;predicted-theta-static;budget-epoch-and-cumulative-ledger;versioned-root|unsupported=over-cap-inner-requires-another-trial-fail-closed|controls=r19r13-parent-bytes;transitive-parents-retained;exact-accounting;reserve-bound;outer-boundary-state;token-fields;token-root;rollback;zero-new-work|routes=soft-cap-projection-invalid;soft-cap-atomic-reserve-exceeded;soft-cap-inner-suspension-unsupported;soft-cap-outer-boundary-incomplete;soft-cap-outer-boundary-suspended|precedence=invalid,reserve,inner,outer,suspended|runs=2-clean-release-builds;1-process-each;byte-exact|work=control-parent-substeps1;new-workspaces0;new-hvp0;new-model0;new-trial0;new-precision0;new-outer0|resume-executed=none;outer6=none;second-solve=none;second-substep=none;macro=none;trajectory=none;timing=none;public-commit=none;physics-mutation=none;live-budget-code-change=none;production-policy-change=none|credit=one-soft-cap-suspension-policy-projection-only
```

Identity SHA-256:
`0d8a39cfefd050febf87c9544c4a33b0cee9251a769bc3b73d6a2f126d4d839f`.

## Required command

Add `--nonlocal-al-soft-cap-suspension-projection`. It must:

1. reproduce exact R13 stdout SHA
   `0f248c4506303055cbf7a967dd1328a36d8628341a04b6861aec210af17f57f9`
   and semantic result
   `3a60f64ee09a85b4ea892f12341ff3b8c4e7dcb63bbe4eb4cbebd99c237e5e57`;
2. retain exact R12/R11/R10/R9 roots frozen above;
3. bind the exact completed outer-state and dual roots;
4. prove trial start `498`, soft limit `512`, existing allowance `34`,
   dynamic ceiling `532`, actual trial charge `25`, completed total `523`,
   overshoot `11` and unused allowance `9`;
5. require trial admission below soft, reserve completion within its dynamic
   ceiling, exact R13 inner/outer completion and denial of outer 6 at 523;
6. construct a versioned continuation-token candidate with exact private
   state, next outer, convergence-history, immutable-input and ledger fields;
7. classify exactly one route under the frozen precedence;
8. prove parent/token rollback and zero new physical work;
9. execute no resume, outer 6, solve/trial, substep, timing or policy change;
10. run one fresh process from each of two clean Release builds.

## Hard failures

Identity/parent bytes, transitive retention, target roots, accounting/reserve,
admission/denial, token field/root, rollback, zero-work scope or build/process
repeat mismatch is hard FAIL.

## Routes and precedence

1. `SOFT_CAP_PROJECTION_INVALID`;
2. `SOFT_CAP_ATOMIC_RESERVE_EXCEEDED`;
3. `SOFT_CAP_INNER_SUSPENSION_UNSUPPORTED`;
4. `SOFT_CAP_OUTER_BOUNDARY_INCOMPLETE`;
5. `SOFT_CAP_OUTER_BOUNDARY_SUSPENDED`.

## Authority boundary

This contract grants one report-only policy/token projection. It does not
authorize budget-code mutation, resume, outer 6, state commit, timing, runtime
policy or production authority.

Closed by the
[D7R19R14 soft-cap suspension evidence](../../development/nonlocal-nsr3b4e2d7r19r14-soft-cap-suspension-evidence-2026-08-24.md).
