# NSR3-B4DR1C3 pressure-cap sweep evidence -- 2026-08-21

Status: `PASS / FIRST_CONVERGENCE_AT_220 / R1C4_DESIGN_AUTHORIZED`

## Result

All eight ascending one-step Hydro points are finite, deterministic in their
single execution and publish no payload. The residual decreases monotonically
without stagnation. Cap 200 still fails the `0.1` threshold; cap 300 permits
the solver to stop normally after iteration 220 at residual
`0.099204208738899444`.

The sweep therefore identifies a bounded cold-pressure convergence problem,
not a divergence, timestep, NaN or contact problem. It does not retroactively
pass R1C.

## Build identity

| Input | Attested value |
|---|---|
| implementation commit | `59e68884862f656c575abdcfd692866d61cc1b06` |
| R1C3 identity | `926e594fedec03e9c3b08aa76fc57a22988049e97f60c5e47113387aae879678` |
| executable | 1,568,552 bytes; `acdc73f65307f18928822b7b4393ada6b0abca6c25ae9810870cc6ee2663ca8d` |
| ELF build ID | `21b15016301388fa2578b72f507f0428e64bb4ab` |
| focused tests | `10/10 PASS` |

The R1B and R1C1 manifest reports retain their exact historical hashes.
Allowed-cap relative paths and cap `37` both reject before Simulation
creation. Every sweep stderr is empty and every external output directory has
zero entries.

## Sweep

| Cap | Iterations | Pressure residual | Threshold ratio | Converged | Report SHA-256 | Wall |
|---:|---:|---:|---:|---|---|---:|
| 25 | 25 | `3.5413221595861795` | `35.4132x` | no | `b9ea61fdb317aa9a0abe1dc301a9ec1b93c4c28a34bfd70034f22d3b8947a77b` | 0.15 s |
| 50 | 50 | `2.1166904827170185` | `21.1669x` | no | `08ae857ed6fb004bb727823a12630e8774421cc412c1b923c12af092a37c4369` | 0.26 s |
| 75 | 75 | `1.3112975183548161` | `13.1130x` | no | `e6ac7316659521aba83e22866f6c45379aa51132e78342165926ffe9814289ce` | 0.36 s |
| 100 | 100 | `0.82744058228594985` | `8.2744x` | no | `653ac098252c2bf984972148c4c333ace48b5f28df7a1433b60f1ef8c4b21422` | 0.46 s |
| 125 | 125 | `0.52711172223798819` | `5.2711x` | no | `f91a59a88fe2723bc69a5db2af2aa4bf200ec826d71c254127089fee5526e834` | 0.57 s |
| 150 | 150 | `0.33783250142153842` | `3.3783x` | no | `af3a67e8fa536bf3a432dd34acbcf606e3b8cda7faa29264531f8955a24c7b5d` | 0.67 s |
| 200 | 200 | `0.14044762612212555` | `1.4045x` | no | `4a2a30e3e7a922bcb97d4cbbdd4b2d11e50c3f6b2716aca09025905f755f9878` | 0.89 s |
| 300 | 220 | `0.099204208738899444` | `0.9920x` | yes | `1473b2e414729520f07e0462735a5f312e9cba4f5a7f7184b53a9b8a399ea212` | 0.97 s |

Raw residual bits are respectively
`400c54a0b6602e8f`, `4000eefb6b783140`, `3ff4fb131b4a8eee`,
`3fea7a64ac09a4ac`, `3fe0de19670b3670`, `3fd59f0c3648688c`,
`3fc1fa30147f681b` and `3fb965727028bcc7`.

## Calibration check

Source inspection confirms SPlisHSPlasH initializes volume as
`0.8 * diameter^3` specifically to reduce startup pressure. For the frozen
50 mm cubic lattice and cubic support radius 100 mm, the analytical infinite-
lattice kernel sum is `7999.7797287283411`. Thus:

- physical `spacing^3 = 0.000125` gives density ratio
  `0.99997246609104284`;
- the upstream startup heuristic `0.0001` gives `0.79997797287283434`.

Changing to the heuristic would intentionally make the interior about 20%
underdense and change particle mass from `0.125` to `0.1 kg`. That would remove
startup pressure by changing the material/reference problem, not by improving
solver convergence. It is rejected for the like-for-like comparator.

## Decision

Select a new-root R1C4 reclosure with pressure maximum 300 and all other
physics unchanged. The cap is a limit, not a forced iteration count; step 1
still stops at 220. Re-run the 24-step scenarios in the original paired,
cost-aware order. Any later step reaching 300 without convergence fails
closed. R1D remains blocked until all three pairs pass byte-identically.

