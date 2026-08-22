# NSR3-B4EP10SIRDA -- directed scratch-liveness audit contract

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4ep10sirda-directed-scratch-audit|v1|parent=809807a71e2aafa46758b8635d0ba3f366ef4caee40787282fb287dba3b1eaa4:fdccbf3bcbd0ee23eb702aac04c7237f0a16dec5ac74feb01807689808c965f0:62a2205d6ff010ab65d7730c41479305ec2ec15a5dd45c248ccec38e25d10aa3|implementation=4946af8abc21320685ed0294045df6692fb92ff3|command=nominal-hydro-directed-scratch-audit|candidate=unchanged-split-incoming-return;shadow-liveness-only|proof=active-source-slot-written-once;active-slot-read-twice;inactive-slot-unread;all-targets-assigned;source-endpoint-valid;sequential-depth1|work=full-init-slots;active-write-slots;high-water-growth-slots;eliminated-init-slots|calls=plan226;evaluation226;hvp459|capacity=audit<=67108864;projected-reuse<=67108864|gates=two-fresh-processes;stdout-byte-exact;sii-result-f7b1542f30fef20a08da57c83bb200878cdf426d87b627445e422c9a72825fb2;reuse/full<=0.01;failures0|negatives=missing-write;duplicate-read|timing=none|reference=closed|credit=scratch-reuse-implementation-contract-research-only
```

Identity SHA-256:
`4ae408eb091ee617fa74325a8a2b53652407dd80ba07533b0e6fc8e190c95743`.

## Implementation boundary

Add only:

```text
--nominal-hydro-directed-scratch-audit
```

It executes the unchanged B4EP10SII candidate without phase timing. Shadow
audit data cannot enter any floating branch, value, root or returned
evaluation/HVP. Defaults and all old commands remain unchanged.

## Exact audit

For each of 226 evaluation plans, construct a shadow written/read certificate
from exact source rows, compression and the split incoming plan. Require:

- active slots marked exactly once and inactive slots never marked;
- every target traversal reads marked slots only;
- every marked slot has exactly two reads after all target rows;
- all targets are assigned and every slot's source/other endpoint is valid;
- the tape retains a valid certificate consumed by all 459 HVP calls;
- audit scratch depth never exceeds one.

Accumulate checked full-initialization, active-write, target-read, high-water-
growth and eliminated-initialization counts. Full initialization equals the
sum of directed sizes over 226 evaluation plus 459 HVP invocations. Active
writes equal frozen evaluation/HVP directed-value counters; target reads equal
twice active writes. High-water growth equals the maximum directed size for a
single never-shrunk transaction-local buffer. Require growth/full `<=0.01` and
both audit and projected reusable buffer payloads `<=67,108,864` bytes.

The first eligible plan must also reject shadow missing-write and duplicate-
read injections. Neither negative may alter returned work.

## Execution and exit

Two fresh serialized processes on CPUs `0..7` must exit zero, emit empty
stderr and byte-identical stdout. Both reproduce B4EP10SII semantic result,
all roots, work/candidate counters and zero audit failures.

PASS authorizes only research/freeze of an opt-in directed scratch-reuse
implementation/A-B contract. Failure preserves B4EP10SII and all earlier
negative evidence. B4E2, broad corpus, runtime, CUDA/GPU, schema and production
remain blocked.
