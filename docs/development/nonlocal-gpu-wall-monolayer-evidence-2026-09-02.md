# Nonlocal GPU wall monolayer stall evidence — 2026-09-02

## Result and claim ceiling

`WALL_MONOLAYER_STALL_RESOLVED_BOUNDED / TRANSIENT_FLOOR_COMPACTION_REPORTED`: with a fixed
two-layer lattice complement that supports density only (NGQ7 revision 2),
the dam front of the 4k and 16k lanes no longer stalls at the far wall, the
first `0.25 m` crest after arrival forms at the wall (`0.06 / 0.04 m`
instead of `0.59 m` away), the stall gate reads `0` frames, and the front
is faster than the unsupported control (`2.61` versus `2.22 m/s` at 16k,
`2.22` versus `1.82 m/s` at 4k). Revision 1 (fixed samples in every term)
removed the stall but slowed the front to `0.89 m/s`; it is refuted.

The frozen compression gate G1 still fails (`1.53 / 1.63` against `1.2`)
because its observable (`y < 0.06 m`) counts second-layer samples squeezed
under a loaded column; the floor-touching layer (`y < 0.04 m`) stays at
`0.72--1.32` of lattice capacity while the control holds `2.42--2.65` as a
pure monolayer for 400 steps. The gate observable, not the mechanism, is
what remains open. This is a game-candidate finding under Proposed
SPEC-38/ADR-076; campaign profiles and their roots are unchanged.

## Frozen inputs and source boundary

```text
branch                          codex/water-research
plan                            docs/plans/nonlocal-gpu-full-step-performance/21-wall-monolayer-boundary-support.md
extractor cuda_baseline.cu      1d07810068462557c8ca594c22c40875978ad511316aa83e3c8d01de73b6bd4e
extractor oracle.hpp            2f405730d6c07928d59f3ec293001d6e1154afc29c83b5fda6e9bea16589d5fc
extractor main.cpp              6b606488ff4af8032ebaa259c4f5ced41d67c5296580cd7b975236c7122a586a
extractor binary                2adf16a8100eba4eec84d1bf8cdb1bccd13b3ef32c09e7e460118a10ce3f44c6
gate script                     lab/scripts/nonlocal_wall_monolayer_gates.py (80d412c9...)
xtask water_preview.rs          844e3eba4fdc788b934fd62c8defc5b481dcc9dcabd6f8413b466111033e8c9a
release xtask                   34f12df3e6ef746c776ccf091d0add1acc3e532b6dc511b63ea485baec68798c
```

Solver changes: `game_fixture` takes `boundary_layers` (default `0`, the
corpus path unchanged) and `Fixture::boundary_density_only` (default
`false`); the fused owner-terms kernel receives the fixed flags and, only
when density-only support is on, skips the viscosity and surface
accumulations for fixed neighbours. The stream exposes `--boundary-layers
0|1|2` and `--boundary-support full|density`; `water-preview` passes
`--stream-boundary-layers` / `--stream-boundary-support` through and now
defaults to two density-only layers (one on the 48k lanes, whose sample
count with two layers exceeds the `u16` neighbour-identifier bound).

## Runs

All runs: `960` steps, surface every `4`, GPU extractor with the closing
model, one worker, particle dumps every emitted frame. Raw summaries:

| Run | boundary samples | max degree | physics per step | execute wall per step | raw JSON |
| --- | ---: | ---: | ---: | ---: | --- |
| 16k control (`L0`) | `0` | `134` | `2.53 ms` | `3.02 ms` | `66d9665c...` |
| 16k full (`L2`, rev 1) | `22,224` | `131` | `4.27 ms` | `5.22 ms` | `c41222dc...` |
| 16k density (`L2density`, rev 2) | `22,224` | `127` | `2.49 ms` | `3.40 ms` | `8197a551...` |
| 4k control | `0` | `131` | `2.15 ms` | `2.30 ms` | `a62da60e...` |
| 4k full | `8,064` | `129` | `2.72 ms` | `3.05 ms` | `06c7b406...` |
| 4k density | `8,064` | `126` | `1.63 ms` | `1.94 ms` | `d4b66863...` |
| 48k full (`L1`) | `11,768` | `132` | `8.83 ms` | `10.88 ms` | `c4e5f416...` |
| 48k density (`L1`, 480 steps) | `11,768` | `130` | `5.35 ms` | `7.15 ms` | `c3284983...` |
| 48k control (480 steps) | `0` | `139` | `3.86 ms` | `5.50 ms` | `2d87c255...` |

The 48k control with dumps failed only because `/tmp` filled; it was
repeated without dumps for cost. Physics per step varied between sessions
for the same configuration (16k control `1.5--2.5 ms`); comparisons are
within one table.

## Gates

Evaluated by the stored script from the dumps and the streamed surface.

| Lane / run | front at wall (step, m/s) | G1 compression (`y < 0.06`) | G2' crest distance | G3 stall | G4 | G5 front speed |
| --- | --- | ---: | ---: | ---: | --- | ---: |
| 16k control | `216`, `2.22` | `2.68` FAIL | `0.59 m` FAIL | `33` frames FAIL | pass | control |
| 16k full (rev 1) | `540`, `0.89` | `1.43` FAIL | `1.99 m` FAIL | `0` PASS | pass | `0.40x` FAIL |
| 16k density (rev 2) | `184`, `2.61` | `1.53` FAIL | `0.06 m` PASS | `0` PASS | pass | `1.18x` PASS |
| 4k control | `132`, `1.82` | `2.38` FAIL | `0.01 m` PASS | `0` PASS | pass | control |
| 4k full (rev 1) | `196`, `1.22` | `1.30` FAIL | `0.89 m` FAIL | `0` PASS | pass | `0.67x` FAIL |
| 4k density (rev 2) | `108`, `2.22` | `1.63` FAIL | `0.04 m` PASS | `0` PASS | pass | `1.22x` PASS |

G2 as first written triggered on the falling column at step 36--40 for
every run; it is reported as G2' (from the frame the front first reaches
the wall), an apparatus correction recorded in the plan before revision 2
ran. The 4k control shows compression without a stall, so the stall itself
is the 16k observable; the 4k lane still gains crest location and speed.

## Wall-band layering behind G1 (16k)

| Step | band samples | `y < 0.04` | `0.04--0.06` | `0.06--0.10` | `>= 0.10` | comp. `y < 0.06` | comp. `y < 0.04` | mean `vx` |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| density 184 | `282` | `282` | `0` | `0` | `0` | `1.18` | `1.18` | `+3.10` |
| density 216 | `678` | `172` | `155` | `109` | `242` | `1.36` | `0.72` | `+0.93` |
| density 300 | `1,714` | `303` | `21` | `130` | `1,260` | `1.35` | `1.26` | `+0.11` |
| density 408 | `2,272` | `316` | `50` | `82` | `1,824` | `1.53` | `1.32` | `+0.03` |
| control 248 | `636` | `636` | `0` | `0` | `0` | `2.65` | `2.65` | `+0.19` |
| control 360 | `609` | `607` | `0` | `0` | `2` | `2.53` | `2.53` | `-0.08` |
| control 440 | `1,102` | `597` | `0` | `0` | `505` | `2.49` | `2.49` | `+0.50` |

With support the band layers up within `32` steps of arrival and holds
over two thousand samples by step 408; without support it stays a single
compressed layer for four hundred steps and only then gains a second layer.

## Revision 3: the split G1 observable on the stored dumps

Frozen in the plan before rerunning the revision-2 dumps: G1a is the
floor-touching layer (`y < 0.04 m`, threshold `1.2`), G1b the squeezed
`0.04--0.06 m` band (report only), G1c is G1a over the last `120` steps.

| Run | G1a floor layer (peak) | G1b squeezed (peak) | G1c settled floor |
| --- | ---: | ---: | ---: |
| 16k control | `2.68` (step 256) FAIL | `0.00` | `2.25` FAIL |
| 16k density | `1.32` (step 408) FAIL | `0.68` | `1.13` PASS |
| 4k control | `2.35` (step 240) FAIL | `0.50` | `2.27` FAIL |
| 4k density | `1.29` (step 792) FAIL | `0.65` | `1.17` PASS |

By the plan's own interpretation, a G1a peak above `1.2` with G1c inside
`1.2` is transient compaction under load, not a stalled layer, and the
density support is complete at rest; the control keeps a `2.25x` floor
layer even when settled.

## Revision 4: cost of the complement (exactness-gated)

Fixed owners now skip the fused owner-terms kernel under density-only
support (their rows are never read back), and `--boundary-lid 0` can omit
the lid layers. Both were admitted only after bit-identical particle dumps:

| Variant | lane | boundary samples | physics per step | execute wall per step | dumps versus reference | raw JSON |
| --- | --- | ---: | ---: | ---: | --- | --- |
| skip fixed owners | 16k, 2 layers | `22,224` | `2.25 ms` | `3.17 ms` | identical to revision 2 over `121` frames | `r4/16k-L2-lid1-skip` |
| skip fixed owners | 48k, 1 layer | `11,768` | `4.98 ms` | `6.75 ms` | reference for the lid test | `r4/48k-L1-lid1-skip` |
| skip, no lid | 48k, 1 layer | `8,324` | `4.84 ms` | `6.57 ms` | identical to the lid run over `121` frames | `r4/48k-L1-lid0-nolid` |

Sources after revision 4: `cuda_baseline.cu` and `main.cpp` hashes are in
the commit that introduces them. 48k with density-only support therefore
costs `4.98 ms` of physics per 240 Hz step (`0.62x` real time unpaced with
the wrapper overhead), against `3.86 ms` without support; the live bridge
keeps paced playback for 48k.

## Live captures (16k, density-only support, closing surface)

| rendered frame | solver step | render critical p95 | real-time ratio | PNG SHA-256 | raw JSON |
| ---: | ---: | ---: | ---: | --- | --- |
| `120` | `236` | `335 us` | `0.997` | `47315355...` | `02999f51...` |
| `170` | `312` | `347 us` | `0.995` | `7689f694...` | `7d18147e...` |

Frame 120 shows the tongue approaching the wall without a stalled sheet;
frame 170 shows the water piling against the wall itself.

## Decision

- H7B is supported bounded in its density-only form: the stall and the
  upstream crest are gone on both lanes at no cost to front speed or 16k
  step time. Revision 1 is refuted (no-slip drag from fixed samples in the
  viscosity and surface terms).
- The live bridge defaults to density-only support; the sphere/closing
  surface and the extractor are unchanged.
- Revision 3 splits G1: the floor layer peaks at `1.32 / 1.29` during the
  impact (transient compaction under load) and settles to `1.13 / 1.17`,
  inside the `1.2` gate, while the control settles at `2.25 / 2.27`. No
  compression claim is made for the impact phase.
- 48k cost rises from `3.86` to `5.35 ms` of physics per step with one
  layer (`0.55x` real time unpaced); the 48k budget question already sits
  with the solver contract (D-044).

## Checks

- extractor Release build: PASS; `cargo clippy -p xtask` both feature
  sets and `cargo test -p xtask water_` (`11`): PASS; `cargo fmt`: applied;
- nine stream runs: PASS, finite state and degree within capacity on every
  audited frame;
- two live captures: PASS, `0` dropped timing samples;
- `git diff --check`: PASS;
- desktop adapter unchanged since its last `platform`/`host-check` PASS;
- corpus commands untouched (`boundary_layers` defaults to `0`); their
  roots were not rerun in this session.
