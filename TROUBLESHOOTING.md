# Next Engine 1.0 — Linux troubleshooting

Run package commands from the package root so the relative `project` and
`source/reference-alpha` paths resolve correctly. Do not move individual files
out of the package: the manifest describes one exact tree.

## No window or Vulkan initialization failure

Confirm that the monitor is powered, uses the selected HDMI/DisplayPort input
and is connected to the Vulkan-capable GPU. In the active desktop session:

```bash
xrandr --current
vulkaninfo --summary
```

`xrandr` must expose a non-zero desktop extent (or the equivalent Wayland
display must be active), and `vulkaninfo` must identify a hardware Vulkan 1.3
device. A virtual/software display is not a supported release substitute.
Update the vendor graphics driver if the package reports a Vulkan loader,
device or required-extension diagnostic.

## Missing system library or incompatible glibc

Inspect `runtime_profile` in `package.manifest.jcs`. It records the exact ELF
interpreter, maximum required glibc version, direct system libraries and stable
diagnostic codes for external Vulkan/desktop prerequisites. Install the
missing distribution package or use a supported Linux system; do not copy
random shared libraries into `bin/`.

## Audio device unavailable

The game attempts to reopen the default audio device and continues with a
declared silent fallback if output remains unavailable. Check the desktop
session's PipeWire/PulseAudio route and selected output device. Audio failure
does not change authoritative gameplay state.

## Save or preference recovery

The game uses the normal Linux user-state location, not the installation
directory. A corrupt preference profile is quarantined and bounded defaults
are used. Save loading rejects a corrupt newest generation, preserves its
original bytes for diagnosis and falls back to the latest complete prior
generation without rewriting it.

If a stale local state is intentionally being discarded during testing, move
that product-specific user-state directory aside rather than editing package
files. Keep a copy if the failure is being reported.

## Stable diagnostics

Commands write a versioned JSON report to standard output and explanatory text
to standard error. Record the diagnostic code, package manifest hash, release
version and the command used. Do not attach saves, credentials, private keys,
personal paths or protected/imported game data to a public report.

Windows binaries and Windows support are not part of Next Engine 1.0. Existing
historical Windows code or results do not describe a supported release target.
