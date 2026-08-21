# NSR3-B4DR1B standalone contact-adapter evidence -- 2026-08-21

Status: `PASS / R1C_DESIGN_AUTHORIZED / NO_TRAJECTORY_AUTHORITY`

## Result

The frozen v2 contact adapter passes its process/ABI preflight, six parent
vectors, two algebra sentinels, independent clearance/chord validation and two
structural profile mutations. Two fresh executions have byte-identical stdout
and zero stderr. No particle world, DFSPH step or trajectory was initialized.

This closes only R1B. It authorizes freezing the R1C scenario manifests; it
does not authorize executing those manifests before their contract is
committed, and it does not authorize B4E.

## Frozen implementation and build identity

| Input | Attested value |
|---|---|
| implementation commit | `7f537c839b18d5d9eb5ed51b38de3805b039447c` |
| R1B contract | `c65346ec7b215a7a173eabfdd6c91e20869d4a0679b9d014a1e897369cb71f84` |
| parent B4DR1 | `ade621f889a08fd26ca713592625c50316b893af8c4f1797d25f4e4d4c96b86a` |
| upstream static library | `172e6777027564566d6282f8679f4d93cbd4cdc193fc236eb9fdaa210ea07d20` |
| tracked input root | `83388aea04ab165178e7483951352e1a47c010f7a7e4ad8e8c71aef1119dfe1d` |
| normalized compile/link root | `ab68bffb5699d8fcf3d35ca59d37aaa536160893703c34087c088eee153dbf79` |
| executable | 1,488,880 bytes; SHA-256 `c1150fadf3829ed60b14e4b578f295602b98d91b3eae45c06fa28ef8c715a5da` |
| ELF build ID | `9e889d47950520511d5597452d54ebd2ac98cee3` |

The tracked-input root is SHA-256 over the concatenated GNU `sha256sum` rows,
in this exact order: the reused `nonlocal-feasibility/src/sha256.cpp/.hpp`,
then adapter `CMakeLists.txt`, `README.md`, `contact_adapter.cpp/.hpp` and
`main.cpp`. The individual adapter source hashes are:

| File | SHA-256 |
|---|---|
| `CMakeLists.txt` | `c326265170270ec4c37c92061ce43ad5bab57586dbbe6423d6c5322e27f25317` |
| `README.md` | `58a66845d2d733eb589e02352cbaaec252d37f2cdbba5315beaceab422aa8ce2` |
| `src/contact_adapter.cpp` | `49687bf42dede570b4894edf689a6d4a878d2347dc55db35e12cce7609c4ee4b` |
| `src/contact_adapter.hpp` | `c4bf38120e802b4453aeb3d5bd63980758d85a6fdf1723cc1a609d12b0827907` |
| `src/main.cpp` | `8cb3c3577dcecc998d0cb873ecc59c6d4f050a20e707b1921c1dea6b9af7bc71` |

Both builds used Release/Ninja, `/usr/bin/g++`, `BUILD_TESTING=ON`, the exact
R1A source/build roots and the adapter's C++17 strict flags
`-O3 -DNDEBUG -ffp-contract=off -fno-fast-math -mno-avx -mno-avx2 -mno-fma
-march=x86-64`. The compiler, CMake and Ninja executable hashes remain the R1A
values. Path normalization replaces the NextEngine source, adapter build and
SPlisHSPlasH source/build roots before hashing all three compile commands and
the final grouped-static-archive link command.

Two distinct adapter build directories produced the same normalized command
root and byte-identical executables. Disassembly of the final executable found
zero AVX-like and zero FMA instructions. The linked binary exports the real
`TimeStepDFSPH::{step,pressureSolve,divergenceSolve}` closure and
`TimeStepDFSPH::METHOD_NAME`; none is invoked.

## ABI and dynamic closure

The default report proves eight-byte IEC-559 `SPH::Real` with 53 binary digits,
eight-byte pointers, little endian, DFSPH method anchor, locale `C`, nearest
rounding, FTZ/DAZ off and one non-dynamic OpenMP thread.

| Runtime object | SHA-256 |
|---|---|
| `libgomp.so.1.0.0` | `645478cbe100d1465b9d45f7047bca3965358f4d399937e544f55ac7a924d5a3` |
| `libstdc++.so.6.0.35` | `5bb0d21308f123b6ad46c6f35b42cedfcb8d6d439a53aa3dae04d880aaffdde3` |
| `libm.so.6` | `beea4eeacfcfa2cd96011b959a826c97cf4a774017e214f6a34d7eea3d49cd88` |
| `libgcc_s.so.1` | `9d339ecb409578d6a5d587e6c537a8f9589b8a13fefba30d167433a4b5758bee` |
| `libc.so.6` | `a3947513a02831ec692ebf13053c07614882ab54a2101fb91a1b15724062ed0c` |
| `ld-linux-x86-64.so.2` | `c5e80a563850d6ab5c2f2482e4202d9c1b71fbf44854b8c399e63527202c64e1` |

`linux-vdso.so.1` is the only non-file `ldd` entry. The embedding executable
defines the logging, timing and counting stores exactly once, matching the
upstream standalone-tool convention. An initial link without those required
definitions failed on the corresponding utility globals. They were added at
the executable boundary instead of weakening the DFSPH anchor.

## Contact and reproducibility result

| Fixture | Canonical velocity (um/s) | Canonical end (um) | Hits |
|---|---:|---:|---|
| outer face | `6000000,0,0` | `975000,500000,500000` | `1` |
| outer high speed | `-114000000,0,0` | `25000,500000,500000` | `0` |
| aperture pass | `30000000,0,0` | `1025000,300000,500000` | none |
| aperture edge graze | `30000000,0,0` | `1025000,225000,500000` | none |
| aperture edge impact | `26544000,4608000,0` | `1010600,239200,500000` | `17` |
| aperture corner sphere | `18000000,0,0` | `975000,200000,400000` | `21` |
| internal-face sentinel | `18000000,0,0` | `975000,100000,500000` | `16` |
| one-ulp edge restart | `19200000,14400000,0` | `1065000,280000,500000` | `17` |

The two fresh-process reports are each 1,730 bytes including the final LF and
have stdout SHA-256
`c6a4950d45270eede2a89a3cecfb4502aa4dd801ecb203d971f312a383203a8a`.
Both exit zero with empty stderr. A one-byte `PASS` to `FASS` capture mutation
changes the hash to
`182125c0f73f5ecaa5de28ed9e7508acddfbfc2564e2bcf362aeadc62a7632e7`.

The frozen base contact-profile root is
`8a40c66a764dc275e831c075c633b7bb6bf4ff3b358c928ea06d000d649957d2`.
Swapping feature-order IDs `17/18` changes it to
`fe4f56e3481a77c72218573842df120f14484bb32d87b8ca210a9d0a09c56837`;
incrementing the radius bits by one changes it to
`c2b7d61a505849a2081cace0c0a750badd5e5c81575aba473c468dcfa43ef7f6`.

## Fail-closed matrix

CTest passes all four registered positive/negative targets. Direct process
tests additionally prove the exact early-rejection boundary:

| Mutation | Exit/reason | Contact/trajectory |
|---|---|---|
| downward rounding | `1 / ROUNDING_NOT_NEAREST` | `false / false` |
| FTZ on | `1 / FTZ_ENABLED` | `false / false` |
| locale `C.UTF-8` | `1 / LOCALE_NOT_C` | `false / false` |
| `OMP_NUM_THREADS=2` | `1 / OMP_NUM_THREADS_NOT_ONE` | `false / false` |
| `OMP_DYNAMIC=TRUE` | `1 / OMP_DYNAMIC_NOT_FALSE` | `false / false` |
| unknown argument | `1 / UNKNOWN_ARGUMENT` | `false / false` |
| extra argument | `1 / UNKNOWN_ARGUMENT` | `false / false` |

## Decision

R1B passes. Preserve the rejected v1 geometry root as negative evidence and
select only R1C scenario-manifest design next. R1C execution, R1D generation,
R1E attestation, B4E, runtime/schema, CUDA and production remain blocked.
