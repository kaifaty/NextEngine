# Original Nonlocal GPU dynamic visual corpus — revision 1

| Field | Value |
| --- | --- |
| Research ID | `NGQ2` revision 1 |
| Status | `FROZEN / GAME_QUALITY_ONLY / TOOL_ONLY` |
| User decision | Explicitly authorized on 2026-09-01 after the 48k analytic-contact timing pass |
| Architecture status | SPEC-38 and ADR-076 remain Proposed; ADR-081 guardrails remain Accepted |
| Engineering consumer | decide whether the original five-iteration GPU route is visually plausible enough to continue toward a renderer-facing prototype |
| Claim class | finite/profile-bound game-quality and implementation measurement |

## Exact question and ceiling

Can the exact original compact/fused Nonlocal GPU candidate selected by NGQ1
advance a falling water block without catastrophic breakup, escape or numerical
failure, while producing a coherent sphere-depth presentation surface?

This experiment does not require stable-ID agreement with the later corrected
CPU research solver. It does not establish real-water fidelity, production
runtime integration, renderer quality, persistence/replay or a shipping
backend. CPU DFSPH remains the fallback.

## Frozen implementation route

```text
backend       = fused-owner-terms-p1 + compact-csr-u16-p2
profile       = nuv-basin-48k-analytic-contact-game.v5 coefficients
iterations    = 5
spacing       = 0.05 m
radius        = 0.025 m
mass          = 0.125 kg
horizon       = 0.15 m
dt            = 1/240 s
gravity       = (0,-9.81,0) m/s^2
boundary      = analytic componentwise GPU box clamp, no ghost samples
observer      = host-only sphere depth and 8-connected silhouette topology
observer time = excluded from the primary CUDA step timing
```

The dynamic fixture is rebuilt from the accepted host state before each GPU
step, exactly as in the admitted NGQ1 game-quality smoke. No CPU correction,
particle deletion, retry, smoothing or post-step position repair is allowed.

## Frozen lanes and schedule

### `FALLING_DAM_4K`

```text
dynamic lattice = 20 x 10 x 20 = 4,000 samples
box cells        = 40 x 15 x 20
box extent       = 2.0 x 0.75 x 1.0 m
initial x/z      = one half of x, full z width
initial y        = radius + 2*spacing through radius + 11*spacing
steps            = 96 = 0.4 s
surface frames   = accepted states 0, 24, 48, 72, 96
```

### `FALLING_DAM_16K`

```text
dynamic lattice = 40 x 10 x 40 = 16,000 samples
box cells        = 80 x 15 x 40
box extent       = 4.0 x 0.75 x 2.0 m
initial x/z      = one half of x, full z width
initial y        = identical to FALLING_DAM_4K
steps            = 96 = 0.4 s
surface frames   = accepted states 0, 24, 48, 72, 96
```

Run 16k only if 4k passes every apparatus and game-quality gate. A skipped 16k
lane is `NOT_RUN_4K_REJECTED`, not a PASS.

## Visible-sphere observer

Observe along gravity, from `+y` toward `-y`, on the `x-z` plane. The observer
uses the existing SPEC-38 debug-sphere representation:

```text
sphere radius = spacing/2 = 0.025 m
pixel pitch   = spacing/4 = 0.0125 m
pixel centre  = box_min_xz + ((index + 0.5) * pitch)
covered       = dx^2 + dz^2 <= radius^2
depth         = y + sqrt(radius^2 - dx^2 - dz^2)
pixel depth   = maximum covered contribution
```

Flood-fill wet pixels using 8-connectivity in increasing pixel order. A
material component contains at least four pixels. Report wet pixels, material
component count, largest-component fraction, satellite-area fraction and
nearest-rank depth p50/p95. The unsmoothed depth buffer is a product-facing
diagnostic, not the final fluid renderer.

The observer writes optional PPM montages only after the simulation; file I/O
and all observer work remain outside CUDA timings and cannot affect PASS.

## Predeclared apparatus gates

Every attempted step and frame must satisfy:

```text
finite positions, velocities, density and metrics
exact dynamic sample count
neighbor build succeeds within N*123 directed-pair capacity
compact u16 neighbor identity remains selected
analytic contact correspondence error <= 1e-4 m/s
maximum box penetration              <= 1e-12 m
surface observer non-empty and dimensionally exact
all five keyframes produced in order
```

An apparatus failure stops the lane and cannot select a physical/game result.

## Predeclared game-quality gates

For each executed lane:

```text
maximum speed over trajectory                    <= 10 m/s
vertical centre-of-mass drop                      >= 0.075 m
rightmost particle/front advance                  >= 0.10 m
final/initial wet-pixel area ratio                 in [1.05, 3.00]
every keyframe largest silhouette component       >= 98%
every keyframe satellite silhouette area          <= 2%
every keyframe material component count            <= 4
final particle-graph largest component             >= 98%
final particle-graph satellite fraction            <= 2%
final depth p95 lower than initial depth p95 by    >= 0.025 m
```

The generous component and speed bands deliberately accept stylized game
water. They reject only visually obvious spray clouds, disconnected chunks,
explosive velocity and a block that never falls/spreads.

## Controls and evidence

Before interpreting a PASS:

1. identical reruns must preserve the semantic trace, surface-frame and result
   roots exactly after excluding only executable/path identity;
2. an empty or non-finite frame must fail the observer;
3. deleting a wet pixel from the largest component or changing one depth value
   must change the frame root;
4. surface file emission disabled/enabled must not change simulation or result
   roots;
5. the prior `--game-quality-smoke`, CUDA self-test and H3 CPU profile corpus
   must remain PASS.

Preserve exact commit, binary/profile/input/output hashes, device identity,
per-step CUDA timings, first failing step/gate and raw JSON outside Git. A
successful author run is `SUPPORTED_BOUNDED` only after one independent
read-only review of the frozen source and evidence.

## Stop rule and successor

- Stop at the first 4k apparatus or game-quality failure; do not loosen a band
  or run 16k to search for a green result.
- If 4k passes and 16k fails, preserve the first 16k failure and diagnose that
  scale boundary without retuning.
- If both pass, the next separately frozen stage may couple the accepted
  trajectory to a renderer-facing smooth surface and measure extraction cost.
- No result here promotes the backend or changes current architecture.
