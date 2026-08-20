# NPR0 — Nonlocal product-profile reclosure contract

Status: `COMPLETE / NONLOCAL_PRODUCT_PROFILE_CANDIDATE / REPORT_ONLY`

## Purpose

NPR0 determines whether the retained Nonlocal equations have a defensible
profile at the scale, cadence, support and geometry of the SPEC-38 basin. It
does not calibrate by appearance, validate full fluid physics, change the
retained P1+P2 arithmetic or authorize integration.

## Frozen inputs

- Product geometry: sealed `4×2×1 m` basin, water depth `0.75 m`, nominal
  lattice `80×15×40 = 48,000`, hard capacity `50,000`, `+Y` up.
- Product sampling: `dx=0.05 m`, mass `0.125 kg`, rest density `1000 kg/m³`.
- Product cadence/support inherited from SPEC-38: `dt=1/240 s`, `h=0.1 m`.
- Source-shaped control: `dx=0.005 m`, mass `0.000125 kg`, `dt=0.001 s`,
  `h=3dx`, `κ=1`, `λ=1.5`, `μ=0`, `γ=0`, five fixed iterations.
- Retained GPU execution: directed gather, pointer swap, specialized terms,
  stable samples, fused owner terms and checked compact CSR.

Changing any frozen input creates a new profile version and cannot inherit an
output root or performance result.

## Bridge ladder

Each step changes one named profile axis. All three initial profiles remain
boundary-free diagnostics; the sealed boundary is a later NPR0 step.

| Identity | Changes from predecessor | Purpose |
|---|---|---|
| `nuv-basin-48k-source-scale.v2` | product lattice orientation, `dx=0.05`, mass `0.125`, `h=0.15`; keeps `dt=0.001` and source coefficients | expose geometric/mass scale behavior while preserving `h/dx=3` |
| `nuv-basin-48k-cadence.v2` | `dt=1/240` | isolate the product cadence |
| `nuv-basin-48k-spec-support.v2` | `h=0.1 = 2dx` | isolate the SPEC-38 support ratio |
| future `nuv-basin-48k-sealed.v2` | exact basin boundary/support/contact | first product-geometry candidate; requires a separate boundary spec |

Unchanged `κ/λ/μ/γ` in the first three identities is a counterfactual control,
not a material calibration. A selected coefficient mapping requires its units,
dimensionless groups, source provenance and physical-quality discriminator to
be written before implementation.

## NPR0-A machine-audit gate

The quarantined tool adds `--production-profile-audit` and must report:

- canonical JSON and SHA-256 for the retained 48k control and all bridge
  profiles;
- exact sample count, lattice axes, origin, spacing, mass, horizon, `h/dx`,
  time step and boundary identity;
- explicit mismatches against the product profile;
- whether coefficients, sealed boundary, canonical publication and physical
  corpus have been selected or remain open.

The command succeeds when the audit is internally consistent. Its semantic
status remains `PROFILE_RECLOSURE_REQUIRED`; a successful command is not a
passing physics gate.

## NPR0-B executable preflight

For each bridge identity, in order:

1. validate profile hash and capacities;
2. run the retained P1/P2 exact correspondence check twice;
3. require finite density/source/matrix/position/velocity and no local solve
   failure;
4. require symmetric current-horizon CSR, exact repeated output/CSR and
   normalized momentum residual within the profile bound;
5. stop at the first failed bridge and record the exact field/iteration.

No long timing run is admitted. If a bridge fails, one bounded diagnosis may
separate float range, time-step stiffness, support deficiency and capacity.

## NPR0-C scale-law and boundary decision

Before any coefficient change or sealed-boundary code, record falsifiable
hypotheses for:

- dimensional scaling of every energy/source/matrix term;
- product cadence stability at fixed versus derived coefficients;
- the effect of changing `h/dx` from `3` to `2`;
- ghost-particle versus analytical/SDF boundary ownership, support and contact;
- the canonical fixed-point publication error at micrometre units.

The minimal physical controls are hydrostatic rest, exact free fall, one
reversible perturbation and a small wall-contact case. They use an independent
CPU implementation and predeclared normalized metrics. Visual similarity is
diagnostic only.

The algebraic scale-law discriminator is complete. Its derived coefficients
remain a hypothesis until the physical controls pass. The sealed-boundary
step must treat two-layer density support and non-penetration as distinct
operations; a fixed ghost shell by itself cannot satisfy the boundary gate.

NPR0-D selected that split and is specified in the
[static-boundary contract](01-static-boundary-discriminator.md). Both full
support profiles pass exact GPU execution preflight, and the negative wall
fixture confirms that fixed ghosts do not provide contact. The first tiny
corpus rejected both v3 profiles. Its frozen hydro remediation rejects h2
through 50 iterations and admits only an h3 support-ratio candidate. Because
exact three-layer full-basin support exceeds the current static-boundary
capacity, a named v4 capacity/profile discriminator and complete NPR0-E rerun
are required. That discriminator now passes audit, repeated exact P1/P2
preflights and all four tiny cases. NPR0 selects the exact v4 h3/16 identity;
NPR1 is eligible while runtime remains blocked.

## Exit and rollback

NPR0 selects `NONLOCAL_PRODUCT_PROFILE_CANDIDATE` only when one exact profile
passes the audit, executable preflight, scale-law controls and sealed-boundary
tiny corpus. Otherwise it remains `PROFILE_RECLOSURE_REQUIRED` or closes
`NONLOCAL_PRODUCTION_RESEARCH_STOP` after two coherent remediation cycles.

Rollback removes only v2 bridge profiles/commands. The completed performance
profiles, P1+P2 implementation, hashes and negative P3/P4 evidence remain
unchanged and selectable.

## Explicit exclusions

PhysX coupling, public contracts, runtime ownership, save/replay, adaptive
iterations, warm start, ML, split/merge, active domains, terrain and Windows.
