# NSR3-B4E2R first-output reference-slice evidence -- 2026-08-22

Status: `PASS / FIRST_OUTPUT_REFERENCE_SLICE / B4E2D_CONTRACT_RESEARCH_AUTHORIZED`

## Result

The new standalone extractor admits the exact content-addressed R1E Dam and
Hydro payloads, validates their complete hashes and selected layouts, and
publishes deterministic canonical first-output slices for Dam step 4 and
Hydro step 24. Two independent Release builds produce the same executable;
one fresh process from each build emits a byte-identical report with empty
stderr.

No Nonlocal transaction ran. No duration, RSS or host path entered the report,
and this PASS creates no performance credit.

## Frozen identity and implementation

| Item | Value |
|---|---|
| B4E2R identity | `681e6e2aab130e0a461dadf575caac754b0931668027db911512a1edac2cccf3` |
| research/contract checkpoint | `d57342f` |
| implementation checkpoint | `e11930c6b2a14ee1fac3107c2eb94ed0d4348d50` |
| source root | `909c5d0a27c862e8080f09d25d070ced734e297b7fc55fd433fab716ce25fbb0` |

Source root definition is `SHA256(domain || NUL || sorted(path || NUL ||
file_sha256 || LF))`, with domain
`nextengine.nonlocal.nsr3b4e2r-source.v1`.

| Source input | SHA-256 |
|---|---|
| shared `sha256.cpp` | `a2c7ef6d047e4c00aaf0a83ea364bdf293a521245b382b86782d205ece3fe13b` |
| shared `sha256.hpp` | `2551f6fb6bfcd6ce30f801f46979f72a5e1bda92a69e1530ee03eb509ba0c76e` |
| slice `CMakeLists.txt` | `0f4883279236a283d744aa9fc0ef014ee6ad67d3fe70c35a26b4f60da5437a52` |
| `main.cpp` | `cd4fc12a43bbe4d9f851d6e958f068ef2080876c0c7c2a5eef17fe2b51c46928` |
| `slice.cpp` | `07ac12b37352d98c8f9cb0b8cc8ca0413c79a07e1eea5734c3ac5c35169d9129` |
| `slice.hpp` | `a78347d6123c011323bb7f3d192676bf5529ad6923bd263876da7ec18aba92ba` |

The target links neither SPlisHSPlasH, the generator/parser, R1E `reader.cpp`,
the Nonlocal solver nor canonical publication code. Its micrometre conversion
independently decomposes binary64 and performs checked integer ties-to-even.

## Builds and positive pair

Raw evidence is retained outside Git under
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4e2r.aqF7er`.
Both builds used GCC 15.2.0, CMake 4.2.3, Ninja 1.13.2 and Release with the
frozen warnings-as-errors/floating-point flags.

| Field | Build/process A | Build/process B |
|---|---:|---:|
| CTest | `3/3 PASS` | `3/3 PASS` |
| exit | `0` | `0` |
| stderr bytes | `0` | `0` |
| report bytes | `1,977` | `1,977` |
| report SHA-256 | `6f2d0ffba91b3056ce1535e5e15ddd29d7e0f49b7b4925a02e98dce1b0c740f1` | same |
| executable bytes | `86,176` | `86,176` |
| executable SHA-256 | `03109daf87a645b3522feae2a74df2bc9900ff84b8e0559f4d1ea55f1ea1aa5d` | same |

GNU Build ID is `32a8f54c86c80dcb7bd7a93fc9baf8b51d0360db`.
The semantic report result is
`8ca3498dbc04556bf86f32a8cab55eccee92ff201aecb737d8ac2760c64504c6`.

## Frozen reference slices

All sums and q99 values are signed integer micrometres or micrometres per
second after exact ties-to-even conversion of the admitted binary64 samples.

| Field | Dam step 4 | Hydro step 24 |
|---|---|---|
| sample root | `1fdebca13509ef85c08bd3886c96083f692dac353493322014ba40c10af57d92` | `d7eee122e5636c811efa4c831a7652505a6b9daccb887142f6a99324409e1099` |
| position sum | `3029244660,2280395480,2999999999` | `3000000212,2404802771,3000000212` |
| velocity sum | `1856527209,1616954663,-4` | `21280,-295879007,21280` |
| q99 x | `992829` | `975000` |
| q99 y | `740902` | `822662` |
| aggregate root | `b8ad20e889d519cc6fdc0a5bbdb228369eb426451ed27ed93058107b9587750c` | `d9a113a3f4c1cab8e12363601da59e796c509272d0008a20f90831da65953c47` |

The DFSPH iteration/density/speed fields remain provenance diagnostics. They
are not Nonlocal comparison tolerances and grant no solver credit.

## Controls and external negatives

Both files pass complete hash, exact manifest/layout, frame-zero, selected-
frame and stable-ID admission. Every frozen control is true:

- private final-file-byte mutation changes the complete hash;
- selected-step mutation rejects;
- swapped first two stable IDs reject;
- decoded first x plus one micrometre changes sample and aggregate roots;
- the synthetic 6,000-value case proves nearest-rank q99 index 5,939;
- integer half cases prove ties-to-even.

Additional descriptor-path negatives are retained under
`/home/kaifaty/.cache/nextengine/external/negative-nonlocal-b4e2r.KmiVAr`:

| Fixture | Exit | First failure | Report SHA-256 |
|---|---:|---|---|
| missing profile | `1` | `CW-DAMBREAK-001:PROFILE_OPEN_REJECTED` | `756c1ae3ce6a1f4b0eaa6d00947b152634c1a9ac86211fa9eb85f28ded9637db` |
| symlink payload | `1` | `CW-DAMBREAK-001:SYMLINK_REJECTED` | `ddd435b5e4eb535217325e07167fa41cf4cff86d7936e1096967f5922a8ee94e` |

Both negative stderr files are empty.

## Decision

B4E2R passes and selects `FIRST_OUTPUT_REFERENCE_SLICE`. These immutable
slice roots authorize only research and contract design for a Dam-first B4E2D
multi-macro physical pilot. Hydro execution, broad B4E3/B4E4, performance
selection, runtime/GPU/schema and production remain blocked.

