# NSR3-B4EP10SI split self/incoming plan research -- 2026-08-22

Status: `COMPLETE / THREE_PART_FOLD_AUDIT_SELECTED`

## Input

B4EP10CTD proves that a compression-independent current-topology plan has the
right stable-subsequence semantics, but rejects its `1.213341x` full scan.
The full plan duplicates every directed value into two target entries:

```text
directed value from source s to participant p
  -> +value in target row s
  -> -value in target row p
```

The first entry does not require a reverse relation. All slots owned by source
`s` are already contiguous in the canonical source CSR.

## Selected representation

Retain only participant/incoming reverse entries. For a fluid target `t`, the
canonical global slot order decomposes exactly into:

```text
incoming slots whose source < t
own source row [offset[t], offset[t + 1])
incoming slots whose source > t
```

The own row is included only when `compression[t] > 0`. Incoming entries are
included only when their source compression is positive. Support targets have
no own row and consume only incoming slots.

Because directed slots are source-major, lower incoming slots are strictly
before the own interval and upper incoming slots strictly after it. The
three-part fold can therefore reproduce the active target row without sorting
or changing floating association.

## Exact work projection

B4EP10CTD's measured full target entries split evenly into self and incoming:

| Component | Evaluation + HVP entries |
|---|---:|
| full incoming scan | 454,936,226 |
| retained own-row fold | 374,945,086 |
| total projected visits | 829,881,312 |
| selected active target entries | 749,890,172 |
| projected ratio | 1.1066704738730726 |

This is below a new frozen `1.15` structural gate. It is not a timing or
speedup claim. Incoming plan payload should also fall from three directed
arrays to two directed arrays plus target offsets.

## Alternatives

- **Full current target rows:** rejected by B4EP10CTD's scan gate.
- **Active source-span lists:** preserve order but add span traversal close to
  the directed count while retaining a full target plan.
- **Scatter self contributions separately after incoming:** rejected because
  it moves the own fold relative to lower-source contributions.
- **Atomics or partial reductions:** rejected because they lose canonical
  addition order.

## Discriminator

B4EP10SID constructs an audit-only incoming view from the exact full current
plan. For every target it reconstructs the active slot stream using lower
incoming, active own row and upper incoming, then compares it byte-for-byte
with the selected active target row.

Across 226 evaluations and 459 HVP projections it requires the exact work
counts above, bounded incoming-plan payload, and separate corrupt own-boundary
and incoming-target rejection. The returned floating path remains B4EP10I;
no duration is admitted.

PASS authorizes only research and freezing of a topology-compaction builder
for the incoming view. It does not authorize floating integration or A/B.

## Decision

Freeze B4EP10SID before code. B4E2, broad corpus, runtime, GPU, schema and
production remain blocked.
