# NP1-P2 — compact directed CSR

Status: `SPECIFIED / IMPLEMENTATION_NEXT / EXACT_WORK / REPORT_ONLY`

## Hypothesis

`compact-csr-u16-p2` reduces neighbor-stream bandwidth by storing directed
neighbor sample IDs as checked 16-bit unsigned values whenever the fixture has
at most `65,535` samples. Row offsets remain checked 32-bit values. The
candidate should improve pair-stage p95 by at least `10%` or total p95 by at
least `5%` against retained P1, without regressing either exact-50k decision
profile by more than `2%`.

P2 does not change pair membership, row order, arithmetic association or
executed nonlinear work. It is a storage representation of the same logical
CSR, not a sparse-neighborhood approximation.

## Frozen identities

The adjacent denominator is the retained P1 stack:

```text
nuv-gather-directed-r0
+ pointer-swap-o1
+ nuv-terms-specialized-o2
+ stable-sample-v0
+ fused-owner-terms-p1
+ csr-neighbor-u32-v0
```

The only candidate delta is:

```text
+ compact-csr-u16-p2
```

The 100k report fixture cannot encode all sample IDs in `u16` and therefore
must select the retained 32-bit neighbor representation. That fallback is
reported explicitly and must reproduce the retained result exactly.

## Representation and construction

- Neighbor IDs use `uint16_t` only when `sample_count <= 65,535`.
- Row offsets and counts retain their checked 32-bit representation. The
  current CUDA lab uses non-negative `int32` storage for this logical unsigned
  domain; changing offset type is outside P2.
- Count, exclusive scan, capacity clamp and error signaling are unchanged.
- The fill kernel writes directly into the selected neighbor buffer. A compact
  run must not allocate a shadow 32-bit neighbor array.
- Every candidate ID is range-checked by the fixture limit before launch and
  converted exactly when loaded by a consumer kernel.
- Host capture may widen compact IDs to `int` after timing so existing logical
  CSR hashing and independent validators remain representation-neutral.
- Capacity stays frozen per profile. Compact storage saves exactly two bytes
  per reserved directed-pair slot and introduces no new per-pair temporary.

The first implementation uses the retained `256`-thread launch geometry.
Compile-time `128/256/512` block-size variants are tested only if compact IDs
alone miss their adjacent gate or profiler evidence shows occupancy/latency as
the next discriminator. A block-size choice becomes part of the candidate
identity and receives a fresh adjacent run.

## Arithmetic and timing

Density, fused owner terms, final density and energy capture decode an ID with
an exact integral conversion before indexing sample arrays. Kernel bodies,
slot traversal order, local accumulator order and commit order remain the P1
versions byte-for-byte apart from the neighbor element type.

The timed region includes neighbor construction and every CSR consumer. It
excludes setup, result capture, host widening and report hashing as in NP0/P1.
The tournament performs the frozen 256-execution conditioning window, 32
formal warm-up rounds and 96 alternating measured rounds in one process.
Dynamic identities advance the same 32-step trace in lockstep.

## Correctness and capacity gates

Before timing, P2 must pass:

1. retained P1 CUDA and tiny CPU `f64` oracle gates;
2. exact ordered output and logical CSR equality on a compact tiny fixture;
3. exact stiff-surface `gamma=1000`, i2 correspondence;
4. exact coherent, permuted and every advected trace-step output, logical CSR
   and handoff correspondence;
5. finite-state, symmetric-topology and normalized-momentum bounds;
6. exact 100k `u32` fallback correspondence;
7. allocation evidence proving the compact candidate owns no 32-bit neighbor
   buffer and saves `2 * pair_capacity` bytes;
8. unchanged v0 profile/input/output/CSR hashes.

Any truncated/wrapped ID, changed logical CSR order, added pair allocation,
capacity overflow or repeated arithmetic mismatch rejects P2 before a long
run. Tolerances are not widened.

## Retention and rollback

Retain P2 only if every correctness/capacity gate passes and either pair-stage
p95 improves by at least `10%` or total p95 by at least `5%`, while neither
coherent nor advected total p95 regresses by more than `2%`. Retained P1 is the
rollback identity.

If compact IDs miss the gate, record the negative result. Test compile-time
block sizes only when the measured miss is plausibly launch/occupancy related;
otherwise proceed to P3 under the roadmap's consecutive-low-gain stop rule.
