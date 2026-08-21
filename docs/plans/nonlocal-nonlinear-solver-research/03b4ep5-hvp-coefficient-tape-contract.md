# NSR3-B4EP5 -- HVP invariant-coefficient tape A/B contract

Status: `FROZEN / IMPLEMENTATION_AND_AB_AUTHORIZED / RESEARCH_ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4ep5-hvp-coefficient-tape|v1|parent=4240148cadb765b33d6b72b55b84d523cbcce5456774dad1d3918faecdeee42e:171f1544bfbd65a999f393e88d7c32afd6f36df4fdf7fc360fb5527acf614779|candidate=e617043b55349894356ce3eda95aca538e99aeec8b127c418d8273bc408a1474:99a7e4b183f844390dca81beba0d83585bf938cb08e0dd6bd6794c82079fd3fe:b0ed87ff3e9cd1131b0c188c84634ab4c01e0453b91342abd4d6f5bb3ae99055|implementation=65ce739e17def07266f0d2f72b00178451a1fb3e|cache=transaction-only;topology=selected;coeff=gradient,second;build=once-per-pair-per-workspace;hvp=lookup;fallback=none|defaults=byte-unchanged;parent=full-state-canonical;candidate=work-only-cached|physics=bit-exact-roots+counters|work=queries226;hvp459;coeff-builds226;coeff-pairs85716150;kernel-evals171432300;hvp-lookups971831424;payload<=15360000|runs=candidate2-byte-exact;regressions=b4ep1,b4ep3,b4ep3i-byte-exact|timing=BASELINE,CANDIDATE,CANDIDATE,BASELINE,BASELINE,CANDIDATE;gate=3/3-wins;median-speedup>=1.10|watchdog=900s|reference=closed|credit=b4ep6-design-only
```

Identity SHA-256:
`46224e0e70c3fa1a21b3a1fa3b8e5aec81f10fcc6312d99314caec6c91e45e40`.

## Implementation boundary

Add optional HVP coefficient arrays to the internal pressure tape and an
opt-in transaction trace policy. Existing defaults leave the arrays empty and
must preserve every old report byte. Add only:

```text
nonlocal-formula-reclosure --nominal-hydro-hvp-coefficient-ablation
```

The command executes the unchanged canonical/full-state parent and the same
B4EP3I work-only cached-topology transaction with invariant scalar coefficients
enabled. Coefficients are computed once from each tape's already stored radius:

```text
gradient[p] = weight_gradient(radius[p])
second[p]   = weight_second(radius[p])
```

The HVP uses only those arrays when the opt-in policy is enabled. It changes no
other expression, traversal, reduction, solver policy or publication step. A
partial/missing coefficient tape rejects the command; no uncached fallback is
allowed inside an opted-in HVP.

## Correspondence and work gate

Require exact B4EP3I physical roots, counters, cache facts and ownership.
Require additionally:

- 226 coefficient-tape builds over exactly 85,716,150 pair records;
- exactly 171,432,300 coefficient kernel evaluations;
- 459 HVP calls consuming exactly 971,831,424 coefficient lookups;
- maximum added coefficient payload at most 15,360,000 bytes per workspace;
- zero coefficient fallback or mismatch;
- parent full hashes `1/0` and candidate transaction full hashes `0/226`.

Run the candidate twice across independent Release builds and require
byte-identical JSON. Require unchanged stdout SHA-256 for B4EP1
`4d63f5f05811357b958b18380ec483cd97073ae02c3a0228098e255d73da8112`,
B4EP3
`4d62367830fd5a32f2f1ec32d07ebae6833cf2491c91af255022efdb988d7095`
and B4EP3I
`b0ed87ff3e9cd1131b0c188c84634ab4c01e0453b91342abd4d6f5bb3ae99055`.

## Timing gate

From identical final Release binaries, run six fresh processes in order:

```text
BASELINE, CANDIDATE, CANDIDATE, BASELINE, BASELINE, CANDIDATE
```

`BASELINE` is B4EP3I cached topology; `CANDIDATE` additionally caches the two
HVP scalars. Pair positions `(1,2)`, `(4,3)` and `(5,6)`. Every candidate must
win and median paired baseline/candidate speedup must be at least `1.10`.
Timing/RSS remain external under the 900-second watchdog.

## Exit

PASS selects `HVP_INVARIANT_COEFFICIENT_TAPE_CANDIDATE` and authorizes only
B4EP6 residual profiling/design. Correspondence failure rejects the candidate
regardless of speed. Speed failure preserves B4EP4 and returns to HVP research.
B4E2, runtime/CUDA, references and production remain blocked.
