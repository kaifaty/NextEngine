# NSR3-B4E2D4 -- Dam step-two strain-refinement contract

Status: `FROZEN / NOT_RUN / DIAGNOSTIC_ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d4-step2-strain-refinement|v1|parent=38da5cf5d6409d15ce1cf0221f0ead6b4054b3cbb9f456c9636c6191d92696f3:a126e8a12f2473f2ff8b8329d6453b504ced47481018c3fe63d9a5e64c749039:8619688fd9b810c5e46cc74a7e1e9603d0571641013289baadd76fe68356b410|alignment=0d567ba5512ba237a48e5e0b828a670a398f1bf23a35ac269729cad535f374d7:fb2b8f8b4c0227cf5d8a7a43ce518ed72b2e5d5cda31fb8e8c17a5727c24ba13:c9afea4e49863db57b4099e3dee5d56d8d3b6ca9bcb03ba100e5870c46e29477:f1598838ff82272113fd9746683d1d857d81f692997a57a22b82151216efdda4:59f17359b06f9d428409399aa7a7acf03f0be56dc2fbdf67ad5f98a8a3e7f2b6:c045627c6abe5ab8bb003fca68796bb06f2b84f14ad8e6be33c4ccdf6ea74c48|solver=35a1d41b78d132429334a34d8c99e6d2870b2b8a68ee949beb5a3c69375dff10:b4f847cb4f19b09e951534649515a4504bc07044a13e6c636598b33f247777e9:e713a61649fc230b189fca9eda3628b69f9369c706df35f0a2b080a4bd189a70|experiment=dam-step2-from-committed-step1;adaptive-40-80-reproduced;fixed-private=80-reuse,160,320;dt=1/240;workers8;work-only;static-index-once;flat-csr;topology-cache-lane;coefficient-cache;fused-tape;split-incoming;directed-scratch-lane|physics=unchanged;kappa-unchanged;tolerances-unchanged;strain-limit=0.001;numeric-run-gates-unchanged|classification=publication-admission-missing-if-private80<=0.001-and-decoded80>0.001;resolved-if-gate160-320-and-abs-strain-delta<=0.00005;adaptive-admission-missing-if-resolved-and-private320<=0.001;finite-penalty-compressibility-if-resolved-and-private320>0.001;temporal-unresolved-otherwise|observables=state-root;substeps;work;private-strain;decoded-strain;penetration;kkt;closure;gate80-160;gate160-320|runs=2-release-builds;2-processes;byte-exact;watchdog=900s;timing=none|failure=no-tune;diagnostic-only|reference=closed|credit=redesign-route-research-only
```

Identity SHA-256:
`30f958a25899ba511313eb5d7d8621470a03dcb69b960af9febdb95893a97b75`.

## Required command

Add `--nominal-dam-step2-strain-refinement`. It must:

1. reconstruct external-reference binary64 frame zero and decoded topology;
2. run exact B4E2D3 step one and require its frozen frame/aggregate roots;
3. run exact B4E2D3 step two and require its 40/80 selection, failed-step
   frame/aggregate roots and peak strain;
4. reuse that private 80 lane and independently run fixed 160 and 320 lanes
   from the exact committed step-one state;
5. use eight workers, work-only evidence, immutable static support, flat CSR,
   lane-local certified topology cache, coefficient cache, fused tape, split
   incoming construction and lane-local directed scratch;
6. emit per-lane state root, work counts, private peak strain, the separate
   decoded step-two strain, penetration, KKT residual, support closure and the
   existing 80/160 and 160/320 gates;
7. emit exactly one frozen route from the research table.

Every fixed lane must pass the unchanged `SmokeRun` numerical gate and its
ownership/cache certificates. No publication, retry, tuning or step-three run
is allowed for 160/320.

## Execution and authority

Build Release twice and require byte-identical executables. Run two fresh
processes under separate external 900-second watchdogs, no timing wrapper, and
require exit zero, empty stderr and byte-identical stdout. A
`TEMPORAL_UNRESOLVED` route may still be a diagnostic PASS.

PASS authorizes only research/contract design for the selected route. It
grants no solver correction, new physical tolerance, full Dam/Hydro corpus,
performance, GPU, runtime/schema, PhysX or production authority.
