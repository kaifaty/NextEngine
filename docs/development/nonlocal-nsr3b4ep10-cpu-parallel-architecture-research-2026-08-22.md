# NSR3-B4EP10 deterministic CPU-parallel architecture research -- 2026-08-22

Status: `COMPLETE / OWNER_COMPUTES_SELECTED / B4EP10D_REQUIRED`

## Problem

B4EP9 measures 92.19% median conservative parallelizable work on a 16-core,
32-thread Ryzen 9 3950X, but the present exact implementation intentionally
uses one core. The opportunity is real; a naive `parallel for` is not valid.
Density, gradient and HVP currently scatter floating-point contributions into
shared particles. Atomics would remove data races but would not preserve the
canonical addition order, and per-worker partial arrays would regroup the
sum. Either can change binary64 results and therefore the solver path.

## Repository and external constraints

The production Rust water solver already contains the right scheduling
precedent: 64 fixed logical partitions are mapped onto a bounded Rayon pool,
outputs have unique owners, fragments merge by logical ordinal, and a worker
failure rejects before canonical merge. This research must reuse those
invariants conceptually without coupling the isolated C++ oracle to runtime.

OpenMP documents non-SIMD `schedule(static)` as reproducible and assigns chunks
in a specified thread order ([OpenMP 5.1 worksharing-loop specification](https://www.openmp.org/spec-html/5.1/openmpsu48.html)).
That makes it a viable later execution backend, but scheduling alone does not
make a floating-point scatter deterministic.

oneTBB offers `parallel_deterministic_reduce`, yet its own specification notes
that the deterministic result may differ from the equivalent sequential
linear algorithm ([oneTBB specification](https://oneapi-spec.uxlfoundation.org/specifications/oneapi/latest/elements/onetbb/source/algorithms/functions/parallel_deterministic_reduce_func)).
That is insufficient for the current bit-exact oracle identity. The ongoing
C++ reproducibility proposal likewise describes floating-point reproducibility
as an explicit constrained choice rather than a default language guarantee
([WG21 P3375R2](https://www.open-std.org/jtc1/sc22/wg21/docs/papers/2025/p3375r2.html)).

## Selected dataflow

Use fixed logical partitions over canonical indices and unique-owner output
slices. Worker count may change scheduling, never partition boundaries,
arithmetic within one output or merge order.

### Topology/filter/CSR

1. Compute one active flag for every canonical superset pair independently.
2. Build a canonical exclusive prefix over flags.
3. Compact active pairs in original pair order.
4. Rebuild each centre-owned CSR row from the superset row and the active-pair
   map. Each row is written by exactly one logical owner.

The first superset construction occurs only once in 226 queries and remains
serial initially. The repeated filter/CSR path is the admitted target.

### Fused evaluation/tape

1. Compute radius, density scalar and HVP coefficients once per pair into
   pair-indexed arrays.
2. Compute each density by gathering its CSR row in stored pair order. That
   order is the restriction of the original global pair order to the target
   particle, so the additions can remain bit-identical.
3. Compute compression and per-centre energy independently; fold scalar energy
   serially by ascending centre index.
4. Compute one directed gradient value per active source-row slot.
5. Build a transpose list by scanning source centre/slot order, then gather
   each target particle's values in that unchanged order.

No atomics or cross-owner writes are permitted.

### HVP

1. Compute `compression_direction` independently per active centre, retaining
   the original row order.
2. Compute one HVP vector per active directed source slot.
3. Gather each target from the same transpose list in global source-slot
   order, with the original unary sign operation.

The transpose plan is immutable per tape. HVP scratch is transaction-local and
bounded; it is not public or persistent state.

## Failure and scheduling model

- Later parallel execution uses 64 fixed logical partitions and an explicit
  worker count; initial scaling points are `1,2,4,8,16`, excluding SMT 32.
- Every worker writes only its declared output slice or logical-partition
  status slot.
- Allocation, capacity and topology validation occur before parallel work.
- Kernels do not throw across a worker boundary. A reported worker failure is
  selected by lowest logical ordinal after the barrier and rejects before the
  candidate becomes observable.
- Dynamic scheduling, atomics, floating reductions, work-stealing-dependent
  merges, affinity requirements and nested parallelism are excluded.
- OpenMP is the selected research backend after the serial dataflow audit,
  because GCC 15.2 and libgomp are already present in the isolated reference
  toolchain. This does not select a production dependency.

## Rejected alternatives

- **Atomics:** race-free but addition order is completion-dependent.
- **Per-worker full output plus tree merge:** deterministic trees can still
  regroup the serial sum and consume `O(workers * particles)` memory.
- **oneTBB deterministic reduction:** repeatable, but not required to equal the
  serial linear reduction.
- **`std::async` per phase:** the file uses it for coarse independent test
  lanes, but 226 workspace and 459 HVP calls need a persistent team.
- **Parallel initial superset build first:** only one of 226 topology queries;
  repeated filtering is the better first target.
- **GPU now:** B4EP9 establishes a CPU decomposition question only; NSR5 and
  its correspondence gates remain separate.

## Sequential roadmap

| Stage | Purpose | Exit |
|---|---|---|
| B4EP10D | serial owner-computes/topology-plan audit | 226 topology, 226 evaluation and 459 HVP alternatives match the existing oracle bit-for-bit; nominal added peak at most 64 MiB |
| B4EP10I | opt-in OpenMP implementation | exact results at worker counts `1,2,4,8,16`; failure/capacity negatives reject before publication |
| B4EP10S | serialized scaling experiment | balanced Release timing selects worker count and establishes real utilization/speedup, not just Amdahl potential |
| B4EP10R | residual attribution | reprofile the selected count before another optimization or B4E2 decision |

Each stage is frozen before implementation. Failure at B4EP10D rejects the
owner-computes design; it does not relax exactness. Failure at B4EP10I retains
the serial fused path. No stage here creates runtime or production authority.

## Decision

Freeze B4EP10D first. Do not add OpenMP or threads until the serial
owner-computes transformation and topology plan reproduce every admitted
oracle result exactly.
