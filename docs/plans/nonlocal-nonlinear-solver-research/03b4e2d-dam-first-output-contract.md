# NSR3-B4E2D -- Dam first-output physical-pilot contract

Status: `FROZEN / NOT_RUN / DAM_STEP_4 / RESEARCH_ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d-dam-first-output|v1|parent=681e6e2aab130e0a461dadf575caac754b0931668027db911512a1edac2cccf3:8ca3498dbc04556bf86f32a8cab55eccee92ff201aecb737d8ac2760c64504c6:b8ad20e889d519cc6fdc0a5bbdb228369eb426451ed27ed93058107b9587750c|alignment=c53a112830cb94c4139da75c2044d28f61673cb1aa05c116098967aee41f7bbe:79a8932136a64c1cfa953caafe323cb9e860cb134c1d4dc0907be7f40891587a:8d0a0a85adba50d4784245d460c83757bcce92841d804f4739b81faf27fd5f09:a2d97ab6f26383d826366eba2a3d4392f5ef9610dda87e89509e93bd9daf61e8:c330a0aecb913d92e95478dc3325f9bc326d5e8d3057485723dd62eef1493889:37d83c159ff913afef290b9dc1cc7affe9f6018d726dd7b13ab9308f3bf741d3|solver=35a1d41b78d132429334a34d8c99e6d2870b2b8a68ee949beb5a3c69375dff10:b4f847cb4f19b09e951534649515a4504bc07044a13e6c636598b33f247777e9|publication=e713a61649fc230b189fca9eda3628b69f9369c706df35f0a2b080a4bd189a70|trajectory=dam;steps1..4;dt=1/240;workers8;work-only;static-index-once;flat-csr;topology-cache-transaction;coefficient-cache;fused-tape;split-incoming;directed-scratch-transaction|state=decoded-canonical-handoff;fine-only;commit-prefix-then-global-roots;failure-preserves-prefix;no-retry-tune|temporal=embedded-adjacent;initial-spectrum-each-step;accepted<=192;attempted-level<=768|physics=strain<=0.001;penetration<=0.0025;kkt-ledger<=1e-9;strict-finite;support-closure<=1e-10;pressure-abs<=0.01-energy;mechanical-abs<=0.01-energy;creation<=0.01-energy+mechanical-abs;publication-impulse-balanced-bound;no-all-pairs;live0|reference=count6000;mass750;step4;position=3029244660,2280395480,2999999999;velocity=1856527209,1616954663,-4;q99=992829,740902;center-normalization=4,1,1;front-normalization=4;height-normalization=1;rmse<=0.05;max<=0.10|runs=first-pass-then-second;2-release-builds;byte-exact;watchdog=900s;timing=none|failure=stop-first;no-second-after-physical-fail|reference=closed|credit=b4e2h-contract-research-only
```

Identity SHA-256:
`282b6ee16135d036363f1a613e4dbfa4c8d2065dfb030810050772edd1ff671e`.

## Frozen input and selected path

Use B4E0 Dam exactly: scenario root `8d0a0a85...fd5f09`, 6,000 fluid
samples, 16,384 immutable two-layer support samples, `80 x 20 x 20` box,
mass `0.125 kg`, rest density `1000 kg/m3`, gravity `-9.81 m/s2`, spacing
`0.05 m` and macro step `1/240 s`. Require the B4E0 static-index, initial
pair and initial aggregate roots bound in the identity projection.

Use the selected SIRDI research path with eight fixed workers, work-only
transient evidence, flat CSR, coefficient cache, fused evaluation/tape,
split incoming construction and transaction-local directed scratch reuse.
Build the immutable support index exactly once. Build a fresh dynamic topology
cache per macro transaction; require one superset rebuild, zero certificate
failures/fallbacks and at least one certified reuse in every transaction.

## Four atomic macro transactions

Run steps 1 through 4 sequentially. Before every step, require position and
velocity to equal the exact decode of the initial frame or preceding committed
frame. Each transaction must pass, select an adjacent fine level, commit only
that fine state, append exactly one matching frame and ledger entry, use at
most 192 accepted substeps and attempt no level above 768 substeps.

Only after a step passes may the committed state/prefix advance. On first
failure stop without retry/tuning and retain the preceding prefix. After step
four require contiguous global steps, exact final decode, and independently
recomputed four-frame trajectory, legacy-ledger and policy-ledger roots.

## Cumulative correctness gate

Across private accepted states, decoded states and ledger entries require:

- maximum positive density strain `<= 1e-3`;
- maximum private/decoded penetration `<= 0.0025 m`;
- maximum KKT-scale residual `<= 1e-9`, strict residual finite;
- maximum support-reaction closure `<= 1e-10`;
- cumulative absolute pressure and mechanical publication delta individually
  `<= 0.01 * energy_scale`, where `energy_scale` is the maximum of initial
  absolute mechanical energy, `N*M*|g|*spacing`, and `1e-12`;
- non-negative energy creation `<= 0.01*energy_scale + cumulative absolute
  mechanical publication delta`;
- cumulative publication impulse within
  `4*M*sqrt(3)*0.5e-6 + gamma(44)*max(norm(impulse),1e-30)`;
- exact work accounting, retained/scratch acquire-release balance, no
  candidate all-pairs evaluation/HVP and zero live workspaces after each step.

## Step-four reference gate

Require 6,000 stable IDs and checked step-four canonical aggregates. The
independent B4E2R reference is:

```text
position_sum_um   = 3029244660,2280395480,2999999999
velocity_sum_um_s = 1856527209,1616954663,-4
q99_position_um   = 992829,740902
aggregate_root    = b8ad20e889d519cc6fdc0a5bbdb228369eb426451ed27ed93058107b9587750c
```

Compute candidate and reference centres from position sums. Normalize centre
errors by `(4e6,1e6,1e6) um`; compute front as `(q99_x+25000)/4e6` and height
as `(q99_y+25000)/1e6`. For the centre vector and q99 vector separately,
require RMSE `<= 0.05` and maximum component error `<= 0.10`. Velocity sums
are provenance diagnostics only in this first-output gate.

## Builds, stop order and authority

Build Release twice. Run A in a fresh process under an external 900-second
watchdog, without `/usr/bin/time` or any timing claim. Gate in order: identity
and alignment, steps 1--4 stop-first, cumulative correctness, reference
comparison. Run B only if A passes; require exit zero, empty stderr and
byte-identical stdout.

FAIL preserves B4E2R/SIRDI and authorizes diagnosis only. PASS authorizes only
B4E2H Hydro-first research/contract design. It grants no full-corpus,
performance, runtime/GPU/schema/PhysX or production authority.

