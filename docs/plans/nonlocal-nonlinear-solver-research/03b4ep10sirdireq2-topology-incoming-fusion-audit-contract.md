# NSR3-B4EP10SIRDIREQ2 -- topology/incoming fusion audit contract

Status: `FROZEN / IMPLEMENTATION_PENDING`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4ep10sirdireq2-topology-incoming-fusion-audit|v1|parent=6518ed9875e227f25b298eeb6fd5b3eb3708d1e7014357125a700e9c0d112964:e5ddff76cff198c62120a7f26dcb6efe5ec0c9a7c908e325704af4047259f14f:e4c4dfd2bd472cec5eea51c444f594e22a57b12eb23b2b688a062239fcdeb4e0|sirdi=b4f847cb4f19b09e951534649515a4504bc07044a13e6c636598b33f247777e9|implementation=edadb2c3044d95f7fe1acfcdb1bfe9ee6ed8cd71|command=nominal-hydro-topology-incoming-fusion-audit|candidate=piggyback-incoming-degree-on-topology-metadata;source+pair-endpoint-on-row-fill;canonical-pair-target-fill|order=pair-sorted-fluid-participant;target-sources-increasing;directed-slots-increasing;exact-sicd-plan|work=plans226;pairs85716150;directed150845996;incoming150845996;baseline-standalone665142896;candidate-standalone85716150;ratio<=0.13;added-regions0|capacity=combined-added<=67108864;pending-plan-depth1;release226;live0|gates=two-fresh-processes;stdout-byte-exact;sirdi-result-b4f847cb4f19b09e951534649515a4504bc07044a13e6c636598b33f247777e9;plan-byte-exact;counts-exact;negatives=endpoint,target-order|timing=none|reference=closed|credit=topology-incoming-candidate-contract-research-only
```

Identity SHA-256:
`1b84bc737e11cd1e3469b35c604ce50ecff32e0897bcc651267fb0c96073273f`.

## Implementation boundary

Add only:

```text
--nominal-hydro-topology-incoming-fusion-audit
```

Run the unchanged SIRDI transaction with an audit-only shadow fused plan. The
accepted SICD builder, split incoming plan, floating evaluation/HVP loops,
formulas, partitions, arithmetic and roots remain unchanged. Existing
commands must not allocate or execute audit storage.

The shadow must collect incoming degrees during topology metadata and source/
pair endpoint slots during topology row fill, then fill incoming target rows
with one canonical current-pair traversal. Compare `source_by_slot`, target
offsets, target slots and payload bytes to the accepted SICD plan before that
plan is moved into the pressure tape. A pending shadow plan has depth one and
must be released after each comparison.

## Exact gates

Require:

- 226 build/comparison/release events and zero final live plan;
- 85,716,150 current-pair visits;
- 150,845,996 incoming-degree increments, source writes, endpoint writes,
  incoming entries, endpoint reads and target writes;
- baseline standalone work 665,142,896 entries, candidate standalone pair
  work 85,716,150 and ratio at most 0.13;
- zero added parallel regions or logical partitions;
- maximum combined selected-path, shadow-plan and shadow-scratch payload at
  most 64 MiB with checked arithmetic;
- byte-exact plans, strictly increasing target rows and zero order, coverage,
  lifetime or fallback failure;
- one corrupted endpoint and one swapped target-order negative both rejected.

Run two fresh serialized processes. Both must exit zero with byte-identical
stdout and empty stderr, retain SIRDI result `b4f847cb...777e9`, SII result,
correspondence, work/lifetime counts and five physics roots. Old SIRDI must
retain stdout SHA-256 `539f1ec5...e7e7`.

## Stop and authority

Do not record or compare wall/process CPU duration. Audit failure retains
SIRDI and closes this fusion hypothesis. PASS authorizes only a separately
researched and frozen candidate implementation contract; it does not authorize
that implementation directly.

No result grants wall speed, B4E2, broad corpus, runtime/GPU/schema or
production authority.
