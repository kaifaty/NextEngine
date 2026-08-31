# NCGP7 Eulerian step-112 diagnostic — 2026-08-31

## Verdict

`H7C_INCONCLUSIVE / PERFORMANCE NOT_RUN`.

The exact NCGP6 hydrostatic step-112 witness reproduces bit-for-bit. On both
predeclared Eulerian grids, GPU and independent CPU states have close bulk
mass, density, velocity and centre of mass. The free-surface decision is not
resolution-stable: the 50 mm grid misses the H7A surface-RMSE band, while the
25 mm grid passes that band but misses the wet-column-set band. Neither grid
reaches the predeclared H7B clear-divergence region.

This does not waive the NCGP6 stable-ID p99 failure and does not admit
dam-break, orifice, 16k/50k correctness or timing. It narrows the remaining
uncertainty to the visible free-surface quantity of interest rather than a
bulk loss of water state.

## Frozen identity and reproduction

- contract commit: `fbfe67dbf43f0c382e845f0ed5b79e87e2cdcf87`
- implementation/final source commit:
  `8ad5caa4389ab512dfd474c51b983d8ef5348619`
- source tree: `f0e6c3d188c96b31af56179a79b6f7d0f83231fb`
- NCGP7 leaf-contract SHA-256 before implementation:
  `da90df5caf82464112af6969c508ead9f017c40dd56333a2ee07e0bcab5118c3`
- aggregate contract root:
  `89417c4aa4a7a234b46dd95ae3651aea51fff6bd5b70eb5410da4757dadfb17c`
- source root:
  `be0d16ce57b906a198a3863a44953d34e44655deade06b549e8849a9abe885df`
- two fresh Release binaries, byte-identical SHA-256:
  `41d5d28a78f03c0ca150b32a531d3f5d48637f1daec528319281a7fe81e40253`
- host: NVIDIA GeForce RTX 3080, SM 8.6, CUDA runtime/driver 13.3
- allocated device memory: `137251397` bytes
- flags: C++ `-O3 -Wall -Wextra -Wpedantic -Werror -ffp-contract=off
  -fno-fast-math`; CUDA `-O3 --fmad=false --prec-div=true --prec-sqrt=true
  --ftz=false`; `SM=86`

The exact apparatus command is:

```text
nonlocal-corrected-cuda-eulerian-diagnostic --eulerian-field-self-test
```

Both builds return PASS with byte-identical stdout SHA-256
`f0f0fd93c66e30ca5208dad650b863b37fede83cc5fe4733327b186c45f978e0`
and result root
`48aa522370564048ef58137c0a2ac3bc4b28dae8abcfba4b50c8b3bc8fd5ee7b`.
Exact-zero fields, stable-ID relabelling, whole-state translation, sample
deletion, omitted channels, quantile, field, work and grid mutations all take
their declared routes.

The exact witness command is:

```text
nonlocal-corrected-cuda-eulerian-diagnostic \
  --diagnose-hydro-step112-eulerian
```

Both fresh processes return 0 with byte-identical stdout SHA-256
`1b6d31323018bf02b231a241a0af25f112a7adbc987fef69b52446d775f60abb`.
The result root is
`d11a79d7360b9ea4b1fc3cc85c974d4e97592bc29cdc09483a6dde7f6fe70abc`,
the 112-step receipt root is
`5dcff7ae2f7f9f6f6758a42aea9806b4631d71fe2aeb1b03cb78dab5e2c19b9b`
and the input root is
`f96ad2a8bd7c7fcee2917ff6168c1aac59d7da56d2f5844a3cfcdab396eda80f`.
The exact parent NCGP6 result root
`4e72a97b1add42e7c58f839fc66dd918879b4325c23f75e5e5f1cf696ea57009`
is bound into the result.

Raw JSON remains outside Git at `/tmp/ncgp7-final-a-step112.json` and
`/tmp/ncgp7-final-b-step112.json` for this host session.

## Exact witness and retained observables

The NCGP6 position witness is exact:

| Observable | Value |
| --- | ---: |
| position RMSE | `0.780996279 mm` |
| nearest-rank position p99 | `2.576882866 mm` |
| position maximum | `20.810782528 mm` |
| maximum GPU / CPU HVP | `125 / 110` |

Corrected and stable-ID-permuted GPU state/field roots are exact. Particle
count is exactly 4,000 and mass is exactly 500 kg. Maximum basin penetration
is zero; all centres remain inside the analytical basin. The normalized
GPU/CPU total-momentum difference is `0.00174968%`. The internal final
solver-energy correspondence is reported diagnostically as `1.074815%`; it
is not the previously frozen positive mechanical-energy-excess/reversibility
gate and was not introduced as a post-result acceptance threshold.

## Eulerian results

| Observable | 50 mm coarse | 25 mm fine | H7A limit |
| --- | ---: | ---: | ---: |
| mass total variation | `0.280293%` | `0.823499%` | `<= 1%` |
| density RMSE | `0.156208%` | `0.172655%` | `<= 1%` |
| normalized velocity RMSE | `0.093921%` | `0.126068%` | `<= 1%` |
| centre-of-mass error | `0.008428 mm` | `0.008428 mm` | `<= 2.5 mm` |
| wet-column symmetric difference | `0.288184%` | `1.742627%` | `<= 1%` |
| surface-height RMSE | `17.769195 mm` | `8.647731 mm` | `<= 12.5 mm` |
| surface-height p95 | `0.912828 mm` | `0.764343 mm` | `<= 25 mm` |
| diagnostic surface maximum | `325.768857 mm` | `199.025905 mm` | report only |

The coarse surface RMSE is dominated by a rare tail: its p95 remains below
1 mm. The fine grid improves surface RMSE while changing the wet support from
the tight H7A band. This is exactly the frozen H7C case. Coarse metrics remain
far below all H7B clear-divergence bands.

## Retained controls and correction record

All retained exact-build controls pass:

| Control | Raw stdout SHA-256 |
| --- | --- |
| profile | `e7905a64dae24b74cc473cee294f113f33866ec912d7d51a5802f3b37aeb5056` |
| graph | `5d2be29a316b3ebffde935bfff49e78981eb894380d2b47da62d846db2b5e3ac` |
| boundary | `4fba27dc1badff9fc61a77b47cd2db995b578bd1a7e3af1aab7a9c71567087e8` |
| transaction | `8c97ff87d5067ee1a03ba1ff788f5005e3491c1f90806811f12e2763813c889b` |
| physics/HVP/predictor | `7f7c5fd3dbc859549ff5f3540fb0228f782cb05a2b75095ab7f91b52604b7cd7` |
| NCGP6 product-gate apparatus | `70edb1c967835da7a8b135857a86976af11112e7d405fe664ce0720e6204deab` |

An intermediate clean run returned `APPARATUS_INCONCLUSIVE` because the
implementation mistakenly applied a new 1% threshold to internal
CPU/GPU solver-energy correspondence. That threshold is absent from the
frozen contract and is not equivalent to the retained mechanical-energy
controls. Commit `8ad5caa4` removed only that invented predicate, retained
the value as a sealed diagnostic, and rebuilt/repeated all final evidence.
The intermediate raw stdout SHA-256 is
`f5108fbe83c49ee043251c3be2fed155b7c209658aec15af45eba0c87edb7afd`.

Compute Sanitizer and independent review are `NOT_RUN`: NCGP7 did not reach a
positive product decision and does not admit performance. They remain
mandatory if a successor passes the complete numerical gates.

## Decision

Close NCGP7 revision 1 as `H7C_INCONCLUSIVE`. Keep NCGP6
`REFUTED_BOUNDED`; do not reinterpret the old p99 gate, adjust the current
surface thresholds, add a third voxel resolution or time the stopped
candidate.

The smallest next action is a separately frozen product-facing
free-surface diagnostic whose observables correspond to visible geometry,
for example robust surface area/height distribution and connected wet-region
topology. It must distinguish a few edge columns from a persistent visible
surface defect and must be accepted explicitly before restarting
dam-break/orifice, 16k/50k correctness and complete-step timing.
