# NSR3-B4DR1E reference-attestation evidence -- 2026-08-21

Status: `PASS / NEW_EXTERNAL_DFSPH_REFERENCE_CANDIDATE / B4E_CONTRACT_DESIGN_AUTHORIZED`

## Result

The standalone reader independently accepts all three content-addressed R1D
references, reconstructs their decoded semantic and aggregate roots, and
rejects both serialized and decoded mutations. Two fresh positive processes
produce the same canonical report. Four external negative fixtures reject
deterministically without changing the published files.

This closes only reference integrity and parser independence. It does not
compare a Nonlocal trajectory, authorize B4E execution, grant old W0I/W1
credit, create runtime/public schemas or support GPU/production claims.

## Frozen identity

| Item | Value |
|---|---|
| R1E identity | `9cf5fc571fee7bc0be5585d9b467f90cd8a27f40d9cf39d6be999b0c466cbccc` |
| parent R1D profile | `ba34b4e3b12986ebc831320d6811551d5311a6774a64f079aabe3a5eaa6bb746` |
| generator source root | `1c23acc5680c724447a2353cbf68f85facafc9042ba65957663e05f00a2433ce` |
| generator build root | `b8d5a5be6891ba2e6c449c4820c61d432a299ae236e04348a00af3f5103cf9db` |
| payload-profile root | `71bed406888d2516b567f44761fb77bb4d50a3349e2ebd2f754db45583ec8f2c` |
| aggregate-profile root | `2a992210cd745f859636a3032309ec587081e42e54f6fac55626c4696e5cebff` |
| reader implementation commit | `8b9c65c74ef981ab5e2f7a327e9b331fb794d5c8` |

## Independent reader closure

The reader is a standalone C++17 executable. It shares only the SHA-256
primitive and neither links SPlisHSPlasH nor reuses the generator's manifest,
parser, serializer, q99 or contact code. It opens every absolute path
component through `openat` with `O_NOFOLLOW`, admits only regular files at or
below 67,108,864 bytes, checks descriptor identity before and after reading,
then independently parses and canonically reconstructs the full payload.

Reader source root:
`754bf1b9087e499b9cd457c8a243f16df60b2cd82429d176b17ae5049745d96a`.

| Reader input | SHA-256 |
|---|---|
| shared `sha256.cpp` | `a2c7ef6d047e4c00aaf0a83ea364bdf293a521245b382b86782d205ece3fe13b` |
| shared `sha256.hpp` | `2551f6fb6bfcd6ce30f801f46979f72a5e1bda92a69e1530ee03eb509ba0c76e` |
| reader `CMakeLists.txt` | `0adf0af4ffd3360e05ec16684186aece68ead209f93dcc84b5f5e6859691daa9` |
| `main.cpp` | `015e6de9f7e611c6c6730ec1a6e8a25e341294d793967b795a439fa6bedc8165` |
| `reader.cpp` | `36886fa737f16946f4409163452361da9e71093140f1dc60ffa55e82b618e290` |
| `reader.hpp` | `a026b07d947d376a841ec7bbf0db661122f82eb4ce83c7a9f3be1ab1d52bfd72` |

Two independent Release builds each pass all four CTest cases and produce the
same 86,120-byte ELF:

| Field | Value |
|---|---|
| executable SHA-256 | `8c4e7d6103340f6e941975b273cd8f4a5ee850718ec133adae1114b4a7f155ea` |
| GNU Build ID | `069d7f3cb9d65f190425ea3c91dad880ab71974a` |
| profile self-test report | 625 bytes, `99a1f4e990ed40785e031a75b31bb8d71087323731d5ff003373a91a2ed76dcf` |
| CTest | `4/4` in each build |

## Positive pair

Two fresh reader processes attest the same explicit persistent artifact root.
Both exit zero with empty stderr and their reports compare byte-for-byte.

| Field | Process A | Process B |
|---|---:|---:|
| report size | 1,557 bytes | 1,557 bytes |
| report SHA-256 | `60e5575b5e6f5cb332cedda4d59c8ded30960cd5a07a14ccec4cd10ea6c6630e` | same |
| wall time | 3.03 s | 3.03 s |
| maximum RSS | 294,596 KiB | 295,244 KiB |

For Hydro, Dam and Orifice every path/type/capacity/read/hash/manifest/parse,
semantic/aggregate and two-layer mutation gate is true. The report explicitly
keeps `trajectory_started=false`, `b4e_execution_authorized=false`,
`runtime_authority=false` and `production_authority=false`.

## External negative fixtures

The fixtures live outside Git and outside `/tmp`. Every process exits 1 with
empty stderr and a canonical report containing the expected diagnostic.

| Fixture | Diagnostic | Report bytes | Report SHA-256 |
|---|---|---:|---|
| missing payload | `CW-HYDRO-001:MISSING_PROFILE` | 1,509 | `33a7db3cfa9a55a0ec60e4d8d27672b180ad58a4ea7f925d14fd151f39384b14` |
| symlink payload | `CW-HYDRO-001:SYMLINK_REJECTED` | 1,501 | `e42d8f75e4b0f05260362f2b6aabf5f0439c75560e25b53cc049e842e6ca0ecb` |
| 64 MiB + 1 sparse file | `CW-HYDRO-001:REFERENCE_INPUT_CAPACITY_EXCEEDED` | 1,533 | `4b60306e69fc49684a3a94ec8e01f5cbea0c57b72455f3dce2f2caf022688c8d` |
| final-byte mutation | `CW-HYDRO-001:COMPLETE_FILE_HASH_MISMATCH` | 1,583 | `afbc0f2500d7c54eafad33fd3a0b0cb0fed9b1234128660769135363a84ad8ae` |

The three published payload hashes were rechecked unchanged after the negative
suite.

## Decision

R1E is `PASS`. The accepted result is
`NEW_EXTERNAL_DFSPH_REFERENCE_CANDIDATE`; B4E nominal-corpus contract design
may begin. B4E execution remains blocked until that separate contract freezes
the Nonlocal candidate, comparison observables, tolerances, failure policy and
resource budget.
