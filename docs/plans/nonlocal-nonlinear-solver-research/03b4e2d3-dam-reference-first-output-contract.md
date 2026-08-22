# NSR3-B4E2D3 -- repaired Dam reference-binary64 first-output contract

Status: `FROZEN / NOT_RUN / DAM_STEP_4 / RESEARCH_ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d3-dam-reference-binary64-first-output|v1|parent=6ad559bfb28a79468f2810ff30615f1f1493cc13dad66f64ba4ecb0015a63e22:4805530ed85d6a6b1f05ca855c8661fe036b16e8201fcbf745864de9e78d1be1:343f6c424b43bccba57eb59e192fd545c968ab382e6abd539ce748c686e86a3d|failed-parent=282b6ee16135d036363f1a613e4dbfa4c8d2065dfb030810050772edd1ff671e:b469c0897d9303242bdc6eda802047f4207d6d37251dd934727e45c5b8f808af|reference-bits=c3fb3522dba71768a7933c293523ebbbbaf08a0b5ee408a57b27a2300a2e22d8:0d567ba5512ba237a48e5e0b828a670a398f1bf23a35ac269729cad535f374d7|alignment=8d0a0a85adba50d4784245d460c83757bcce92841d804f4739b81faf27fd5f09:a2d97ab6f26383d826366eba2a3d4392f5ef9610dda87e89509e93bd9daf61e8:fb2b8f8b4c0227cf5d8a7a43ce518ed72b2e5d5cda31fb8e8c17a5727c24ba13:342502:611520:120:37d83c159ff913afef290b9dc1cc7affe9f6018d726dd7b13ab9308f3bf741d3|solver=35a1d41b78d132429334a34d8c99e6d2870b2b8a68ee949beb5a3c69375dff10:b4f847cb4f19b09e951534649515a4504bc07044a13e6c636598b33f247777e9|publication=e713a61649fc230b189fca9eda3628b69f9369c706df35f0a2b080a4bd189a70|trajectory=dam;steps1..4;dt=1/240;workers8;work-only;static-index-once;flat-csr;topology-cache-transaction;coefficient-cache;fused-tape;split-incoming;directed-scratch-transaction|state=reference-frame0-binary64;decoded-canonical-handoff-steps2..4;fine-only;commit-prefix-then-global-roots;failure-preserves-prefix;no-retry-tune|temporal=embedded-adjacent;initial-spectrum-each-step;accepted<=192;attempted-level<=768|physics=strain<=0.001;penetration<=0.0025;kkt-ledger<=1e-9;strict-finite;support-closure<=1e-10;pressure-abs<=0.01-energy;mechanical-abs<=0.01-energy;creation<=0.01-energy+mechanical-abs;publication-impulse-balanced-bound;no-all-pairs;live0|reference=count6000;mass750;step4;position=3029244660,2280395480,2999999999;velocity=1856527209,1616954663,-4;q99=992829,740902;center-normalization=4,1,1;front-normalization=4;height-normalization=1;rmse<=0.05;max<=0.10|runs=first-pass-then-second;2-release-builds;byte-exact;watchdog=900s;timing=none|failure=safe-empty-prefix-report;stop-first;no-second-after-physical-fail|reference=closed|credit=b4e2h-contract-research-only
```

Identity SHA-256:
`38da5cf5d6409d15ce1cf0221f0ead6b4054b3cbb9f456c9636c6191d92696f3`.

## Repair boundary

Add a new `--nominal-dam-reference-first-output` command. Retain every B4E2D
trajectory, solver, state ownership, cumulative physics, reference tolerance,
worker and watchdog byte except:

1. require the B4E2D1 external frame-zero raw-bit root;
2. require the B4E2D2 decoded pair root/counts/degree;
3. start step one from that reference-binary64 state and steps 2--4 from the
   preceding decoded committed frame;
4. if zero frames commit, publish empty root fields plus the exact first
   failure instead of calling positive-count trajectory-root APIs.

The failed B4E2D identity/result remains historical evidence. Do not change a
formula, coefficient, convergence gate, physical/reference tolerance or
selected SIRDI architecture.

## Execution and authority

Build Release twice. Run A under the external 900-second watchdog without a
timing wrapper. Stop on first identity/capacity/step/cumulative/reference
failure and do not run B. Only after A fully passes, run B and require exit
zero, empty stderr and byte-identical stdout.

All gates in the original
[B4E2D contract](03b4e2d-dam-first-output-contract.md) apply with the repaired
alignment above. PASS authorizes only B4E2H Hydro-first research/contract
design. It grants no full-corpus, speed, runtime/GPU/schema/PhysX or production
authority.

