# Nonlocal continuum NP1-P4 evidence — 2026-08-20

Status: `P4_CORRECTNESS_STOP / P2_FINALIST / FIXED_WORK_DECISION_NEXT / REPORT_ONLY`

## Decision

Stop `verlet-skin-p4` before timing and retain P2 as the NP1 finalist. The
displacement certificate proves geometric coverage, but it does not preserve
the retained canonical CSR order when samples cross horizon-grid cell
boundaries. The first reused dynamic step has the same active pair count but a
different ordered CSR; f32 association then changes the accepted state.

This is not evidence that Verlet lists are physically invalid. It is a precise
incompatibility between neighbor reuse and this research root's bit-exact
cell-traversal order. The frozen rule says one repeated association mismatch
stops an exact-work family, so no skin tuning or long tournament was run.

## Artifact identity

- branch: `codex/nonlocal-continuum-n0`
- executable SHA-256:
  `8cbaf460f2f1d2f7eb5427df51cc8ead4ad18a36a5ae7a60962c9841750a0f33`
- denominator: retained P1+P2 stable storage
- candidate: `0.04h` compact superset, double-precision max-displacement
  certificate and exact `h` filter
- added memory: exact `12N + 8` bytes
- candidate capacity: existing 123 fixed / 192 advected neighbor limits
- no adjacent timing admitted after the dynamic correctness stop

## What passed

- CPU self-test and independent CPU gather self-test;
- retained CUDA self-test;
- unchanged v0 profile/input/output identity;
- compact tiny CPU `f64` P4 cold/reused checks;
- stiff surface `gamma=1000`, i2: one cold rebuild plus two reuses, exact
  output and active CSR;
- fixed-state certificate and cache memory accounting;
- dynamic seed cold rebuild: exact output and active CSR.

The stiff result has `1,699,688` active/candidate directed pairs, output
`52a3d852c05b9cc7931a3133816e0ddb1445695b88ea25193d0981022080b65e`
and CSR
`a0304020eeb98889b47c0e179c85aaf4f93fb671bf58f15a99d3e034f1ad4698`.

## Reproduced dynamic failure

Two fresh invocations produced byte-identical failure JSON. The candidate and
retained seed are exact. At step 1 the certificate legitimately permits reuse
with maximum anchor displacement `0.000292574 m`; both paths still contain
`5,471,308` active directed pairs, but their ordered CSR differs:

| Receipt | Retained P2 | P4 reuse |
|---|---|---|
| step-1 output SHA-256 | `fb0642b90b974459615bc32b7edd6df9b46b7f41a37e4b61f5bc9d7900a6fef9` | `be900343fb5a8eef510891cc7bb15c2d3cda42d1093a0951cb6dfc9f1e74b276` |
| step-1 active CSR SHA-256 | `7a2f8afc7cdd2707db023cad779a000313f88815647d8103218911c77056a8db` | seed order `1b176356938914f036682127fbfb322595fb2b6112043bb3f44247c843990392` |
| active pair count | 5,471,308 | 5,471,308 |

At step 2 the `0.000603734 m` displacement forces a rebuild and the active CSR
again matches retained, but state already diverged from the prior association
change. The two complete failure reports both hash to
`a6e9c1ec138bd4ac58c877f9d367ad96a50a65833f158e6acdc4a501a6c5cf4e`.

## Root cause

The geometric proof is sound:

```text
2 * max endpoint displacement <= skin
=> every current h-pair existed in the anchor (h + skin) superset
```

What it does not prove is order stability. The retained builder orders a row
by current relative grid cell and then stable sample ID. A sample can cross a
cell boundary while moving much less than `skin/2`; the active membership is
unchanged, but its reduction slot moves. The cached superset retains anchor
cell order, so exact f32 accumulation is no longer the retained computation.

Three possible continuations were rejected for this stage:

1. require that no sample changes cell — correct, but this lattice places many
   samples near cell boundaries and would eliminate useful dynamic reuse;
2. rebuild/sort active row order every step — restores exactness but largely
   recreates the work P4 is meant to avoid and needs a second active layout;
3. redefine canonical arithmetic as global stable-ID order — plausible future
   architecture, but it is a new numerical root requiring CPU/GPU reclosure,
   not a P4 exact-work optimization.

## Raw hashes

| Artifact | SHA-256 |
|---|---|
| stiff final check | `6964f2adb1b73db6552d22f46fc40500b86520af6d7f3dcbeb6c332e9e88060a` |
| reproduced advected failure | `a6e9c1ec138bd4ac58c877f9d367ad96a50a65833f158e6acdc4a501a6c5cf4e` |
| CPU self-test | `5b007b5dc6aa8e9c18ba580779aae04694b9e54d92da7109a4f5d1ffeb597c91` |
| CPU gather self-test | `0f422b76cbff71e51db8b9dc697e6a295a9a86cc9a1c9574c1fbfc8ced4606f9` |
| CUDA retained self-test | `de6f11b0d5b26d4fa798212efe94571f9ba44f7e448981cd085dfe29d3658adb` |
| v0 repeatability | `9a096d88cb4a9e64b1d491c4a583864f8e7d9fcbb91ca060d8d4e4ade796c40d` |

Raw JSON and binaries remain outside Git under `/tmp`.

## Consequence

NP1 is complete. Its finalist is P1+P2; P3 and P4 remain selectable research
diagnostics but are not retained. The next step is the frozen 64-warm-up,
512-sample fixed-work decision campaign on coherent and advected exact-50k,
followed by NP4 selection or conditional NP2.
