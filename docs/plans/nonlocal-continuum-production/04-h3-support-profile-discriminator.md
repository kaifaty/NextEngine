# NPR0-R1B — h3 support-profile discriminator

Status: `SPECIFIED / IMPLEMENTATION_NEXT / REPORT_ONLY`

## Purpose

This bounded discriminator decides whether the only passing NPR0-R1 point can
become one exact product-profile candidate. It does not amend SPEC-38, grant
runtime authority, start NPR1 physics validation or claim production
performance.

The h3 point was selected by the frozen remediation matrix rather than visual
tuning. Every value below is fixed before the v4 profile executes.

## Frozen v4 identity

| Field | Value |
| --- | ---: |
| Profile | `nuv-basin-48k-static-support-h3-physical.v4` |
| Fluid lattice | `80*15*40 = 48,000` |
| Spacing / radius | `0.05 m / 0.025 m` |
| Mass / rest density | `0.125 kg / 1000 kg/m3` |
| Cadence / gravity | `1/240 s / 9.81 m/s2 toward -Y` |
| Horizon | `0.15 m = 3dx` |
| kappa / lambda / mu / gamma | `9196.875 / 360 / 0 / 0` |
| Fixed iterations | `16` |
| Boundary | exact three-layer rooted lattice complement plus separate swept-sphere contact |
| Static boundary samples / capacity | `38,856 / 38,856` |
| Total solver participants | `86,856` |
| Maximum neighbors | `123` |
| Maximum directed pairs | `10,683,288` |
| Global neighbor ID | checked `u32`; `u16` is ineligible |

The fluid hard capacity remains 50,000 and is not charged with rooted static
support. The larger static capacity is a versioned research candidate above
SPEC-38's current 32,768 value. No truncation, support thinning or hidden
capacity borrowing is allowed.

## Ordered gates

### V4-A profile audit

The canonical profile record and SHA-256 must expose every frozen field,
explicitly identify both deviations from SPEC-38 (`h=3dx` and static capacity
38,856), and retain `runtime_authority=false` and `npr1_authorized=false`.

### V4-B full-basin exact preflight

Run P1 and retained P2 twice at the profile's 16 iterations. Require:

- exact 48,000 fluid, 38,856 fixed and 86,856 total counts;
- checked u32 neighbor fallback and no u16 truncation;
- maximum degree at most 123 and directed pairs at most 10,683,288;
- P1/P2 correspondence within the profile tolerances;
- finite density, source, local matrices, positions and velocities;
- no local solve, capacity, CSR symmetry or repeatability failure;
- identical repeated output and CSR roots.

No wall-clock value participates in this gate and the old 3.2 ms result is
not inherited.

### V4-C complete tiny-corpus rerun

Run all frozen NPR0-E cases with h3-consistent execution:

- TPF-1 uses 16 SISSM iterations; the isolated free-fall recurrence and gates
  remain unchanged;
- TPH-1 uses h3, three rooted layers, 16 iterations and all unchanged gates;
- TPR-1 uses h3 and 16 iterations with the unchanged rigid-mode gates;
- TPW-1 uses an exact three-layer complement and separate swept contact, with
  unchanged feature, penetration, reaction and fixed-support gates.

The profile passes only if every case passes. Positive-compression remains the
frozen NPR0-E hydro metric; its one-sided limitation is carried into NPR1 and
is not silently reinterpreted as a long-horizon density proof.

## Disposition

1. If V4-A, both repeated V4-B checks and V4-C pass, emit
   `NONLOCAL_PRODUCT_PROFILE_CANDIDATE` with the profile, audit, P1/P2,
   tiny-corpus and CSR roots. NPR1 becomes eligible; runtime remains blocked.
2. Any physics, exactness, repeatability or declared-capacity failure emits
   `NONLOCAL_PRODUCTION_RESEARCH_STOP`. This is the second bounded remediation
   boundary from the NPR0 contract; no further coefficient/support sweep is
   authorized in this lineage.
3. A machine/environment failure that prevents execution is reported as an
   execution blocker and does not become physics evidence.

## Non-regression

The v0-v3 profiles, old hashes, retained P1/P2 implementation, split contact,
negative P3/P4 evidence and DFSPH production authority remain unchanged.
