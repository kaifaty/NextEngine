# Nonlocal continuum feasibility tool

This directory contains the quarantined, report-only implementation for the
NR1 Nonlocal/SISSM research stage. It is not linked into the Rust workspace,
does not expose an engine API and does not publish authoritative simulation
state.

The tool contains an independent CPU `f64` oracle, frozen machine-readable
workload profiles and a separately implemented source-shaped CUDA `f32`
baseline. The two numerical paths do not share pair-contribution code.

Build outside the repository and run the bounded CPU and CUDA controls:

```sh
cmake -S crates/continuum-water/tools/nonlocal-feasibility \
  -B /tmp/nextengine-nonlocal-feasibility-build -G Ninja
cmake --build /tmp/nextengine-nonlocal-feasibility-build
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-corrected-cuda-terms \
  --self-test
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-corrected-cuda-full-step \
  --graph-self-test
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-corrected-cuda-full-step \
  --operator-self-test
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-corrected-cuda-full-step \
  --solver-self-test
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-corrected-cuda-full-step \
  --tiny-solver-correspondence
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-corrected-cuda-full-step \
  --correspondence-4k 64
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-corrected-cuda-full-step \
  --trajectory-4k hydrostatic-hold 16 128
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-corrected-cuda-full-step \
  --gpu-trajectory-probe hydrostatic-hold 16 128
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-formula-reclosure \
  --normalized-kernel-reclosure-self-test
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-formula-reclosure \
  --manufactured-multistep-self-test
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-formula-reclosure \
  --temporal-stiffness-diagnostic
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-formula-reclosure \
  --floor-limited-temporal-oracle
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-formula-reclosure \
  --acoustic-substep-policy-self-test
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-formula-reclosure \
  --pressure-tangent-spectrum-self-test
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-formula-reclosure \
  --spectral-substep-policy-self-test
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-formula-reclosure \
  --embedded-spectral-error-controller-self-test
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-formula-reclosure \
  --transactional-multistep-controller-self-test
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-formula-reclosure \
  --fine-state-ownership-self-test
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-formula-reclosure \
  --split-static-boundary-self-test
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-formula-reclosure \
  --boundary-composition-smoke-self-test
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-formula-reclosure \
  --boundary-reaction-accuracy-self-test
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-formula-reclosure \
  --displacement-ownership-self-test
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-formula-reclosure \
  --finite-precision-merit-self-test
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-formula-reclosure \
  --floor-stationarity-trajectory-self-test
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-formula-reclosure \
  --owned-gradient-self-test
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-formula-reclosure \
  --owned-residual-trajectory-self-test
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-formula-reclosure \
  --owned-boundary-composition-self-test
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-formula-reclosure \
  --closed-box-eligibility-self-test
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-formula-reclosure \
  --tiny-pressure-corpus-self-test
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-formula-reclosure \
  --box-contact-kkt-self-test
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-formula-reclosure \
  --box-contact-kkt-face-self-test
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-formula-reclosure \
  --tiny-pressure-contact-kkt-self-test
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-formula-reclosure \
  --contact-onset-forecast-self-test
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-formula-reclosure \
  --tiny-pressure-contact-forecast-self-test
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-formula-reclosure \
  --joint-neighborhood-self-test
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-formula-reclosure \
  --joint-neighborhood-one-pass-self-test
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --describe-profile nuv-water-48k.v0
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility --cpu-self-test
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility --cpu-gather-self-test
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility --self-test
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --self-test --accumulation nuv-gather-directed-r0
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --self-test --accumulation nuv-gather-directed-r0 \
  --handoff pointer-swap-o1
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --self-test --accumulation nuv-gather-directed-r0 \
  --handoff pointer-swap-o1 --term-kernels nuv-terms-specialized-o2
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --repeatability nuv-surface-16k.v0 --iterations 20 --runs 10 \
  --accumulation nuv-gather-directed-r0 --handoff pointer-swap-o1
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --check nuv-water-48k.v0 --iterations 5
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --benchmark nuv-water-48k.v0 --warmup 5 --runs 50 \
  --accumulation nuv-gather-directed-r0 --handoff pointer-swap-o1 \
  --term-kernels nuv-terms-specialized-o2
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --self-test --accumulation nuv-unique-pair-segmented-o3 \
  --handoff pointer-swap-o1 --term-kernels nuv-terms-specialized-o2
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --layout-tournament nuv-water-16k.v0 --warmup 32 --runs 96
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --self-test --accumulation nuv-gather-directed-r0 \
  --handoff pointer-swap-o1 --term-kernels nuv-terms-specialized-o2 \
  --storage cell-sorted-o4
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --locality-tournament nuv-water-48k.v0 --warmup 32 --runs 96
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --retained-tournament nuv-water-48k.v0 --warmup 32 --runs 96
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --np0-baseline nuv-water-50k-coherent.v1 --warmup 64 --runs 512
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --np0-baseline nuv-water-50k-advected.v1 --warmup 64 --runs 512
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --p1-check nuv-surface-stiff-16k-i2.v1 --iterations 2
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --p1-tournament nuv-water-50k-advected.v1 --warmup 32 --runs 96
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --p2-check nuv-water-100k-report.v1 --iterations 5
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --p2-tournament nuv-water-50k-advected.v1 --warmup 32 --runs 96
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --p3-check nuv-surface-stiff-16k-i2.v1 --iterations 2
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --p3-tournament nuv-water-50k-permuted.v1 --warmup 32 --runs 96
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --p4-check nuv-water-50k-advected.v1 --iterations 5
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --p4-tournament nuv-water-50k-advected.v1 --warmup 32 --runs 96
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --p2-decision nuv-water-50k-advected.v1 --warmup 64 --runs 512
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --production-profile-audit
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --cpu-scale-law-self-test
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --cpu-boundary-discriminator
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --cpu-tiny-physical-corpus
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --p2-check nuv-basin-48k-source-scale.v2 --iterations 5
/tmp/nextengine-nonlocal-feasibility-build/nonlocal-feasibility \
  --p2-check nuv-basin-48k-static-support-derived.v3 --iterations 5
```

Each command writes one JSON value to stdout. Build trees, binaries, raw JSON
and profiler captures stay outside Git.

Commands without `--accumulation` retain the frozen `source-atomic-v0`
baseline. `nuv-gather-directed-r0` selects the NR1-RC1 owner-only directed
gather counterfactual explicitly. Commands without `--handoff` retain the
`copy-v0` state handoff. `pointer-swap-o1` is valid only with the directed
gather path and selects the report-only NR2-O1 candidate explicitly. Commands
without `--term-kernels` retain `nuv-terms-runtime-v0`.
`nuv-terms-specialized-o2` is valid only with pointer swap and either directed
gather or the O3 segmented layout. On directed gather it selects the retained
report-only NR2-O2 path.

Commands without `--storage` retain stable fixture/sample order.
`cell-sorted-o4` is valid only with retained gather, pointer swap and O2 term
specialization. It stores solver fields in stable packed-cell order while
remapping captures and logical CSR back to sample-ID order. Map setup, exact
map/CSR digests, memory and neighbor-storage distances are reported
separately.

`nuv-unique-pair-segmented-o3` selects the report-only NR2-O3 candidate and
is valid only with `pointer-swap-o1` plus `nuv-terms-specialized-o2`. It keeps
the frozen CSR, builds and validates an immutable symmetric reverse-slot map,
evaluates each non-self pair once, writes two 48-byte endpoint fragments and
reduces them in owner CSR order. Reverse-map setup time and the additional
layout allocation are reported separately.

`--layout-tournament` co-resides historical atomic, retained gather and O3
segmented instances, then rotates their order `A/G/S`, `G/S/A`, `S/A/G` for
32 warm-up and 96 measured rounds. It is an evidence command, not runtime
integration. The O3 execution record rejected the candidate at the stiff
surface correspondence gate, so retained work continues to use
`nuv-gather-directed-r0 + pointer-swap-o1 + nuv-terms-specialized-o2`.

`--locality-tournament` co-resides stable-sample and cell-sorted instances and
alternates their order for 32 warm-up and 96 measured rounds. Exact remapped
output/CSR, storage capacity and before/after state are required before its
same-process timings are admissible.

`--retained-tournament` is the final NR2 aggregate comparator on the two
correctness-valid HN-3 workloads. It co-resides and alternates the historical
source-atomic/copy/runtime denominator with the final retained
gather/pointer-swap/specialized/stable-sample stack. It is not valid for the
known-failed atomic stiff-surface profile.

`--np0-baseline` accepts only the separately rooted v1 profiles. It preserves
the retained NR4 solver identity, validates a bounded v1 CPU subset and stiff
surface i2 control, and runs one preallocated same-process benchmark. Advected
profiles feed each accepted output position/velocity into the next substep,
rebuild neighbors from that reference and reset only at a complete 32-step
epoch boundary. The command records every trace-state/output/CSR hash, raw
stage timing, degree/locality distribution and capacity result. A fixed 256-run
GPU-conditioning window precedes (and is separate from) the requested 32/64
formal warmups so P8-to-boost transitions do not contaminate percentiles.

The dynamic surface performance profile uses `gamma=100`; the independent
`nuv-surface-stiff-16k-i2.v1` correctness gate retains `gamma=1000`. The
rejected combination `gamma=1000`, 20 iterations and 32 sequential substeps
exceeded its bounded CSR at step one and is not a supported performance
profile.

`--p1-check` and `--p1-tournament` select the exact-work
`fused-owner-terms-p1` traversal on top of the retained NP0 identity. Density
remains a separate global barrier. Enabled post-density terms share one owner
CSR visit but retain independent f32 accumulators and the original
incompressibility/viscosity/surface commit order. The check requires exact
tiny/stiff/target output and CSR. The tournament co-resides retained and P1
instances, conditions both, alternates their order and advances dynamic traces
in lockstep. P1 adds no device storage and remains selectable as a rollback
layer for P2.

`--p2-check` and `--p2-tournament` compare retained P1's 32-bit neighbor IDs
with `compact-csr-u16-p2`. Eligible fixtures build directly into a 16-bit
neighbor array; row offsets remain checked 32-bit values. Capture widens IDs
outside timing for representation-neutral logical CSR hashing. Profiles above
65,535 samples explicitly fall back to the retained 32-bit representation.

`--p3-check` and `--p3-tournament` add `dynamic-cell-local-p3` to retained P2.
Every solve radix-sorts stable material IDs by horizon cell, gathers physical
SoA state, executes the exact P1+P2 arithmetic and scatters an advected result
back to stable-ID order. Sort, map, gather and scatter are timed. Captures
validate inverse maps and remap output/CSR to canonical IDs outside timing.

`--p4-check` and `--p4-tournament` compare retained P2 with the certified
`verlet-skin-p4` cache. A fixed `0.04h` superset is reused only while the
double-precision maximum-displacement certificate proves coverage. Every CSR
consumer applies the retained `h` predicate; captures distinguish candidate
superset CSR from exact active logical CSR. Cache state is invalidated at each
explicit trace reset.

`--p2-decision` accepts only coherent or advected exact-50k v1 water. It first
replays exact P1/P2 trace correspondence, releases the comparator, then runs a
single P2 finalist instance through 256 conditioning, 64 formal warm-up and
512 measured executions. It reports raw totals and the standalone `4/6 ms`
p95/p99 gate; it does not grant runtime or W2 authority.

`--production-profile-audit` starts NPR0 without changing the retained
performance identities. It hash-binds three boundary-free v2 bridge profiles:
product geometry/mass at the source `h/dx` and cadence, then product cadence,
then the SPEC-38 `h/dx`. Its command status validates the audit itself while
its semantic status remains `PROFILE_RECLOSURE_REQUIRED`. The v2 coefficients
are unchanged counterfactual controls, not a selected material calibration.

`--cpu-scale-law-self-test` independently checks the algebraic similarity
transform of the CPU `f64` directed-gather equations on one all-term tiny
fixture. For length ratio `s` and time ratio `t`, it checks
`kappa*s^4/t^2`, `lambda*s^3/t`, `mu*s^3/t` and `gamma*s/t^2`, scaled
position/source, invariant density/matrix and scaled velocity. PASS proves
only the implemented equation transform; it does not calibrate a product
material, boundary, energy curve or canonical authority.

The CPU boundary discriminator independently builds a two-layer complement
around a tiny box and sends one high-speed sample through the bottom wall.
PASS requires the fixed ghost-only solve to expose penetration and the
separately evaluated SPEC-38 swept-sphere counterfactual to stop it with exact
fluid/reaction impulse closure. Its semantic result is
SPLIT_BOUNDARY_REQUIRED: support samples are not a contact mechanism.

The separate `--split-static-boundary-self-test` command evaluates the FCR2
fluid-centred pressure energy with immutable static support, its analytic
fluid HVP and virtual support reaction. It also compares two/three layers and
runs independent face/edge/corner swept-contact controls. PASS is a bounded
formula candidate only; it does not execute a boundary trajectory or authorize
runtime integration.

`--boundary-composition-smoke-self-test` executes the separately frozen B3
post-solve swept-contact hypothesis. Its current expected semantic result is
FAIL at `MOMENTUM_LEDGER`: the selected scale-aware smooth stop does not yet
certify the virtual support reaction at the stricter per-substep tolerance.
The command preserves that negative boundary and grants no physical-corpus or
runtime authority.

`--boundary-reaction-accuracy-self-test` is the non-aborting B3D diagnostic.
It decomposes stationarity, translation, reconstruction and contact defects,
computes an inactive binary64 forward bound and charges an active reaction-
aware replay. Its current valid disposition rejects the cumulative inactive
certificate and authorizes no B3 retry.

The v3 static-support profiles add the exact 24,704-sample two-layer outer
complement to 48,000 fluid samples. Their 72,704 total solver indices
deliberately exceed u16; P2 must report an explicit checked u32 fallback.
The GPU preflight evaluates density/constitutive support only. It does not
execute the external swept contact or claim a sealed production step.

The CPU tiny physical corpus evaluates both v3 coefficient profiles against
the predeclared free-fall, hydro, reversible-mode and face/corner gates. The
command succeeds when the bounded decision is produced; inspect
selection.disposition and selection.selected_profile_id for the semantic
outcome. A remediation result is not a selected profile.
