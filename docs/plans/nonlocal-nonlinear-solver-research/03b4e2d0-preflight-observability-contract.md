# NSR3-B4E2D0 -- Dam preflight observability contract

Status: `FROZEN / PASS / PAIR_IDENTITY_MISMATCH / NO_TRAJECTORY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d0-preflight-observability|v1|parent=282b6ee16135d036363f1a613e4dbfa4c8d2065dfb030810050772edd1ff671e:22aa521b3acea2b9776d8b4a9eb8ebc468fb9463:b469c0897d9303242bdc6eda802047f4207d6d37251dd934727e45c5b8f808af|scope=projection,scenario,initial-canonical,decoded-raw-exact,static,pairs,evaluation,counts|trajectory=none|failure-report=total;no-empty-root|runs=2-processes;byte-exact|route=first-mismatch-only|timing=none|credit=b4e2d1-reclosure-research-only
```

Identity SHA-256:
`2b09ce8435fc185bd91322e17f1ee34f04a3ec59dc1a47dc5bba04cef23c6641`.

## Diagnostic command

Add `--nominal-dam-first-output-preflight`. It must execute only the bounded
B4E2D construction/alignment prefix and must never call the macro adaptive
transaction, KKT solve or trajectory/ledger root over a frame prefix.

Publish actual plus expected values and a separate boolean for:

1. B4E2D projection SHA;
2. OpenMP process policy;
3. scenario root;
4. raw B4E0 initial canonical aggregate and canonical-layout check;
5. exact binary64 equality between raw lattice state and decoded frame zero;
6. decoded-state static-index root;
7. decoded-state pair root, unique pairs, directed records and maximum degree;
8. sample/support counts, finite evaluation and one static-index build.

The report must contain a deterministic ordered mismatch list and complete
normally even when one or more B4E2D expectations fail. Diagnostic PASS means
only that all facts were observed and serialized; it does not require the
parent B4E2D alignment gate to pass.

Run two fresh processes from the already built Release executable after the
observability-only implementation and require exit zero, empty stderr and
byte-identical LF-terminated stdout. No timing wrapper or speed claim.

B4E2D0 PASS selects only a separately frozen B4E2D1 preflight reclosure for
the first observed mismatch. It cannot resume the four-step pilot, change a
physical coefficient/tolerance, or authorize Hydro/full corpus/runtime/GPU/
production work.

B4E2D0 passes twice byte-identically and isolates the pair root/counts after
frame-zero decode; see the
[dated evidence](../../development/nonlocal-nsr3b4e2d0-preflight-observability-evidence-2026-08-22.md).
Because the external payload appears to select the decoded binary64 state,
B4E2D1 must bind that external raw-bit fact independently before topology is
reclosed.
