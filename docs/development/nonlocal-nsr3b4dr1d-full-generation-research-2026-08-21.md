# NSR3-B4DR1D full-generation research -- 2026-08-21

Status: `COMPLETE / SCHEDULE_RECLOSURE_SELECTED / IMPLEMENTATION_NEXT`

## Question

What is the smallest reproducible full-generation extension of the accepted
R1C5 trajectory path, and how can it use more host capacity without changing
the deterministic one-thread solver profile?

## Findings

R1D is not merely a larger loop bound. The R1C scenario blocks normatively say
`steps=24` and `outputs=0..24/every=1`; prepending an R1D profile identity to
those unchanged blocks would create a reproducible but self-contradictory
payload. R1D therefore needs new scenario blocks whose only semantic changes
are the schedule and scenario-domain label. Fluid positions/IDs, boundary
positions, mass/volume, solver, contact and corrected domain ownership remain
unchanged.

The frozen schedules produce 51 Hydro frames and 181 Dam/Orifice frames. With
the unchanged 312,156-byte frame layout, expected complete payload sizes are
15,920,965, 56,501,239 and 56,501,341 bytes. The densest payload remains
10,607,523 bytes below the 64 MiB hard cap.

One upstream process must remain `OMP_NUM_THREADS=1`; changing its internal
thread reduction would create a different floating-point profile. The three
independent scenarios can instead run concurrently. Execute one Hydro, Dam and
Orifice process in the first wave and their fresh repeats in the second wave,
never exceeding the parent limit of three processes. This improves machine
utilization without changing any per-file reduction or serialization order.

The content hash is unknown until the complete payload exists. Each process
therefore writes to a distinct fresh directory under the explicit external
artifact root. Only after same-scenario byte equality may one verified regular
file be atomically published at the content-addressed path. `/tmp`, symlinks
and an unverified copy receive no credit.

## Aggregate design

For every serialized frame, calculate nearest-rank q99 independently over all
6,000 raw binary64 x and y positions. Rank is
`ceil(99 * sample_count / 100)` with one-based indexing. Preserve the selected
value's bits rather than decimal-rounding it.

Three domain-separated binary streams record q99-x, q99-y and receiver count.
Each begins with its ASCII domain plus NUL, then little-endian sample/frame
counts, then ordered `(step, value)` records. Their SHA-256 roots enter the
canonical process report. This is a compact aggregate identity, not a
replacement for complete payload hashing.

## Selection

Freeze a distinct R1D profile and scenario-manifest identity. Add a zero-
Simulation manifest/capacity preflight plus a forced schedule mismatch. Then
add `--r1d-generate` by parameterizing the accepted R1C5 loop with total steps
and output stride. Count convergence/contact diagnostics across every solver
step but serialize only schedule frames.

Run the full pairs only after source review, focused tests and a committed
implementation. R1D PASS may authorize R1E design, but cannot itself authorize
R1E execution, B4E, runtime integration or production claims.
