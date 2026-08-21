# NSR3-B4DR1E -- new external-reference attestation contract

Status: `FROZEN / IMPLEMENTATION_AND_EXECUTION_AUTHORIZED / NEW_ROOT_ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4dr1e-reference-attestation|v1|parent=ba34b4e3b12986ebc831320d6811551d5311a6774a64f079aabe3a5eaa6bb746|generator-source=1c23acc5680c724447a2353cbf68f85facafc9042ba65957663e05f00a2433ce|generator-build=b8d5a5be6891ba2e6c449c4820c61d432a299ae236e04348a00af3f5103cf9db|payload-profile=71bed406888d2516b567f44761fb77bb4d50a3349e2ebd2f754db45583ec8f2c|aggregate-profile=2a992210cd745f859636a3032309ec587081e42e54f6fac55626c4696e5cebff|reader=independent-cxx17;openat-nofollow;max67108864;cwrefv2-full-parse;canonical-reencode|mutations=file-final-byte-xor1;decoded-first-x-xor1|runs=2-byte-exact|credit=new-external-dfsph-reference-candidate
```

Identity SHA-256:
`9cf5fc571fee7bc0be5585d9b467f90cd8a27f40d9cf39d6be999b0c466cbccc`.

## Authority and exclusions

The [R1D PASS](../../development/nonlocal-nsr3b4dr1d-full-generation-evidence-2026-08-21.md)
authorizes this stage. R1E is an integrity/parser attestation only. It cannot
compare a Nonlocal trajectory, issue old W0I/W1 credit, execute B4E, change a
solver, create runtime/public/persistence schemas or make production/GPU
claims.

## Frozen generator source and build

Generator source root definition is:

```text
SHA256(
  "nextengine.nonlocal.nsr3b4dr1e-generator-source.v1\0" ||
  for each path in byte-sorted order:
    path_utf8 || NUL || lowercase_file_sha256 || LF)
```

| Tracked input | SHA-256 |
|---|---|
| `crates/continuum-water/tools/nonlocal-feasibility/src/sha256.cpp` | `a2c7ef6d047e4c00aaf0a83ea364bdf293a521245b382b86782d205ece3fe13b` |
| `crates/continuum-water/tools/nonlocal-feasibility/src/sha256.hpp` | `2551f6fb6bfcd6ce30f801f46979f72a5e1bda92a69e1530ee03eb509ba0c76e` |
| `crates/continuum-water/tools/nonlocal-reference-adapter/CMakeLists.txt` | `7767577d791f7bfe970b0c6e288709e58b6f2f8994b40adc74bf72964e7b8bcd` |
| `crates/continuum-water/tools/nonlocal-reference-adapter/patches/splishsplash-eccce861-cold-start.patch` | `e89cf9befc2a08a9c15bd6b290a97b3f6815da1ea699508c41c15645f86c33bc` |
| `crates/continuum-water/tools/nonlocal-reference-adapter/src/contact_adapter.cpp` | `02a9cf21f47e6b2eafe842a78abaec528f0ad1ca5c036f290327c2aa4d4a59aa` |
| `crates/continuum-water/tools/nonlocal-reference-adapter/src/contact_adapter.hpp` | `1ea245cda5af51fb40d22bf710345eaeb33916335f40ebf94bf12927fa34fa51` |
| `crates/continuum-water/tools/nonlocal-reference-adapter/src/main.cpp` | `8c26c5d2e4c1311ca0a1d22292d232cf4f5d2b24e5a826c5542e3c506e15787d` |
| `crates/continuum-water/tools/nonlocal-reference-adapter/src/r1c_manifest.cpp` | `3a67fa2dec712cf400c792cdfab657c20aea9af1d010d54bbe436fa96980d884` |
| `crates/continuum-water/tools/nonlocal-reference-adapter/src/r1c_manifest.hpp` | `ca926845a83cd8bce91cae5aa3d171cccaec3ca6e788fa4f97f4cf0ef716304e` |
| `crates/continuum-water/tools/nonlocal-reference-adapter/src/r1c_trajectory.cpp` | `77d7361bf317aa7d952f99da9f0e4ff6ecc952d20e05df773c5f7a1cbf91b81e` |
| `crates/continuum-water/tools/nonlocal-reference-adapter/src/r1c_trajectory.hpp` | `2eb41e4d55a7726cacbc41a339c49080ceae69528f9819a25206f153cbf7adc9` |

The resulting source root is
`1c23acc5680c724447a2353cbf68f85facafc9042ba65957663e05f00a2433ce`.
Generator implementation commit is
`fbbd6eb2d0f53e049e4098d0b57bb50b31f35136`.

Build projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4dr1e-generator-build.v1|source=1c23acc5680c724447a2353cbf68f85facafc9042ba65957663e05f00a2433ce|commit=fbbd6eb2d0f53e049e4098d0b57bb50b31f35136|upstream=eccce86155776f6ac52d5080b1f720a52bf29450|upstream-lib=172e6777027564566d6282f8679f4d93cbd4cdc193fc236eb9fdaa210ea07d20|patch=e89cf9befc2a08a9c15bd6b290a97b3f6815da1ea699508c41c15645f86c33bc|compiler=g++-15.2.0|cmake=4.2.3|generator=ninja-1.13.2|mode=Release|cxx=17|float=binary64,avx-off,fma-off,nearest,ftz-off,fast-math-off|omp=1,dynamic-false|binary=1595984:8ba20fc747ecd5a098e978469c67209cc915e897047629271931a5396eb49737
```

Build root is
`b8d5a5be6891ba2e6c449c4820c61d432a299ae236e04348a00af3f5103cf9db`.

## Frozen payload and aggregate profiles

Payload projection fields per scenario are
`bytes:file_sha256:scenario_manifest_root:decoded_semantic_root`:

```text
nextengine.nonlocal.nsr3b4dr1e-payload-profile.v1|hydro=15920965:6d6b70c3feb6fd583d8740a901cc2577de7e3744d4b6c8dbd76a73d2a2710c2f:c430b679dfeec33a6ac12c51df75ddee7e7bc48484c6c05188219f0a727909a0:b5a831e370e25d5e8c1370df3ebe2fa4997b708b04a668b7d6e1ee74c496f0f6|dam=56501239:a0b030805034538420e0a5d90390f7117e645119013e9223ff1b334564cbcc13:8d0a0a85adba50d4784245d460c83757bcce92841d804f4739b81faf27fd5f09:17fdc16b4348eb8dc27c513d56a5eaf5e3376b1cca60e6b1e2751526c259d2f5|orifice=56501341:5c16d4dccd351a8dab0bf613a80702a8a5a3da1517371e1bb751cb8485cbc626:53d0db457091d7a7d6ede58a0628688b701850d6de30dacdb14ee9951d036961:50d18e45b3622cc92591637b7ad14b7aefd8dd1b87ea33a90a08dd9d96c47079
```

Payload-profile root is
`71bed406888d2516b567f44761fb77bb4d50a3349e2ebd2f754db45583ec8f2c`.

Aggregate fields are `q99_x_root:q99_y_root:receiver_count_root`:

```text
nextengine.nonlocal.nsr3b4dr1e-aggregate-profile.v1|hydro=6c5ce1fb89d335657d91922b513b29d0903d09f585e8815b6e37a289f3add73e:b65d550c8bcb0052f249b9cbde2d4fefe175bb690466560e8796db5682ddaf7a:7253b247811382836c7c7678f0e3df2e66dca1248f6b4ac04ce3758e144de498|dam=1a4fce89db7c09dcc28dbb2eb46abc03fceb4b2a0658d687bcd181ff96f600c8:16157e8fcb1ab8acad53cbe369fb2917940bde1303d01c0e0709af9e920b0158:893f43e62fb8adbb2627302d61a4a153cffc9d7616568e460012823f4fa8afbe|orifice=b6c06e1f68b3bc467b5bdf738b2b8034df87b7b206299ec3cd31ecb397a0af3d:d4d0ade221dd14b776076e4364f52319cc89ea3e844e0ccd09236e75c7990344:ff07ee7f84d50cf98745c1d3ec0aca9b79c7494bf9138797da1e63006761466e
```

Aggregate-profile root is
`2a992210cd745f859636a3032309ec587081e42e54f6fac55626c4696e5cebff`.

The semantic root is independently reconstructed as:

```text
SHA256(
  "nextengine.nonlocal.nsr3b4dr1e-decoded-frames.v1\0" ||
  sample_count_u32 || frame_count_u32 ||
  every parsed frame/diagnostic/sample field re-encoded canonically)
```

## Independent reader

Implement standalone C++17 target under
`crates/continuum-water/tools/nonlocal-reference-reader`. It may share only
the existing SHA-256 primitive. It must not link SPlisHSPlasH or call/reuse the
generator's manifest, parser, serializer, q99 or contact code.

The CLI is:

```text
nonlocal-reference-reader --profile-self-test
nonlocal-reference-reader --attest <absolute-artifact-root>
```

`--profile-self-test` verifies all exact projection roots without opening an
artifact. `--attest` rejects relative roots and any root under `/tmp`. It opens
the root, fixed profile directory, scenario directory and hash-named payload
using `open/openat`, `O_NOFOLLOW` and exact directory/regular-file checks.

Before allocation, require size at most 67,108,864 and exactly the scenario
size. Read through the admitted descriptor, compare pre/post `fstat` identity
and size, then require complete SHA-256. Parse independently:

- `CWREFV2\0`, exact manifest length/bytes, 6,000 samples and frame count;
- exact step cadence and total frame/file length;
- step-zero diagnostics, bounded nonzero iterations and receiver count;
- finite binary64 diagnostics, positions and velocities, ordered stable IDs;
- non-inverted density range and exact end-of-file;
- decoded semantic root and independently regenerated aggregate roots.

For every positive file, flip the final serialized byte in a private copy and
require complete-hash rejection. Flip the low bit of the first decoded x value,
canonically reconstruct and require semantic-root rejection. Restore neither
mutation into the published file.

## External negative fixtures and execution

Outside Git and outside `/tmp`, create isolated roots proving deterministic
rejection of:

1. missing profile/file;
2. a symlink in the fixed payload path;
3. an oversized sparse regular file before allocation;
4. a regular complete-size copy with its final byte flipped.

Never mutate or replace the published R1D files. Positive attestation runs
twice against the same explicit root. Reports exclude the host-specific root
string, are canonical LF text, must be byte-identical and record per scenario
path/type/capacity/read/hash/manifest/parse/semantic/aggregate/mutation gates.

The report must keep `trajectory_started=false`, `b4e_execution_authorized=false`,
`runtime_authority=false` and `production_authority=false`. Complete PASS
selects only `NEW_EXTERNAL_DFSPH_REFERENCE_CANDIDATE` and authorizes B4E
nominal-corpus contract design. Any failure leaves B4E blocked.
