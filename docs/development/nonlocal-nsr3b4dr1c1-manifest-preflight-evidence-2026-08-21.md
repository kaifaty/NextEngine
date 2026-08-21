# NSR3-B4DR1C1 manifest-preflight evidence -- 2026-08-21

Status: `PASS / TRAJECTORY_IMPLEMENTATION_AUTHORIZED / NO_SOLVER_EXECUTED`

## Result

The corrected manifest-only gate independently reproduces the exact hydro,
dam-break and orifice scenario, fluid and boundary projections. Two fresh
processes from byte-identical builds produce byte-identical reports with zero
stderr. The forced manifest-root mismatch exits nonzero before creating a
Simulation or starting a trajectory.

No SPlisHSPlasH Simulation, fluid model, boundary model, time step, DFSPH
iteration or output payload was created. This closes only the zero-physics
R1C1 gate and authorizes implementation of the already-frozen R1C 24-step
trajectory path. R1D and B4E remain blocked.

## Frozen implementation and build identity

| Input | Attested value |
|---|---|
| implementation commit | `d3c22e88d4ad2b0e7e0653f91ef4f1ab54fa95f8` |
| R1C1 contract | `865570e18864ec55cdbbbbad8b9cfa3f200a085087ecf272366c342144488927` |
| rejected R1C parent | `a061f43ea3bc60fc3ff3af03aa242c298ad059e094e2215bab71642902f199d0` |
| upstream commit | `eccce86155776f6ac52d5080b1f720a52bf29450` |
| future trajectory patch | `e89cf9befc2a08a9c15bd6b290a97b3f6815da1ea699508c41c15645f86c33bc` |
| tracked input root | `e35c41c85f98676d3db11654d6627b0094389bc344f473121183793564cd3e01` |
| executable | 1,510,480 bytes; SHA-256 `c8933e011f842b0e9d5bf145555dd01a74b9da103cabc3bb6d5c0598a027b6ca` |
| ELF build ID | `c2c2591593e3e2031fae94bba44ab69990c9880b` |

The tracked-input root is SHA-256 over concatenated GNU `sha256sum` rows in
this exact order: shared `sha256.cpp/.hpp`, adapter `CMakeLists.txt`, README,
`contact_adapter.cpp/.hpp`, `main.cpp`, then `r1c_manifest.cpp/.hpp`.

The two original Release/Ninja builds used GCC 15.2 and the unchanged R1A/R1B
strict binary64/no-fast-math/no-AVX/no-FMA flags, and their final executables
were byte-identical. A later audit found that one retained source tree did not
have a complete Git object database. The historical implementation was
therefore rebuilt against a verified true full clone and reproduced the same
executable and report hashes with `6/6` focused tests. See the
[dated provenance correction](nonlocal-nsr3b4dr1a-full-clone-provenance-correction-evidence-2026-08-21.md).
Disassembly finds zero AVX-like and zero FMA instructions.

The default R1B report remains byte-identical at SHA-256
`c6a4950d45270eede2a89a3cecfb4502aa4dd801ecb203d971f312a383203a8a`.

## Positive process evidence

Both fresh-process stdout files are 1,744 bytes and have SHA-256
`6d2933283281591cc1dd259053de2b68b1f27188e945c7e753ef21323eb558f3`.
Both stderr files are empty. The exact scenario rows are:

| Scenario | Manifest | Fluid | Boundary |
|---|---|---|---|
| `CW-HYDRO-001` | 444 bytes; `88d7b5ee...11b7` | 6,000; `7d4e661d...5606` | 5,824; `25de85b5...8d62` |
| `CW-DAMBREAK-001` | 441 bytes; `4c126bd7...cc05` | 6,000; `9c12e445...6e76` | 16,384; `1cf0fd17...830d` |
| `CW-ORIFICE-001` | 543 bytes; `5bb0a197...d42e` | 6,000; `21307ab2...8425` | 5,792; `5d23bd8c...2cb3` |

The full hashes are exactly those frozen by R1C/R1C1. The stable-ID swap
changes the hydro fluid root to
`2f3e78cfff1824feff82bbf048990c9f8ead25dc33b5978bfb0fc90d997efc11`.
Moving the first hydro boundary x coordinate by one micrometre changes its
root to
`e26da4150557301dd0ef0e5b10b89742dd1f41e31be68c98269e7def3f81971b`.
Neither mutation collides with its base root.

## Negative process evidence

`--r1c-negative-manifest-mismatch` exits `1` with empty stderr. Its 246-byte
stdout has SHA-256
`6eb9e4b2724ecb16d3e1f02f00d1c18bb0bd7714d2b35d0355fca68fe617e87b`
and contains:

```text
status=REJECTED
reason=FORCED_MANIFEST_ROOT_MISMATCH
simulation_created=false
trajectory_started=false
```

CTest passes all `6/6` registered contact, environment, manifest and negative
manifest tests in both build directories.

## Decision

R1C1 passes and restores only conditional R1C trajectory authority. Next,
create a new ordinary full clone, apply only the tracked cold-start/diagnostic
patch, prove its exact dirty-path/build closure, then implement the 24-step
path. Do not modify either clean R1A clone and do not run R1D schedules.
