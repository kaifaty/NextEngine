# NSR3-B4E2D7R10 -- private divided-reduction inner contract

Status: `CLOSED / PASS / PRECISION_CERTIFICATE_REQUIRED / PRIVATE_INTEGRATION_ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r10-private-divided-inner|v1|parent=1193920362d57931c518afabb73cc0fb31b466eb:5866926c59596624bfea7341d2c3d45eef4bc15ecb149f5c7e6ac0bbea5dd0a7:2f9a935e4a8f24bc60b8b146424d73c98b5d8dc47756bcf6a70b367f03f47e64|states=eta1e-8:21ad77e22bdba4520ca231bb78d51947a1b67e4263e08dae33af13b0aafabd05;eta1e-9:075442656934aeed156642091e9fc1ed41bc739513b5710990cdfb231d8aabc2;eta1e-10:299e4ce372a8a3d418ac6f354c70c362fd772acdf278464fb603a3881527901c|solver=d7r4-step-norm-trust;limits=64-trials,8-rejects,min-radius1e-14;initial-radius=0.25dx;hvp-and-model-unchanged|change=divided-actual-for-ratio,acceptance,rejected-radius-interpolation;thresholds=0.1,0.25,0.75;radius-bounds-unchanged|audit=every-trial-binary64-repeat;every-candidate-accept-long-double-sign;resolved=1024-extended-ulp;no-resolved-negative-accept|effect=at-least-one-parent-raw-reject-candidate-accept|routes=resolved-sign-contradiction;precision-certificate-required;private-inner-convergence-candidate;inner-policy-still-insufficient|precedence=contradiction,certificate,convergence,insufficient|controls=d7r9r1-complete-bytes;d7r9-fail-bytes;state-roots;common-tight-state;forced-rollback|runs=2-release-builds;2-processes;byte-exact;timing=none|outer-updates=none;trajectory=none;public-commit=none;physics-mutation=none|credit=one-private-inner-integration-classification-only
```

Identity SHA-256:
`438994d226709b3a75d7ca0abfa7b2f4aee2d4e2328e14499362b82248136d5c`.

## Required command

Add `--nonlocal-al-divided-difference-private-inner`. It must:

1. reproduce complete D7R9R1 PASS and D7R9 FAIL bytes plus all three state
   roots and the common tight-state control;
2. run one independent bounded private inner at each frozen stationarity
   request using unchanged D7R4 model/HVP/radius/work limits;
3. use the divided reduction only for actual ratio, acceptance and rejected-
   radius interpolation;
4. retain raw/direct reductions and all topology/branch counts as evidence;
5. reevaluate every divided reduction twice and require bit equality;
6. audit every candidate-accepted trial with the independent extended formula
   and frozen resolved-sign rule;
7. require at least one inherited-raw rejection that the candidate accepts;
8. preserve the input multiplier, force rollback and publish no state;
9. emit exactly one route under the frozen precedence.

Parent, identity, input-state, common-state, binary64 repeat, nonfinite,
unchanged model/HVP/radius policy, candidate-effect, accepted-sign ledger,
rollback or precedence mismatch is hard FAIL.

## Routes

1. `RESOLVED_SIGN_CONTRADICTION`: any candidate acceptance has a resolved
   negative extended reduction.
2. `PRECISION_CERTIFICATE_REQUIRED`: no preceding route; all three inners
   converge and at least one candidate acceptance has unresolved extended
   sign.
3. `PRIVATE_INNER_CONVERGENCE_CANDIDATE`: no preceding route; all inners
   converge and every candidate acceptance is resolved positive.
4. `INNER_POLICY_STILL_INSUFFICIENT`: no preceding route; a finite inner still
   reaches an unchanged reject, trial or minimum-radius limit.

PASS is private classification only. It grants no outer AL integration,
unresolved production acceptance, cap/tolerance, pressure gate, `beta`,
kernel, state precision, trajectory, performance, GPU/runtime or production
authority.

## Closed result

Two clean Release builds/processes reproduce stdout SHA
`ee7b1d4eb0b5eb37334415fa38a1ee2c2a716fa9ab5b628985e7fed06c1c710d`
and semantic result
`48b498487db1265d28123dc2ae2edca92a66bddc2cee7714b3f6ffb77216e77f`.
All three inners converge in one accepted trial and two HVPs with no rejects.
Two accepted signs are resolved positive, none negative and the tight sign is
unresolved. Select `PRECISION_CERTIFICATE_REQUIRED`; see the
[dated evidence](../../development/nonlocal-nsr3b4e2d7r10-private-inner-evidence-2026-08-22.md).

