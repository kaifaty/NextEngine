# NSR3-B4DR0 external-reference recovery audit -- 2026-08-21

Status: `PASS / NEW_REFERENCE_PROFILE_REQUIRED / B4E_BLOCKED`

## Result

The read-only audit selects `NEW_REFERENCE_PROFILE_REQUIRED`. Neither the
three exact W0I payloads nor the exact generator lineage needed to recreate
them is retained locally or publicly discoverable by its recorded hashes.

## Evidence

| Recovery path | Evidence for | Evidence against | Result |
|---|---|---|---|
| exact payload | frozen paths, byte counts and SHA-256 values remain known | no file in `/tmp`, user/worktree/download/trash locations; no Git blob has either exact payload size | unavailable |
| exact generator | pinned upstream commit, high-level contact algorithm, source/diff/binary hashes and GCC version remain known | upstream checkout is absent; no adapter source/patch/binary file is retained; none of 98 unreachable blobs matches any recorded generator SHA-256 | unavailable |
| public restore | upstream commit remains accessible from the official repository | exact searches for all payload hashes and `hard-contact-final.bin` return no result | diagnostic negative only |
| fresh reconstruction | local GCC is still `15.2.0`; the documented physical/geometry intent is sufficiently detailed to design a new adapter | exact C++ ordering, patch bytes and original binary are not derivable from hashes or prose | feasible only under a new identity |

The current local CMake is `4.2.3`; the original report recorded a Release
build with `USE_DOUBLE_PRECISION=ON`, `USE_AVX=OFF`, GCC `15.2.0` and
`OMP_NUM_THREADS=1`, but not a complete hermetic dependency/toolchain image.
Matching the compiler version therefore does not restore the original binary.

The [official pinned repository](https://github.com/InteractiveComputerGraphics/SPlisHSPlasH/tree/eccce86155776f6ac52d5080b1f720a52bf29450)
is still accessible and documents SPlisHSPlasH as an MIT-licensed SPH library
with bundled external dependencies and a CMake build. That supports
feasibility of a new external build, not identity with the lost custom adapter
or payloads.

## Irreducible missing input

The first irreducible historical input is the actual adaptation diff with
SHA-256
`4effa812553649c89135ca9515aaac410e47eca1fe250890766c0d6183165a4b`.
Without its bytes, the comparator source and binary hashes cannot be reproduced
or audited; without one of those exact paths, regenerated payloads cannot
inherit the W0I root.

## Rejected shortcuts

- Reconstructing binary payloads from published RMSE/maxima or trajectory
  roots: those aggregates do not contain the per-output particle positions.
- Treating prose-equivalent contact code as the old source: floating order,
  tie handling and build bytes are identity-relevant.
- Using the old non-clearance hydro/dam files: W0I already rejects their
  geometry.
- Declaring B4D PASS from the manifest hashes alone: availability and reader
  mutation gates remain unexecuted.
- Reusing W1 credit for newly generated data: a new comparator must receive
  new provenance, payload and aggregate roots.

## Decision

Preserve historical W0I/W1 evidence and B4D's exact failure. Freeze B4DR1 as a
new, reproducible external generator profile before cloning or building the
pinned upstream project. The new profile should track only engine-owned
adapter/patch material and manifests in NextEngine; upstream source, binaries
and generated trajectories remain external. It must make the durable artifact
location content-addressed rather than rely solely on ephemeral `/tmp`.
