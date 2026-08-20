# NP0 — exact-50k corpus and retained baseline

Status: `SPECIFIED / IMPLEMENTATION_NEXT / REPORT_ONLY / NO_W2_CREDIT`

## Purpose

NP0 replaces inference from the closed coherent 48k fixture with a separately
rooted v1 workload. It changes no solver mathematics and grants no performance
credit. NP1 may start only after this corpus is reproducible on the retained
`nuv-gather-directed-r0 + pointer-swap-o1 + nuv-terms-specialized-o2 +
stable-sample-v0` implementation.

The v0 profile JSON, profile hashes, fixture hashes and output digests are
compatibility controls. Adding v1 must not change any of them.

## Frozen generators

All coordinates below use metres, stable sample IDs are zero-based, lattice
coordinates are enumerated with `x` fastest, then `y`, then `z`, and
`linear(x,y,z) = x + X * (y + Y * z)`.

| Profile | Lattice | Samples | Iterations | Initial storage |
|---|---:|---:|---:|---|
| `nuv-water-50k-coherent.v1` | `100 x 25 x 20` | 50,000 | 5 | sample ID equals lattice linear index |
| `nuv-water-50k-permuted.v1` | `100 x 25 x 20` | 50,000 | 5 | lattice index `(32749 * id + 7919) mod 50000` |
| `nuv-water-50k-advected.v1` | `100 x 25 x 20` | 50,000 | 5 | coherent stable IDs; dynamic seed below |
| `nuv-viscous-16k-advected.v1` | `40 x 20 x 20` | 16,000 | 20 | coherent stable IDs; dynamic seed below |
| `nuv-surface-16k-advected.v1` | `40 x 20 x 20` | 16,000 | i2 correctness, then 20 | coherent stable IDs; dynamic seed below |
| `nuv-water-100k-report.v1` | `100 x 50 x 20` | 100,000 | 5 | coherent; report-only scaling control |

The affine permutation is bijective because `gcd(32749, 50000) = 1`. It
changes only sample-to-storage association: after remapping by lattice index,
mass, position, velocity, fixed flag and material coefficients are identical
to the coherent fixture.

All v1 profiles retain the v0 material constants unless the profile name says
otherwise: spacing `0.005`, horizon `0.015`, mass `0.000125`, rest density
`1000`, timestep `0.001`, gravity `[0,-9.81,0]`, and no boundary. Water uses
`kappa=1`, `lambda=1.5`; viscous uses `kappa=1`, `lambda=200`, `mu=1`;
surface uses `kappa=1`, `lambda=0.2`, `gamma=1000`.

## Dynamic seed and trace

Advected positions begin on a lattice scaled by `0.99` about its centre. This
keeps the first and next lattice shells away from the exact support boundary.
For each sample let `nx`, `ny`, `nz` be its centred coordinate divided by that
axis half-extent. Its initial velocity is:

```text
vx = -0.20 * ny
vy =  0.20 * nx
vz =  0.05 * sin(pi*nx) * sin(pi*ny) * cos(pi*nz)
```

The trace has exactly 32 sequential solver substeps. At substep `s`, the
canonical reference position and initial velocity are the accepted final
position and reconstructed velocity from `s-1`; substep zero uses the frozen
seed. Neighbor membership is rebuilt from that reference position at every
substep and frozen only across that substep's nonlinear iterations.

The fixed grid includes an additional `0.025 m` margin on every side. Any
reference position outside that bound, pair capacity overflow, nonfinite
state or local-solve failure rejects the trace instead of silently clamping
the workload.

After the implementation preflight, the committed trace record must contain:

- seed fixture SHA-256;
- ordered state and logical CSR SHA-256 for steps `0`, `1`, `15` and `31`;
- an aggregate trace SHA-256 over every step's input state and active CSR;
- the first topology-preserving and first topology-changing interval, or a
  typed NP0 rejection if either class is absent;
- per-step directed-pair, mean/max degree and storage-distance distributions.

If the prescribed seed does not produce both interval classes, NP0 stops and
this specification is revised before any timing. The runner may not select or
discard trace frames based on candidate performance.

## Persistent runner

The NP0 command owns one preallocated solver instance and emits one JSON value.
It supports fixed-state and sequential modes:

```text
nonlocal-feasibility --np0-baseline <profile-id> --warmup <n> --runs <n>
```

- coherent and permuted profiles replay their exact frozen input;
- advected profiles advance through the 32-step trace and reset to the exact
  seed only at an epoch boundary;
- warm-up advances are discarded and the measured epoch always begins at
  trace step zero;
- seed reset and process/report I/O are outside timing; device-side state
  publication from one substep to the next is inside the substep total;
- each measured record names its trace step and rebuild reason;
- raw totals and all stage totals are retained so min/median/p95/p99/mean are
  recomputable without rounded aggregates.

The adjacent tier is exactly 32 warm-up executions and 96 measured executions.
The decision tier is exactly 64 warm-up executions and at least 512 measured
executions. Dynamic runs must contain whole 32-step epochs; larger run counts
therefore remain a multiple of 32.

## Correctness and equivalence

Before a percentile run:

1. all existing CPU/CUDA tiny and stiff-surface controls pass;
2. all closed v0 profile hashes and retained output digests match their
   pre-NP0 values;
3. the v1 one-iteration pair/oracle subset passes the inherited numeric
   tolerances, which are frozen in each v1 profile rather than inherited by
   name;
4. coherent and permuted seed states match after remapping by lattice index
   within the declared f32 correspondence tolerance;
5. repeat execution of a fixed frame has exact ordered output and logical CSR;
6. every dynamic CSR is symmetric, includes self, remains within capacity and
   is built from the recorded prior accepted state.

Full CPU `O(N^2)` execution is not a valid 50k oracle. NP0 uses the independent
CPU f64 tiny/pair/one-iteration subsets plus exact retained-vs-candidate full
GPU correspondence. NP2 later creates a separate high-iteration stationary
oracle.

## Required report fields

In addition to the common contract, the v1 report records generator version,
permutation parameters, trace length/step, reset count, rebuild reason, input
and prior-state digests, logical CSR digest, ordered output digest, directed
pairs, min/mean/p95/max degree, mean/p95 storage distance, bytes per sample and
edge, capacity headroom, binary/device/compiler identity and raw stage samples.

Profile, seed and trace hashes are populated in this document by the NP0
implementation checkpoint, before the first NP1 candidate is built.

## NP0 exit

NP0 closes only when:

- v0 compatibility controls are unchanged;
- all six v1 generators are hash-bound;
- the advected corpus contains both required interval classes;
- adjacent retained reports reproduce in a fresh process;
- exact-50k coherent and advected decision denominators have at least 512 raw
  samples and valid p95/p99;
- a dated NP0 evidence document records the result and updates the roadmap.

Failure closes as `NP0_REJECTED` or `NP0_EVIDENCE_INCOMPLETE`; it is not
silently converted into an NP1 optimization problem.
