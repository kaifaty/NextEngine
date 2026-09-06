# Nonlocal GPU capacity-160 performance evidence — 2026-09-01

## Result

`QUALITY_AND_BUDGET_SUPPORTED_BOUNDED` for the exact original Nonlocal GPU
game candidate on the finite NGQ2/NGQ3 fixtures.

The same versioned capacity-160 profile now owns both sides of the decision:

- 4k and 16k falling-dam visual/invariant corpus: PASS;
- 48,000-particle performance campaign: PASS in two independent processes;
- total GPU p95: `3.225760 / 3.231616 ms` against `4 ms`;
- total GPU p99: `3.269312 / 3.334560 ms` against `6 ms`.

This is a tool-only RTX 3080 result for plausible stylized game water. It is
not a shipping-backend, renderer-integration, corrected-research-solver or
laboratory-fidelity claim.

## Frozen identity

```text
branch                         codex/water-research
base before this change        715de74b
contract                       NGQ3 revision 1
contract SHA-256               86d86bf42709f3ff7444cb9b5b0fcd2a90a0f17a941bb722ff0c8b1a74b39f54
profile                         nuv-basin-48k-analytic-contact-game-cap160.v6
profile SHA-256                3f482db40880099b9088c6b2ad8d7d427e455d78750267cb04410ff5ac040661
binary SHA-256                 822b5d9735d0a372a625f3301e00bb2813cc0b1bf5f16e0ca09d34a2333e2854
CUDA/compiler                  13.3.73 / RTX 3080 sm_86
build directory                /tmp/nextengine-nonlocal-game-quality-build.zdvs81
```

Measured source SHA-256 values before commit:

```text
profiles.cpp       f822fcbceaedb36004b32d5535086a483ec711bebaf0833972b7d085ae896605
cuda_baseline.cu   f891162703032b456d7284eeb71b2e0343370071e5e388cf572d0b5b5ed499da
```

## Exact configuration

- 48,000 dynamic samples (`80 x 15 x 40`);
- zero static ghosts;
- H3 `kappa=9196.875`, `lambda=360`;
- five fixed Nonlocal iterations;
- horizon `0.15 m`, time step `1/240 s`;
- compact u16 neighbor IDs;
- `160` neighbor slots and `7,680,000` directed-pair capacity;
- analytic GPU box contact included in primary timing;
- host observers excluded from primary timing.

Relative to the retained version-5 allocation, device memory increased from
`20,917,770` to `24,469,770` bytes: exactly `3,552,000` bytes, equal to
`48,000 * (160 - 123) * sizeof(u16)`. Actual graph work on the reset 48k
fixture remains `5,200,628` directed pairs with maximum degree `123`.

## Commands

```bash
cmake --build /tmp/nextengine-nonlocal-game-quality-build.zdvs81 \
  --target nonlocal-feasibility -j2

/tmp/nextengine-nonlocal-game-quality-build.zdvs81/nonlocal-feasibility \
  --p2-check nuv-basin-48k-analytic-contact-game-cap160.v6 \
  --iterations 5

/tmp/nextengine-nonlocal-game-quality-build.zdvs81/nonlocal-feasibility \
  --game-visual-corpus

/tmp/nextengine-nonlocal-game-quality-build.zdvs81/nonlocal-feasibility \
  --p2-decision nuv-basin-48k-analytic-contact-game-cap160.v6 \
  --warmup 64 --runs 512
```

The final command was run in two sequential processes. Each process performed
256 conditioning executions, 64 warmups and 512 measurements.

## Timing

| Metric | Process A | Process B | Gate |
| --- | ---: | ---: | ---: |
| total median | `3.067648 ms` | `3.074976 ms` | diagnostic |
| total p95 | `3.225760 ms` | `3.231616 ms` | `<= 4 ms` |
| total p99 | `3.269312 ms` | `3.334560 ms` | `<= 6 ms` |
| total mean | `3.081599 ms` | `3.090396 ms` | diagnostic |
| contact p95 | `0.005120 ms` | `0.005120 ms` | included |
| contact p99 | `0.005120 ms` | `0.005120 ms` | included |

Both JSON reports have `decision_gate=true`, 512/512 valid measurements and
no first trace mismatch.

Raw JSON SHA-256:

```text
Process A  19987ad588464e2b5221dd002e29aecd2b87863a4b54c3f3dbd4fb2fdf312bf7
Process B  94c934d74319998a67cc223d24c56d3025018883e42f33c99350886e7434e76a
```

## Semantic correspondence

Both processes match exactly on:

```text
input                  775f0c4a9057b49cd2dd859c81036d3ea88c277e5ee7ea19856c4a9481afe6f0
trace                  5b5a0b3ffc6b2a6805c5b27c7dc65c55396209660f31063c574413053fc0aecc
ordered output         b49be7bb0b1617832267428a73f45a833e1fd3cc7aaa8817620341917b6fe58d
logical CSR            bee4107f065c40f7277b27e0a96e56317b754693571842cc96fcf1e8f24d0988
semantic subset        a50edd3e4d8090f4f94ae3b2458d7d3a09f4a24e7cce505cf1c5e6ad2d6410e4
```

The version-6 visual rerun also preserves every prior NGQ2 semantic root:

```text
corpus result          c2f1f6e75f1238409298cb6db6f16c2aeb0baca3ad4ebbeff216ba3c39f98274
4k trace/result        369ac07d... / 18f48386...
16k trace/result       61c45089... / 8e3e9f95...
```

The visual rerun raw JSON SHA-256 is
`48d9517a8c6869b58b3a48ae7193bde163a470d0d434729a80079cc44b6fd699`.

## Checks

- focused CMake Release build: PASS;
- version-6 profile description/audit: PASS;
- P2 correctness and retained/compact correspondence: PASS;
- version-6 4k/16k visual corpus: PASS, semantic roots unchanged;
- two sequential 512-measurement processes: PASS;
- CUDA self-test: PASS, raw SHA-256
  `2d22c16079c12cfb8f5abb689b9e3dffe845d5077fb98eeacfca231033b08216`;
- CPU H3 corpus: PASS, retained raw SHA-256
  `0b02fc8c12d8870b1b9ae1d9c135fbabe684daa450d7ce73debc59ba4e581950`;
- independent review: `NOT_RUN` (no separately authorized reviewer);
- `git diff --check`: PASS.

## Decision and next risk

The original compact/fused five-iteration GPU path is now bounded-supported
for both the frozen visual corpus and the near-50k game budget on one explicit
capacity profile. The next smallest action is presentation-only surface
smoothing/extraction over the accepted frames. That work must be timed
separately and must never feed back into simulation state.

The first visible limitation to watch is the final 4k top-view satellite area
of `0.846%`; the particle graph itself remains one connected component. CPU
DFSPH remains the fallback, while SPEC-38 and ADR-076 remain Proposed.
