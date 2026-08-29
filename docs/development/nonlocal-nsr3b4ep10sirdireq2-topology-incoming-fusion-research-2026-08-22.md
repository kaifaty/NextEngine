# NSR3-B4EP10SIRDIREQ2 topology/incoming fusion research -- 2026-08-22

Status: `CLOSED / AUDIT_PASS / CANDIDATE_CONTRACT_RESEARCH_AUTHORIZED`

## Question

What exact structural work causes the Q1 topology leader, and is there one
bounded redundancy that can be removed without revisiting rejected gather-plan
representations or using unqualified wall timing?

Q1 assigns median `0.301239` of transaction process CPU to topology. The sum
of its nine existing topology subphases leaves
`5.258894099 / 6.399587042 / 5.332367200 s` unclassified inside the topology
stage: `48.02%--50.74%` of topology CPU and `14.67%--15.26%` of transaction
CPU. This residual is not itself attributed exclusively to one function, but
it contains the split-incoming construction that runs after topology finalize.

## Existing structural cost

For the exact SIRDI transaction, `b4ep10sicd_build_incoming_plan` performs:

| Standalone work | Aggregate entries |
|---|---:|
| source/endpoint map over directed rows | 150,845,996 |
| two current-pair passes for support CSR | 171,432,300 |
| target-count scan | 171,432,300 |
| target-fill/validation scan | 171,432,300 |
| total | 665,142,896 |

It also launches three regions per topology, 678 across 226 plans. The current
topology builder has already visited the same active pairs, counted rows and
filled the exact center-major directed slots before this construction begins.

## Alternatives considered

1. Reuse a masked superset plan. B4EP10PI already rejects this route: 28.85%
   scan expansion raises CPU and misses the speed gate.
2. Build an active plan with partitioned counting sort. B4EP10PCI is exact but
   its extra scans/regions raise CPU and miss the speed gate.
3. Build a full current-topology plan. B4EP10CTD already fails its scan gate.
4. Cache the current active topology. The fixed lattice includes horizon-edge
   pairs, so a global active-set margin is not an established certificate;
   pair-count stability is not pair-identity proof.
5. Fuse incoming construction into the already selected topology passes. This
   uses no expanded pair set and preserves the exact current-pair identity.

Select alternative 5 for one timing-free structural audit.

## Selected construction

The audit shadows, but does not replace, the accepted SIRDI path:

1. During existing topology metadata, increment incoming degree for both
   fluid endpoints or only the support endpoint. The aggregate is exactly the
   current directed-slot count.
2. During existing topology row fill, write `source_by_slot` and the source/
   participant slot for each active pair.
3. Prefix the incoming degrees after metadata.
4. After row fill, traverse current pairs once and append their endpoint slots
   to the corresponding target rows.

Canonical pairs are sorted by `(fluid, participant)`, with a fluid-fluid pair
stored at its lower fluid endpoint. Directed slots are center-major and each
center row is participant-major. For a fixed fluid target, pair order therefore
visits lower sources first and higher sources second, both in increasing source
order; for a support target it visits fluid sources in increasing order. The
resulting target slots are strictly increasing and must match the existing
SICD plan byte for byte.

The candidate adds one standalone pass over 85,716,150 current pairs and no
OpenMP region. Its standalone scan ratio is
`85,716,150 / 665,142,896 = 0.12886877468807845`. Piggyback writes remain
explicit work; this ratio is not a speedup prediction.

## Decision

Freeze B4EP10SIRDIREQ2 as an audit only. Require all 226 fused plans to match
the accepted plan, exact structural counts, bounded depth/lifetime/capacity,
two corruption negatives and two byte-identical fresh-process reports. Do not
record duration.

PASS may authorize only research and freezing of a separate opt-in candidate
implementation contract. It grants no implementation, wall speed, B4E2,
broad-corpus, runtime/GPU/schema or production authority.

## Closure

The audit passes in both frozen fresh processes. All 226 shadow plans are
byte-exact, the standalone ratio is `0.12886877468807845`, no parallel region
is added and every corruption/lifetime/capacity gate passes. See the
[dated evidence](nonlocal-nsr3b4ep10sirdireq2-topology-incoming-fusion-evidence-2026-08-22.md).

The next admitted action is research and freezing of a separate opt-in
candidate implementation contract. The audit itself remains shadow-only and
grants no speed or production claim.
