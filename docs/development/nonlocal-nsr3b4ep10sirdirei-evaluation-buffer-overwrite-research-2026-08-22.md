# NSR3-B4EP10SIRDIREI evaluation buffer overwrite research -- 2026-08-22

Status: `COMPLETE / F64_OVERWRITE_CONSTRUCTION_SELECTED`

## Question

How can the exact full-write proof from B4EP10SIRDIREA remove redundant value
initialization without changing workspace sizes, arithmetic or ownership?

## Pool limitation

A direct pool of ordinary `std::vector` instances does not realize the audit's
high-water projection by itself. The nominal pair extent averages 379,275 but
reaches 380,511, so pair vectors change logical size. Shrinking a vector and
later growing it with `resize` value-initializes the regrown range even when
capacity is already sufficient. Keeping `.size()` at high-water would instead
change published tape sizes, validation, hashes and iteration bounds.

Avoiding that behavior with raw storage or a new span-based representation
would widen this performance experiment into a storage-identity redesign. The
two-workspace lifetime proof remains useful if later allocation profiling
selects such work, but it is not the smallest next change.

## Narrow candidate

C++17 allocator construction can start the lifetime of a scalar `double` by
default-initialization without assigning a value. Define one internal stateful
allocator mode:

- default mode forwards normal construction and preserves every old command;
- candidate mode skips only the value assignment for zero-argument `double`
  construction;
- construction with values and all non-`double` objects is forwarded normally;
- allocator mode propagates through the existing fused-result move into the
  returned workspace.

Enable candidate mode only for the six audited `double` roles: density,
radius, compression, HVP gradient, HVP second and density contribution. The
gradient `Vec3`, center energy, plans, directed scratch and all other vectors
remain unchanged. B4EP10SIRDIREA proves 345,576,600 such slots, or
2,764,612,800 redundant initialization bytes, are overwritten before read.
The unchanged gradient accounts for the remaining 64,133,376 measured bytes.

This changes neither vector size nor capacity and needs no buffer pool or
release hook. The proof obligation is exact candidate semantics plus counters
showing precisely the six-role scope.

## Measurement design

Freeze one opt-in candidate command against uninstrumented SIRDI. After one
warmup per command, run balanced `AB`, `BA`, `AB` pairs on physical cores
`0..7`. Require exact SIRDI/SII/correspondence/roots/work/reuse/retention in all
candidate processes, three wall wins, at least `1.05x` median paired speedup,
candidate range ratio at most `1.10`, RSS delta at most 16 MiB and median total
CPU ratio at most `1.02`.

## Decision

Freeze B4EP10SIRDIREI as overwrite construction for six `double` buffers only.
Do not add a pool, change vector sizes, touch `Vec3`, fuse arithmetic or reuse
the mode outside the exact candidate. If it passes, reprofile the candidate
before deciding whether allocation pooling remains useful. If it fails, retain
SIRDI and the audit result without widening storage representation.
