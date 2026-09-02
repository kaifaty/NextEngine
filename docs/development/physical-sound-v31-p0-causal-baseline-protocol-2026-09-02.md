# Physical sound V31 P0 — causal baseline protocol

| Field | Value |
| --- | --- |
| Date frozen | `2026-09-02` |
| Status | `FROZEN_BEFORE_SOLVER_VALUES / SYNTHETIC_ONLY / ZERO_SIGNAL` |
| Roadmap package | V31 `P0` |
| Profile | [`physical-sound-v31-p0-causal-baseline.v1.json`](../../lab/profiles/physical-sound-v31-p0-causal-baseline.v1.json) |
| Product effect | None; external research only, with authored clips authoritative |

## Question and bounded claim

Can the next classical owner expose the correct causal response to isolated
changes in elastic material, geometry, excitation, contact and support before
any solver output, audio, model or real material value is opened?

P0 freezes an executable contract for that question. It can claim only that
the contract is complete, deterministic and internally consistent. It does not
implement the P1 solver and cannot claim waveform quality, real Steel/Glass/
Wood identity, validator qualification, admission, cooking or runtime use.

## Owner boundary

The owner is an external CPU reference process. Its inputs are synthetic
geometry, an acoustic elastic material, contact, pickup, support and normal
impulse. Its future P1 output is a canonically sorted modal field and bounded
PCM render. It never reads raw PhysX callbacks and does not reuse collision
friction or restitution as acoustic properties.

All scalar inputs use SI units and finite IEEE-754 binary64 values. The
reference path is single-threaded, has no randomness and uses nearest-even
rounding. Modes sort by `(frequency_hz, family_index_a, family_index_b)`;
canonical JSON is UTF-8, key-sorted, two-space-indented and LF-terminated.
P1 rendering is mono `48 kHz` with at most `144000` frames.

The frozen analytic controls are:

```text
Kirchhoff-Love simply supported plate:
  D = E h^3 / (12 (1 - nu^2))
  omega_mn = pi^2 sqrt(D / (rho h)) ((m/a)^2 + (n/b)^2)
  phi_mn(u,v) = sin(m pi u) sin(n pi v)

Euler-Bernoulli rectangular beam:
  omega_n = beta_n^2 / L^2 sqrt(E I / (rho A))
  A = width height; I = width height^3 / 12
  beta_1(cantilever) = 1.875104068711961
  beta_1(simply-supported) = pi
```

These formulas inherit the successful V24 T0 analytic controls, but V31 adds
support, contact-node and whole-owner fallback requirements. P1 may reuse the
V24 implementation only after conforming to this profile.

## Frozen modal and render output

Each fixture retains exactly the ten lowest positive modes. Plate candidates
enumerate positive `(m,n)` pairs through `1..10` on both axes, sort by the
numeric profile and take ten. The beam uses the profile's first ten frozen
positive cantilever roots; the simply-supported counterfactual uses
`beta_n = n*pi` without a numerical root finder.

Every modal record contains:

```text
ordinal, family_index_a, family_index_b,
frequency_hz, decay_per_second,
contact_participation, pickup_participation, signed_gain
```

Plate participation is the declared `phi_mn`. Beam mode shapes use the V24 T0
cantilever normalization (maximum absolute displacement one on `u in [0,1]`);
the simply-supported beam uses `sin(n*pi*u)`. `signed_gain` is contact times
pickup participation. `decay_per_second` is the fixture's declared synthetic
loss rate; no audio fit participates.

The mono reference render is:

```text
y(t) = 0.01 * impulse_Ns * sum_i(
  signed_gain_i * exp(-decay_i*t) * sin(2*pi*frequency_i*t)
)
```

It is stored as little-endian float32 WAV (`WAVE_FORMAT_IEEE_FLOAT`) at the
frozen sample rate/frame count. The scale `0.01 per N*s` is common to both
fixtures and all interventions. Per-mode, per-contact and per-render peak
normalization is forbidden. P1 rejects non-finite samples, clipping, growth
after an unforced modal onset or any result outside the frozen envelope.

## Frozen fixtures and isolated interventions

The profile declares one non-real rectangular plate and one non-real
rectangular beam. Each has ten modes and a nested `17 x 13` / `33 x 25` remesh pair. The
material ID is deliberately `synthetic-elastic-reference`, not a product
material name.

Exactly one causal axis changes in each intervention:

| Axis | Frozen mutation | Expected invariant and response |
| --- | --- | --- |
| Young's modulus | `E x 4` | contact/support unchanged; every bending frequency `x 2` |
| Density | `rho x 4` | geometry/contact/support unchanged; frequency `x 0.5` |
| Thickness | `h x 2` | material/in-plane geometry unchanged; frequency `x 2` |
| Uniform scale | all lengths `x 2` as one scale coordinate | material and normalized contact unchanged; frequency `x 0.5` |
| Impulse | normal impulse `x 2` | frequencies unchanged; linear modal amplitude `x 2` |
| Contact | plate contact moves to `u=.5,v=.5` | frequencies unchanged; `(m=2,n=1)` participation is exactly zero |
| Support | beam cantilever becomes simply supported | material/geometry/contact unchanged; first frequency ratio `(pi/beta_1)^2 = 2.8070425317861318...` |

No test may combine axes, select a favorable mode after values, normalize each
render separately or infer a material label from a frequency match.

## Resources and typed fallback

One P1 run is bounded to 64 modes, 100,000 mesh vertices, 200,000 elements,
144,000 render frames, 256 MiB output, 1 GiB peak RSS and 300 seconds wall
time. It uses no network, GPU, optimizer, model or runtime dependency.

Unsupported geometry, acoustic material, support, contact, numeric value or
resource request returns `FallbackOutOfDomain` with one frozen reason code and
publishes no partial modal or PCM artifact. The selected authored clip remains
the successful product result. Contract corruption returns `ContractReject`,
also without partial publication.

## P0 gates

P0 passes only when the owning CLI:

1. strictly validates every profile field, fixture, formula, limit and required
   intervention;
2. independently recomputes all expected ratios and the plate nodal rule;
3. proves each intervention changes exactly one declared causal coordinate;
4. writes two byte-identical contract/report sets to fresh external paths;
5. records zero network, audio, signal and model access;
6. rejects duplicate/unknown fields, non-finite values, profile drift, unsafe
   outputs and missing fallback semantics.

A pass authorizes only P1 implementation. It grants no ProductCheck, real
material, perceptual, validator, release or runtime credit.

## Stop and non-retry rules

- No real audio, protected payload, generated waveform or checkpoint opens in P0.
- Current physics materials are not silently promoted to acoustic materials.
- No raw backend callback or ECS mutation path is introduced.
- Failure revises the protocol before P1 values; it does not tune a threshold
  after observing a solver result.
- Generated artifacts stay outside Git. Git contains only the protocol,
  profile, owner and compact tests.
