# NSR3-B4EP10SICD -- incoming-plan construction audit contract

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4ep10sicd-incoming-construction-audit|v1|parent=5b730b2e84676f8b47aa6b16ca2e519ae8b7dff43e935a1fc2ba642bed976b4c:a1fbe1e255ca3b95c97c4b446c0f750448faf82b1bb9a461f2acfd35882188c0:34a15ad1d08d86f736328b4b701b1518315396eec1cbd49e64c9278c14c36bbf|implementation=d5aa920a237618e2e9b0c2e47ebedd4623198f43|command=nominal-hydro-incoming-construction-audit|candidate=source-slot+pair-endpoints;fluid-target-own-row;support-target-pair-csr|phases=parallel-source-map,parallel-target-count,serial-offset-prefix,parallel-target-fill-validate|order=neighbor-source-ascending;support-pair-global-order|audit=plans226;directed150845996;incoming150845996;arrays-byte-exact|executor=additional-regions678;additional-logical-partitions43392;workers8|capacity=combined-added<=67108864|negatives=pair-endpoint;support-cursor|runs=2;stdout-byte-exact;stderr-empty|timing=none;old-commands-exact|reference=closed|credit=b4ep10sii-integration-contract-only
```

Identity SHA-256:
`64dba1abc15451b19874df204411a88f34c99f60ca6f7ec296663708bc853267`.

## Implementation boundary

Add only:

```text
--nominal-hydro-incoming-construction-audit
```

The returned evaluation/HVP remains B4EP10I. Build B4EP10SID's full-plan
oracle and the independent pair-endpoint candidate beside it for every current
topology; discard both after their tape/workspace lifetime. Old commands must
not allocate candidate arrays or enter its three regions.

The candidate uses no compression, atomics or floating reduction. Endpoint
slot writes have unique pair/endpoint ownership. Target fill has unique target
ownership. Support rows derive from current pairs in preserved global
`(fluid,participant)` order. Any missing/duplicate endpoint, degree mismatch,
non-increasing row or payload overflow rejects without fallback.

## Exact work and capacity

Across 226 plans require exactly 150,845,996 directed source slots and the
same number of incoming target entries. Candidate `source_by_slot`, incoming
target offsets and incoming target slots must equal the B4EP10SID oracle
byte-for-byte.

Require exactly 678 added parallel regions and 43,392 logical partitions.
Report fluid-fluid/support pair visits, support CSR records, endpoint writes,
target counts and fills. Maximum candidate plan, endpoint/support scratch and
existing owner payload combined must not exceed 67,108,864 bytes.

Dedicated copies/injection must reject one wrong pair endpoint and one wrong
support cursor before an exact candidate is reported.

## Runs and regressions

Run two fresh Release processes pinned to CPUs `0..7` with fixed OpenMP
placement and dynamic teams off. Require zero exit, empty stderr and
byte-identical stdout. No duration receives evidence credit.

The final binary must preserve exact B4EP10I worker-8 stdout
`c47e9393b88208ade76274cff1ee66af00d7da6bef7a461df7b3575b6f4c2ee3`,
B4EP10SID stdout
`2a7f044d402f1f7d7b0a4fa5a2da288101dc21f257ab76d0f549482bac64548a`
and B4EP10R1 semantic result
`a296ee658196958c69b54421a8a29813b7ce6928f154a215989f5d7f7630560b`.

## Exit

PASS authorizes only separately frozen B4EP10SII floating integration/A-B
research. Failure retains B4EP10SID structural evidence and B4EP10I selected
execution. B4E2, broad corpus, runtime/GPU/schema and production remain
blocked.
