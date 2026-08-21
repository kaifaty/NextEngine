# Nonlocal external-reference adapter

This is a standalone research tool for the B4DR1 new-root external comparator.
It is not runtime code, a public ABI or a solver plugin. The default mode is
the closed R1B contact gate; the R1C manifest-only mode creates no physical
world and is the prerequisite for the separately gated trajectory path.

The tool independently implements the frozen analytical hard-contact vectors
and validator. It links the exact external SPlisHSPlasH R1A static closure only
to attest the real binary64 ABI and DFSPH method anchor. It does not call a
solver step or initialize particles.

As required by SPlisHSPlasH's standalone embedding model, `main.cpp` owns the
single process-wide logging, timing and counting stores. Those definitions
only close the static-library link boundary; the adapter does not install a
log sink or call any timing/counting API.

Configure a fresh out-of-tree build with the same GCC profile used by R1A:

```text
cmake -S <nextengine>/crates/continuum-water/tools/nonlocal-reference-adapter \
  -B <external-build>/contact-r1b -G Ninja \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_CXX_COMPILER=/usr/bin/g++ \
  -DSPLISHSPLASH_SOURCE_DIR=<external-source> \
  -DSPLISHSPLASH_BUILD_DIR=<external-splish-build>
cmake --build <external-build>/contact-r1b --parallel
ctest --test-dir <external-build>/contact-r1b --output-on-failure
```

The source/build inputs must already pass the R1A hashes. Execute with:

```text
LC_ALL=C OMP_NUM_THREADS=1 OMP_DYNAMIC=FALSE \
  <external-build>/contact-r1b/nonlocal_reference_adapter
```

`--negative-rounding` and `--negative-ftz` are fail-closed evidence modes.
They must exit nonzero before contact starts. Unknown arguments are rejected.
`--r1c-manifest-preflight` independently regenerates the frozen R1C scenario,
fluid and boundary roots without creating a SPlisHSPlasH model or time step.
`--r1c-negative-manifest-mismatch` must reject with
`simulation_created=false` and `trajectory_started=false`. Neither manifest
mode writes files.

After R1C1 manifest evidence passes, the patched full-clone build additionally
supports:

```text
nonlocal_reference_adapter --r1c-trajectory \
  <CW-HYDRO-001|CW-DAMBREAK-001|CW-ORIFICE-001> \
  <absolute-empty-output-directory>
```

This mode writes exactly one atomically published 25-frame `CWREFV2` research
payload. It is valid only when linked against the frozen cold-start/diagnostic
patch. Missing, relative, nonempty, non-directory or symlink output directories
reject before Simulation creation. Full R1D schedules are not implemented by
this mode.

After the first R1C Hydro pressure failure, R1C2 adds only:

```text
nonlocal_reference_adapter --r1c-diagnose-trajectory \
  CW-HYDRO-001 <absolute-empty-output-directory>
```

It executes the same trajectory path but emits exact failure step/phase,
iteration, residual-bit, convergence and time-step-bit fields under a distinct
diagnostic-only identity. It does not tune the solver or authorize R1C/R1D.

R1C3's report-only first-step sweep uses:

```text
nonlocal_reference_adapter --r1c3-pressure-cap \
  <25|50|75|100|125|150|200|300> \
  <absolute-empty-output-directory>
```

It changes only the pressure iteration cap, executes one Hydro step and writes
no payload. The external directory is still required as a fail-closed sentinel.
