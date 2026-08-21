# NSR3-B4DR1D full-generation evidence -- 2026-08-21

Status: `PASS / R1E_CONTRACT_DESIGN_AUTHORIZED / NO_B4E_RUNTIME`

## Result

Hydro, Dam and Orifice each complete the frozen full schedule twice in fresh
one-thread processes. All six exit zero with empty stderr. Complete canonical
reports and `CWREFV2` payloads are byte-identical within every same-scenario
pair.

Every physical step runs DFSPH convergence validation and the accepted swept
contact projection; only the frozen output cadence is serialized. Hydro keeps
the short-gate iteration maxima over 1,200 steps. Orifice completes 720 steps
with 1,172 final receiver samples. No solver, calibration, boundary, contact or
frame-format change was needed after R1C5.

## Implementation and build closure

| Input | Attested value |
|---|---|
| implementation commit | `fbbd6eb2d0f53e049e4098d0b57bb50b31f35136` |
| R1D identity/profile root | `ba34b4e3b12986ebc831320d6811551d5311a6774a64f079aabe3a5eaa6bb746` |
| executable | 1,595,984 bytes; `8ba20fc747ecd5a098e978469c67209cc915e897047629271931a5396eb49737` |
| ELF build ID | `2d68ef197135bd771c3b6f30011cb9978d2a0a29` |
| focused tests | `15/15 PASS` in both build directories |

An incremental build and a new out-of-tree Release/Ninja/GCC 15.2 build are
byte-identical. Both bind the same verified ordinary full upstream clone,
commit `eccce86155776f6ac52d5080b1f720a52bf29450`, eight-library closure and
tracked cold-start/diagnostic patch as R1C5.

The final executable also preserves earlier roots:

- R1B canonical report:
  `c6a4950d45270eede2a89a3cecfb4502aa4dd801ecb203d971f312a383203a8a`;
- R1C1 manifest report:
  `6d2933283281591cc1dd259053de2b68b1f27188e945c7e753ef21323eb558f3`;
- R1C5 Hydro report/payload:
  `cb5c19fad36bd9390dff4b09dd259ea1b2b2a4270bdf68a53b60b8beec747835` /
  `9c58556d8ab93ef346181c1fe96befb1d6533e181ac6c2acf689f23bf3d73c63`.

## Manifest and negative gates

Two manifest-only processes produce the same 1,004-byte report SHA-256
`45d5f972caad2a8a329b357a393105c64d570da43e44a38c4f6a72466be51e64`.
They verify all three schedule manifest roots, exact 51/181/181 frame counts,
payload capacities and initial-lattice nearest-rank q99 without creating a
Simulation.

The forced Orifice schedule-root mismatch exits one before Simulation with
296-byte report root
`d9c091ac04582cacd0d5f0594c39875f02bbc94d660fe3b34737f4d73603de73`.
A relative output path also exits one before Simulation with 239-byte report
root `ce697b893ca923f3d630a52d55f163c036723d592a600db007242952358eb257`.
All four preflight/negative stderr streams are empty.

## Full paired results

| Scenario | Frames | Payload bytes / SHA-256 | Report SHA-256 | Max P/V | Contact hits | Final receiver | Wall pair | Peak RSS pair |
|---|---:|---|---|---|---:|---:|---:|---:|
| `CW-HYDRO-001` | 51 | 15,920,965 / `6d6b70c3feb6fd583d8740a901cc2577de7e3744d4b6c8dbd76a73d2a2710c2f` | `84ad8450af0e8e39e92744d4652de97237e7a40ace29bc45869737724f663ef6` | `220 / 12` | 1,787,879 | 0 | 7:44.41 / 7:42.60 | 57,612 / 58,272 KiB |
| `CW-DAMBREAK-001` | 181 | 56,501,239 / `a0b030805034538420e0a5d90390f7117e645119013e9223ff1b334564cbcc13` | `24bfe0b38a65032bf4857d3f59bd21cb7bc1c0bdffa7fa4d4e20ee6047d78dfa` | `203 / 9` | 1,142,368 | 0 | 1:03.52 / 1:03.97 | 179,064 / 178,732 KiB |
| `CW-ORIFICE-001` | 181 | 56,501,341 / `5c16d4dccd351a8dab0bf613a80702a8a5a3da1517371e1bb751cb8485cbc626` | `ee8505c8261b63d76c726173708fb3019a0a359ceacd936759f80c1ac5345ea3` | `205 / 13` | 1,268,897 | 1,172 | 3:22.92 / 3:23.10 | 175,848 / 175,664 KiB |

Reports are 921, 925 and 927 bytes respectively. Direct `cmp`, not report
hashes alone, establishes both report and payload pair equality.

## Aggregate roots

| Scenario | q99-x root | q99-y root | receiver-count root |
|---|---|---|---|
| Hydro | `6c5ce1fb89d335657d91922b513b29d0903d09f585e8815b6e37a289f3add73e` | `b65d550c8bcb0052f249b9cbde2d4fefe175bb690466560e8796db5682ddaf7a` | `7253b247811382836c7c7678f0e3df2e66dca1248f6b4ac04ce3758e144de498` |
| Dam | `1a4fce89db7c09dcc28dbb2eb46abc03fceb4b2a0658d687bcd181ff96f600c8` | `16157e8fcb1ab8acad53cbe369fb2917940bde1303d01c0e0709af9e920b0158` | `893f43e62fb8adbb2627302d61a4a153cffc9d7616568e460012823f4fa8afbe` |
| Orifice | `b6c06e1f68b3bc467b5bdf738b2b8034df87b7b206299ec3cd31ecb397a0af3d` | `d4d0ade221dd14b776076e4364f52319cc89ea3e844e0ccd09236e75c7990344` | `ff07ee7f84d50cf98745c1d3ec0aca9b79c7494bf9138797da1e63006761466e` |

## External publication

Pair candidates remain under persistent external suite root
`/home/kaifaty/.cache/nextengine/external/r1d-full-suite.oY7yxl`. After pair
equality, one copy per scenario was copied to a fresh partial file, compared,
rehashed, fsynced, atomically renamed and compared/rehashed again. The final
objects are regular files, not symlinks, under:

```text
/home/kaifaty/.cache/nextengine/reference-artifacts/
  ba34b4e3b12986ebc831320d6811551d5311a6774a64f079aabe3a5eaa6bb746/
    CW-HYDRO-001/6d6b70c3feb6fd583d8740a901cc2577de7e3744d4b6c8dbd76a73d2a2710c2f.cwrefv2
    CW-DAMBREAK-001/a0b030805034538420e0a5d90390f7117e645119013e9223ff1b334564cbcc13.cwrefv2
    CW-ORIFICE-001/5c16d4dccd351a8dab0bf613a80702a8a5a3da1517371e1bb751cb8485cbc626.cwrefv2
```

The two parallel waves take about 927 seconds of elapsed critical-path time
versus 1,461 seconds for the sum of all six process wall times, a `1.58x`
harness-wall improvement. This is independent-scenario resource utilization,
not a DFSPH or runtime throughput claim. Hydro's long late-state cost is a new
performance-roadmap fact; it does not change the reference profile.

## Decision

R1D passes and authorizes only design of the R1E reader/profile attestation.
R1E must freeze the actual source/build/binary/scenario/payload roots above,
parse the complete files within capacity, reject full-file and in-memory
mutations and execute twice byte-identically.

R1D alone does not authorize B4E, candidate-vs-reference comparison, old W1
credit, runtime use, GPU work, public schema or production claims.
