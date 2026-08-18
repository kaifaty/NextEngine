# W0I — External reference geometry attestation

Status: `REFERENCE_GEOMETRY_ATTESTATION_FROZEN / W1_AUTHORIZED / RESEARCH_ONLY`.

## Decision

Keep the W0F geometry/contact formulation, W0G energy semantics and W0H
pressure solver unchanged. Reclose only the independent-reference profile:
an external trajectory is admissible for W1 credit only when it satisfies the
same hard particle-clearance geometry as the candidate and its complete
`CWREFV1` payload SHA-256 is frozen by this profile.

The earlier SPlisHSPlasH aggregate files are rejected as W1 evidence. Their
boundary pressure was allowed to move particle centres through the analytical
wall. Dam-break first exceeds the `2.5 mm` canonical penetration allowance at
step 4, first escapes the box at step 28 and reaches `30.365 mm` equivalent
penetration at step 188. Hydro first exceeds the allowance at step 24, first
escapes at step 936 and reaches `25.687 mm` at step 984. Aggregate curves from
those states cannot be a geometry oracle.

This reclosure does not widen the frozen `5%` RMSE or `10%` maximum curve
thresholds. It does not select SPlisHSPlasH particle state, density, pressure,
convergence or contact as Next Engine authority.

## Independent generator profile

The generator remains an external build of
[SPlisHSPlasH](https://github.com/InteractiveComputerGraphics/SPlisHSPlasH)
commit `eccce86155776f6ac52d5080b1f720a52bf29450`. No external source or large
trajectory is copied or linked into Next Engine.

Common settings are:

- Release, binary64, AVX disabled and `OMP_NUM_THREADS=1`;
- fixed `1/240 s`, CFL disabled, DFSPH, cold pressure/divergence solves and all
  optional non-pressure models disabled;
- `6,000` samples, `0.000125 m³` volume and `0.125 kg` mass per sample;
- Akinci 2012 boundary pseudo-volumes;
- an independently implemented post-pressure predictive sphere projection;
- `25 mm` outer clearance on every accepted float step;
- fail-closed outer, internal-clearance and swept aperture-crossing checks.

Hydro and dam-break use the support-complete two-layer outer lattice and the
outer-box projection. Orifice uses the two-layer support on the solid side of
the initially occupied left chamber plus an eight-hit swept projection against
the internal plane, four aperture-edge capsules and four aperture-corner
spheres. The adapter runs face, high-speed crossing, aperture pass, exact edge
graze, edge-impact and corner-impact self-tests before an internal-geometry
trajectory.

The one-sided orifice support is an aggregate comparator profile, not a
general two-sided wall model. Symmetric four-layer support was rejected before
the full run because it reconstructed up to `3.43 × rho0`, reached `72 m/s`
and produced a canonical chord through solid. Surface and native-regular
Akinci variants remained geometry-safe but reached approximately
`2.7 × rho0` and `106 m/s`, repeatedly exhausting both DFSPH solves. The
selected source-side profile remains bounded and its transfer curve is the
only external quantity consumed by W1.

## Provenance and exact outputs

| Input or artifact | SHA-256 |
|---|---|
| tracked upstream adaptation diff | `4effa812553649c89135ca9515aaac410e47eca1fe250890766c0d6183165a4b` |
| external comparator source | `f87a598a03b1893646de8188c393b41c262fb335e4f980a3672ea099d8461c11` |
| external comparator binary | `ee0e12ea5ef6afc6090a6404a259d75769bae28448119edc9764a96bb705d0aa` |
| `CW-HYDRO-001` reference | `84ae867f5b336cd0bd51be6f29a6a2a1f27f702c424f1dbd0a8f735b9f4bb435` |
| `CW-DAMBREAK-001` reference | `853d965489a40082a024aeee5a19f98aef054417014af8556fa212687d88d12c` |
| `CW-ORIFICE-001` reference | `60e9b3538d621ef1a3f1ae77569640740df471fbe4e5ef8eaa813d3751930849` |

Each complete external scenario was generated twice with the final binary and
matched byte for byte. The three files remain outside Git. The canonical W1
reference-attestation projection root is
`186e1e31c0aa2636525bbc54e4fe4335b8432e7e99eddf3221d08e0380b65c90`.

## Runner contract

`xtask continuum water run-w1-linux` publishes the W0I root, expected file
hash, observed file hash and attestation result. In normal `frozen-successor`
mode a required reference with any other SHA-256 fails before the candidate
trajectory starts with `WATER_REFERENCE_CORPUS_MISMATCH`.

The explicit `diagnostic-frozen-observe-energy` mode may import another
well-formed reference for bounded research. It labels a matching curve
`PASS_UNATTESTED_RESEARCH_ONLY`, keeps external completion false and cannot
issue W1 or ProductCheck credit.

The report also publishes the output step and both normalized curve values at
the maximum absolute error. This is diagnostic provenance only; it does not
change either reduction or threshold.

## Closure evidence

Against the geometry-attested files, unchanged W0H produces:

| Metric | RMSE | Maximum error | Frozen limits |
|---|---:|---:|---:|
| dam-break front | `0.3156528%` | `0.8418750%` at step 144 | `5% / 10%` |
| dam-break height | `2.3295631%` | `6.4657000%` at step 488 | `5% / 10%` |
| orifice transfer | `0.1624230%` | `0.3000000%` at step 556 | `5% / 10%` |

Hydro retains zero canonical penetration and `0.3377402%` maximum absolute
energy drift under W0H. Its external file is still mandatory provenance, but
the frozen comparator intentionally has no hydro aggregate curve entry; the
scenario's analytical centre-of-mass, energy, density and clearance rules
remain blocking.

Detailed counterexamples, generator diagnostics and report hashes are in the
[W1 hard-clearance reference evidence](../../development/continuum-water-w1-hard-clearance-reference-reclosure-2026-08-18.md).

## Downstream closure

W0I itself only authorizes the evidence profile; it does not manufacture W1
credit. Downstream W1 closed at clean commit
`e00999e96f0f55ae02426e806457f625d0a4844f`: two complete Linux runs used the
three exact external files, passed every W0F/W0G/W0H/W0I preflight and metric,
and reproduced corpus root
`d38d6bc8a8e98e87402202a926685dbe4867e3de6a7d8679362885be46e96835`.
Their reports are identical after removing only diagnostic wall-clock fields,
so W1 issues `CONTINUUM-WATER-REF-P1=PASS / LINUX_W1_PASS`.

Windows remains outside the current W1 scope by user decision and is deferred,
not waived, for production promotion. W1 supplies no W2 performance, PhysX,
persistence, runtime or shipping evidence.
