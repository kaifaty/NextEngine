# NSR3-B4E2R -- first-output reference-slice contract

Status: `FROZEN / PASS / FIRST_OUTPUT_REFERENCE_SLICE / NO_TRAJECTORY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2r-first-output-reference-slice|v1|parent=9cf5fc571fee7bc0be5585d9b467f90cd8a27f40d9cf39d6be999b0c466cbccc:60e5575b5e6f5cb332cedda4d59c8ded30960cd5a07a14ccec4cd10ea6c6630e|alignment=c53a112830cb94c4139da75c2044d28f61673cb1aa05c116098967aee41f7bbe:79a8932136a64c1cfa953caafe323cb9e860cb134c1d4dc0907be7f40891587a|payload=ba34b4e3b12986ebc831320d6811551d5311a6774a64f079aabe3a5eaa6bb746;dam:a0b030805034538420e0a5d90390f7117e645119013e9223ff1b334564cbcc13;hydro:6d6b70c3feb6fd583d8740a901cc2577de7e3744d4b6c8dbd76a73d2a2710c2f|slice=dam:step4;hydro:step24;frame1;stable-id6000|parser=standalone-cxx17;no-generator-reader-reuse;openat-nofollow;full-hash-before-parse;max67108864|canonical=ties-even-micrometre;sample-root;sum-position;sum-velocity;q99-rank5940|controls=step,id,position-um,q99-synthetic,file-final-byte|runs=2-builds;2-processes;byte-exact|trajectory=none|timing=none|credit=b4e2d-contract-research-only
```

Identity SHA-256:
`681e6e2aab130e0a461dadf575caac754b0931668027db911512a1edac2cccf3`.

## Authority and executable

R1E PASS and B4E0 PASS authorize this extraction-only stage. Create a new
standalone target under
`crates/continuum-water/tools/nonlocal-reference-slice`. It may share only
`sha256.cpp/.hpp`; it must not link SPlisHSPlasH, the generator parser, the R1E
`reader.cpp`, the Nonlocal solver or canonical publication implementation.

The CLI is exactly:

```text
nonlocal-reference-slice --first-output <absolute-artifact-root>
```

Reject an unknown option, a relative root and any root under `/tmp`. Admit
every path component through `open/openat`, `O_NOFOLLOW`, directory/regular-
file checks and pre/post-read descriptor identity. Reports contain no supplied
host path.

## Frozen input and slice

Read in order `CW-DAMBREAK-001`, then `CW-HYDRO-001`, from profile
`ba34b4e3...a6bb746`. Before parsing, require at most 67,108,864 bytes and the
exact size/hash:

| Scenario | Bytes | Complete SHA-256 | Selected step |
|---|---:|---|---:|
| Dam | 56,501,239 | `a0b03080...cbcc13` | 4 |
| Hydro | 15,920,965 | `6d6b70c3...10c2f` | 24 |

Require `CWREFV2`, exact R1D profile plus scenario manifest, 6,000 samples,
181/51 frames, stride 4/24, frame width 312,156 bytes and exact total layout.
Parse frame zero and frame one without trusting a stored offset. Frame zero
must have step zero, zero diagnostics/features and exact stable IDs. Frame one
must have the selected step, admitted finite diagnostics and exact stable IDs.
No later frame is decoded by this command.

## Canonical slice

Decode each selected sample as stable ID plus binary64 position/velocity.
Require finite values and convert all six components to signed micrometres
under verified `FE_TONEAREST` ties-to-even semantics with overflow checks.

For each selected frame publish:

- exact scenario, step and sample count;
- a domain-separated SHA-256 over all ordered IDs and six little-endian signed
  64-bit canonical components;
- checked signed 64-bit sums for x/y/z position and velocity;
- nearest-rank q99 x/y using sorted integer micrometres at zero-based index
  5,939;
- a domain-separated aggregate SHA-256 over all fields above.

Do not publish or gate on per-particle Nonlocal error, DFSPH pressure/
divergence iterations, density extrema or visual output. Iteration and density
fields remain provenance diagnostics only.

## Controls, builds and result

Before PASS require deterministic rejection/change for:

1. a private final-file-byte xor that fails the complete hash;
2. a selected-step mutation;
3. swapped first two stable IDs;
4. decoded first x plus exactly one micrometre changing the sample and
   aggregate roots;
5. a synthetic 6,000-value q99 set proving rank 5,940 and rejecting the next
   value after a controlled mutation.

Build Release twice with repository warnings-as-errors. Run one fresh positive
process from each build against the same persistent artifact root; require
exit zero, empty stderr and byte-identical LF-terminated reports. Record source
roots, executable SHA/size/Build ID and report SHA in dated evidence.

Any failure preserves R1E/B4E0/SIRDI and authorizes only extractor diagnosis.
PASS selects `FIRST_OUTPUT_REFERENCE_SLICE` and authorizes only B4E2D Dam
first-output simulation research/contract design. It grants no timing/speed,
Hydro trajectory, broad corpus, runtime/GPU/schema or production authority.

B4E2R passes across two independent builds/processes; see the
[dated evidence](../../development/nonlocal-nsr3b4e2r-first-output-reference-slice-evidence-2026-08-22.md).
The selected result authorizes only B4E2D Dam-first pilot research and
contract design.
