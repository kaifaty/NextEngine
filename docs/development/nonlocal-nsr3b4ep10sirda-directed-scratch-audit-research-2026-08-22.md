# NSR3-B4EP10SIRDA directed scratch audit research -- 2026-08-22

Status: `COMPLETE / STRUCTURAL_AUDIT_SELECTED`

## Question

B4EP10SIRD establishes that evaluation/HVP directed phases own 38.91% of the
transaction, but those timers combine arithmetic with construction and value-
initialization of full `std::vector<Vec3>` buffers. We need to determine
whether transaction-local scratch reuse is semantically possible before
building or timing it.

## Dataflow observation

On the selected split-incoming path, directed slots are source-row owned:

1. each active source row writes every slot in its CSR interval exactly once;
2. the target fold reads a slot only when that slot's source compression is
   positive;
3. each active slot is read once by its source target and once by the pair's
   other endpoint, in unchanged canonical row order;
4. inactive-source slots are not read;
5. all target outputs are assigned, not accumulated onto a prior invocation;
6. HVP calls are sequential at the transaction level; OpenMP parallelism is
   internal and writes disjoint source-row slices.

If all six properties hold over the exact 226 evaluation and 459 HVP calls,
a single transaction-local directed buffer can retain a high-water size.
Active slots are overwritten before read; inactive contents are irrelevant.
This changes neither formulas nor floating addition order.

## Audit design

Add an opt-in shadow certificate only. For every evaluation plan, mark active
source slots and replay the split target traversal with byte-sized read
counters. Require one write and exactly two reads for every active slot, zero
reads for every inactive slot, complete target assignment and valid source/
endpoint ownership. Store the certificate on the tape; every HVP must consume
a certified tape and report sequential scratch depth one.

Count, without changing the returned path:

- full directed slots currently value-initialized by all evaluation/HVP
  invocations;
- active slots overwritten by arithmetic;
- the maximum directed high-water mark and its byte capacity;
- projected reusable growth initialization, which is exactly the cumulative
  increase of one never-shrunk buffer;
- projected eliminated initialization slots and their ratio.

Inject one missing-write and one duplicate-read shadow mutation into the first
eligible plan. Both must reject without touching returned physics.

## Decision

Freeze B4EP10SIRDA as a timing-free structural audit over the exact
split-incoming candidate. Two byte-identical processes, exact SII semantics,
all liveness identities, both negatives, capacity at most 64 MiB and projected
reusable/full initialization ratio at most `0.01` are required before scratch-
reuse implementation research. No timing or speed credit is available.
