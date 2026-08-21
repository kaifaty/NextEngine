# NSR3-B4DR1C4 -- pressure-cap trajectory reclosure contract

Status: `FAIL / HYDRO_DAM_PASS / ORIFICE_DOMAIN_EXTENT_REJECTED`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4dr1c4-pressure-cap-reclosure|v1|parent=865570e18864ec55cdbbbbad8b9cfa3f200a085087ecf272366c342144488927|diagnostic=926e594fedec03e9c3b08aa76fc57a22988049e97f60c5e47113387aae879678|change=pressure-max:100->300|observed=hydro-step1:iterations220,error:0x3fb965727028bcc7,threshold:0x3fb999999999999a|volume=.000125|mass=.125|all-else=r1c1|runs=2-fresh-byte-exact-per-scenario|order=hydro,dam,orifice;stop-first-failure|credit=new-root-only
```

Identity SHA-256:
`7490aa5390296c5f7fc29a56f7039458ced1969ddee555bfe0ea07f50d49be41`.

## Correction boundary

R1C4 changes exactly:

- pressure `MAX_ITERATIONS` from 100 to 300;
- trajectory report schema/contract identity;
- the UTF-8 profile identity prepended to every successful `CWREFV2`
  scenario manifest and therefore the new payload roots.

Pressure minimum 2, error `0.01%`, divergence `1..100 @ 0.1%`, cold starts,
volume, mass, density, kernel, timestep, gravity, boundary geometry, contact,
stable IDs, frame layout and publication rules are inherited unchanged from
R1C1. R1C's failed report and missing payload remain negative evidence.

## Implementation

Add a distinct CLI:

```text
--r1c4-trajectory \
  <CW-HYDRO-001|CW-DAMBREAK-001|CW-ORIFICE-001> \
  <absolute-empty-output-directory>
```

The mode must select cap 300 and the R1C4 identity without accepting a caller-
provided cap. Every nonzero frame requires pressure iterations `2..300` and
convergence true; all other validators are unchanged. Existing R1B, R1C1 and
R1C2/R1C3 nonphysical reports remain exact.

## Execution and exit

Run two fresh Hydro processes, compare report and payload bytes, then do the
same for Dam and Orifice in that order. Each process gets a new external empty
directory. Stop on the first nonzero exit, output mismatch or validator
failure. Record binary, report and payload identities plus timing.

R1C4 passes only if all six processes pass and each same-scenario pair is
byte-identical. PASS authorizes only R1D full-generation execution under this
new cap-300 profile. R1E, B4E, runtime and production remain blocked.

Hydro and Dam pairs pass byte-identically, but the first Orifice process
rejects because implementation derives analytical `x_max=1.0` from source-
support `boundary_nx=20`. See the
[dated evidence](../../development/nonlocal-nsr3b4dr1c4-trajectory-evidence-2026-08-21.md).
R1C4 therefore fails overall; only R1C5 may correct the ownership split.
