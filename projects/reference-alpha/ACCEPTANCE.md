# Frontier Relay manual acceptance

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
