# NSR3-B4E2D7R20R63ZH independent review request

Status: `REVIEW_REQUEST_FROZEN / REVIEW_NOT_TESTED`.

This packet is the first-pass input for a reviewer who did not author the
R63ZH executable snapshot. It intentionally excludes the author's research
synthesis and requested verdict. Read the frozen pre-execution contract, the
executable diff and the raw output before reading any later R63ZH evidence or
task-state narrative.

Do not edit or repair the candidate during review. Report all initial findings
in one batch. At most one clean rebuild and one focused rerun belong to this
initial pass.

## Frozen input

| Item | Identity |
|---|---|
| Parent commit | `7a09048d0050202c4b329e25cd63eba446d786b4` |
| Author snapshot | `12bd39864931278c5598c3342cbc9cdb1f2bd041` |
| Executable-only diff scope | `crates/continuum-water/tools/nonlocal-feasibility` |
| Executable-only diff SHA-256 | `74fbb15c104369356c0f9c1bb17abbbd204d15a95b8862b17494ec020f62f986` |
| Complete commit diff SHA-256 | `afa28aec5d006256770fdc1275cd002e2965ddd611a6ccabf06bb164c7c34068` |
| Frozen contract blob SHA-256 | `4e8d08a13f3011cc2a11cb52beb0394c1a7f92f53576381facd9957f59e3d54e` |
| Raw dev/Release stdout SHA-256 | `181ac246c7a0d6e1c24543fb70e684a9357a0c183858c371867dcd068fe96722` |
| Release binary SHA-256 | `05f2ff12d44fa8668007808886e09dac8997c2aaafa5ffd584b4f25032fa52f0` |
| Parent-cache SHA-256 | `23dbf605ad7b6ae12c4cf6a80404ead9617354ff2848bd010b52c7fa7f83bb84` |
| Boost development package SHA-256 | `7b89698c907fd5d33ccd439674fa53803139923822181c87b910579827aca379` |

The executable-only hash is deliberately path-scoped. The complete commit
also contains dated research, task-state, plan and roadmap text, so hashing
the unscoped commit diff must produce the separate complete-diff identity
above.

Verify the identities before interpreting output:

```bash
git rev-parse 12bd39864931278c5598c3342cbc9cdb1f2bd041^
git diff \
  7a09048d0050202c4b329e25cd63eba446d786b4 \
  12bd39864931278c5598c3342cbc9cdb1f2bd041 \
  -- crates/continuum-water/tools/nonlocal-feasibility | sha256sum
git diff \
  7a09048d0050202c4b329e25cd63eba446d786b4 \
  12bd39864931278c5598c3342cbc9cdb1f2bd041 | sha256sum
git show \
  7a09048d0050202c4b329e25cd63eba446d786b4:docs/development/nonlocal-nsr3b4e2d7r20r63zh-center-producer-decomposition-research-2026-08-27.md \
  | sha256sum
sha256sum \
  /tmp/nextengine-r63zh-dev.json \
  /tmp/nextengine-r63zh-release-run1.json \
  /tmp/nextengine-r63zh-release-run2.json \
  /tmp/nextengine-r63zh-parent-cache.bin \
  /tmp/nextengine-r63zh-build/nonlocal-formula-center-producer-release \
  /tmp/nextengine-r63zh-deps/libboost1.90-dev_1.90.0-6ubuntu1_amd64.deb
```

The frozen regressions are:

| Raw artifact | SHA-256 |
|---|---|
| `/tmp/nextengine-r63zh-r63zg-regression.json` | `ca2a0f80b692a466aa97b4725fcc0ac3f553db95bb7ddf72494a754a7e2029c9` |
| `/tmp/nextengine-r63zh-r63zf-regression.json` | `be74e412ec6d708128506eeef5cd7284a56a2f150de8e1decb5be12ca394de0e` |
| `/tmp/nextengine-r63zh-r63ze-regression.json` | `4fcb94cd94c176e515aedcdb7d0d724e17bac77ec0a94c215ced2dbf4d8ef4a4` |
| `/tmp/nextengine-r63zh-kernel-regression.json` | `e4ff7dc634c04338d980d61214de0ca95dd9480ee34f4a139e62feb26cd381d3` |

If a frozen object is absent or mismatched, report that fact. Do not replace it
with a similarly named artifact or infer correspondence from a successful
build.

## Contract and source surface

Read the contract from the parent commit, not the post-execution research
file:

```bash
git show \
  7a09048d0050202c4b329e25cd63eba446d786b4:docs/development/nonlocal-nsr3b4e2d7r20r63zh-center-producer-decomposition-research-2026-08-27.md
```

Inspect the complete executable diff and these author-snapshot source blobs:

| Source blob | SHA-256 |
|---|---|
| `src/formula_center_producer_main.cpp` | `d94a223af13125fd0256fc5e3cefb5ce311db3e8c29d0de84c280cdc43f3d950` |
| `src/formula_certificate_detail_main.cpp` | `9575e7b107ee1a63b915e085249883d3242384fdc7ae6467af91d66cd48513bc` |
| `src/formula_probe_api.hpp` | `d316d5c399b69cffb735b9896865031afcbecdc0ff21cae0407602cdad593b65` |
| `src/boundary_reference.cpp` | `5960e1906e8ff910f4ce363fef0a7a0bac5b37c894111b167d9789a76c613e4c` |
| `CMakeLists.txt` | `3a7fc34cb903c753f3190cdef9e46e5933ee1561aa08f4af1c425413ce5a2180` |

The paths are relative to
`crates/continuum-water/tools/nonlocal-feasibility/`. Verify a blob without
checking out the snapshot with, for example:

```bash
git show \
  12bd39864931278c5598c3342cbc9cdb1f2bd041:crates/continuum-water/tools/nonlocal-feasibility/src/formula_center_producer_main.cpp \
  | sha256sum
```

## Required adversarial checks

Check the candidate and controls for the same defect. A successful compiler,
author control or byte-identical rerun is not sufficient.

1. Confirm that the executable implements the exact contract rather than an
   adjacent residual, interval or solution-representation statement.
2. Independently derive the binary64-to-dyadic decode for zero, subnormal and
   normal finite values, including sign and exponent limits. Check zero
   normalization, removal of powers of two, exponent alignment, comparison
   and overflow/failure precedence.
3. Trace both width-two solution expansions from the private DTO back to the
   frozen fixture. Verify component/radius counts, endpoint selection, source
   roots and resealing. The DTO must expose no mutable candidate state.
4. For every exact center expression, verify operand order, the sign in
   `v_i - sum_j M_ij x_j`, row-major matrix indexing and both expansion lanes.
   Confirm that the exact oracle does not consume a candidate center or call
   Dot2Err/candidate reduction helpers.
5. Verify that containment compares each exact center with the intended
   immutable R63Y closed interval and has no fitted epsilon, hidden widening,
   point removal or retry. Check endpoint equality and an interval that really
   excludes its center.
6. Independently derive the exact and finite common maximum-row selection,
   including absolute-value and tie behavior. Check that the selected interval
   excludes zero and that the maximum-row seal binds the vector term, all
   tangent/common products, displacement transport, exact interval and all
   per-column contributions.
7. Verify literal canonical-dyadic equality for
   `r_c-r_t=-M(x_c-x_t)` in all rows. Check subtraction direction, matrix
   transpose faults, row/column loop bounds and that the two sides do not share
   a derived intermediate that could force equality.
8. Re-derive every work count from dimension, lane and loop bounds before
   comparing it with the JSON ledger. Confirm that candidate, operator, solver
   and certificate update counts are zero and that oracle-internal work is not
   credited to them.
9. Inspect classifier exhaustiveness, invalid/nonfinite/exponent failures,
   stale and resealed mutations, sign/transpose faults, interval controls,
   parent-fixture corruption, result sealing and publication precedence.
   Determine whether each control can actually distinguish its stated defect.
10. Verify result/audit/work/control identity coverage and parent R63ZE,
    R63ZF, R63ZG, kernel-smoke and cache identities. Check that the observable
    supports only the contract's bounded fixed-profile correspondence claim
    and no runtime, timing, GPU or production claim.

## Optional bounded rebuild and rerun

Use no more than one clean rebuild and one focused run. The captured author
environment used CMake 4.2.3, GNU C++ 15.2.0, strict `-ffp-contract=off
-fno-fast-math`, OpenMP with one runtime thread, and Boost 1.90 headers from
the frozen package. A detached worktree avoids reviewing later documentation
as if it were part of the candidate:

```bash
review_root=$(mktemp -d /tmp/nextengine-r63zh-review.XXXXXX)
review_tree="$review_root/tree"
review_build="$review_root/build"
git worktree add --detach "$review_tree" \
  12bd39864931278c5598c3342cbc9cdb1f2bd041
cmake \
  -S "$review_tree/crates/continuum-water/tools/nonlocal-feasibility" \
  -B "$review_build"
CPLUS_INCLUDE_PATH=/tmp/nextengine-r63zh-boost/usr/include \
cmake --build "$review_build" \
  --target nonlocal-formula-center-producer-release --parallel 2
OMP_NUM_THREADS=1 \
"$review_build/nonlocal-formula-center-producer-release" \
  /tmp/nextengine-r63zh-parent-cache.bin \
  > /tmp/nextengine-r63zh-review.json
sha256sum /tmp/nextengine-r63zh-review.json
cmp /tmp/nextengine-r63zh-release-run1.json \
  /tmp/nextengine-r63zh-review.json
```

A rebuilt binary may embed build-path-dependent bytes; the required focused
comparison is the stdout payload against the frozen raw output. Record the
rebuilt binary hash separately rather than silently replacing the frozen
author-binary identity.

## Required review response

Return one response containing:

- `snapshot_verified: YES | NO`, with every checked hash and any unavailable
  artifact;
- findings first, ordered by load-bearing severity, each with exact file and
  line or symbol, violated contract clause, and the smallest counterexample or
  reasoning that demonstrates impact;
- `correspondence_review: GO | NO_GO | NOT_TESTED`;
- the exact bounded statement the executable output can support and its
  ceiling;
- commands executed and stdout/binary hashes for the optional rebuild/rerun;
- explicit confirmation that the candidate was not repaired during review.

Use `GO` only when no load-bearing defect remains in the frozen snapshot.
Use `NO_GO` for any defect that can alter the declared observable, exact
identity, interval correspondence, route, sealing or work ledger. Use
`NOT_TESTED` when independence or required frozen input is unavailable. Notes
that cannot affect the bounded claim should be labelled non-load-bearing.
