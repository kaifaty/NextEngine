# Nonlocal GPU engine 48k live lane evidence — 2026-09-02

## Result and claim ceiling

`ENGINE_VULKAN_48K_LIVE_PACED_BOUNDED`: the SPEC-38 production fixture size
(`48,000` samples, `4 x 1 x 2 m` basin, `0.75 m` fill) now streams live into
the renderer through the same two-process bridge, but not at real time on
this host. The accepted five-iteration solver spends `3.87 ms` of GPU time
per 240 Hz step (`4.17 ms` budget) and the research `execute` wrapper adds
about `0.9 ms` of host overhead, so the live lane runs at `0.74x` real time
unpaced and plays smoothly when paced at `--stream-rate 0.7`. GPU extraction
stays at `1.4 ms` per frame on a worker thread and is not the limit; the
renderer critical path stays near `0.3 ms`.

Presentation-only, developer tool, Proposed SPEC-38/ADR-076. The 48k stream
lanes use the accepted profile and capacity but their trajectories are not
corpus evidence (float device state, sparse audit). CPU DFSPH remains the
fallback.

## Frozen inputs and source boundary

```text
branch                          codex/water-research
base before change              0936a1c1 (D-043 commit)
extractor cuda_baseline.cu      8b08af9fa225587cb292c5e2a188e3e58866445bd0ada4fbabbdfbec0eae0de5
extractor main.cpp              7f252bd72733dfb84d4eaaf75dcb15f460aa70d4d7e3d185ed85051ab8726a0f
extractor binary                7acbf2bf11c40a296a43c49a8c35a96dfcea98e0083c47ae99faaf31066d496b
xtask water_preview.rs          e86654abc03adb5651bc821495ffd449ba403a728b5db96191372542a2129ce3
release xtask                   003154e959f4ec848d0d629e24b17c682806de2de37bc859cda9be0f55f13c5c
```

The desktop adapter is unchanged since the D-042 commit. The only solver
change is two new stream lanes and a `lattice_y` parameter on the visual
seeding helper; the `4k`/`16k` lanes and corpus commands are unchanged.

## Lanes

| Lane | Box (m) | Seed lattice | Samples | Surface grid | Note |
| --- | --- | --- | ---: | ---: | --- |
| `48k` | `4 x 1 x 2` | `80 x 15 x 40` | `48,000` | `320 x 160` | production basin and fill, released from the visual lanes' `0.1 m` lift so it slams and sloshes |
| `48k-dam` | `4 x 2 x 2` | `40 x 30 x 40` | `48,000` | `320 x 160` | same count as a `2 m` column so the dam-break front is visible at 48k |

Both lanes use the accepted `nuv-basin-48k-analytic-contact-game-cap160.v6`
profile (capacity `160`, five iterations, analytic box contact). Maximum
observed neighbour degree was `140` (`48k`) and `138` (`48k-dam`) over 960
steps, inside capacity.

## Standalone timing (frames to a file, GPU extractor, one worker)

| Lane | steps / every | GPU physics per step | execute wall per step | wall per step | extraction per frame | raw JSON |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| `48k` | `960 / 4` | `3.97 ms` | `5.60 ms` | `5.71 ms` | `1.45 ms` | `ef71f217...` |
| `48k-dam` | `960 / 4` | `4.23 ms` | `5.92 ms` | `6.02 ms` | `1.41 ms` | `bf267dd3...` |
| `48k` | `480 / 480` (no extraction) | `3.87 ms` | `4.80 ms` | `4.80 ms` | n/a | `e03b7e25...` |
| `16k` | `480 / 480` (no extraction) | `1.51 ms` | `1.77 ms` | `1.77 ms` | n/a | `5e7138aa...` |

The wall totals include the first step's `~0.4 s` device warm-up. The
`execute` overhead beyond the GPU event span (`~0.26 ms` at 16k, `~0.9 ms`
at 48k) is host work inside the research solver's timing apparatus (per-call
event creation and stage timing collection, four synchronous flag copies);
it is instrumentation, not physics, and was left unchanged.

GPU extraction verified against the CPU reference on the `48k` lane:
`61` frames, `0` closed-mask or mesh-count mismatches, maximum depth
difference `1.13e-7 m` (`809ab405...`), inside the `1e-6 m` gate.

## Live Release runs (900 rendered frames)

| Metric | `48k` rate `1.0` | `48k-dam` rate `1.0` | `48k` rate `0.7` |
| --- | ---: | ---: | ---: |
| frames received / published / skipped | `253 / 250 / 3` | `242 / 240 / 2` | `239 / 237 / 1` |
| published stream seconds / wall seconds | `4.183 / 5.658` | `4.000 / 5.634` | `3.950 / 5.650` |
| real-time ratio | `0.739` | `0.710` | `0.699` (paced target `0.7`) |
| solver steps before close | `1,040` | `976` | `976` |
| render critical p95 / p99 / max | `310 / 378 / 7,887 us` | `317 / 363 / 566 us` | `321 / 376 / 719 us` |
| refresh-frame upload p95 / max | `216 / 293 us` | `223 / 295 us` | `220 / 298 us` |
| extraction per frame p95 | `1.85 ms` | `1.70 ms` | `1.84 ms` |
| physics per frame p95 | `17.8 ms` | `19.6 ms` | `17.9 ms` |
| engine-owned device allocation | `41,858,448 B` | `40,531,088 B` | `41,858,448 B` |
| raw JSON | `6ccc651b...` | `5c8638c6...` | `9e4b90af...` |

The single `7.9 ms` maximum in the first `48k` run is one frame; its p99 is
`378 us`. At `--stream-rate 0.7` the paced feed no longer waits on the
solver (`1` skipped frame) and the motion is continuous.

## Decision

- Real time at 240 Hz for 48k is out of reach on this host by the solver
  alone (`3.87 ms` of `4.17 ms`) before any bridge cost; no bridge change can
  close it and the physics cadence is fixed by the profile.
- The developer path for 48k is paced playback at `0.7x`; `--stream-rate`
  already provides it and the feed stays continuous.
- Removing the `~0.9 ms` execute overhead (event pooling in the research
  timing apparatus) would bring unpaced 48k to about `0.87x` but not to
  real time; it is a later, separately justified change because it touches
  the campaign's measurement code.

## Checks

- extractor Release build: PASS;
- `cargo clippy -p xtask --features desktop-sdl-ash --all-targets`: clean;
- `cargo fmt -p xtask`: applied;
- standalone `48k`/`48k-dam` GPU streams and `48k` verify: PASS;
- three live Release runs: PASS, `0` dropped timing samples, one dynamic
  draw per frame;
- `git diff --check`: PASS;
- desktop adapter unchanged since its `platform`/`host-check` PASS;
- screenshot/capture: `NOT_RUN`.
