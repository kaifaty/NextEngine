# 03 — Water presentation

## Outcome

Make the accepted water state visible while proving that presentation cadence,
device loss and surface reconstruction cannot change physical results.

## Extraction boundary

After an accepted continuum step, extract a bounded immutable record containing
region/profile identity, source physics tick and either stable samples or a
derived scalar field. The exact public shape is deferred; the lab may use a
private record. Previous/current samples support interpolation. No renderer
buffer, depth texture, camera value or surface mesh returns to physics.

## Candidate render paths

1. debug points/spheres as the mandatory diagnostic path;
2. anisotropic particle splatting plus reconstructed mesh for quality;
3. screen-space fluid as an optional performance path;
4. optional spray/foam derived from vorticity/curvature diagnostics.

Mesh/SDF/foam/wetness state is reconstructible. Missing compute support or
device loss falls back to debug/low-quality presentation while the same
continuum state continues.

## Checks and exit

- render at 30/60/144 Hz from one 30/60 Hz physical trace;
- repeat with extraction disabled, renderer disabled and injected device loss;
- verify identical continuum trajectory/conservation roots;
- bound extraction records and reject stale/profile-mismatched input;
- capture dam break, moving boundary and floating body for human inspection.

Exit requires authoritative equality across presentation variants and a useful
debug view. Pixel equality is not a physical correctness oracle.
