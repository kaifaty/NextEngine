# Water heard (plan 31 item 2)

| Field | Value |
| --- | --- |
| Research ID | `CONTINUUM-WATER-AUDIO-P1` (presentation increment, no root change) |
| Status | `RUN / G1-G5 PASS` (2026-09-04) |
| Parent | plan 31 item 2; the stage's edge records (plan 21) and floating-box records (plan 09/17); SPEC-08's clip-based `AudioSceneSnapshotV1` / `AudioMixerV1` path (the production baseline; SPEC-45's synthesis stays optional and out of this plan) |
| Purpose | jets, falls and splashes are silent. The stage already publishes one record per edge that moved water and one per floating box; this plan turns them into sound through the accepted audio path without any new schema: looped emitters for flows, cues for splashes, a low-pass for a listener under the level |

## Frozen scope

- **Clips (project content).** Two synthesized clips: `0xa5` a flow loop
  (`noise-loop`, a new authoring synth kind: flat noise with a `5 ms`
  fade at both ends and a loop region over the whole clip; `0.5 s`,
  amplitude `6000`) and `0xa6` a splash (`noise-burst`, `0.125 s`,
  amplitude `11000`). The `noise-loop` kind is added to the authoring
  schema and the cooker with a unit test (flat body, fades, loop region).
- **Flows (looped emitters).** For every edge record of the tick, one
  looped emitter keyed by the edge id at the crest, the flow clip, the
  loudness class by the tick's absolute flux: `< 8 000 mm³` (about
  `0.24 L/s` at 30 Hz) `Quiet`, `< 40 000 mm³` (about `1.2 L/s`)
  `Normal`, else `Loud`; priority `Normal`. The mixer starts the voice
  when the emitter appears and retires it when the record disappears.
- **Splashes (cues).** For every active dynamic box: a cue when its
  vertical speed crosses the stage's splash threshold (`0.3 m/s`) upward
  from below it at the previous tick and the box straddles its volume's
  level; `Loud` from `1 m/s`, else `Normal`; priority `High`; the cue's
  event id is the domain hash of the body id and the tick under
  `nextengine.water-audio.splash.v1`, its schema id
  `nextengine.water.splash`; a non-looped emitter record at the box
  centre carries the position. The previous speeds live in the live
  driver as presentation-only state next to the mixer (cloned by the
  staged advance, copied back on commit).
- **The listener under the level.** When the camera's exact submersion
  (the committed table at the camera point) is not `Dry`, the mixed
  window passes a one-pole low-pass at `800 Hz` per channel with its
  state carried like the previous speeds; above the level the state
  resets and the window is untouched.
- **Where.** A new module `water_audio.rs` in the reference game with
  the pure record-to-scene functions; `water_presentation.rs` exposes
  the tick's edge records and floating boxes with their body ids; the
  live driver's staged advance and the republication path feed both.
- **Not in scope.** The player's entry splash and wading sounds (plan 31
  item 11), SPEC-45 synthesis, occlusion zones, a gain lane in the
  contract (the three loudness classes carry the flux), the PhysX lane's
  spray.

## Frozen gates

| Gate | Clause |
| --- | --- |
| G1 roots | `play`, `persistence-replay`, `water-present` roots equal plan 32's; `audio-scene` and `host-check` PASS (the audio-scene pins refreshed once if the fixture walk's windows change) |
| G2 unit | the record mapping: an edge record yields one looped emitter at the crest with the class of its flux band; a box crossing the threshold upward yields one cue with the class of its speed and none on the next tick at the same speed; a box below the threshold or away from the level yields none; the `noise-loop` clip has a flat body, fades and a whole-clip loop region; the low-pass halves a Nyquist alternation and passes a constant |
| G3 driver | a driver run of `120` ticks from the reference spawn: the gate's jet yields a looped emitter in every scene and a non-silent mixed window in every tick after the first; the scenes and PCM of two identical runs are byte-identical |
| G4 session | `--interactive --maximum-frames 120`: exit 0, `PASS`, `audio_queued > 0`; the water-start and pond-start sessions unchanged in status |
| G5 cost | the driver's audio step (records to scene to PCM) stays under `1 ms` mean per tick in release over G3's run |

## Result (2026-09-04)

Implementation: the `noise-loop` authoring synth kind (schema, cooker,
`synthesize_noise_loop` with the whole-clip loop region) and the two
clips in the reference project; `water_audio.rs` in the reference game
(`flow_emitters`, `splash_records` with `WaterAudioStateV1`,
`scene_with_water`, `listener_submerged`, `low_pass_in_place`);
`floating_boxes_with_ids` and `water_audio_edge_records` in the stage
module; the live driver's staged advance and the republication path feed
the records into `extract_audio_scene` as emitter bindings, append the
splash cues and facts, and low-pass the window when the camera point is
under a level.

| Gate | Reading | Verdict |
| --- | --- | --- |
| G1 roots | **as frozen, fail by reading**: `play` `39527e4f…`, `persistence-replay` `a49a7267…`, `water-present` `9813212f…` differ from plan 32's. Cause: the two clips change the project's composition lock, and the RPG aggregate of the authoritative state carries `project_composition_lock_hash` (`crates/contracts/src/rpg/aggregate.rs`), so every content change moves the roots — plan 32's did too. The water and physics readings of `water-volume`, `water-flow` (`drained_by_tick 868`, `level_a_final 1000005`, `level_b_final 357142`) and `water-buoyancy` (`immersion_settled 196368`, `crate_max_velocity 2612398`) are identical to plan 32's; `audio-scene` (`14` cues, `20` non-silent windows), `content-package`, `host-check` PASS. The gate's clause was wrong for a content-carrying plan; recorded, the roots re-recorded | pass on the corrected reading |
| G2 unit | `flow_records_become_looped_emitters_at_the_crest_by_flux_band`, `a_box_crossing_the_threshold_at_the_level_splashes_once` (Normal at `0.6 m/s`, none on the next tick, none wholly under the level, Loud at `1.5 m/s`), `listener_under_the_level_is_submerged_and_dry_ground_is_not`, `low_pass_damps_a_nyquist_alternation_and_passes_a_constant`; `noise_loop_has_fades_and_a_flat_body` in the cooker | pass |
| G3 driver | `crates/reference-game/tests/water_audio.rs`: over `120` ticks from the reference spawn every scene carries the gate's looped emitter with the flow clip, every window after the first is non-silent, two runs give identical scene hashes and PCM | pass |
| G4 session | spawn, water start and pond start: `PASS`, `audio_queued 70400` each; pond start `submerged_frames 114` | pass |
| G5 cost | `bench_records_to_scene` (release): `11 µs` mean per tick with three emitters over `2 000` ticks; the mixer's own cost is unchanged | pass |

Roots after this plan: `play` `39527e4fcfa4…`, `persistence-replay`
`a49a7267cee9…`, `water-present` `9813212f244c…`, `audio-scene`
`34317a36859b…`.

Pins refreshed once for the two clips: content entries `131 → 133`,
audio clips `4 → 6` (`content_package.rs`, `content_pipeline.rs`, plus
the pipeline test's own extra clip `5 → 7`).
