# NSR3-B4E2D7R20R63ZJ independent re-review request

Status: `REPAIR_FROZEN / SINGLE_RE_REVIEW_NEXT`.

This packet is the single batched-repair re-review allowed after the initial
independent `NO-GO`. Review the frozen contract, the initial snapshot and only
the repair diff before reading later author synthesis. Do not edit or repair
the candidate. Return findings by severity and a final `GO` or `NO-GO`.

## Frozen identities

| Item | SHA-256 / Git identity |
|---|---|
| Initial reviewed snapshot | `caaa16b4f91a622dd36bf55026529c0853fd6d16` |
| Repair snapshot | `9d3482bfc557cd20367c8082a769f36884bc20fd` |
| Repair parent | `caaa16b4f91a622dd36bf55026529c0853fd6d16` |
| Repair diff | `e644da04a688fa797a3698bed3ff1efd98d0b926dc9352e29a013c70d495b505` |
| Frozen contract blob | `713f9ab948528e5f8db7fc1db2b0daa211d5957425b951a25862e545ca263696` |
| Initial Release stdout | `f7f25439fd05555ceb84c4eb99803fb7d4cf5db625ff58e67422fc607ec049a3` |
| Repaired Dev/Release stdout | `b9ded7d71e0c19aed85ee51f8ba9d923569aeee9c11b02a7060a267672b27f8a` |
| Repaired Release binary | `a468e64c932721234a5a3e8192dd0282b2a7e109867ff33daaff9489c63fd0e2` |
| Parent cache | `23dbf605ad7b6ae12c4cf6a80404ead9617354ff2848bd010b52c7fa7f83bb84` |

Repair-snapshot source blobs:

| Source | SHA-256 |
|---|---|
| `src/boundary_reference.cpp` | `3922f5f7cc317f50146593a7225490b503ab8e66adf251ae32783924448e21cb` |
| `src/formula_probe_api.hpp` | `9a733210f5a543d4316f7b146532ab28cac2d11c2c20b8d2ef980e5447315006` |
| `src/formula_product_precision_localization_main.cpp` | `9b80c264156980d35583eeab7b670ba8c2e3dfdf736680580f901379f9f6a5da` |
| `CMakeLists.txt` | `2b612758b2281eec18b5c1ab02658fede2522ea4de68b426f901564510ac52c8` |

Verify without checking out later documentation:

```bash
git rev-parse 9d3482bf^
git diff caaa16b4 9d3482bf | sha256sum
git show 9d3482bf:docs/plans/nonlocal-nonlinear-solver-research/03b4e2d7r20r63zj-product-precision-localization-contract.md | sha256sum
git show 9d3482bf:crates/continuum-water/tools/nonlocal-feasibility/src/boundary_reference.cpp | sha256sum
```

## Initial findings that must be closed

1. **Callback/product identity:** the old validator admitted any nonempty
   callback root, did not bind product sites/inputs to recurrence states and
   could accept a resealed finite output because the exact audit derived a
   radius from the observed error.
2. **Frozen endpoints:** dense/common rejection was checked only as
   `12+/24-/66?`, without exact recurrence, certificate and sign roots.
3. **Status:** the research note still said `IMPLEMENTATION_NEXT`.

The repair claims to close these findings by sealing one callback identity,
the independent binary128 high/low kernel roots and projected execution root
at each of `Kx0`, `Kp0` and `Kp1`; linking each site to its exact state input
and consuming state; replaying the callback from the frozen fixture; adding
fully resealed callback/input/product controls; freezing dense/common endpoint
roots; and updating the status note. Verify the behavior, not just the
presence of the new fields.

## Required adversarial review

1. Re-derive the callback identity and all three trace-root formulas. Confirm
   every root binds the fixture, tangent, sigma, method, site, input, separate
   high/low products, K2 output and execution root exactly once in the stated
   order.
2. Trace `Kx0`, `Kp0`, `Kp1` back to `state0.solution`, `state0.direction` and
   `state1.direction`. Confirm each state operator root is the corresponding
   executed product and no baseline/verifier/oracle value supplies an input.
3. Inspect independent callback replay. Verify exact binary64 component
   reconstruction, separate binary128 products, K2 projections, canonical
   addition and bit-exact output comparison. Look for shared intermediates or
   self-fulfilling roots that would allow finite drift.
4. Reconstruct the callback-identity, input and product negative controls.
   Confirm they reseal all public dependent roots before validation and still
   reject for the intended semantic mismatch rather than a stale outer hash.
5. Confirm the exact audit remains post-transaction and that its self-derived
   radius cannot authorize a mutated callback now that deterministic replay is
   load-bearing. If it still can, return `NO-GO`.
6. Verify dense and common recurrence roots, all six rejecting certificate
   roots and reject sign root against the actual frozen endpoints. Confirm an
   equal-count but different endpoint cannot reach the causal route.
7. Re-derive candidate work (`3/6`, `1890/612`, `192780/192780`,
   `612/612/306`) and distinguish candidate work from review/control replay.
   Flag any unsealed work that affects the scientific route.
8. Recheck route precedence and result sealing. The only success routes remain
   the two contract outcomes; controls, parents and audits must precede them.
9. Confirm the repair leaves R63ZI, R63ZG and R63ZH stdout byte-exact and does
   not alter the frozen dense product roots when optional callback trace fields
   are empty.
10. State the exact bounded claim and ceiling. A `GO` may authorize only the
    next portable operator-representation discriminator; it grants no
    runtime, GPU, timing, corpus, generalization or production authority.

## Optional clean rebuild

Use at most one detached clean Release build and two focused executions:

```bash
review_root=$(mktemp -d /tmp/nextengine-r63zj-rereview.XXXXXX)
git worktree add --detach "$review_root/tree" 9d3482bf
cmake \
  -S "$review_root/tree/crates/continuum-water/tools/nonlocal-feasibility" \
  -B "$review_root/build" \
  -DCMAKE_CXX_FLAGS=-I/tmp/nextengine-r63zh-boost/usr/include
cmake --build "$review_root/build" \
  --target nonlocal-formula-product-precision-localization-release --parallel 2
"$review_root/build/nonlocal-formula-product-precision-localization-release" \
  /tmp/nextengine-r63zh-parent-cache.bin > /tmp/r63zj-rereview-1.json
"$review_root/build/nonlocal-formula-product-precision-localization-release" \
  /tmp/nextengine-r63zh-parent-cache.bin > /tmp/r63zj-rereview-2.json
cmp /tmp/r63zj-rereview-1.json /tmp/r63zj-rereview-2.json
sha256sum /tmp/r63zj-rereview-1.json \
  "$review_root/build/nonlocal-formula-product-precision-localization-release"
```

Expected stdout is `b9ded7d7...7f8a`. A clean binary may be path-dependent;
record its hash separately. ProductChecks remain `NOT_RUN` because this is a
localized offline C++ discriminator under Proposed SPEC-38/ADR-076.

## Required response

Return:

- snapshot/diff/blob verification;
- findings with severity and exact `file:line` evidence;
- independent build/run hashes if executed;
- `GO` only if every initial finding and contract-load-bearing control closes;
- otherwise `NO-GO`, which ends R63ZJ as `INCONCLUSIVE` under the single
  re-review budget;
- the exact permitted claim and prohibited production inferences.
