# NP1-P3 — dynamic cell-local storage

Status: `COMPLETE / NOT_RETAINED / P2_ROLLBACK / P4_INPUT / EXACT_WORK / REPORT_ONLY`

## Hypothesis

`dynamic-cell-local-p3` may recover memory locality when stable material IDs
are not spatially ordered. It rebuilds a packed-cell storage permutation at
every substep while preserving stable sample identity and the exact logical
CSR/reduction order. P3 is tested on deliberately permuted and advected v1
inputs; coherent water is a mandatory negative control, not a reason to apply
sorting universally.

The adjacent denominator is retained P1+P2:

```text
nuv-gather-directed-r0
+ pointer-swap-o1
+ nuv-terms-specialized-o2
+ stable-sample-v0
+ fused-owner-terms-p1
+ compact-csr-u16-p2
```

The candidate adds only:

```text
+ dynamic-cell-local-p3
```

P3 does not change the accepted iterate, number of samples, pair membership,
logical row/slot order or arithmetic association. It cannot claim adaptive or
algorithmic credit.

## Stable identity and dynamic remap

Stable material sample ID remains the canonical identity for input, trace
handoff, hashes and reports. At the beginning of every solve:

1. compute a checked packed horizon-cell key from canonical reference
   positions;
2. stable radix-sort `(cell_key, sample_id)` into `storage -> sample`;
3. construct and validate the bounded inverse `sample -> storage` map;
4. gather reference position, velocity and fixed state into cell-local SoA;
5. build the directed CSR by physical owner, storing physical `u16` neighbor
   IDs but retaining each logical owner's fixed 27-cell/sample-ID slot order;
6. execute the retained P1+P2 kernels in physical storage order;
7. scatter accepted position/velocity back to canonical sample-ID order before
   an advected substep publishes its handoff.

Sorting, map construction/validation, gather, scatter and the one required
cell-range build are inside total timing. The remap sort also supplies the
cell ranges for neighbor construction and is not repeated in the same solve.
Host map capture and logical widening/remapping remain outside timing as
diagnostics only.

Map fields are immutable during one solve and may change only between complete
substeps. A capture records both map hashes. Any non-bijection, out-of-grid
sample, wrapped compact ID or stale map is a typed failure.

## Memory and work

P3 may allocate exactly:

- sorted reference positions: `12N` bytes;
- sorted velocities: `12N` bytes;
- sorted fixed flags: `N` bytes;
- two 32-bit permutation maps: `8N` bytes;
- one storage error flag: `4` bytes.

Total added storage is `33N + 4` bytes. Existing radix-sort scratch is reused;
no second CSR, shadow neighbor array or per-edge remap is allowed. P2 compact
CSR remains direct and mandatory for eligible P3 profiles.

## Correctness gate

Before timing:

1. retained P2 checks remain PASS;
2. a coupled tiny fixture matches retained P2 and the CPU `f64` oracle;
3. stiff surface `gamma=1000`, i2 matches retained P2 exactly;
4. coherent and permuted seed output/logical CSR are exact after remap;
5. every 32-step advected output, logical CSR and canonical handoff is exact;
6. every captured map is a bounded inverse pair with deterministic hashes;
7. physical and logical topology are finite, symmetric and within capacity;
8. added memory equals `33N + 4`, with no changed P2 neighbor capacity;
9. v0 retained hashes remain unchanged.

Exact here means identical ordered output and logical CSR digests. Tolerance
widening, reordered logical reductions and rebuilding from predicted rather
than frozen reference positions are forbidden.

## Tournament and retention

The tournament co-resides P2 stable and P3 cell-local instances, performs the
frozen 256-execution conditioning window, 32 warm-up rounds and 96 alternating
measured rounds, and advances dynamic traces in lockstep.

P3 is a conditional storage capability, not the new universal default:

- permuted 50k must improve total p95 by at least `5%`;
- advected 50k must improve by at least `5%` or remain within `2%` of P2;
- no coupled 16k control may regress total p95 by more than `2%`;
- coherent 50k is reported as the negative control and remains on stable P2
  unless P3 unexpectedly improves it;
- all correctness, memory and capacity gates must pass.

If these gates pass, retain P3 only behind an explicit disordered-storage
selection boundary; P2 stable remains the coherent/default rollback. If the
permuted gain cannot amortize remap or advected/coupled regression exceeds its
bound, record P3 as a negative result and proceed to P4 with P2.

P3 proved a `1.779x` permuted gain but failed the two coupled non-regression
gates, so it is not retained. See the
[dated evidence](../../development/nonlocal-continuum-np1-p3-evidence-2026-08-20.md).
