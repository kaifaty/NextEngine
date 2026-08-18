# Continuum water W1 hard-clearance reference reclosure — 2026-08-18

Status: `REFERENCE_PROFILE_SELECTED / EXACT_HASH_ATTESTATION_IMPLEMENTED / CLEAN_W1_PENDING`.

## Outcome

The frozen W0H solver is retained. The W1 dam-break mismatch was caused by an
external reference that violated the candidate's hard geometry, not by the
W0H pressure equations. An independently implemented C++ hard-contact
extension of pinned SPlisHSPlasH produces geometry-admissible dam-break and
orifice curves that unchanged W0H passes under the original `5%` RMSE and
`10%` maximum-error thresholds.

No solver coefficient, curve formula, time alignment, output interval or
threshold changed. W0F, W0G and W0H remain immutable. W0I freezes the new
external provenance/output hashes and the W1 runner rejects any other required
reference in production-credit mode.

`CONTINUUM-WATER-REF-P1` remains `NOT_RUN` in this report. All comparison runs
below used a dirty research tree and the explicit
`diagnostic-frozen-observe-energy` mode; they are architecture discriminators,
not clean corpus evidence.

## Counterexample to the old reference

The original support-complete SPlisHSPlasH files had SHA-256 values:

- hydro: `886af72d70ee3ad114bea9d5f381cf75cbb4393a51788f869c6edf8f8b64c03f`;
- dam-break: `67931a638ece5782109bb111c227f7c8a493fdffc263eaeaea381dcad5a5d6bc`.

Their raw binary64 positions were checked directly against the W0F outer-box
clearance rule.

| Scenario | First `>2.5 mm` penetration | First centre escape | Maximum equivalent penetration |
|---|---:|---:|---:|
| hydro | step 24 | step 936 | `25.687 mm` at step 984 |
| dam-break | step 4 | step 28 | `30.365 mm` at step 188 |

The old dam-break height is therefore partly a curve of particles occupying
states that Next Engine must reject. A no-contact W0H counterfactual follows
that curve more closely but also violates geometry. Every tested hard-contact
variant lowers the splash height.

The strongest geometry-safe density-barrier candidate passed hydro with
`0.6508873%` maximum energy drift, `2.5 mm` maximum canonical penetration and
47 pressure-operator applications. It still missed the invalid dam-break
height by `15.4494%`. Moving its continuation beyond the first exterior
lattice layer reduced terminal contact loss from approximately `26.35%` to
`0.068%`, yet worsened the height maximum error to `19.1356%`. This falsified
contact activation location as the cause and stopped further density-map or
barrier tuning.

## Independent hard-contact extension

The external checkout is pinned to SPlisHSPlasH commit
`eccce86155776f6ac52d5080b1f720a52bf29450`. The comparator remains outside
Next Engine and uses no Rust implementation code. It saves the pre-step
position by stable particle ID, lets SPlisHSPlasH perform its DFSPH step, then
projects the resulting velocity before replacing the terminal position with
`x_n + dt * v_projected`.

Outer contact clamps each velocity component to the exact radius-offset box.
The orifice extension independently sweeps against the internal plane, four
edge capsules and four corner spheres with a fixed eight-hit schedule. It
fails immediately if the final position violates `25 mm` clearance or if the
straight accepted transition crosses the wall outside the radius-safe opening.
The adapter also executes six fixed face/aperture contact vectors before the
orifice run.

One continuous-float edge case was found during the 24-step discriminator. A
particle resting a fraction of an ulp inside the contact plane produced a
slightly negative time of impact. Next Engine cannot retain that state because
every step republishes integer micrometres. The external guard now handles an
already touching or microscopically embedded inward state at `t=0`; clearance
validation remains unchanged.

## Boundary-profile discriminator

The external solver cannot express W0F's per-fluid-side oriented internal
density support without invasive solver changes. Four profiles were evaluated
before the full reference run.

| Profile | 24-step result | Decision |
|---|---|---|
| symmetric four-layer internal support | density up to `3.43 rho0`, speed up to `72 m/s`, unsafe final chord rejected | reject |
| surface Akinci wall | density up to about `2.64 rho0`, speed up to `106 m/s`, both solvers repeatedly capped | reject |
| native regular Akinci wall | density up to about `2.70 rho0`, speed about `38 m/s`, both solvers repeatedly capped | reject |
| source-side two-layer support | hard geometry pass, speed at most about `4.14 m/s` through step 24 | select for aggregate transfer only |

The selected orifice profile is intentionally narrow: the receiver is empty at
step zero, and W1 consumes only the chamber-count transfer curve. It does not
validate a general two-sided Akinci wall, local densities or particle
identity. The later low-density value near `318.31 kg/m³` is approximately an
isolated particle's self contribution and is recorded as a limitation of the
external comparator.

## Reproducible external artifacts

Build facts:

| Fact | Value |
|---|---|
| upstream tracked adaptation diff | `4effa812553649c89135ca9515aaac410e47eca1fe250890766c0d6183165a4b` |
| comparator source | `f87a598a03b1893646de8188c393b41c262fb335e4f980a3672ea099d8461c11` |
| comparator binary | `ee0e12ea5ef6afc6090a6404a259d75769bae28448119edc9764a96bb705d0aa` |
| compiler | GCC `15.2.0` |
| CMake profile | `Release`, `USE_DOUBLE_PRECISION=ON`, `USE_AVX=OFF` |
| execution | `OMP_NUM_THREADS=1`, fixed `1/240 s`, CFL off |

Each final file was generated twice with the final binary and matched byte for
byte.

| Scenario | Outputs | Bytes | SHA-256 |
|---|---:|---:|---|
| hydro | 51 | 7,344,252 | `84ae867f5b336cd0bd51be6f29a6a2a1f27f702c424f1dbd0a8f735b9f4bb435` |
| dam-break | 181 | 26,064,772 | `853d965489a40082a024aeee5a19f98aef054417014af8556fa212687d88d12c` |
| orifice | 181 | 26,064,772 | `60e9b3538d621ef1a3f1ae77569640740df471fbe4e5ef8eaa813d3751930849` |

The files remain outside Git at `/tmp/cwref-*-hard-contact-final.bin`. The W0I
attestation projection containing the source, binary, semantic-profile and
output hashes has root
`186e1e31c0aa2636525bbc54e4fe4335b8432e7e99eddf3221d08e0380b65c90`.

## W0H comparison

| Scenario/metric | Result | Frozen threshold |
|---|---:|---:|
| hydro maximum absolute energy drift | `3,377,402 ppb` | `<=10,000,000 ppb` |
| hydro maximum canonical penetration | `0 um` | `<=2,500 um` |
| dam-break front RMSE / maximum | `3,156,528 / 8,418,750 ppb` | `<=50,000,000 / 100,000,000 ppb` |
| dam-break height RMSE / maximum | `23,295,631 / 64,657,000 ppb` | `<=50,000,000 / 100,000,000 ppb` |
| orifice transfer RMSE / maximum | `1,624,230 / 3,000,000 ppb` | `<=50,000,000 / 100,000,000 ppb` |

The dam-break maxima occur at steps 144 and 488; the orifice maximum occurs at
step 556. The W1 report now includes those steps and candidate/reference values
without changing the reductions.

Diagnostic report SHA-256 values are:

- hydro `740f5e1c69397ca743a61a9a62859eb4ce3cbcfe85ff32616c52933534cbb6d3`;
- dam-break `8e2f041ad5dd542ce272410ecc1006f5dfdb80b0ff7ea85cd6def22bc5157b98`;
- orifice `4d5bf729f0b53a70cdf1d16aa82df4bae9ec40801e339cd37b450f1f249161f9`.

## Decision and next action

1. Retain W0H APG and sequential analytical contact unchanged.
2. Reject the old non-clearance reference files and all density/barrier solver
   branches tested against them.
3. Require exact W0I reference hashes before a `frozen-successor` trajectory;
   permit alternatives only in explicit no-credit research mode.
4. Run the complete Linux W1 corpus twice from a clean committed tree with the
   three exact references. Identical target-local roots and all blocking
   checks are required before `CONTINUUM-WATER-REF-P1 = PASS`.
5. Keep Windows outside current W1 scope and explicitly deferred for production
   promotion. Do not infer cross-target evidence from this result.

