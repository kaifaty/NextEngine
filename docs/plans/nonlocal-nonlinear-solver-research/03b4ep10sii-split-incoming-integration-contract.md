# NSR3-B4EP10SII -- split incoming integration/A-B contract

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4ep10sii-split-incoming-integration|v1|parent=64dba1abc15451b19874df204411a88f34c99f60ca6f7ec296663708bc853267:0062f28f10e1737b48016df723f8a44124cac25f9740236c03058a7941d515d2:995283db5a74283459eadbbbbdf548074db48e7aa6d4da1e5d3e95b701275200|implementation=41d90e89ab808e853ee4486ae25bcfdfc8363545|commands=baseline:nominal-hydro-owner-parallel-8,candidate:nominal-hydro-split-incoming-plan-8|ownership=topology-result-owned;move-to-tape;transaction-lifetime|construction=pair-endpoint;builds226;regions678;partitions43392|fold=lower-incoming,active-own-row,upper-incoming;canonical|work=incoming-full454936226;incoming-retained374945086;own-retained374945086;target-logical749890172|capacity=actual-added<=67108864|semantics=b4ep10sid+b4ep10sicd;physics-roots-exact;old-commands-exact|timing=external-monotonic+gnu-time;one-warmup-each;three-pairs=AB,BA,AB;serialized;affinity=0-7|gates=candidate-exact-3of3;wins3of3;median-paired-speedup>=1.05;candidate-range-ratio<=1.10;rss-delta-kib<=16384|failure=retain-b4ep10i-active-plan|reference=closed|credit=split-incoming-residual-timing-research-only
```

Identity SHA-256:
`9a496e6129ce4669af31aa056446f3743404bfe38ebf458edff1f6cb7b816774`.

## Implementation boundary

Add only:

```text
--nominal-hydro-split-incoming-plan-8
```

Only this command constructs B4EP10SICD's incoming plan for each current
topology and moves it exactly once into the evaluation tape. It must not build
the serial active plan, full current plan, masked superset plan or any audit
oracle. Old commands must not allocate, move or dereference candidate storage.

Retain the three audited builder regions per topology. Do not fuse phases in
this stage. The target fold uses active lower incoming slots, the active own
source row, then active upper incoming slots. No atomics, floating partials,
runtime option or public schema is allowed.

## Exactness, work and capacity

Require unchanged frame, aggregate, trajectory and both ledger roots; frozen
query/work receipts; exact energy, KKT and adaptive-level gates; 226
evaluation and 459 HVP calls; zero candidate failure or fallback.

Require 226 incoming plan builds and total 4,089 executor regions / 261,696
logical partitions. Candidate floating counters must report exactly:

- 454,936,226 incoming entries scanned;
- 374,945,086 incoming entries retained;
- 374,945,086 own-row entries retained;
- 749,890,172 logical target contributions.

Maximum candidate plan, construction scratch and owner evaluation/HVP scratch
must not exceed 67,108,864 bytes. The three measured candidate stdout streams
must be byte-identical.

The final binary must preserve exact B4EP10I worker-8 stdout
`c47e9393b88208ade76274cff1ee66af00d7da6bef7a461df7b3575b6f4c2ee3`,
B4EP10SICD stdout
`bf4164f6d293c69c5d9774b96d3ece10275b273a95d29c07d56c38fa166f6c5c`
and B4EP10R1 semantic result
`a296ee658196958c69b54421a8a29813b7ce6928f154a215989f5d7f7630560b`.

## External A/B

On the same final Release binary and otherwise idle host:

1. pin both commands to physical CPUs `0..7` with fixed OpenMP placement and
   dynamic teams off;
2. run one unmeasured warmup per command;
3. run three serialized pairs in order `AB`, `BA`, `AB`;
4. record monotonic wall nanoseconds, GNU Time user/system and maximum RSS;
5. require zero exit, empty program stderr and exact stdout before admitting
   duration.

PASS requires candidate exactness `3/3`, candidate wins `3/3`, median paired
speedup at least `1.05`, candidate wall max/min at most `1.10`, and candidate
median RSS no more than 16,384 KiB above baseline median.

## Exit

PASS authorizes only separately frozen residual timing research. Functional
failure rejects the implementation. Performance failure retains B4EP10I but
may route to a separately frozen construction-fusion discriminator. B4E2,
broad corpus, runtime/GPU/schema and production remain blocked.
