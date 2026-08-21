# NSR3-B4DR1 -- reproducible external-reference generator contract

Status: `FROZEN / R1C_FAIL / R1C3_PASS / R1C4_TRAJECTORY_RECLOSURE / NEW_ROOT_ONLY`

Identity projection:

```text
nextengine.nonlocal.nsr3b4dr1-reference-generator|v1|upstream=eccce86155776f6ac52d5080b1f720a52bf29450|adapter=engine-owned-standalone|format=CWREFV2|compiler=gcc-15.2.0|float=binary64,avx-off,fma-off,nearest,ftz-off|solver=dfsph,dt=1/240,cfl-off,omp1|artifacts=external-content-addressed|preflight=contact6+24step|full=3-process-independent|credit=new-root-only
```

Identity SHA-256:
`ade621f889a08fd26ca713592625c50316b893af8c4f1797d25f4e4d4c96b86a`.

## Scope and non-inheritance

B4DR1 builds a new external DFSPH aggregate comparator. It does not reproduce,
supersede or invalidate W0I/W1. No historical adapter, binary, payload,
scenario, corpus or attestation root transfers. `CWREFV2` and every profile
root are new.

This contract authorizes only research tooling. No upstream source, external
binary, generated trajectory, dataset, build tree or log is committed to
NextEngine. The adapter and reproducibility manifests are not runtime code and
cannot become a public solver/plugin ABI.

## R1A -- external bootstrap

Clone the official repository recursively into a newly created external
directory at commit
`eccce86155776f6ac52d5080b1f720a52bf29450`. Record:

- exact repository `HEAD`, clean status and recursive submodule status;
- upstream tree/archive and license hashes;
- license identities for linked external dependencies;
- `g++` executable SHA-256/version, target triple and configured C++ standard;
- CMake executable SHA-256/version and selected generator;
- Release, `USE_DOUBLE_PRECISION=ON`, `USE_AVX=OFF` and the minimal non-GUI
  target needed by the standalone adapter;
- compile and link command hashes, linked-library list and adapter ABI facts.

The strict candidate additionally requires `-ffp-contract=off`,
`-fno-fast-math`, round-to-nearest, FTZ/DAZ disabled, `OMP_NUM_THREADS=1`,
`OMP_DYNAMIC=FALSE` and locale `C`. If upstream or a dependency defeats one of
these controls, stop and redesign; do not silently weaken the profile.

R1A may download/build only after this contract is committed. It cannot add an
upstream remote to NextEngine or modify the engine workspace.

R1A closure records upstream-library compile/link closure. Adapter-specific
compile/link commands, linked-library list and ABI facts are necessarily R1B
exit evidence because R1B requires the adapter source to be frozen before it
can be compiled. This stage ordering does not waive those facts.

## R1B -- adapter/contact gate

Freeze adapter source before executable contact work. The adapter must remain
standalone and independently implement:

1. outer-face contact from an interior start;
2. high-speed outer crossing;
3. aperture pass with radius-safe clearance;
4. exact aperture-edge graze;
5. edge impact against the capsule feature;
6. corner impact against the sphere feature.

It uses a fixed eight-hit internal-contact schedule, stable feature order and
the already-touching inward `t=0` rule. Each result must remain finite, inside
`25 mm` clearance and free of an outside-opening wall chord. A mutated feature
order and one altered clearance value must change the adapter/profile root.

## R1C -- 24-step trajectory gate

The first
[R1C contract](03b4dr1c-trajectory-preflight-contract.md) is rejected because
its shortened dam ID is inconsistent with its own fluid/boundary roots. The
[R1C1 reclosure](03b4dr1c1-manifest-identity-reclosure-contract.md) changes
only that identity and derived manifest root. Its manifest-only preflight must
pass and receive dated evidence before the first solver object or trajectory
is authorized.

Freeze three scenario manifests before execution. Common requirements are
binary64 DFSPH, `dt=1/240 s`, CFL/warm starts/viscosity/surface/vorticity off,
6,000 samples, `0.000125 m^3` and `0.125 kg` per sample, Akinci pseudo-volumes,
stable sample-ID serialization and fail-stop behavior.

Hydro and dam use two-layer support-complete outer boundaries. Orifice uses
source-side two-layer support plus the analytical swept internal contact.
Each scenario runs 24 steps twice in separate fresh processes. The complete
`CWREFV2` preflight files must be byte-identical per scenario; all pressure and
divergence solves finish, all values are finite, outer/internal clearance is
exact and the orifice transition validator sees no solid chord.

The first Hydro process failed at pressure convergence and published no
payload. R1C therefore does not pass. The
[R1C2 observability contract](03b4dr1c2-failure-observability-contract.md)
permits one diagnostic Hydro process without changing physics. It does not
authorize a retry under the failed identity or either later scenario.

R1C2 confirms a step-1 pressure cap hit at 100 iterations while divergence
and timestep remain exact. The
[R1C3 cap-sweep contract](03b4dr1c3-pressure-cap-sweep-contract.md) authorizes
only fixed one-step diagnostic caps before any reference-profile reclosure.

R1C3 first converges at iteration 220 under cap 300. The
[R1C4 reclosure](03b4dr1c4-pressure-cap-reclosure-contract.md) changes only
pressure maximum and profile identity, then repeats the paired short gate.

## R1D -- full external generation

Only R1C PASS authorizes full generation. Frozen output schedules remain
hydro `0..1200/24` and dam/orifice `0..720/4`. Each scenario runs twice from a
fresh process. The three scenarios may occupy at most three concurrent
one-thread processes; their report order is hydro, dam, orifice regardless of
completion order.

Every same-scenario pair must be byte-identical. The profile manifest records
complete file size/hash and aggregate q99-x/q99-y/receiver-count roots. Full
payloads remain external under:

```text
<explicit-artifact-root>/<profile-root>/<scenario-id>/<payload-sha256>.cwrefv2
```

No unverified copy, symlink target or `/tmp` staging path receives credit.

## R1E -- new attestation and exit

After R1D, freeze a new reader/profile contract containing actual source,
build, binary, scenario and payload hashes. The reader repeats B4D's capacity,
format, full-hash and in-memory mutation gates over `CWREFV2` and runs twice
byte-identically.

R1E PASS selects only `NEW_EXTERNAL_DFSPH_REFERENCE_CANDIDATE` and authorizes
B4E nominal-corpus contract design. Any earlier failure preserves B4D/B4E as
blocked. It cannot issue old W1 credit, runtime authority, a public schema or a
production claim.
