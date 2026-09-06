# NCGP3 scalable compensated-state graph-control corrigendum

| Field | Value |
| --- | --- |
| Research ID | `NCGP3` revision 2 |
| Status | `FROZEN / CORRECTNESS_FIRST / REPORT_ONLY` |
| Supersedes | Only the graph-control input construction in NCGP3 revision 1 |
| Claim class | Unchanged: bounded 4k correctness; no 50k or performance claim |

## Reason for the corrigendum

NCGP3 revision 1 requires the physical trajectory corpus to start from shared
binary32 bytes. That requirement is retained. Its mandatory graph apparatus
control also asks for identical high parts whose different low parts change
exact support membership. Such a low part cannot be supplied by a one-part
binary32 physical input, so the control needs an explicitly admitted canonical
pair fixture. This corrigendum freezes that fixture before implementation.

## Frozen graph apparatus fixture

The physical trajectory corpus, including all 4k states and ghosts, still
round-trips through shared binary32 bytes before the CPU and GPU routes. The
single graph apparatus control below instead admits the stated canonical
`(hi, lo)` bytes directly. It exercises graph addressing only and cannot be
used as a physical trajectory input or as evidence for formula precision.

For the x coordinate, use these exact binary32 bit patterns:

```text
owner.hi        = 0x3ccccdd9 = 0.02500049956142902374267578125
owner.lo        = 0x30388db0 = 6.7140160098233536700718104839324951171875e-10
candidate.hi    = 0x3e333376 = 0.1750009953975677490234375
candidate.lo    = 0x00000000
horizon_um      = 150000
```

The y and z coordinates are equal, finite, interior binary32 values with zero
low parts. Owner and candidate IDs are distinct and the owner pair must pass
the NCGP2 canonical-pair predicate.

The required reconstructions are:

```text
llrint(double(owner.hi) * 1_000_000)                   = 25000
llrint((double(owner.hi)+double(owner.lo))*1_000_000) = 25001
llrint(double(candidate.hi) * 1_000_000)              = 175001
```

Therefore high-only addressing observes an x distance of `150001 um` and
must exclude the directed cross-pair, while pair-aware addressing observes
exactly `150000 um` and must include it under the frozen `<= horizon` rule.
Both owner rows include self. The correct two-particle CSR therefore has four
directed pairs; the high-only mutation has two. ID permutation must preserve
the canonical graph root after mapping rows back to stable IDs.

The implementation must seal the original high/low bytes, reconstructed
integer coordinates, cell keys, distance predicates, row counts, pair count
and canonical graph root. Merely changing a variant identifier or expected
root is not a successful control.

## Unchanged requirements

All other NCGP3 revision-1 representation, graph, boundary, transaction,
corpus, work, failure, review and stop rules remain unchanged. In particular,
this corrigendum does not authorize a host-origin shortcut, binary64 physical
formulas, tolerance changes, 50k execution or performance timing.
