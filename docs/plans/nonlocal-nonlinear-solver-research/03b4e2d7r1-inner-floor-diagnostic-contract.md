# NSR3-B4E2D7R1 -- inner-floor observability contract

Status: `FAIL / TOPOLOGY_GATE / REPLAY_ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r1-inner-floor-diagnostic|v1|parent=86427f686fd719655497f7171cace1ccaaea9353a5e0d919766a6309f11dfe2d:b384964ddb70aede09d6dc3994d851b90ca3af7153ef340d9ed45423c5c53a63:4b0272df2d46bb53ec09175ea7095b0e8c699ac9e0e4e093c6e20dfd00240801|state=d7-prefix:04a9c03308662d6102b165d8109b23c1b60a3145314603909b7dbfda8385d95e:9bffc61a943f50cab449c052bd0a6791c40128e894d98d9a9589b0969dbb82c2;post-outer9-forced-private:31840abd2f10907491360d75d57f6ecbbe6b0cf94672fa1b703bcb5ffa9d5830|replay=unchanged-d7r-through-first-inner-failure;beta=1226.25;inner-stationarity=1e-8;trust=unchanged;reject-limit=8;no-state-commit|trace=initial-energy-bits;initial-gradient-stationarity;constraint;lambda;9-trials;trust-radius;step-norm;hvp;predicted;raw-actual;ratio;energy-ulp;active-topology|direct-difference=inertia-factor:(trial-current)dot(trial+current-2pred)*mass/(2dt2);al-factor:sum((at-ac)*(at+ac)/(2beta));fixed-order;diagnostic-only|routes=nested-accuracy-and-direct-merit-reclosure-if-model-positive-topology-stable-direct-descent-hidden-by-raw;inner-accuracy-schedule-only-if-raw-and-direct-admit;model-or-active-set-research-otherwise|runs=2-release-builds;2-processes;byte-exact;timing=none|trajectory=none;commit=none;physics-mutation=none|credit=inner-remediation-contract-research-only
```

Identity SHA-256:
`8902a9417b2dad68c51fab118892767e2f21be5191cbea0fcbe2e87e9079d862`.

## Required command

Add `--nonlocal-al-inner-floor-diagnostic`. It must:

1. reproduce D7R through its exact post-outer-9 state and first inner failure;
2. replay that inner call without accepting or publishing a trial;
3. emit initial total/support/inertia energy bits, gradient norm/scaled
   stationarity, constraint and multiplier ranges;
4. emit all trust trials through reject limit with radius, step norm, HVP
   count, predicted/raw/direct actual reductions, raw ratio, current-energy
   ULP and active/topology correspondence;
5. prove public state/commit count remain exact and emit one route.

## Frozen gates and routes

- D7 prefix roots, post-outer-9 forced-private root and D7R
  `INNER:REJECT_LIMIT` reproduce exactly;
- initial and trial values are finite; exactly nine rejected trials execute
  under unchanged trust policy with zero replay-state accepts;
- every reported topology and active set matches the replay initial state;
- `NESTED_ACCURACY_AND_DIRECT_MERIT_RECLOSURE_REQUIRED` requires positive
  predicted and direct actual reduction for every trial, while every raw trial
  is rejected because its raw actual/ratio loses that descent;
- `INNER_ACCURACY_SCHEDULE_ONLY` requires raw/direct sign and admission
  agreement for every trial;
- otherwise select `MODEL_OR_ACTIVE_SET_RESEARCH_REQUIRED` when replay itself
  remains exact; reproduction/nonfinite/count/rollback mismatch is hard FAIL.

Build Release twice and run two fresh processes without timing. PASS means the
diagnostic classified the failure byte-identically. It authorizes one new
remediation research/contract only. It grants no solver PASS, trajectory,
state commit, performance, runtime, GPU/PhysX or production authority.
