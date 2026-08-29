# NSR3-B4DR1E reference-attestation research -- 2026-08-21

Status: `COMPLETE / INDEPENDENT_READER_SELECTED / IMPLEMENTATION_NEXT`

## Question

How can the new R1D external trajectories become a trusted comparison oracle
without trusting their generator, filenames or report hashes?

## Findings

Reusing generator deserialization would preserve a common-mode bug: the same
incorrect layout assumption could write and read identical bytes. R1E should
therefore be a separate small C++17 executable with no SPlisHSPlasH link and
no dependency on generator source. Sharing the reviewed SHA-256 primitive is
acceptable; format parsing, bounds, cadence, IDs, finite checks, q99 and
canonical reconstruction must be independently implemented.

Path validation also needs a stronger boundary than `std::filesystem` checks
followed by a separate open. On Linux, open the absolute artifact root and
each fixed path component with `open/openat`, `O_NOFOLLOW` and directory/file
mode checks. Read the final regular file through the admitted descriptor,
compare pre/post `fstat`, and allocate only after the exact size is below the
64 MiB cap. This closes symlink and basic path-swap ambiguity. Windows remains
out of scope.

A complete SHA proves byte identity but not that the reader understood the
layout. Independently parse every integer/binary64 field and canonically
re-encode the decoded sample/frame stream. A domain-separated semantic root
over that reconstruction must match a root frozen from each R1D payload.
Then flip one bit in the first decoded x value and prove that semantic root
changes. Separately flip the final serialized byte in a private copy and prove
the complete-file hash rejects.

The reader can independently regenerate q99-x, q99-y and receiver-count roots
from decoded frames. These must match the R1D aggregate roots. Contact between
non-output steps cannot be re-derived from sampled payloads and is deliberately
not re-claimed by R1E; the full generator hash and R1D evidence own that fact.

## Selection

Create standalone research tool
`crates/continuum-water/tools/nonlocal-reference-reader`. Freeze generator
source/build roots, all actual payload/manifest/semantic/aggregate roots and
the exact reader failure order before implementation.

Positive attestation must execute twice byte-identically against an explicit
absolute non-`/tmp` artifact root. External negative fixtures must prove
missing/symlink/oversized/full-file-mutation rejection without modifying the
three published references. Only R1E PASS may select the new external DFSPH
reference candidate and authorize B4E contract design.
