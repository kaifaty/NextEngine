# Migration provenance

This repository was initialized as a new repository from a committed architecture-document snapshot of OpenGothic. OpenGothic commit history was intentionally not imported. This repository does not contain the OpenGothic runtime, build system, binaries, or game assets.

## Source

- Repository: <https://github.com/kaifaty/OpenGothic.git>
- Source branch: `AI_GOTHIC`
- Source HEAD: `1f86802216609cc3853e3bc668553ec3bb4b8052`
- Prior packet 1.3 context: `56bb8127bf446f776ffd24b39fc1946cf6cfe865`

Only committed objects were read. Uncommitted files in the source worktree were not copied.

## Snapshot mapping

- Path mapping: `docs/next-engine/` → `docs/architecture/`
- Imported history/tags/remotes: none
- Result branch: `main`
- First standalone commit: contains packet 1.4, repository provenance, governance, and bootstrap workspace as one reviewed baseline

## License origin

The migrated documents were committed in the OpenGothic repository under its root MIT License, including the notice `Copyright (c) 2019 Try`. The standalone project uses Apache-2.0 for new engine code and documentation while retaining the MIT origin and notice for the migrated material in `THIRD_PARTY_NOTICES.md`.

No conclusion about Gothic game data, trademarks, importer distribution, or derived assets is implied. Public release remains blocked until the naming, license, and provenance review defined by the architecture packet is complete.
