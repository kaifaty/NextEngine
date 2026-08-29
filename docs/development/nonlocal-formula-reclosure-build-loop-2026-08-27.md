# Nonlocal formula-reclosure build-loop debt

## Outcome first

The immediate author loop no longer requires an optimized rebuild of the
monolithic formula-reclosure target after every probe edit.

`nonlocal-formula-reclosure-dev` is an opt-in target with the same strict
floating-point and warning policy as Release, but `-O0 -g0` for compile
latency. On the current shared host:

| Profile | Incremental compile after the last probe changes | Focused R63ZC run | Total developer loop |
|---|---:|---:|---:|
| Release `-O3` | user-observed roughly 3 min | roughly 55 s | roughly 4 min |
| Dev `-O0` | roughly 34 s | roughly 110 s | roughly 144 s |
| Trial dev `-O1` | roughly 94 s | roughly 60 s | roughly 154 s |

These are developer-loop observations on a shared host, not solver-performance
evidence. The structural fact is stable: editing the final `.inc` rebuilds one
160,821-line / 7.75-MB translation surface today.

The current R63ZC dev and Release stdout are byte-identical at
`e59c1948892b12f30e7b249c20662792e06372d8eacc280806e2d98a368b8fa3`.
This correspondence is a regression check for the current snapshot, not a
general theorem that optimization level cannot change future floating-point
results.

The implementation core is now a reusable static target. A thin
`nonlocal-formula-probe-api-smoke` executable links the dev core, reproduces
the pre-split CLI regression hash
`d6ba5f8e802966c25283d0c8384ed01beec20b347acb343cf5c7c2bf360d69d9`,
and proves the intended incremental build graph: after changing only its
source, Ninja compiles that one source and relinks without rebuilding
`boundary_reference.cpp`. The first smoke observed about 0.3 s and the final
kernel-smoke rebuild observed about 0.5 s on the current host.

The versioned R63ZC parent fixture is now self-sealed by live payload roots,
ancestor identities and reconstructed endpoint certificates. Its external
author cache has fixture root
`7780543a21d3b32e39a1fd18e5056f61c610d69929b4c6b69640075d1e7c4553`,
file SHA-256
`23dbf605ad7b6ae12c4cf6a80404ead9617354ff2848bd010b52c7fa7f83bb84`
and size `1,033,625` bytes. A validated cache read observed about 2.2 s instead
of replaying the roughly 100-second parent capture. Truncation is rejected.
The cache remains outside Git and has no evidence authority by itself.

`nonlocal-formula-probe-kernel-smoke` consumes only that cache plus the stable
API. It verifies the frozen factor solve, tangent/common products, scalar dot,
R63Y certificate, invalid-dimension controls and consistently resealed
RHS/common/endpoint and tangent/factor/scale/profile mutations. The final
repaired smoke stdout SHA-256 is
`e4ff7dc634c04338d980d61214de0ca95dd9480ee34f4a139e62feb26cd381d3`.
This describes build/integrity tooling only and is not solver-performance
evidence.

R63ZD now proves the intended successor build graph. Its source is a separate
translation unit with dedicated dev and Release targets. Editing it compiles
only that source and relinks against the stable core. The Release producer
regenerated a cache byte-identical at file SHA-256 `23dbf605...bb84`; two
R63ZD Release runs and the author run are byte-identical at stdout SHA-256
`90c73a8a...9461` for the repaired frozen snapshot.

## Usage and evidence boundary

Author iteration:

```text
cmake --build /tmp/nextengine-r20r4-build \
  --target nonlocal-formula-operator-schedule-dev -j 8

/tmp/nextengine-r20r4-build/nonlocal-formula-operator-schedule-dev \
  /tmp/nextengine-r63zc-parent-v1.bin
```

Before interpreting or independently reviewing executable research evidence:

1. build the Release cache producer and focused successor target;
2. regenerate the parent cache and require byte identity with its frozen
   fixture root and file hash;
3. run the focused command twice and freeze source/binary/cache/command/stdout
   hashes;
4. compare dev and Release stdout for the exact candidate snapshot;
5. submit only the Release binary/output as decisive evidence.

The dev target is excluded from the default build and has no scientific,
runtime, GPU or production authority.

## Remaining debt roadmap

### BLT2 -- Stable core target (`COMPLETE`)

Move the unchanged formula-reclosure sources into a reusable static/object
core and keep CLI dispatch in a thin executable. Preserve every existing CLI
stdout/exit code. This prepares reuse but does not by itself remove the
monolithic `boundary_reference.cpp` invalidation.

### BLT3 -- Frozen parent-fixture API (`COMPLETE`)

Expose one narrow, versioned DTO/API for successor probes:

- actual ancestor JSON hashes and parent semantic;
- tangent/common operator payloads and roots;
- factor/permutation/RHS/inverse-scale payloads and roots;
- verifier profile plus live-payload validation;
- independently reconstructed endpoint certificates.

The API must be self-sealed and mutation-tested. Do not expose anonymous
namespace implementation types or make a successor include
`boundary_reference.cpp`.

### BLT4 -- One successor probe per translation unit (`COMPLETE`)

Compile R63ZD and later experiments as separate source files linked to the
stable core/API. Editing one probe must not rebuild the ancestor implementation
object. Keep one CLI route per probe and a dedicated focused target when useful.

### BLT5 -- Parent fixture cache (`COMPLETE`)

Serialize the immutable parent DTO outside Git with a schema/version/hash and
load it during author iteration. Regenerate it only from a frozen Release
producer. A final evidence run must either regenerate and compare the fixture
or prove byte identity with its frozen producer hash.

This removes the second cost currently hidden by the build problem: every
process repeats the unchanged R63Y--R63ZB parent chain before exercising the
new lane.

The Release producer now regenerates the external cache. For R63ZD it
reproduced both fixture root and file hash exactly before the two decisive
Release executions. The cache remains uncommitted and has no standalone
evidence authority.

## Stop rules

- Do not weaken floating-point flags for frozen evidence.
- Do not treat dev/Release equality on R63ZC as blanket correspondence.
- Do not commit raw generated fixture blobs until schema, size and review
  ownership are accepted.
- Do not spend another research-review round on build tooling; normal code
  review plus deterministic regression checks are sufficient here.
