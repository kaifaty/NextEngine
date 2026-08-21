# NSR3-B4DR1D -- full external generation contract

Status: `FROZEN / IMPLEMENTATION_AND_EXECUTION_AUTHORIZED / NEW_ROOT_ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4dr1d-full-generation|v1|parent=3f5a73693f05daf487898fd692160357c1775652a1c11e65205356243c57386f|schedule=hydro:0..1200/every24;dam:0..720/every4;orifice:0..720/every4|format=CWREFV2;frame=unchanged|summary=nearest-rank-q99-x-f64bits;nearest-rank-q99-y-f64bits;receiver-u32;domain-separated-sha256|publication=external-content-addressed;regular-file;no-tmp;verified-copy-only|runs=2-fresh-byte-exact-per-scenario|parallel=max3;omp1|report-order=hydro,dam,orifice|pressure-max=300|all-else=r1c5|credit=new-root-only
```

Identity SHA-256:
`ba34b4e3b12986ebc831320d6811551d5311a6774a64f079aabe3a5eaa6bb746`.

## Authority and invariants

The [R1C5 PASS](../../development/nonlocal-nsr3b4dr1c5-trajectory-evidence-2026-08-21.md)
authorizes this stage. R1D changes only scenario schedule manifests, loop
length, output cadence and aggregate reporting. It retains:

- the exact full-clone/upstream/patch/build and binary64 process closure;
- 6,000 stable samples, positions, IDs, volume `0.000125 m^3` and mass
  `0.125 kg`;
- all boundary samples/roots and explicit analytical domain extents;
- DFSPH `dt=1/240`, cold starts, pressure cap 300, original tolerances and all
  nonpressure terms disabled;
- R1B v2 swept contact, validators and unchanged `CWREFV2` frame layout.

It creates no R1E/B4E execution authority, old W0I/W1 credit, runtime/public
schema, persistence or production claim.

## Exact scenario blocks

Each block includes its final LF. Its root is:

```text
SHA256("nextengine.nonlocal.nsr3b4dr1d-scenario.v1\0" || exact_block_bytes)
```

### Hydro

```text
B4DR1D_SCENARIO_V1_BEGIN
scenario_id=CW-HYDRO-001
kind=hydrostatic-cube
box_um=0,0,0;1000000,1000000,1000000
fluid=20,15,20;first=(25000,25000,25000);velocity=(0,0,0);id=iy-iz-ix
fluid_root=7d4e661d08de08b18d43a76342329b51f6ae98bca9baee3d850e0f403eae5606
boundary=two-layer-outer-complement
boundary_count=5824
boundary_root=25de85b5eeec041c12bbb5de10e00b8374457b4cf09cb61d99dfc4d5511d8d62
steps=1200
outputs=0..1200/every=24
B4DR1D_SCENARIO_V1_END
```

Length is 449 bytes, root is
`c430b679dfeec33a6ac12c51df75ddee7e7bc48484c6c05188219f0a727909a0`,
frame count is 51 and expected payload size is 15,920,965 bytes.

### Dam break

```text
B4DR1D_SCENARIO_V1_BEGIN
scenario_id=CW-DAMBREAK-001
kind=dam-break
box_um=0,0,0;4000000,1000000,1000000
fluid=20,15,20;first=(25000,25000,25000);velocity=(0,0,0);id=iy-iz-ix
fluid_root=9c12e445666c7b0eada3e6e2c258c733323e4eb8ca6474a6f3d5b863f1566e76
boundary=two-layer-outer-complement
boundary_count=16384
boundary_root=1cf0fd172dcb321e995f372119ea956d1e376b8a409b804bc07e31a729aa830d
steps=720
outputs=0..720/every=4
B4DR1D_SCENARIO_V1_END
```

Length is 443 bytes, root is
`8d0a0a85adba50d4784245d460c83757bcce92841d804f4739b81faf27fd5f09`,
frame count is 181 and expected payload size is 56,501,239 bytes.

### Orifice

```text
B4DR1D_SCENARIO_V1_BEGIN
scenario_id=CW-ORIFICE-001
kind=orifice-release
box_um=0,0,0;2000000,1000000,1000000
wall_um=x=1000000;opening_y=200000..400000;opening_z=400000..600000;radius=25000
fluid=20,15,20;first=(25000,25000,25000);velocity=(0,0,0);id=iy-iz-ix
fluid_root=21307ab2d1655ab5da33f3b5d4e887152601ed531679723b8452023df1f48425
boundary=source-chamber-two-layer-minus-safe-opening
boundary_count=5792
boundary_root=5d23bd8c407c1b1d6bb86fc6dda2250c36db4cde1227d9f48fc6239c728a2cb3
steps=720
outputs=0..720/every=4
B4DR1D_SCENARIO_V1_END
```

Length is 545 bytes, root is
`53d0db457091d7a7d6ede58a0628688b701850d6de30dacdb14ee9951d036961`,
frame count is 181 and expected payload size is 56,501,341 bytes.

All sizes use the unchanged 312,156-byte frame and remain at most 64 MiB
(67,108,864 bytes). `manifest_bytes` are the 539-byte identity projection,
one LF and the exact scenario block.

## Aggregate roots

All integers and binary64 bits use the `CWREFV2` little-endian encoding.
For 6,000 positions, nearest-rank q99 is sorted element 5,939 using zero-based
indexing. For ordered serialized frames define:

```text
q99_x_root = SHA256(
  "nextengine.nonlocal.nsr3b4dr1d-q99-x.v1\0" ||
  sample_count_u32 || frame_count_u32 ||
  each(step_u32 || q99_x_f64_bits))

q99_y_root = SHA256(
  "nextengine.nonlocal.nsr3b4dr1d-q99-y.v1\0" ||
  sample_count_u32 || frame_count_u32 ||
  each(step_u32 || q99_y_f64_bits))

receiver_count_root = SHA256(
  "nextengine.nonlocal.nsr3b4dr1d-receiver-count.v1\0" ||
  sample_count_u32 || frame_count_u32 ||
  each(step_u32 || receiver_count_u32))
```

The canonical report records these three roots, complete payload size/hash,
total steps, output stride/frame count, maxima over every solver step, total
contact hits and final receiver count. An individual successful process is a
full-generation candidate only and keeps `r1e_authorized=false` and
`b4e_authorized=false`.

## Implementation and preflight

Add a distinct `--r1d-manifest-preflight` that builds all schedule blocks,
checks roots/counts/sizes and creates no Simulation or output. A forced R1D
schedule mutation must fail this gate. Add:

```text
--r1d-generate <scenario-id> <absolute-empty-output-directory>
```

The implementation executes every step from one through the scenario's total
step. Contact projection and validators run after every step. A frame is
serialized only at step zero or when `step % output_stride == 0`. Any solver,
contact, finite, capacity, publication or identity error fails closed.

## Execution, publication and exit

Run two waves. Each wave contains one fresh one-thread process per scenario,
with at most three simultaneous processes. Use distinct fresh directories
under an explicit persistent external artifact root, not `/tmp`. Reports are
evaluated in Hydro, Dam, Orifice order regardless of completion order.

For each scenario require zero exits, empty stderr, byte-identical reports and
payloads and exact size/hash/root fields. After pair equality, atomically
publish one verified regular-file copy under:

```text
<explicit-artifact-root>/ba34b4e3b12986ebc831320d6811551d5311a6774a64f079aabe3a5eaa6bb746/
  <scenario-id>/<payload-sha256>.cwrefv2
```

Rehash and compare source, copied bytes and final regular file; symlinks and
unverified copies receive no credit. Preserve process reports/timings as
external evidence.

Only complete three-pair PASS authorizes R1E contract design. A first failure
stops publication and preserves R1E, B4E, runtime and production as blocked.
