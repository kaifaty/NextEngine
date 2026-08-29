# NSR3-B4EP10PI masked plan implementation research -- 2026-08-22

Status: `COMPLETE / OPT_IN_MASKED_PATH_SELECTED`

## Input

B4EP10PD proves that every one of 226 active target plans is the same
order-preserving filtered view of one 705,284-slot superset plan. The
structural candidate passes its 1.35 scan and 64 MiB capacity gates, but its
extra 28.85% target reads may still outweigh the removed plan builds.

## Selected ownership

The fixed `JointOwnerGatherPlan` remains owned by the transaction-local
topology cache. Each candidate workspace owns its current
superset-slot-to-current-slot mapping and keeps a non-owning pointer to the
fixed plan. Admission requires the cache to outlive every workspace; the
candidate command releases all workspaces before its local cache is destroyed.
This pointer is research-only and is not added to runtime or persistent state.

Evaluation and HVP target gathers traverse a fixed target row in stored order:

```text
fixed slot
  -> missing current slot? skip
  -> source compression <= 0? skip
  -> read current directed_value[current_slot]
  -> add with the same target/source sign
```

B4EP10PD proves that retained entries equal the current active row in exact
order. Directed-value construction remains indexed by the current compacted
slot. The scalar energy fold remains serial in canonical centre order.

The candidate does not construct the active owner plan. It must report one
fixed build, 225 reuses, zero active builds, 966,239,080 full entries scanned
and 749,890,172 retained entries over 226 evaluation and 459 HVP calls.

## Timing design

Use two explicit commands from one final Release binary:

- baseline: `--nominal-hydro-owner-parallel-8`;
- candidate: `--nominal-hydro-masked-superset-plan-8`.

After one unmeasured warmup of each, run three serialized physical-core pairs
in order `AB`, `BA`, `AB`, pinned to CPUs `0..7`. Record external monotonic
wall, GNU Time user/system and maximum RSS. Program stderr is separate from
the time record.

The candidate must be exact in all three runs, win all three same-round pairs,
reach at least `1.05x` median paired speedup, have a wall range ratio at most
1.10 and increase median RSS by at most 16,384 KiB. Durations are not part of
the semantic result.

## Failure and route

Any exactness, work, lifetime or capacity failure rejects the candidate and
retains the B4EP10I active plan. A speed/RSS failure retains it only as
negative research evidence; structural B4EP10PD remains true.

PASS authorizes only masked-path residual timing research. It does not
authorize B4E2, broad corpus, runtime, GPU, schema or production use.

## Decision

Freeze B4EP10PI before implementation and timing.
