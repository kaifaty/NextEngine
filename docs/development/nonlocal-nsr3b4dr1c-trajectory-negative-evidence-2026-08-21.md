# NSR3-B4DR1C trajectory negative evidence -- 2026-08-21

Status: `FAIL / PRESSURE_NOT_CONVERGED / R1D_BLOCKED`

## Result

The first authorized physical process, `CW-HYDRO-001`, exited nonzero with
`PRESSURE_NOT_CONVERGED`. A SPlisHSPlasH Simulation had been created and the
trajectory loop had started. No `CWREFV2` file or partial file was published,
stderr was empty, and the cost-aware gate stopped before the second Hydro
process, Dam Break or Orifice.

This is a real R1C profile failure. It is not evidence that the Nonlocal
formula is wrong: R1C is an external DFSPH comparator bootstrap, and the
failed process did not execute the engine Nonlocal solver. It also does not
yet distinguish an iteration-cap failure from another false convergence
diagnostic because the frozen failure report omitted the already available
solver fields.

## Frozen implementation and build closure

| Input | Attested value |
|---|---|
| implementation commit | `235e826afbb2dcbbcf49b29dc2a56d0599c285ca` |
| corrected manifest identity | `865570e18864ec55cdbbbbad8b9cfa3f200a085087ecf272366c342144488927` |
| upstream commit | `eccce86155776f6ac52d5080b1f720a52bf29450` |
| tracked source patch | `e89cf9befc2a08a9c15bd6b290a97b3f6815da1ea699508c41c15645f86c33bc` |
| applied Git diff | `6e54baf340651a7c3eefd3fd39c4ff209bcf9948e07cf0829a96345271aab824` |
| patched `TimeStepDFSPH.cpp` | `cb22e2c3006060c07526c813c10e9483a2dacf5073bc7cb57697fff84bfe8c7a` |
| patched `TimeStepDFSPH.h` | `06ddb8131108e0f9fd54ada44c3327d1ef548a7d47a94889e26cc592f9aa18af` |
| generated `Utilities/Version.h` | `975287f2e39757092e5d8d91551b6beaf344008eead09fd00de0b5a348d620ce` |
| executable | 1,559,008 bytes; `6570a7f1415993ec10577d19c957e54643892164886c3bbb56d3589bdcb79810` |
| ELF build ID | `8b349f96102913906b6e739b8e6598eaf1b0fc3f` |
| Ninja command stream | `c92169c1d6c6b44b256085bfa824549b445203ff7067a70d417310543600225e` |

Before adapter configure, the upstream source was an ordinary complete clone:
`.git` was a directory, no promisor configuration existed, and
`git fsck --full --no-dangling` passed. Its exact status contained only the
two patched tracked files and generated untracked `Utilities/Version.h`.
Configure pins that status, the Git diff, all three file hashes and all eight
static-library hashes. It separately rejected a clean unpatched clone, a
promisor partial clone and the retained incomplete-object copy.

The final compile commands contain the strict no-fast-math/no-AVX/no-FMA
flags. Disassembly contains no AVX-like or FMA instructions. Focused CTest is
`7/7 PASS`. The unchanged R1B and manifest reports remain exactly:

- R1B: 1,730 bytes,
  `c6a4950d45270eede2a89a3cecfb4502aa4dd801ecb203d971f312a383203a8a`;
- R1C1 manifest: 1,744 bytes,
  `6d2933283281591cc1dd259053de2b68b1f27188e945c7e753ef21323eb558f3`.

The relative-output negative exits before Simulation creation and has report
root `16b306733f29f593537923cccfa0fd4e35ad6cf597382203ecd23eda63a1d289`.

## Physical process evidence

The process used locale `C`, one static OpenMP thread and the frozen
`CW-HYDRO-001` manifest. It exited `1` after 0.46 seconds wall time with 12,328
KiB maximum resident memory. Its exact 225-byte report root is
`1d3f7c4ed3bd582b657c4180947506d31fe3982fe6427f7a445b3ecd013bc67e`:

```text
schema=nextengine.nonlocal.nsr3b4dr1c-trajectory.v1
contract_identity=865570e18864ec55cdbbbbad8b9cfa3f200a085087ecf272366c342144488927
status=FAIL
reason=PRESSURE_NOT_CONVERGED
simulation_created=true
trajectory_started=true
```

The output directory had zero entries after exit. No payload hash exists.

## Decision

R1C is `FAIL`; R1D, R1E and B4E remain blocked. Do not increase the pressure
cap, loosen tolerance, enable warm starts or run another scenario from this
result. First reclose failure observability only, preserve the same physics,
and execute one diagnostic Hydro process which exposes failing step, phase,
iteration counts, convergence booleans, residual bits and time-step bits.

