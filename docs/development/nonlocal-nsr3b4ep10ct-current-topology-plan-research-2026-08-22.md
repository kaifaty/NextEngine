# NSR3-B4EP10CT current-topology reverse-plan research -- 2026-08-22

Status: `COMPLETE / STRUCTURAL_AUDIT_SELECTED`

## Input

Two exact plan experiments miss the same frozen 5% wall gate:

- B4EP10PI avoids builds but repeatedly scans a fixed superset, expanding
  target reads by 28.85% and increasing CPU work;
- B4EP10PCI keeps compact active rows but adds five integer-heavy parallel
  regions per plan, increasing CPU work for only 3.37% median wall benefit.

The shared mistake is placing pressure-active plan policy after topology has
already been compacted. The current topology contains a smaller,
compression-independent reverse relation that can potentially be constructed
alongside its source CSR and reused by evaluation and every HVP for that
workspace.

## Candidate representation

For each current filtered topology, define a full reverse plan:

```text
source_by_slot[directed_slot]
target_offsets[target]
target_slots[entry]
```

Every current directed slot contributes once to its source target and once to
its participant target. Rows are ordered by current directed slot. Evaluation
and HVP scan those rows and skip sources whose compression is non-positive.
The floating contribution and fold order of retained entries therefore remain
identical if every active row is an exact stable subsequence.

Unlike B4EP10PI, this plan excludes pairs outside the current topology. Unlike
B4EP10PCI, a later design may emit its source and reverse indices during
topology compaction rather than running five new pressure-plan regions.

## Alternatives considered

- **Persistent OpenMP team around the partitioned builder:** deferred. B4EP10R1
  measured low orchestration on the selected path, while B4EP10PCI also adds
  substantial integer scan work; team persistence alone does not remove it.
- **Atomic target cursors:** rejected because arrival order would define
  floating gather order.
- **Parallel radix sort of `(target,slot)`:** deterministic but adds key/value
  buffers and multiple full passes before the smaller topology-owned view is
  tested.
- **Per-target scan of all sources:** rejected as quadratic.
- **Compression-active incremental patching:** deferred because active-set
  insert/delete ordering and transaction rollback are not yet proved.

## First discriminator

B4EP10CTD is audit-only. For every one of the 226 workspaces it builds a
full current-topology plan beside the selected serial active plan by supplying
an all-positive compression sentinel to the existing validated builder. It
then requires:

- exact `source_by_slot`;
- every active target row to equal the stable compression-filtered subsequence
  of the full row;
- exact evaluation and HVP retained counts;
- measured full-row scan ratio at most `1.20`;
- combined side-by-side payload at most 64 MiB;
- corrupt source and target fixtures to reject.

The plan is retained by its evaluation tape only to count projected HVP scans;
the actual evaluation/HVP path continues to use the active plan. No duration
is admitted.

PASS authorizes only construction-dataflow research: how to emit this exact
view during current topology compaction. It does not authorize an
implementation/A-B directly.

## Decision

Freeze B4EP10CTD before code. B4E2, broad corpus, runtime, GPU, schema and
production remain blocked.
