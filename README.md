# Next Engine

**An open-source, AI-first engine and toolchain for systemic single-player RPGs.**

Next Engine is built for games where characters, quests, factions, combat,
dialogue, and the world itself behave as parts of one persistent system.
Player choices should leave durable consequences, autonomous characters should
act for understandable reasons, and creators should be able to extend the game
without modifying the engine.

AI-first does not mean online-only. The core game must remain deterministic,
playable, and correct without a network connection, an LLM, or an external AI
service. Models can enrich the experience when available; declared in-process
fallbacks keep the game working when they are not.

> **Project status:** Next Engine is in active pre-1.0 development. A playable
> Windows reference alpha and its deterministic headless counterpart work
> locally. Native Linux closure, hard release performance evidence, the public
> creator workflow, and the v1 release are not complete. See the
> [roadmap](docs/roadmap.md) for the current stage and open blockers.

Next Engine is an independent project. It is not an OpenGothic port and it is
not intended to be a general-purpose engine for every genre. The name is
provisional.

## The game we want to enable

Next Engine focuses on systemic RPGs: worlds where rules are shared instead of
being rebuilt as one-off scripts for every quest or character.

- **A world that remembers.** Actions are committed atomically, saved in
  versioned state, and can be replayed to reproduce what happened.
- **Characters with agency.** NPCs can perceive, remember, plan, and act through
  the same world rules as the player, with deterministic behavior available
  offline.
- **AI with boundaries.** LLMs and learned policies may propose or improve
  behavior, but they never become the sole authority over gameplay state.
- **Extensions as a product feature.** First-party mechanics and community
  packages follow the same public APIs, capability checks, and validation path.
- **Physical characters without lock-in.** RPG identity, physical embodiment,
  animation, and motor control remain separate so that one model or backend
  does not define the game.
- **Playable progress first.** Every major increment must make the reference
  project richer, more reliable, or easier to create—not merely add
  infrastructure.

## What is playable today

The current reference project, **Frontier Relay**, is a compact product slice.
On the validated Windows path, a player can:

- explore a streamed multi-region world;
- pick up and equip an item, fight an enemy, interact with an NPC, and complete
  a dialogue-driven quest;
- use the HUD, inventory, journal, pause, save, and load flows;
- continue from saved state while the engine preserves authoritative world
  history;
- run the same gameplay path without a renderer for deterministic validation
  and replay.

The slice also exercises data-driven content, Luau and WebAssembly extension
paths, baseline audio and subtitles, controller fallback, and packaged world
streaming. It is a development alpha, not a finished game or a public v1 SDK.

## Direction to v1

The v1 goal is a small but complete foundation for building and shipping a
systemic RPG:

1. a connected movement, interaction, combat, dialogue, and quest loop;
2. reliable save, load, restart, and read-only replay;
3. the same authoritative simulation in interactive and headless modes;
4. deterministic offline NPC behavior with optional AI enhancement;
5. content and mechanics through data, Luau, and WebAssembly packages;
6. physical characters with a shipping-capable procedural fallback;
7. practical local tools for cooking, validating, inspecting, and packaging a
   project;
8. native Windows x86_64 and Linux x86_64 releases.

A full editor, multiplayer, consoles, mobile platforms, macOS shipping,
runtime model training, and a bundled Gothic importer are outside the v1 scope.

## Try the current alpha

The shortest supported local path is Windows x86_64 with a Vulkan-capable GPU,
an up-to-date graphics driver, and the repository-pinned Rust toolchain.

From Explorer, run:

```text
PLAY_NEXT_ENGINE.bat
```

Or start it from a Rust-enabled shell:

```bash
cargo run --release -p next_game --features desktop-sdl-ash -- --interactive
```

Run the same reference project without a window or renderer:

```bash
cargo run -p next_headless
```

This is a source build and may take a while on its first run. Linux is a v1
shipping target, but the current same-commit Windows/Linux release gate remains
open; do not treat an ad-hoc Linux build as a supported release.

## Learn more

- [Product contract](docs/architecture/00-product-contract.md) — the product
  boundary and v1 promises.
- [Roadmap](docs/roadmap.md) — current progress, next milestones, and known
  blockers.
- [Architecture index](docs/architecture/README.md) — specifications and
  accepted decisions.
- [Contributing](CONTRIBUTING.md) — the local development workflow.
- [Security policy](SECURITY.md) — how to report a vulnerability.

## License

Next Engine source code is licensed under [Apache License 2.0](LICENSE).
Third-party notices and migrated-document provenance are recorded in
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) and
[MIGRATION_PROVENANCE.md](MIGRATION_PROVENANCE.md).

Game installations, protected or imported assets, datasets, checkpoints,
generated model artifacts, captures, caches, and secrets do not belong in this
repository.
