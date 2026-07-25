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

## Frozen research annex

Packet 1.5 candidate mechanically copies the committed source blob below without semantic edits:

- Source repository: `https://github.com/kaifaty/OpenGothic.git`
- Source commit: `c56e15f1fa68430eaa618dcc892edc00bff6209d`
- Source path: `docs/physical-avatar-research-spec.md`
- Source Git blob: `ae922aa2a182f14108ae364714dc86072ecccbe8`
- Destination: `docs/architecture/research/physical-avatar-research-spec.md`
- SHA-256: `90533ed15c4c1a5ef41a24f26f4d17cf8c59f467e07619316d3c9744f4d2d79b`

The annex remains a frozen research input, not an Accepted Next Engine implementation decision. Its OpenGothic-specific names describe source provenance and do not cross the importer/runtime boundary.

## License origin

The migrated documents were committed in the OpenGothic repository under its root MIT License, including the notice `Copyright (c) 2019 Try`. The standalone project uses Apache-2.0 for new engine code and documentation while retaining the MIT origin and notice for the migrated material in `THIRD_PARTY_NOTICES.md`.

No conclusion about Gothic game data, trademarks, importer distribution, or
derived assets is implied. A distributable artifact simply excludes material
whose name, license, provenance or redistribution rights are not yet clear; no
architecture packet or separate release-unblocking review is required.
