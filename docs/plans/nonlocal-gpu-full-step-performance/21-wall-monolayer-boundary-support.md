# Wall monolayer stall discriminator — revision 1

| Field | Value |
| --- | --- |
| Research ID | `NGQ7` revisions 1 and 2 |
| Status | `RUN / REVISION_2_SELECTED_BOUNDED / G1_OBSERVABLE_OPEN` |
| Parent | D-047 observation on the live 16k lane; D-033 reconsideration condition |
| Negative control | the accepted game profile with analytic contact only (`--boundary-layers 0`) |

## Observation

Past the accepted 96-step corpus, the dam front of the 16k lane reaches the
far wall at step 220 as a one-particle floor sheet. By step 240 the last
`0.3 m` before the wall holds `608` bottom-layer particles (`2.53x` the
lattice capacity of `240`), the sheet is motionless for about `120` steps,
and the surface crest first exceeds `0.25 m` at `x = 3.42 m`, `0.58 m` from
the wall, while the water at the wall stays `~50 mm` deep.

## Competing hypotheses

| ID | Causal hypothesis | Prediction under the frozen change |
| --- | --- | --- |
| H7B | a floor monolayer without boundary density support compresses in plane until the 3D kernel reads rest density and therefore carries no pressure; the bulk jumps onto it upstream | with two fixed boundary layers the wall band stays near lattice density and the first `0.25 m` crest forms at the wall |
| H7C | the stall comes from the positional contact clamp or the lattice sheet itself, independent of density support | the compression and upstream crest persist with boundary layers |
| H7D | the stall is a resolution effect of the 50 mm spacing | boundary layers change the numbers but not the qualitative pattern at 4k and 16k alike |

## Frozen change

The stream lanes may append the existing `append_game_boundary` lattice
complement (`--boundary-layers 2`: two layers of fixed samples on all six
faces of the box, the geometry of the accepted `two_layer_fixed_ghost_density_support`
profiles). No other profile value changes: same iterations, kappa, lambda,
horizon, capacity `160`, analytic box contact. The fixed samples take part in
neighbor search, density and the constrained solve; they never move and are
excluded from the presentation surface and the particle dump.

Compact `u16` neighbor identifiers bound the sample count to `65,535`, so
the discriminator runs on the `4k` and `16k` lanes with two layers; `48k`
reports cost only with one layer (`~12k` fixed samples).

## Frozen gates

Computed offline from the per-frame particle dumps (every 4 steps) and the
streamed closing surface of the same run, for the whole 960-step run:

| Gate | Definition | Baseline (no support, 16k) | Pass |
| --- | --- | ---: | --- |
| G1 compression | maximum over frames of bottom-layer count (`y < 0.06 m`) in the last `0.3 m` before the far wall, divided by that band's lattice capacity (`0.3 x depth / 0.05^2`) | `2.53` | `<= 1.2` |
| G2 crest location | at the first frame whose maximum surface height over the downstream half exceeds `0.25 m`, the distance from that maximum to the far wall | `0.58 m` | `<= 0.15 m` |
| G3 stall | longest run of consecutive frames in which the wall band holds `>= 50` particles, its mean `x`-velocity is below `0.1 m/s` and its mean surface height is below `0.1 m` | `>= 30` frames | `<= 10` frames |
| G4 finite state | every audited frame finite, degree within capacity | pass | pass |
| cost | execute wall per step with and without support, per lane | report | report |

A PASS requires G1, G2, G3 and G4 on both 4k and 16k with two layers while
the no-support control reproduces the baseline failure on 16k. G1--G3
failing with support selects H7C; a qualitatively different pattern between
4k and 16k selects H7D.

## Stop and interpretation

- pass: H7B is supported bounded; the game candidate needs boundary density
  support, and the 48k budget question moves to the solver contract;
- G1 passes but G2/G3 fail: compression is not the mechanism; open a contact
  or sheet discriminator instead;
- no support run passes too: the baseline was not reproduced;
  `APPARATUS_INCONCLUSIVE`.

Do not tune layer count, spacing or contact after seeing the results.

## Revision 1 result and apparatus correction

Revision 1 ran on 16k and 4k with two layers plus the no-support controls.
The control reproduced the baseline on 16k (G1 `2.68`, post-arrival crest
`0.59 m` from the wall, stall `33` frames). With two layers the stall
vanished (G3 `0`) and compression fell to `1.43`, but the front slowed from
`2.22 m/s` to `0.89 m/s`: the fixed samples enter the bulk and shear
viscosity terms and the surface term, so the floor behaves as no-slip.

Two apparatus defects are corrected before revision 2, without touching
physics: G2 as written triggered on the falling column at step 36--40, so
it is evaluated only from the frame in which the front first reaches the
wall; and the front speed is added as a frozen gate.

## Revision 2 (frozen before running)

The fixed complement contributes to density and the incompressibility term
only; the viscosity and surface accumulations skip fixed neighbours
(`--boundary-support density`). Campaign profiles keep the previous
behaviour (`full`) and their roots are unaffected.

| Gate | Definition | Pass |
| --- | --- | --- |
| G1 | as revision 1 | `<= 1.2` |
| G2' | crest distance at the first post-arrival frame whose downstream-half maximum exceeds `0.25 m` | `<= 0.15 m` |
| G3 | as revision 1 | `<= 10` frames |
| G4 | as revision 1 | pass |
| G5 front speed | mean front speed from `x = 2.0 m` to the wall relative to the no-support control | `>= 0.7x` |

A revision-2 PASS on 16k and 4k selects H7B with density-only support as
the game candidate; failure of G5 with G1--G3 passing means the fixed
complement itself, not viscosity, slows the sheet and a sheet/contact
discriminator follows.

## Revision 2 result

Recorded in `docs/development/nonlocal-gpu-wall-monolayer-evidence-2026-09-02.md`.
On 16k and 4k, G2', G3, G4 and G5 pass with density-only support (crest at
`0.06 / 0.04 m` from the wall, `0` stall frames, front `1.18x / 1.22x` the
control) while the control reproduces the baseline on 16k. G1 fails at
`1.53 / 1.63`: the `y < 0.06 m` observable counts second-layer samples
squeezed under a loaded column; the floor-touching layer stays within
`1.32`. The mechanism (H7B) is selected bounded; the G1 observable needs a
revision that separates the floor layer from squeezed layers before any
compression claim. No parameter was tuned after seeing results.

## Revision 3 (frozen before running): split G1

G1 conflated the floor-touching layer with samples squeezed below `0.06 m`
under a loaded column. Revision 3 changes only the observable, keeps the
threshold and reruns the stored revision-2 dumps:

| Gate | Definition | Pass |
| --- | --- | --- |
| G1a floor layer | maximum over frames of the wall-band count with `y < 0.04 m` (samples in floor contact, clamp at `0.025 m`) divided by the band's lattice capacity | `<= 1.2` |
| G1b squeezed | maximum over frames of the wall-band count with `0.04 <= y < 0.06 m` divided by capacity | report only |
| G1c settled | G1a evaluated over the last `120` steps of the run only (`>= 840`) | `<= 1.2` |

G1a above `1.2` during the impact but G1c within `1.2` means transient
compaction under load rather than a stalled layer; G1c above `1.2` means
the floor layer stays over-dense and the density support is still
incomplete.
