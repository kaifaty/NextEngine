# NSR3-B4E2R first-output reference-slice research -- 2026-08-22

Status: `COMPLETE / STANDALONE_SLICE_EXTRACTOR_SELECTED / NO_TRAJECTORY`

## Question

What is the smallest honest step from the selected SIRDI nominal research
path to a multi-macro physical comparison when short-margin performance
selection is no longer admissible on the shared host?

## Performance and correctness are different boundaries

Q4 proves that this host cannot resolve a three-percent CPU selection gate.
That stops another optimization A/B; it does not make deterministic physics
execution invalid. The accepted SIRDI path reduced the historical one-macro
execution by an order large enough that a coarse watchdog can bound a
first-output pilot without interpreting elapsed time as throughput.

The old B4E2 outline still mixes two independent risks:

1. first-time decoding of 16/56 MiB external `CWREFV2` payloads;
2. four Dam or twenty-four Hydro nonlinear macro transactions.

If those happen in one command, a path/parser/layout failure can be reported
beside a partially advanced trajectory and be mistaken for a physics result.
The reference bytes are already fully attested by R1E, so the missing bridge
is a small immutable comparison slice, not another full reference generator.

## Selected boundary

Add a new standalone C++17 extractor, separate from both the generator and the
R1E reader implementation. It shares only the repository SHA-256 primitive.
It opens the persistent content-addressed root descriptor-safely, requires the
exact Hydro and Dam file sizes and complete hashes, and parses only:

- frame zero, to prove the expected initial schedule and stable IDs;
- Dam frame one at macro step 4;
- Hydro frame one at macro step 24.

The remaining payload does not need a second semantic parse: the complete-file
hash already binds it to the independently full-parsed R1E artifact. Exact
total layout, frame width and schedule arithmetic still have to prove that the
selected offsets belong to the admitted file.

For every selected frame, independently convert position and velocity to
signed micrometres with ties-to-even and checked finite/range admission. Emit:

- a domain-separated root over all 6,000 stable IDs and six canonical integer
  components;
- checked integer position and velocity sums;
- nearest-rank q99 x/y at rank 5,940 (zero-based index 5,939);
- frame diagnostics as reference provenance, not cross-solver iteration gates;
- one aggregate root over the scenario, step, sample root, sums and q99.

This is sufficient for the already selected B4E comparison observables while
keeping per-particle error and DFSPH iteration/density values out of scope.

## Failure and negative policy

The extractor runs no Nonlocal transaction. Missing/symlink/non-regular,
capacity, hash, manifest, header, step, stable-ID, non-finite or canonical
range failures reject before publication. It also proves controls for a final
serialized-byte mutation, wrong selected step, swapped stable IDs, a one-
micrometre decoded-position mutation and the rank-5,940 q99 rule on a synthetic
strictly ordered set.

Two independent Release builds and one fresh process from each must emit
byte-identical path-free reports. No duration or RSS is part of the semantic
report or grants speed credit.

## Why this reopens only bounded physical design

A B4E2R PASS will freeze immutable external first-output slice roots. It may
authorize a Dam-first B4E2D simulation contract, followed conditionally by a
Hydro B4E2H contract. It does not authorize either trajectory by itself, the
broad B4E3/B4E4 corpus, runtime/GPU/schema integration or production claims.

The Dam pilot remains first because it reaches its first output after four
macro steps and exercises free-surface release. Hydro is twenty-four macros
and may run only after the Dam command proves transaction orchestration,
canonical state handoff and comparison plumbing.

## Decision

Freeze B4E2R as a timing-free standalone reference-slice extractor. Do not
implement a multi-macro solver command or embed observed reference values
until two independent extractor builds/processes pass the frozen contract.

