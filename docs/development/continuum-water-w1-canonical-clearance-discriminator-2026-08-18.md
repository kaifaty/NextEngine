# Continuum water W1 canonical-clearance discriminator — 2026-08-18

Status: `REPORT_ONLY / RUNNER_POLICY_DEFECT_CONFIRMED / CONTACT_UNCHANGED / NO_W1_CREDIT`.

## Outcome

The clean Linux W1 run first stopped `CW-ORIFICE-001` at step `95` with all
solver metrics converged but a reported canonical penetration of `1 µm`. The
new successor runner had required `maximum_penetration_um == 0`. That rule was
not part of a frozen W0F or W0G projection and contradicted the inherited
canonical acceptance contract.

The rooted W0B metric definition performs the exact integer branch

```text
distance_squared >= 22_500^2 µm^2
```

and admits reported penetration through `2,500 µm`, inclusive. W0A, SPEC-38
and the existing oracle retain the same limit. W0F's strict-radius-clearance
orifice preflight is a 24-step local contact discriminator; it does not replace
the full-corpus canonical validation threshold. W0F's continuous swept
velocity projection and prohibition on post-integration position repair also
remain unchanged.

The correction therefore restores the pre-existing rooted validation policy;
it does not loosen a threshold after observing a failure, change contact
geometry or issue new solver/profile roots.

## Clean failure

Exact-profile commit `cf8df8b8df6ed22e6f502ed9f7e29d5c8bdf99b0`
stopped at step `95` with:

- density `14 / 78,749 ppb`;
- divergence `1 / 153,149 ppb`;
- reported penetration `1 µm`;
- step `94` as the last accepted frame;
- no retry or continued corpus credit.

The clean failure report has SHA-256
`e2f36b89e088d05c67984311e8e959a29b2b1432e5947ee82399dcdf7b46d5b1`
and remains outside Git at
`/tmp/nextengine-w1-orifice-clean-cf8df8b.json`.

## Canonical witness

A diagnostic-only witness scan reproduced the same step and selected the exact
closest rooted feature:

```text
sample_id=1339
position_um=(975303,204617,403877)
closest_feature_id=19   # aperture z-min edge
distance_squared_um2=624972938
distance_um=24999.458754141058
ideal_radius_shortfall_um=0.541245858942
```

The sub-micrometre continuous shortfall is reported as `1 µm` after the frozen
ties-to-even canonical metric conversion. It is far inside the already frozen
`2,500 µm` acceptance branch and is not an illegal wall crossing. The dirty
diagnostic report has SHA-256
`a655540b381e4e1d6107a5bade245681e9bb17288aa516b584fd7b417368a827`.

## Full bounded counterfactual

Only the runner branch was aligned to the inherited inclusive limit; solver,
geometry, contact, canonical publication, energy semantics and every root were
left byte-identical. The dirty research run then completed `720/720` with:

- maximum penetration `1 µm`;
- density at most `42` iterations and `99,996 ppb`;
- divergence at most `1` iteration and `400,816 ppb`;
- momentum residual at most `44 ppb`;
- positive mechanical-energy excess `0 ppb`;
- mechanical-energy deficit `263,874,944 ppb`, diagnostic under W0G;
- stage-energy closure `0 ppb`;
- exact partition accounting and `6,000` retained samples;
- trajectory root
  `83a67be2871261f8d8273bae04623f8891164b329c010f198515227745c1007f`.

The report has SHA-256
`0764abb4b20a079d4a307eeafcab856cd0ad1afce9ae3e1cf227273bfd092e50`
and remains outside Git at
`/tmp/nextengine-w1-orifice-product-clearance-dev-cf8df8b.json`.
It is dirty-tree evidence and cannot receive W1 credit. The mandatory external
orifice transfer curve was not provided.

## Decision and guard

1. Define one code constant as
   `PARTICLE_RADIUS_UM - MINIMUM_CLEARANCE_UM = 2,500 µm` and use it in both
   historical and successor oracle validation.
2. Admit only the exact inclusive interval `0..=2,500 µm`; negative values and
   `2,501 µm` fail.
3. Retain the closest sample/feature/distance witness on a future clearance
   failure.
4. Do not change swept contact, add positional repair, alter canonical
   rounding or issue new W0F/W0G roots.
5. Repeat the complete orifice scenario on a clean correction commit before
   recording internal W1 progress. Missing or failed external reference input
   continues to block the scenario and ProductCheck.
