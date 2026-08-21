# NSR3-B4EP3I -- hot-path superset-cache A/B contract

Status: `FROZEN / IMPLEMENTATION_AND_AB_AUTHORIZED / RESEARCH_ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4ep3i-hotpath-superset-cache|v1|parent=19c2c1f27c1bec7444743c1d67ce1bb5d69b8acc22804513ef5e8a6cc21e2552:d59a5fbe86e52bedc9d27bc41f7047a5e294311ce73ea28aa16b81b70fb83312:4d62367830fd5a32f2f1ec32d07ebae6833cf2491c91af255022efdb988d7095|oracle=470a0f4ec9b51ac57f1ecb69e937bb9d2756db41299a0dc01fa427ae63a78e99:25b1f00c0c7477a03532dca2acb97314853e9b4184962d9695792273280f5e04:4d63f5f05811357b958b18380ec483cd97073ae02c3a0228098e255d73da8112|cache=transaction-only;trace-owned;skin=0.04h;certificate=fail-closed;fallback=none|parent=full-state-canonical;candidate=work-only-cached;defaults=byte-unchanged|physics=bit-exact-roots+counters;work=queries226,rebuild1,reuse225,max-degree122,certificate-fail0,candidate-active<=1.25|runs=candidate2-byte-exact;audit-oracle=byte-exact|timing=UNCACHED,CACHED,CACHED,UNCACHED,UNCACHED,CACHED;gate=3/3-wins;median-speedup>=1.10|watchdog=900s|reference=closed|credit=b4ep4-design-only
```

Identity SHA-256:
`e617043b55349894356ce3eda95aca538e99aeec8b127c418d8273bc408a1474`.

## Implementation boundary

Add an optional internal transaction-local superset cache to
`JointQueryTrace` and a final optional cache argument to the macro research
runner. Defaults are null and must preserve every existing report byte.

Add only:

```text
nonlocal-formula-reclosure --nominal-hydro-cached-topology-ablation
```

The command executes canonical/full-state parent preflight, then the same
B4EP1 retained-flat work-only transaction with one empty cache. No fallback,
formula, KKT policy, temporal level, capacity, publication or ledger change is
allowed.

Cached construction must reuse the exact B4EP3 builder/filter/certificate.
It still materializes current flat CSR and rebuilds evaluation/tape every
query. Cache state is destroyed after the transaction and never enters public
state or persistence.

## Correspondence and work gate

Require exact B4EP1 physical facts and roots:

- selected level 1; initial/accepted/attempted/discarded `14/28/42/14`;
- outer/rejected `221/0`, nonlinear/spectral HVP `411/48`;
- exact frame, aggregate, trajectory and both ledger roots;
- exact strain, energy, penetration, publication and retention facts.

Require parent full hashes `1/0`, transaction full hashes `0/226`, 226 cached
queries, cache rebuild/reuse `1/225`, zero certificate failure/fallback,
maximum candidate degree 122 and candidate/active visits `<=1.25`. Flat CSR
logical record/ownership counts remain exact; static fluid-record sorting may
decrease and must be reported rather than relabelled as baseline work.

Run the candidate twice across independent Release builds and require
byte-identical JSON. Require unchanged B4EP1 stdout SHA
`4d63f5f05811357b958b18380ec483cd97073ae02c3a0228098e255d73da8112`
and B4EP3 audit stdout SHA
`4d62367830fd5a32f2f1ec32d07ebae6833cf2491c91af255022efdb988d7095`.

## Timing gate

From identical Release binaries, run six fresh processes in order:

```text
UNCACHED, CACHED, CACHED, UNCACHED, UNCACHED, CACHED
```

Use the 900-second watchdog and external `/usr/bin/time -v`. Pair positions
`(1,2)`, `(4,3)` and `(5,6)`. CACHED must win all three pairs and median
`UNCACHED/CACHED >= 1.10`. Timing and RSS never enter deterministic reports.

## Exit

PASS selects `HOTPATH_CANONICAL_SUPERSET_CANDIDATE` for later nominal research
and authorizes only B4EP4 residual profiling/design. Correspondence failure
rejects the candidate regardless of speed. Speed failure preserves B4EP3
feasibility but routes to HVP design. B4E2, runtime/CUDA, references and
production remain blocked.
