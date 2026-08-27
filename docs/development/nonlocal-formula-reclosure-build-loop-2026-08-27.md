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

## Usage and evidence boundary

Author iteration:

```text
cmake --build /tmp/nextengine-r20r4-build \
  --target nonlocal-formula-reclosure-dev -j 8

/tmp/nextengine-r20r4-build/nonlocal-formula-reclosure-dev \
  --nonlocal-al-generalization-v5-operator-input-factorial
```

Before interpreting or independently reviewing executable research evidence:

1. build `nonlocal-formula-reclosure` with the frozen Release profile;
2. run the focused command twice and freeze diff/source/binary/stdout hashes;
3. compare dev and Release stdout for the exact candidate snapshot;
4. submit only the Release binary/output as decisive evidence.

The dev target is excluded from the default build and has no scientific,
runtime, GPU or production authority.

## Remaining debt roadmap

### BLT2 -- Stable core target

Move the unchanged formula-reclosure sources into a reusable static/object
core and keep CLI dispatch in a thin executable. Preserve every existing CLI
stdout/exit code. This prepares reuse but does not by itself remove the
monolithic `boundary_reference.cpp` invalidation.

### BLT3 -- Frozen parent-fixture API

Expose one narrow, versioned DTO/API for successor probes:

- actual ancestor JSON hashes and parent semantic;
- tangent/common operator payloads and roots;
- factor/permutation/RHS/inverse-scale payloads and roots;
- verifier profile plus live-payload validation;
- independently reconstructed endpoint certificates.

The API must be self-sealed and mutation-tested. Do not expose anonymous
namespace implementation types or make a successor include
`boundary_reference.cpp`.

### BLT4 -- One successor probe per translation unit

Compile R63ZD and later experiments as separate source files linked to the
stable core/API. Editing one probe must not rebuild the ancestor implementation
object. Keep one CLI route per probe and a dedicated focused target when useful.

### BLT5 -- Parent fixture cache

Serialize the immutable parent DTO outside Git with a schema/version/hash and
load it during author iteration. Regenerate it only from a frozen Release
producer. A final evidence run must either regenerate and compare the fixture
or prove byte identity with its frozen producer hash.

This removes the second cost currently hidden by the build problem: every
process repeats the unchanged R63Y--R63ZB parent chain before exercising the
new lane.

## Stop rules

- Do not weaken floating-point flags for frozen evidence.
- Do not treat dev/Release equality on R63ZC as blanket correspondence.
- Do not commit raw generated fixture blobs until schema, size and review
  ownership are accepted.
- Do not spend another research-review round on build tooling; normal code
  review plus deterministic regression checks are sufficient here.
