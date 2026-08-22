# NSR3-B4EP10PD -- masked superset owner-plan audit contract

Status: `CLOSED / PASS / B4EP10PI_CONTRACT_RESEARCH_AUTHORIZED`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4ep10pd-masked-superset-plan-audit|v1|parent=ab9f3e0bca369c80e0185d6c73cb8334ff4baa6a9aa8728758b6fe162975d638:ad54e7ab063e88c45dae8ee518a94870c585bc4c5879b2f9d6ae770afe385232|implementation=e6ea63381437a3d5e0179bc164475e6983314836|command=nominal-hydro-masked-superset-plan-audit|baseline=b4ep10i-owner-parallel-8;common=917a04d31bb849a9bee5dd190ad6d15e07c9c90a9c2822130ae1adac6ebcb4ca|cache=queries226;rebuild1;reuse225|candidate=full-superset-target-csr-built-once;stable-active-subsequence;current-slot-map|include=pair-active&&compression[source]>0|proof=target-row-counts;source-order;slot-order;two-target-coverage;plans226|work=current-target-gathers=263974460+485915712;candidate-full-scan-reported;ratio<=1.35|capacity=combined-added<=67108864|negatives=mapping-slot;target-entry|runs=2;stdout-byte-exact;stderr-empty|timing=none;openmp-unchanged|regressions=b4ep10i8;b4ep10r1-semantic|reference=closed|credit=b4ep10pi-implementation-contract-only
```

Identity SHA-256:
`82be83e5131ae5eb3c49a687a764c851f79a81c107417922fd7285e35a986ce1`.

## Implementation boundary

Add only:

```text
--nominal-hydro-masked-superset-plan-audit
```

The returned nominal transaction remains the exact B4EP10I 8-worker path.
The audit may add an optional full target CSR to the transaction-local
topology cache and an optional current-slot mapping to filtered topology. Old
commands may not allocate, populate or inspect either structure. Do not add
timers, a new OpenMP region, atomics, floating reductions or a runtime option.

## Structural oracle

Build the fixed plan once from the canonical B4EP3 superset and its
B4EP10 owner topology rows. Its source order is centre then superset row slot.
For each of 226 current active plans and each target:

1. traverse the fixed target row in stored order;
2. map its superset slot to the current compacted slot;
3. skip missing current slots and sources with non-positive compression;
4. require the resulting current-slot and source sequence to equal the
   existing active plan byte-for-byte;
5. require every retained slot to address exactly its source and participant
   target, with two retained target entries per active directed slot.

All additions, sizes, sentinel conversions and projected multiplications are
checked. Partial state must fail closed.

## Work and capacity gates

Require exactly 226 audits, one fixed-plan build, 225 plan reuses and zero
order, coverage, mapping or fallback mismatch. Report fixed directed slots,
fixed target entries, retained entries and full-plan scan projection across
226 evaluations plus 459 HVPs.

The projection denominator is the exact existing
`263974460 + 485915712 = 749890172` target gathers. The candidate ratio must
be at most 1.35. Maximum combined added nominal payload, including persistent
plan, current-slot mapping and existing owner scratch, is 67,108,864 bytes.

Dedicated copies must reject one out-of-range current-slot mapping and one
wrong fixed target entry without publishing a successful audit.

## Runs and regressions

Run two fresh Release processes pinned to CPUs `0..7`, with
`OMP_PLACES=threads`, `OMP_PROC_BIND=close` and dynamic teams disabled. Both
must exit zero, produce empty stderr and byte-identical stdout. Require the
B4EP10I common correspondence hash and unchanged physical/output roots.

The final binary must retain exact B4EP10I worker-8 stdout SHA-256
`c47e9393b88208ade76274cff1ee66af00d7da6bef7a461df7b3575b6f4c2ee3`
and B4EP10R1 semantic result SHA-256
`a296ee658196958c69b54421a8a29813b7ce6928f154a215989f5d7f7630560b`.

## Exit

PASS authorizes only a separately frozen B4EP10PI opt-in implementation and
controlled A/B. Scan/capacity failure rejects masked reuse and routes to
partition-local stable counting-sort research. Exactness failure preserves
the current active-plan implementation and closes this candidate. B4E2,
broad corpus, runtime/GPU/schema and production remain blocked.

## Closure

B4EP10PD passes; see the
[dated evidence](../../development/nonlocal-nsr3b4ep10pd-masked-superset-plan-evidence-2026-08-22.md).
All 226 filtered row sequences are exact. Projected scan expansion is
`1.288507x` and conservative added payload is 38,044,404 bytes. This
authorizes only B4EP10PI opt-in implementation/A-B contract research.
