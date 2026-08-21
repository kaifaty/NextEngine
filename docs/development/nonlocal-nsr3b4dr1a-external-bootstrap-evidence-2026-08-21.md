# NSR3-B4DR1A external-reference bootstrap evidence -- 2026-08-21

Status: `PASS / R1B_DESIGN_AUTHORIZED / NO_TRAJECTORY_AUTHORITY`

## Result

The pinned external SPlisHSPlasH library and the dependencies needed by its
DFSPH target build reproducibly under the strict B4DR1 binary64 profile.
Different source/build paths produced identical normalized command roots and
byte-identical static artifacts, and a later verified true-full-clone build
reproduced the complete artifact closure. No adapter, contact vector, particle
trajectory or `CWREFV2` payload executed.

The original wording that both first source trees were full Git clones was
too strong. The exact correction and true-full-clone revalidation are recorded
in the [dated provenance correction](nonlocal-nsr3b4dr1a-full-clone-provenance-correction-evidence-2026-08-21.md).

## Source and dependency closure

| Component | Commit | Tree/archive SHA-256 | License identity |
|---|---|---|---|
| SPlisHSPlasH | `eccce86155776f6ac52d5080b1f720a52bf29450` | tree `ed6966d6b85a87df1dd2f127fbd9163a8b55cfb6`; archive `b2f3ead4bf591f282532d35d952c092dc7277c79ceec49258417a159eee4161c` | MIT file `608181acd95c1b672b984157254568d581b6d9eafc3e5785899ce1ac85dc9c29` |
| CompactNSearch | `a9ab7c71ce264487660ecbaf81b5060bda462722` | tree `ae68f93e78fad0e6fb5524fed60eb08935b192e7`; archive `454717eefb08e0d17243f920b386a45d9fbac26c326ac6337a32658549ca8621` | `6f12ab962339fad18e34c0984a6a5065404e697e72545680121cb85835b2c896` |
| Discregrid | `ddf20dc0480874bf02e0bdc6ded76c1f101b17fb` | tree `5948b5d59a94a7415efa80271a67c205939a9133`; archive `2196deab06bae9b56cad04285d33d7670cb70f63a205a2a9abffdd11d942ed47` | `dd7e4401f6af3e9f34497b97ad1e594d6ea23156b7db4ad31b2667c68c7b55a5` |
| GenericParameters | `a4e2744eea526270cfe38b826440d09f66473316` | tree `011c9c324d2588ba68d8c3370405741011b8ad1e`; archive `d70a230154fbad556d23a836e5b5b1b0f92847b9480518833467438e9cc57eb9` | `7de8f2feb1a5f3045575b61ba618c262f47dda657ed21470f7ffa9739f548ce7` |

The upstream commit has no Git submodules. Its CMake target separately pins
the three repositories above. Bundled Eigen, zlib, Partio, MD5 and tinyexpr
are covered by the upstream archive root; their license-bearing file hashes
were inventoried locally. GUI, Python, tests and the PositionBasedDynamics
target were not configured into the minimal `SPH_LIBS_ONLY` build.

## Toolchain and exact profile

| Input | Attested value |
|---|---|
| compiler | `/usr/bin/x86_64-linux-gnu-g++-15`, GCC `15.2.0`, target `x86_64-linux-gnu`, executable SHA-256 `e6718f7e0c7d057c3ff77b550c603da9bc4030e3ede3c053705acce1293dbe4d` |
| CMake | `4.2.3`, `/usr/bin/cmake`, SHA-256 `6e1dccda39845415d68eabb934c598998949c99ec4668625d571aee1827b05c7` |
| generator | Ninja `1.13.2`, SHA-256 `91e9548850cda2799facfdaa7abe33f9e978832f85e98212255708a7bbe437f2` |
| host ABI | Linux x86-64, glibc `2.43` |
| CMake selection | Release, `CI_BUILD=ON`, `SPH_LIBS_ONLY=ON`, `USE_DOUBLE_PRECISION=ON`, `USE_AVX=OFF`, `USE_PERFORMANCE_OPTIMIZATION=OFF`, `USE_PYTHON_BINDINGS=OFF`, `USE_OpenMP=ON`, `USE_THIRD_PARTY_METHODS=OFF`, static libraries |
| strict flags | `-ffp-contract=off -fno-fast-math -mno-avx -mno-avx2 -mno-fma`; upstream Release adds `-O3 -DNDEBUG -march=x86-64`; C++11 and OpenMP 4.5 |

The real main and nested Ninja commands contain the strict flags. They contain
neither `-march=native` nor `-ffast-math`. Disassembly of SPlisHSPlasH,
CompactNSearch and Discregrid found zero AVX-like and zero FMA instructions.
The main archive exports the expected binary64
`TimeStepDFSPH::{step,pressureSolve,divergenceSolve}` symbols.

Path-normalized command-stream roots are:

| Command closure | SHA-256 |
|---|---|
| main `SPlisHSPlasH` target | `136ef47ea5feef79ea23ff658db24a591caa256a97de0ebd083b9eda136e8b7f` |
| CompactNSearch nested build | `28f8e875ab4338a7b0e63f3d4a18f5a4ac43fe2188efe1099c1ae25d19dd5625` |
| Discregrid nested build | `f30a75455636d654896be391edffc37ec77660f66e433323b660243128baa603` |

## Reproducibility result

The two original builds matched, and a later verified true-full-clone build
reproduced the same identities:

| Artifact | Bytes | SHA-256 | Result |
|---|---:|---|---|
| `libSPlisHSPlasH.a` | 5,016,786 | `172e6777027564566d6282f8679f4d93cbd4cdc193fc236eb9fdaa210ea07d20` | exact after revalidation |
| `libUtilities.a` | 15,604 | `93c6dd16ae500aae89b370b8a3bca902da3ba5a32e91a99630f66dfa89a1d0c4` | exact after revalidation |
| `libtinyexpr.a` | 28,228 | `63e8e8bda564e8738dfbb085a4032ba893207b4dffb2ba0b3acb25f723126cbc` | exact after revalidation |
| `libMD5.a` | 15,110 | `e983a90676a7200508467fc2a8e1ae3c0453515cdc77d9ca3c0e620ab0d11239` | exact after revalidation |
| `libpartio.a` | 927,100 | `a0f9ae32d8661ea47e848fbed167ea2ed9d1f652293736b3700eb6d360892494` | exact after revalidation |
| `libzlib.a` | 152,300 | `70f93ec9b3400961560606548e672b036106d82e6ba8b37aa2d870a00cbf7d97` | exact after revalidation |
| `libCompactNSearch.a` | 86,922 | `5d3df5a05d8148b2fae02a7cd9a81a2fe09d06af82c5a98a041255f772c87e61` | exact after revalidation |
| `libDiscregrid.a` | 172,112 | `8f6fa4d0f0992e0154e02554a836673ad676e3b52bc68ef773bc9c2c3885c19a` | exact after revalidation |

All builds generated the same upstream `Utilities/Version.h`, SHA-256
`72be70658e43ed1f6e310de446ad5153ccb3f505f853b2040ffcf99a1ed8c900`.
GNU `ar` used deterministic member metadata.

## Negative evidence and bootstrap rule

- A blobless partial checkout made 805 HEAD blobs resolve through slow lazy
  requests. The retained recipe must use a complete checkout or explicitly
  batch-fetch every missing HEAD blob before checkout.
- Upstream writes untracked `Utilities/Version.h` into its source tree.
  Provenance therefore requires a clean status immediately before the first
  configure and records the generated header separately afterward.
- A linked Git worktree makes the old upstream revision module emit
  `HEAD-HASH-NOTFOUND`. Linked worktrees are forbidden for reference builds;
  use an ordinary full Git clone.
- Final adapter compile/link command hashes, runtime linked-library closure,
  `sizeof(Real)` and floating-environment facts cannot exist before adapter
  source is frozen. They are mandatory R1B exit evidence, not silently waived.

## Decision

R1A passes and authorizes only R1B contract/design. R1B must freeze the
standalone contact adapter and its six vectors before compiling or executing
it. It must fail closed unless locale `C`, round-to-nearest, FTZ/DAZ off,
`OMP_NUM_THREADS=1` and `OMP_DYNAMIC=FALSE` are proven at process start.
B4E and every particle trajectory remain blocked.
