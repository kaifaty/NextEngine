# Nonlocal GPU game-quality and timed-contact evidence — 2026-09-01

## Scope and claim ceiling

This report evaluates the retained `fused-owner-terms-p1 + compact-csr-u16-p2`
CUDA implementation as a game-water candidate. It does not claim laboratory
accuracy or agreement with the later pressure-QP research solver.

The admitted claim is bounded to:

- the three small product-facing smoke scenarios described below;
- the five-iteration H3-coefficient route;
- one reset 48,000-particle CUDA step with analytic box contact included in the
  CUDA-event interval;
- an RTX 3080 under the observed desktop load.

It does not yet establish a long 48k trajectory, a rendered dam break, visible
free-surface quality at product scale, runtime integration or shipped status.
SPEC-38 and ADR-076 remain Proposed, and CPU DFSPH remains the fallback.

## Implementation under test

- CUDA backend: `fused-owner-terms-p1 + compact-csr-u16-p2`.
- Dynamic samples: `48,000`.
- Static ghost samples: `0`.
- Iterations: `5`.
- Coefficients: H3 `kappa=9196.875`, `lambda=360`, normal/bulk viscosity;
  surface and shear terms disabled as in the selected profile.
- Contact: one componentwise analytic box clamp on the GPU after the position
  solve and before final density/velocity publication.
- Contact work: included in primary CUDA total timing and reported separately.
- Host observers: excluded from primary timing.

Profile `nuv-basin-48k-analytic-contact-game.v5` has SHA-256
`cd6dd8f9ea4304a603b9f58d2891787fc5f9173c50edc4d635efa6fbc7b5901b`.
The fixture input root is
`51117c157030ab9bc715e120879230aa701b9198e09232073a7c78cd21eb5274`.
The measured binary SHA-256 is
`b6b047031c488e89d574f1ca026068d217a2075b1021ab550ffb6105adddab6a`.

## Game-quality smoke

Command:

```text
nonlocal-feasibility --game-quality-smoke
```

The smoke uses the actual published GPU position and velocity as the next-step
state. The host independently recomputes box projection and velocity only for
correspondence and product observables.

Scenarios:

1. `confined_hold`: 24 steps, finite/contained, one component, zero satellites,
   vertical COM drift `7.046 mm` at five iterations against `25 mm`.
2. `release_contact`: 48 steps, bottom contact with lateral/top clearance,
   finite/contained, one component, zero satellites, forward COM travel
   `150.002 mm` against `50 mm`.
3. `face_corner_contact`: manufactured high-speed face and corner crossings,
   exact expected contact features and zero penetration.

Maximum GPU-published velocity discrepancy against the independent contact
oracle is `1.779e-6 m/s` against `1e-4 m/s`. Both five and sixteen iterations
pass after ghost removal; five is selected because it is the first passing and
cheaper lane. No threshold was weakened after observing the result.

Two smoke processes have byte-identical non-timing content with normalized
SHA-256
`ca03f14cc9cab75171ea87411f01c3a1a010c56df331f0f34cd3be8c88c18c68`.
The semantic result root is
`29816d43122ff831888fa17fda2f8d3c84abaccbce400142456ee1ae5a28de44`.

## 48k timing

Each process executed 256 conditioning steps, 64 warmups and 512 measured
steps:

```text
nonlocal-feasibility --p2-decision \
  nuv-basin-48k-analytic-contact-game.v5 --warmup 64 --runs 512
```

| Metric | Process A | Process B | Gate |
| --- | ---: | ---: | ---: |
| total p95 | `3.797568 ms` | `3.798176 ms` | `<= 4 ms` |
| total p99 | `3.808544 ms` | `3.874656 ms` | `<= 6 ms` |
| total median | `3.596448 ms` | `3.599520 ms` | diagnostic |
| contact p95 | `0.005120 ms` | `0.005120 ms` | included |
| contact p99 | `0.005120 ms` | `0.005120 ms` | included |

Both processes report `trace_exact=true`, `trace_memory_exact=true`,
`measurement_valid=true` and no failed measurement. The shared trace SHA-256 is
`e65e683ec1c0415e16c59ddfd290ad2b592d186864ce6f849ac104bb35909e59`.
The ordered output root is
`b49be7bb0b1617832267428a73f45a833e1fd3cc7aaa8817620341917b6fe58d`;
the logical CSR root is
`bee4107f065c40f7277b27e0a96e56317b754693571842cc96fcf1e8f24d0988`.
Capacity is `20,917,770` device bytes, two-byte neighbor IDs,
`5,200,628` directed pairs and maximum degree `123`.

Raw JSON SHA-256 values are:

- Process A: `072f11955d4c2e5931b76b36280029738d7d67b6ebbe0c95b304e78d6cb70b4a`.
- Process B: `1bea379cefc7ea8c0ba75db76b11aa6944c018ee2254be01398839817097c29f`.

## Discriminating negative result

The 48k H3 profile with 38,856 static ghost samples remained numerically exact
at five iterations but used 86,856 total solver samples, fell back to u32
neighbor IDs and took approximately `14.94--16.05 ms` for one step. This
rejects the ghost-shell representation for the four-millisecond game budget;
it does not reject the five-iteration Nonlocal compute path.

## Checks

- focused CMake Release build: PASS;
- `--game-quality-smoke`, two processes: PASS and normalized-exact;
- `--self-test`: PASS, every case passed;
- `--cpu-h3-profile-corpus`: PASS, retained SHA-256
  `0b02fc8c12d8870b1b9ae1d9c135fbabe684daa450d7ce73debc59ba4e581950`;
- `--production-profile-audit`: exit 0;
- two 48k performance processes: PASS, 512/512 measurements each;
- `git diff --check`: PASS.

## Next smallest action

Run a bounded 4k/16k dynamic release or dam trajectory with the same GPU
contact path and product-facing surface/topology observers. Do not add the
pressure-QP research solver unless that corpus exposes a concrete gameplay
failure attributable to the missing semantic block.
