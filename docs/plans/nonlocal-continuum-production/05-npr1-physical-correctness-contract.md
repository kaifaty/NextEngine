# NPR1 — independent physical and canonical reclosure

Status: `EXECUTED / NPR1_B_FAIL / NONLOCAL_PRODUCTION_RESEARCH_STOP`

## Purpose

NPR1 attempts to falsify the exact NPR0 profile
`nuv-basin-48k-static-support-h3-physical.v4` as a water candidate. It adds an
independent canonical f64 route and broader physical evidence. It does not
choose production authority, expose public schemas, integrate PhysX or award
performance credit.

The profile SHA-256 is fixed to
`624678f6ad4dbf2d3657b30880ad1cff2445d0348194549b37c71c1e55804327`.
Changing coefficients, h3 support, 16 iterations, cadence, mass or boundary
capacity returns to a newly authorized profile lineage; NPR1 may not tune
them.

## Execution outcome

NPR1-A passed, but the first NPR1-B term discriminator failed twice at the
same exact boundary. The source-shaped cubic gradient is `dW/dq`, not the
published `dW/dr`; active bulk viscosity and the dormant shear/surface terms
also fail force/energy consistency. Independent analytical derivatives and
the literal formula repair probe both match finite differences below
`3.45e-9`, so the failure is not numerical noise or a sign/index ambiguity.

The exact evidence is recorded in the
[NPR1-B report](../../development/nonlocal-continuum-npr1b-term-controls-evidence-2026-08-20.md).
Per the frozen first-failure rule, Poiseuille, 16/32/64 convergence, NPR1-C,
NPR1-D and NPR1-E were not run. The selected v4 lineage terminates as
`NONLOCAL_PRODUCTION_RESEARCH_STOP`; NPR2 is not authorized.

## NPR1-A canonical publication

Build a separate CPU binary64 target so canonical flags do not rewrite the
retained NPR0 CUDA laboratory. It must use no fast math or contraction and
must publish every accepted substep exactly once as:

```text
NonlocalCanonicalSampleV1 {
  sample_id: u32,
  position_um: [i64; 3],
  velocity_um_s: [i64; 3],
}
```

Conversion is an integer IEEE-754 binary64 decoder with checked
nearest-ties-to-even rounding at 1,000,000 units per metre. It normalizes
negative zero, rejects nonfinite and checked overflow, and enforces the
existing lab bounds of +/-16,000,000 um position and +/-64,000,000 um/s
velocity.

Frame root:

```text
SHA256(
  "nextengine.nonlocal.canonical-frame.v1\0" ||
  profile_sha256[32] || scenario_sha256[32] || step_u32_le ||
  sample_count_u32_le ||
  repeated ascending SampleId:
    sample_id_u32_le || position_i64_le[3] || velocity_i64_le[3]
)
```

Trajectory root uses the domain
`nextengine.nonlocal.canonical-trajectory.v1\0`, scenario/profile roots,
frame count and the ordered frame roots. The next substep decodes only the
published integers. Private float positions, velocities, SISSM matrices and
neighbor data never cross the boundary.

Required self-tests include exact half-even vectors, negative zero,
normal/subnormal values, i64 and lab-bound edges, NaN/infinity, N-1/N/N+1
sample capacity, duplicate/unsorted SampleId rejection and a hand-computed
frame root.

NPR1-A now passes. Its exact roots and non-regression checks are recorded in
the [dated evidence](../../development/nonlocal-continuum-npr1a-canonical-evidence-2026-08-20.md).

## NPR1-B independent term and convergence controls

The f64 implementation must remain structurally independent from CUDA kernels
and pass:

- cubic-kernel value/gradient goldens and central finite differences;
- incompressibility, bulk/shear viscosity and surface pair-energy directional
  derivatives at relative error <=1e-7;
- translation invariance and equal/opposite pair closure <=1e-12;
- zero shear response for a rigid translation;
- monotonically non-increasing kinetic energy as shear viscosity increases in
  the rotating-sphere control, with angular-momentum loss explicitly reported;
- Poiseuille normalized profile RMSE <=5% and maximum error <=10% for the
  paper's separate lambda=1, mu=0, kappa=1, gamma=0 control;
- finite 16/32/64-iteration outputs and a reported Eq. 28 update ratio for
  every smoke case.

Poiseuille and surface/shear controls do not select water coefficients and do
not enable those terms in the v4 water profile.

## NPR1-C deterministic f64 trajectory runner

Replace the quadratic diagnostic neighbor scan with a checked sorted-cell
grid. Membership uses canonical integer squared distance and each row is
sorted by stable SampleId, with rooted static support in a separate stable ID
space. All counts and products are pre-admitted before allocation.

The runner must support the frozen outer box and the exact internal plane,
closed aperture, edge/corner swept contact required by the orifice case.
Three rooted layers cover the complete h3 horizon; support is neither thinned
nor charged against fluid capacity. A failed step publishes no frame.

NPR1 f64 is serial and deterministic first. Parallel/GPU authority is outside
this stage.

## NPR1-D smoke and reference preflights

Before any nominal run:

1. attest the three external files at their frozen hashes and reject one
   deliberately changed byte before trajectory execution;
2. run small hydro, free-fall, dam-break, still, orifice, sealed and storage-
   order analogues twice;
3. require exact canonical roots across repeats and identity/reverse/affine
   storage orders;
4. require exact count/mass, finite metrics, no leak, stable contact features,
   penetration <=2.5 mm and normalized momentum/external-work residual <=1%;
5. compare 16 and 32 iterations on the smoke aggregate curves; RMSE must be
   <=2.5% and maximum difference <=5%.

The first failed case stops the smoke batch. One named bounded diagnosis may
localize arithmetic, support, contact, convergence or physical-profile
failure; thresholds and the v4 profile remain immutable.

## NPR1-E nominal water corpus

Reuse the exact W1 scenario geometry, output cadence, external files and
aggregate formulas:

| Scenario | Blocking rule |
| --- | --- |
| hydro, 6k/1200 | final-window COM x/z within 2.5 mm and y within 25 mm; density signed/absolute/positive p50/p95/p99 reported; penetration/work/momentum gates pass |
| free fall, 1k/96 | every canonical frame equals semi-implicit gravity recurrence; no internal term activates |
| dam break, 6k/720 | front and height each <=5% RMSE and <=10% maximum error against the attested curve |
| still tank, 4k/7200 | exact count/mass, lateral COM drift <=2.5 mm, no growing work/momentum residual above 1%, exact repeat root |
| orifice, 6k/720 | exact partition/total mass; transfer <=5% RMSE and <=10% maximum error |
| sealed product, 48k/480 | all samples and mass remain; no leak; penetration/work/momentum pass; exact repeat root |
| order, 1152/240 | identity/reverse/affine trajectory roots equal |

Energy for static-wall equilibrium/free-fall/still/order uses two-sided <=1%
drift. Impact/orifice/sealed stress forbids positive energy creation above 1%
and reports deficit separately. These formulas and thresholds are inherited
unchanged from the accepted DFSPH reference contract.

The v4 16-iteration result must also remain within 2.5% curve RMSE and 5%
maximum aggregate difference from a diagnostic 32-iteration run. The 32-run
does not replace the selected profile and cannot be used for best-of-two
selection.

## NPR1-F outcome

`NONLOCAL_PHYSICAL_CANONICAL_CANDIDATE` requires NPR1-A through NPR1-E plus
the applicable NPR1-B controls, exact same-target canonical repeat/order
roots and no first failure. It authorizes NPR2 to decide authority only.

After one bounded diagnosis, a repeated physical, canonical, capacity,
reference or convergence failure emits `NONLOCAL_PRODUCTION_RESEARCH_STOP`.
No further coefficient/support sweep is authorized in this roadmap. An
execution environment failure is reported as blocked evidence, not physics
failure.

That stop state is selected by NPR1-B. A formula-corrected implementation is
a new solver/profile lineage and cannot resume at NPR1-C or inherit v4
coefficients, roots or physical credit.

## Explicit exclusions

Moving rigid bodies, PhysX reaction, public/durable schemas, save/replay,
parallel authority, GPU exactness, performance optimization, ML, adaptive
iterations, split/merge, surface reconstruction, terrain and Windows.
