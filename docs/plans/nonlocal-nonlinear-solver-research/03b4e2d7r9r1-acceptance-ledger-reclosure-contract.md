# NSR3-B4E2D7R9R1 -- acceptance-ledger reclosure contract

Status: `FROZEN / NOT_RUN / PRIVATE_DIAGNOSTIC_ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r9r1-acceptance-ledger-reclosure|v1|parent=b15390b347fb1b6894a520ab327c3014425056d0:b5a0f921a65d82c07a019ee08be970468b80aeec6075c3da706f7623cd346ce9:2c45e93d4153edf714b0f490be3f3d760b14a1ead1e8ed8ae9528d2555ec91f0|correction=inherited-acceptance-ledger;eta1e-8-trial7;eta1e-9-trial5;eta1e-10-none;total2;new-acceptance0|candidate=d7r9-binary64-formula-and-scoring-unchanged|oracle=11-positive;0-negative;12-unresolved|gate=all-11-resolved-signs;relative-error<=0.5;repair-at-least-one-causative;finite;branch-ledger-exact|controls=d7r9-complete-fail-bytes;d7r8-complete-bytes;state-roots;work-acceptance;forced-rollback|routes=compensated-absolute-candidate;divided-difference-candidate;branch-reclosure;stronger-arithmetic|precedence=absolute,divided,branch,stronger|runs=2-release-builds;2-processes;byte-exact;timing=none|trajectory=none;new-trial-acceptance=none;public-commit=none;physics-mutation=none|credit=one-binary64-actual-reduction-candidate-only
```

Identity SHA-256:
`13edb2e1b7c7cd2e761c52c08a1b8b62c6d343546631bf8f8133f0d566138f59`.

## Required command

Add `--nonlocal-al-divided-difference-reclosure`. It must:

1. reproduce complete D7R9 FAIL bytes and D7R8 parent bytes;
2. reproduce inherited acceptance only at `1e-8/trial 7` and
   `1e-9/trial 5`, with none in the tight replay;
3. report two inherited and zero new accepted trials;
4. preserve D7R9 candidate values, branch ledger, oracle ledger, state roots,
   rollback and scoring exactly;
5. emit one unchanged D7R9 route under the same precedence.

Any parent, identity, acceptance-ledger, candidate value, oracle, branch,
state, work, rollback or precedence mismatch is hard FAIL. PASS grants one
binary64 reduction candidate only. It grants no solver integration, new trial
acceptance, cap/tolerance, pressure gate, `beta`, kernel, state precision,
trajectory, performance, GPU/runtime or production authority.

