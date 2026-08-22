# NSR3-B4E2D7R4 -- step-norm trust inner contract

Status: `PASS / STEP_NORM_TRUST_INNER_CANDIDATE / PRIVATE_INNER_ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r4-step-norm-trust-inner|v1|parent=eac500f20a578a6aac8d091476c86a43a9d7e7ef72e0b96cb61ea9b58049c604:6b64edba428fe38544ad6a9bc4d4a8e2ba5100749e15a57a33b94a9a6072d01c:b52f9a599f2b0312fbe73ed8686f623a7142a967b497f507345ed7a47bc00f3c|state=post-outer9-forced-private:31840abd2f10907491360d75d57f6ecbbe6b0cf94672fa1b703bcb5ffa9d5830|solver=unchanged-al-inner;private-copy;stationarity=1e-8;outer-cap=64;reject-cap=8|reject-update=interior-if-norm<0.9delta;direct-delta-f;denom=2*(delta-f-gts);alpha=-gts/denom;new-delta=min(max(alpha,0.25)*norm,0.5*old-delta);valid-finite-positive-smaller;else-quarter-fallback|acceptance=unchanged-raw-actual;raw-ratio>=0.1;positive-model|trace=all-trials;radius-owner;raw-direct;hvp;stationarity;topology-roots|controls=first-event-d7r3-exact;nonpositive-denom-quarter-fallback;rollback|runs=2-release-builds;2-processes;byte-exact;timing=none|trajectory=none;outer-updates=none;commit=none;physics-mutation=none|credit=outer-integration-contract-research-only
```

Identity SHA-256:
`3d7bf83ce177ccd2161dada39caa48c6172277ab5c37f0c2e3c6036676d4d96a`.

## Required command

Add `--nonlocal-al-step-norm-trust-inner`. It must:

1. reproduce the exact D7R failed private state;
2. execute only a separate candidate inner solve with unchanged math and raw
   acceptance, applying the frozen radius rule on rejected interior steps;
3. emit every trial's pre/post radius, owner (`STEP_NORM`, `QUARTER`, `NONE`),
   step norm, HVP count, model/raw/direct reductions, raw/direct ratios,
   acceptance, stationarity and active/fluid/boundary roots;
4. reproduce D7R3's first interpolation radius and first recomputed proposal;
5. converge with scaled stationarity `<=1e-8`, reject count `<=8`, trial count
   `<=64`, finite state and positive model/raw reduction on every accept;
6. prove a nonpositive-denominator scalar control selects exact quarter
   fallback;
7. leave multiplier, outer/public state and commit count exact.

## Gates

- identity, parent stdout and failed-state root are exact;
- the first full rejection and step-norm recompute reproduce D7R3 values;
- at least one `STEP_NORM` update executes and no invalid interpolation owns a
  radius;
- every accepted step passes the unchanged raw `actual>0`, model `>0`,
  `raw_ratio>=0.1` gate;
- final stationarity, work bounds, finiteness, fallback and rollback pass;
- two clean builds/processes are byte-exact and every parent command is
  unchanged.

PASS selects `STEP_NORM_TRUST_INNER_CANDIDATE` and authorizes only research of
a full private outer-AL/confirmation integration contract. It grants no outer
convergence or pressure commit, trajectory, coefficient/tolerance change,
performance, runtime, GPU/PhysX or production authority.
