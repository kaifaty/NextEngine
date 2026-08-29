# NSR3-B4EP7D -- evaluation/tape dataflow audit contract

Status: `CLOSED / PASS / B4EP7I_DESIGN_AND_AB_ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4ep7d-evaluation-tape-dataflow-audit|v1|parent=75b8a8b7ba137674091413e8f0eb7ace46f32a2c09059de260c06c36955db24d:e7690846477ab60a753510926f146dcb5d1eaa6661db31b6d164d6bf5cfa2945|candidate=46224e0e70c3fa1a21b3a1fa3b8e5aec81f10fcc6312d99314caec6c91e45e40:5cd61e3eb82f6a3cedfe4d7d8e39cbb5aca65c6e00c5ef9cd9e7555a71b9bf23:dac62e7528e08bc6d9dec91458bd2f7d78a03b75c0e8554f86d1fda5e89ae73b|implementation=7263d8929491b66ea94eb74713fab4c136efe5be|audit=transaction-only;post-tape-derived;no-hot-loop-increments|baseline=eval-radius=N+D;tape-radius=N;coefficient-gradient=N;coefficient-second=N;compression=2C|fused=radius=N;gradient=N;second=N;compression=C;density-order=unchanged;gradient-order=unchanged|gate=queries226;N=85716150;C=1356000;D>0;radius-current=2N+D;radius-fused=N;gradient-current=N+D;gradient-fused=N;zero-fallback|runs=2-byte-exact;regressions=b4ep1,b4ep3,b4ep3i,b4ep5-byte-exact|timing=none|reference=closed|credit=b4ep7i-design-only
```

Identity SHA-256:
`261bcd72315c16836f782751738a4e4a1bcd8a10dd9ba5cab9ff5478ecb987d0`.

## Implementation boundary

Add transaction trace totals for fluid-center visits and active directed
records. Update them once after each pressure tape passes; do not add hot-loop
increments. Add only:

```text
nonlocal-formula-reclosure --nominal-hydro-evaluation-tape-dataflow-audit
```

The command runs the unchanged canonical/full parent and exact B4EP5 cached,
coefficient-enabled transaction. It reports derived baseline/fused counts; it
does not execute a fused path.

## Gate

Require exact B4EP5 physical roots, schedule, topology-cache facts,
coefficient counts, full/work-only ownership and zero fallback. Require:

- 226 transaction queries;
- `N=85,716,150` pair visits and `C=1,356,000` center visits;
- deterministic nonzero `D` equal to the sum of tape `active_directed`;
- current radius work exactly `2N+D`, projected fused radius work `N`;
- current gradient-kernel work exactly `N+D`, projected fused work `N`;
- current/fused compression work `2C/C`;
- positive removable radius and gradient work with checked arithmetic.

Run across two independent clean Release builds and require byte-identical
audit stdout. Require unchanged stdout SHA-256 for B4EP1, B4EP3, B4EP3I and
B4EP5. No timing is admitted.

## Exit

PASS freezes `D` and authorizes only B4EP7I fused evaluation/tape design and
controlled A/B. Failure preserves B4EP6. B4E2, references, runtime/CUDA,
parallelism, solver-policy and production remain blocked.

Observed PASS freezes `D=131,987,230`, 71.75% removable radius work and
60.63% removable gradient-kernel work. See the
[dated evidence](../../development/nonlocal-nsr3b4ep7d-evaluation-tape-dataflow-evidence-2026-08-22.md).
