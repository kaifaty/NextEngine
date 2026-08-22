# NSR3-B4E2D7R3 -- globalization-policy discriminator contract

Status: `PASS / STEP_NORM_AWARE_TRUST_SELECTED / REPLAY_ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r3-globalization-policy-discriminator|v1|parent=034d39324962a5552d5a87d3cb5759e18961401fc87cd9b49e4581e4414b45d0:238dca8baa3434b284acaf88dac3f0279312bee1aad312bf4f559d7064f33a06:36a6c45b08e595818a4ce86a7bde3d88d3a923f250e0ed9d859973aeb083786e|state=post-outer9-forced-private:31840abd2f10907491360d75d57f6ecbbe6b0cf94672fa1b703bcb5ffa9d5830|base=first-rejected-newton;direct-actual;ratio-gate=0.1;no-accept|backtrack=alpha=2^-e,e=0..6;reuse-direction;first-admissible|step-trust=alpha-hat=-gts/(2*((ftrial-fcurrent)-gts));delta=min(max(alpha-hat,0.25)*norm-step,0.5*delta-old);one-steihaug-recompute|legacy=quarter-radius-to-first-binding;one-recompute;report-repeated-work|stability=active-root;fluid-root;boundary-root|routes=step-norm-aware-trust-selected;hybrid-backtrack-selected;legacy-binding-policy-research;steihaug-fallback-research|runs=2-release-builds;2-processes;byte-exact;timing=none|trajectory=none;commit=none;physics-mutation=none|credit=one-policy-implementation-contract-research-only
```

Identity SHA-256:
`eac500f20a578a6aac8d091476c86a43a9d7e7ef72e0b96cb61ea9b58049c604`.

## Required command

Add `--nonlocal-al-globalization-policy-discriminator`. It must reproduce the
exact D7R2R parent and emit:

- base gradient slope, full-step norm/model/raw/direct reductions and roots;
- all seven backtrack rows with rounded step, reductions, ratio and set roots;
- interpolation numerator/denominator, `alpha_hat`, selected step-relative
  radius and its ratio to the rejected norm;
- the recomputed step-norm-aware Steihaug proposal with HVP count, boundary
  status, reductions, ratio and set roots;
- first legacy binding shrink/radius, repeated proposal/HVP/objective counts
  before binding, and the recomputed binding proposal;
- exactly one route under the precedence below.

## Gates and routes

All lanes are finite, replay-only and use the unchanged objective, Hessian,
Steihaug solve, reduction and `0.1` admission ratio.

1. Select `STEP_NORM_AWARE_TRUST_SELECTED` if the interpolated radius is
   positive and smaller than the rejected norm, the recomputed step reaches
   that radius within binary64 relative error `1e-12`, changes no active/fluid/
   boundary set, and has positive predicted/direct reduction with direct ratio
   `>=0.1` using at most two new HVPs.
2. Otherwise select `HYBRID_BACKTRACK_SELECTED` if the frozen ladder finds a
   moved, topology-stable admitted row.
3. Otherwise select `LEGACY_BINDING_POLICY_RESEARCH` if the first-binding
   recompute is admitted and topology-stable.
4. Otherwise select `STEIHAUG_FALLBACK_RESEARCH` if replay and finiteness gates
   pass.

Base D7R2R facts, parent stdout, set roots, rollback and parent command bytes
must remain exact. Any mismatch is hard FAIL. PASS authorizes research/freeze
of one tiny inner-policy implementation contract only. It does not authorize
installing a policy, changing reject cap/initial radius/tolerance/beta/formula,
completing outer AL, committing state, trajectory, performance, runtime,
GPU/PhysX or production work.
