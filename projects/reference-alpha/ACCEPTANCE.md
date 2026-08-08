# Frontier Relay manual acceptance

## Architecture-cleanup accepted package — 2026-08-08

**Result:** `PASS`

The fresh Windows package produced after Architecture Cleanup passed the full
Frontier Relay manual flow without debug commands. The accepted immutable
inputs are:

- source baseline:
  `3825ab919f5d67756ea2a7001ebfdd1c7f89ddd1`;
- package manifest SHA-256:
  `71599dbb989116f34d650b9d7f8c6deec4886130459d71eee5cb798d7a15822e`;
- packaged `game` binary SHA-256:
  `3333c1b98fb8ed11c5b46219ba5cec30061888a8729af891cd0e76535e270844`;
- project composition lock SHA-256:
  `73a52631264237bd01c5d5ce5f0a6bce65d546ad081818cf107835aae4b32c2a`.

The run verified quest acceptance and `Active` journal recovery; pickup and
equipment through the production movement/contact path; Raider health
`100 → 50 → defeated`; separate combat and activated-relay Save/Load cuts;
explicit Resume and authoritative WASD after Load; final `Completed` journal,
active cyan relay and `Frontier Relay restored` action panel. All four visible
rock colliders stopped continued movement at their inset silhouettes, and the
window passed real resize plus fullscreen enter/exit. The packaged process
closed normally with `status = PASS` and `close_result = Saved`.

This record closes the manual acceptance item for Architecture Cleanup on
Windows. It does not claim the unpublished native-gate bundle whose performance
preflight was not ready, Linux/R1 cross-target closure, performance blocker
B-12 or v1 shipping.

## Prior accepted R2 package — 2026-08-08

**Result:** `PASS`

The complete 20–30 minute Windows acceptance run passed for
`r2-reference-alpha-visual-v5` without debug commands. The accepted immutable
inputs are:

- source baseline: `fe87ae3`;
- package manifest SHA-256:
  `fe903b4ae8dd225d1636e4483e11e3767ef516dc04661b3b995f3e63b4e0a138`;
- packaged `game` binary SHA-256:
  `50e4fb7f2fbc9b38c75acc623c0cf2fe3e8b0a3f902088a5c59e2bf5e810534a`;
- project lock SHA-256:
  `84742a69c5b32e10fb5093d40c89f7e06f4e3aa6ccbf35feeca78cbb5a20a828`.

The run verified Save confirmation, an observable world change, Load rollback,
explicit Resume and authoritative WASD movement; visible collider boundaries,
all four rock collisions, combat/relay state, UI, resize and fullscreen also
passed. This record closes the Windows R2 manual gate. It does not claim Linux
or R1 cross-target closure and does not close performance blocker B-12.

Target duration: 20–30 minutes on the Windows `game` root. The short automated
flow exercises the same stable action IDs and committed RPG/world outcomes;
this run adds human pacing, exploration, UI and recovery observation.

Keyboard controls: `WASD` move, mouse moves the camera, `E` interacts,
`Q` picks up the focused item, `R` equips/uses it, and `F` performs melee.
`I` opens inventory, `J` opens the journal, and `Escape` opens/closes the pause
menu; use the arrow keys and `Enter` to choose Save, Load, Resume, or dialogue
options. A generic controller maps to the same semantic actions.

At launch the world must no longer read as an empty plane: the tiled route,
relay platform, two ruined pillars and four-rock field form the approach to the
gate. A separate cyan HUD panel shows the next production action. It progresses
through talk, pickup, equip, combat, relay, return and completion; it is a
read-only projection and never bypasses normal targeting or command validation.

The relay collision follows its visible left/right pillars, top beam and central
switch. The open space around the structure is traversable: there must be no
invisible wall extending beyond the visible gate. A defeated sentry remains
visible with a dark material while its physical body is still solid; no hidden
solid NPC collider should remain in the route.

To verify Save/Load unambiguously, select **Save game** and press `Enter`.
The selected label must change to **Saved**; the pause menu intentionally stays
open so you can choose Resume or Load. Resume, create an obvious authoritative
difference (for example, move to another place and advance the quest), then
open the pause menu and choose **Load game**. The menu closes and the player,
quest and world state return to the saved point. Loading immediately after
saving can look unchanged because the restored state is exactly the current
state.

1. Start a clean `reference-alpha` project and speak to the relay keeper NPC.
   Accept **Frontier Relay** and save/reload. The journal must remain `Active`.
2. Explore the start zone, find the worn relay blade, pick it up and equip it.
3. Cross into the frontier zone. Defeat the single hostile sentry, then
   save/reload; its health/result must not roll back or duplicate.
4. Activate the relay switch and save/reload. The switch must remain active.
5. Return to the keeper, finish the dialogue and complete the quest. The
   journal must show `Completed`, the relay remains active and replay roots
   must match the scripted `game`/`headless` run. The action panel must finish
   with `Frontier Relay restored`.

Keyboard/mouse is always available. A connected generic controller uses the
same action IDs; disconnecting or starting without one falls back to keyboard
bindings without changing project or gameplay state.
