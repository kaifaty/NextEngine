# NSR3-B4E2D1 -- external frame-zero binary64 contract

Status: `FROZEN / NOT_RUN / NO_SOLVER / NO_TRAJECTORY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d1-frame-zero-binary64|v1|parent=2b09ce8435fc185bd91322e17f1ee34f04a3ec59dc1a47dc5bba04cef23c6641:ee0c8d94799420a112103917252a4bc0e3b31a62bc652c7ebda7e03ccf23845f:be8e9ecf9bc35204ed422ecdb35032e5dabaa3688e5b7f788826d78ef0626d4e|reference=681e6e2aab130e0a461dadf575caac754b0931668027db911512a1edac2cccf3:8ca3498dbc04556bf86f32a8cab55eccee92ff201aecb737d8ac2760c64504c6:a0b030805034538420e0a5d90390f7117e645119013e9223ff1b334564cbcc13|slice=dam;frame0;step0;stable-id6000;position+velocity-u64bits|parser=standalone-cxx17;openat-nofollow;full-hash-before-parse;frame0-only|candidates=micrometre-division,addition-lattice;velocity-zero;ids-iy-iz-ix|comparison=external-root-equals-exactly-one-candidate;vector-mismatch-counts|controls=serialized-position-bit,candidate-position-bit,id-swap|runs=2-builds;2-processes;byte-exact|trajectory=none|timing=none|credit=b4e2d2-topology-reclosure-research-only
```

Identity SHA-256:
`c3fb3522dba71768a7933c293523ebbbbaf08a0b5ee408a57b27a2300a2e22d8`.

## Executable and input

Add `nonlocal-reference-slice --initial-binary64 <absolute-artifact-root>` to
the standalone B4E2R target. Retain its exact descriptor-safe path admission,
64 MiB capacity, complete Dam file size/hash, manifest/layout, frame-zero
diagnostic and stable-ID checks. The new mode reads no selected/later frame
and keeps the old `--first-output` report byte-exact.

The target may share only SHA-256. It must not link the generator/parser,
R1E reader, Nonlocal runner, balanced canonical implementation or
SPlisHSPlasH.

## Raw-bit roots and candidates

For each frame-zero sample hash little-endian stable ID followed by the raw
little-endian u64 bits of position xyz and velocity xyz under a dedicated
domain. Publish the external root.

Independently generate the same ordered sample set twice:

1. `decoded`: `(25000 + 50000*axis_index) / 1000000.0`;
2. `addition`: `0.025 + axis_index*0.05`;

where IDs are `iy-iz-ix` and all velocity bits are positive zero. Publish both
roots plus counts of complete position vectors bit-exact to external. Require
the external complete root to equal exactly one candidate root and the two
candidate roots to differ. Name the selected representation explicitly.

Before PASS require a private serialized first-position-bit mutation, a
candidate first-position-bit mutation and swapped first IDs to change/reject
their roots. Restore all bytes before the positive report.

## Runs and authority

Build Release twice, run one fresh process from each build against the same
persistent artifact root, and require exit zero, empty stderr and byte-exact
LF-terminated reports. Record executable/report roots and result in evidence.
No timing wrapper or speed claim.

PASS authorizes only B4E2D2 topology-alignment research/contract design using
the selected frame-zero representation. It grants no B4E2D rerun, Hydro/full
corpus, runtime/GPU/schema/PhysX or production authority.

