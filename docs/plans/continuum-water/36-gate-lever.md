# The gate lever — a flow edge the player commands (plan 31 item 4, first cut)

| Field | Value |
| --- | --- |
| Research ID | `CONTINUUM-WATER-GATE-P1` (gameplay increment with recorded roots) |
| Status | `RUN / G1-G5 PASS` (2026-09-04) |
| Parent | plan 31 item 4; ADR-103 (the flow network) and its `WaterFlowCommandV1` (`SetGate`, `SetPump`, `SetSource`, used so far by the `water-flow` harness under a Tool principal, D-003); plan 34 (the flow heard) |
| Purpose | the flow network can only be watched. A lever beside the vessels lets the player close and reopen the gate between vessel A and vessel B with the accepted flow command, through the accepted command path (a principal, a stream, a capability, the command log), so the level, the jet and its sound answer the player |

## Frozen scope

- **The lever (content and physics).** A static `0.5 m` cube at
  `x 16.4 m`, `z −0.6 m` (south of vessel B, `0.9 m` ahead of the
  vessels' start), body `0x9f`, the crate mesh with the base material,
  an environment binding. No RPG aggregate: the lever is not an
  interactive object of the core profile but a reference-game
  affordance.
- **The principal.** A reference-game system principal
  `nextengine.reference.water-gate` (`InternalSystem`, like the
  interaction system) with the one grant `WATER_FLOW_CAPABILITY_ID` and
  its own stream from the bootstrap. The command's sequence is the
  target tick (at most one gate command per tick, strictly increasing,
  restart-safe).
- **The toggle.** On the interact action's `Started` edge with no
  dialogue open and the avatar's horizontal distance to the lever within
  `1 m` (the reference interaction reach), the staged advance reads the
  committed gate state (`edge_states[gate]`: opening and record
  revision) and enqueues `SetGate { expected_record_revision,
  opening_permille: 0 if open else 1000 }` next to the root-motion
  command. The runtime validates, orders and logs it like any command;
  a stale revision is rejected, never patched.
- **The prompt.** `LiveHudStatusV1` gains the lever prompt (in reach:
  the gate's state); the HUD status panel shows "E / Pad South  Close
  gate" or "… Open gate" (two text ids in both catalogs).
- **Diagnostics.** `--start-at-vessels` (the existing `at_vessels`
  spawn, in reach of the lever).
- **Not in scope.** Pumps, sources and sinks as play (the same path,
  their own levers later), a lever mesh, an animation of the lever, the
  player's own splash.

## Frozen gates

| Gate | Clause |
| --- | --- |
| G1 checks | `water-volume`, `water-flow`, `water-present`, `water-buoyancy`, `physics-collision`, `audio-scene`, `play`, `persistence-replay`, `content-package`, `host-check` PASS after the one refresh of pins; the new roots recorded |
| G2 unit | the toggle payload from a gate state (open → `0`, closed → `1000`, the state's revision as expected), the reach test (in reach at `0.9 m`, out of reach at `1.1 m`), the prompt text by state |
| G3 driver | a driver run from the vessels' start: the HUD carries the lever prompt with the "close" text; tapping `E` closes the gate within two ticks (the committed opening `0`, no edge record, the flow emitter gone from the audio scene) and a second tap reopens it (opening `1000`, the record and the emitter back); the command log shows two accepted flow commands |
| G4 session | `--interactive --start-at-vessels --maximum-frames 120`: exit 0, `PASS`; the prompt visible on the capture |
| G5 harness | the `water-flow` check (a Tool principal on its own stream) still passes unchanged |

## Result (2026-09-04)

Implementation: `water_gate.rs` (the prompt and the toggle payload from
the committed gate state, the reach test, the command on the water-gate
stream with the tick as its sequence); the lever body and its binding;
the water-gate system principal with the flow-control grant and its
stream in the session; the interact edge in the staged advance enqueues
the command next to the root-motion command; the HUD prompt in the
status panel (`HUD_GATE_ELEMENT_ID`, two text ids in both catalogs);
`--start-at-vessels`.

| Gate | Reading | Verdict |
| --- | --- | --- |
| G1 checks | `water-volume` (`82270ec3…`), `water-flow` (`568fa955…`, `drained_by_tick 868` unchanged), `water-present` (`5a376a0d…`), `water-buoyancy` (`7982b322…`), `physics-collision`, `audio-scene` (`3130730a…`), `play` (authoritative root `1af7bbc8…`), `persistence-replay` (`c7cb4302…`), `content-package`, `host-check` PASS after the pin refresh below | pass |
| G2 unit | `the_toggle_closes_an_open_gate_and_opens_a_closed_one_at_its_revision`; the reach cases live in the driver test (`0.9 m` in reach at the vessels' start, the reference spawn `16 m` away out of reach) | pass |
| G3 driver | `crates/reference-game/tests/water_gate.rs`: the close prompt at the start, one tap closes the gate (opening `0` after the commit tick, no gate record, the flow emitter gone), the prompt flips to open, a second tap reopens (opening `1000`, the record and the emitter back); the out-of-reach driver shows no prompt and a tap changes nothing | pass |
| G4 session | `--interactive --start-at-vessels --maximum-frames 120`: `PASS`; the capture at frame 60 shows "E / Pad South  Close gate" in the status panel; the lever cube stands directly ahead of the avatar and is hidden by it from the third-person camera | pass |
| G5 harness | `water-flow` PASS with its own Tool principal and stream, `drained_by_tick 868` as before | pass |

Pins refreshed once: the command ledger's final hash in the play
acceptance (`bdfba580… → 08502169…`, the water-gate stream is one more
stream of the ledger; the archive root, the identity index root, the
event and tick counts unchanged); rendered objects and indexed draws
`16 → 17`, scene records `18 → 19`, bindings `17 → 18` for the lever.

Observation: the lever shares the crate's mesh and stands south of the
vessels, which sit off the reference floor; a walk from the spawn to the
lever crosses the floor's edge. The vessels' placement predates this
plan (plan 07) and is left as is.
