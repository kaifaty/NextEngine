# Next Engine 1.0 — Linux package quick start

Next Engine 1.0 supports native `x86_64-unknown-linux-gnu` only. Windows is
outside the current product scope indefinitely. The release package is
self-contained apart from the system libraries and graphics/audio services
listed in `package.manifest.jcs`.

## Requirements

- 64-bit Linux with glibc 2.35 or newer;
- an X11 or Wayland desktop session for the interactive game;
- a Vulkan 1.3-capable GPU and vendor driver;
- the system runtime libraries listed under `runtime_profile` in
  `package.manifest.jcs`.

No network service, LLM or external AI process is required. The authoritative
gameplay path and NPC fallback remain local and deterministic.

## Start the packaged game

Extract or copy the complete package to one directory, open a terminal in that
directory and run:

```bash
./bin/next_game --interactive --project project
```

The game creates user saves and preferences through the normal Linux user-state
location; it does not write operational state into the package directory.

For a displayless deterministic run of the same cooked project:

```bash
./bin/next_headless --project project
```

To validate the frozen public authoring source with the packaged creator tool:

```bash
./bin/next project validate --project source/reference-alpha
```

Each command emits a versioned JSON result on standard output. A successful
command reports `"status":"PASS"`; failures return a stable diagnostic code
and a non-zero exit status.

## Package integrity and dependencies

`package.manifest.jcs` contains the exact file inventory, hashes, release
version, Linux ABI/runtime profile and clean-install receipts.
`DEPENDENCY_INVENTORY.jcs` lists the selected locked Cargo dependency closure,
checksums, license expressions and the copied files under
`THIRD_PARTY_LICENSES/`.

See `TROUBLESHOOTING.md` for display, Vulkan, audio, state and diagnostic
guidance. Creator workflows beyond the frozen reference project are documented
in the repository's Creator SDK guide; they are not required to play the
packaged reference game.
