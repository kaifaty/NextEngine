# NCGP1 matrix-free operator evidence — 2026-08-30

| Field | Value |
| --- | --- |
| Research ID | `NCGP1` revision 1, operator checkpoint |
| Result | `AUTHOR_SUPPORTED_BOUNDED / MATRIX_FREE_OPERATOR_PASS` |
| Independent review | `NOT_RUN` |
| Product status | `REPORT_ONLY`; no nonlinear solve, trajectory or performance claim |

## Outcome

The scalable CUDA path evaluates the corrected FCR density, f64 scalar energy,
f32 gradient, diagonal and matrix-free HVP without materializing a Hessian.
Pressure includes both `J^T J` and the active geometric term; viscosity consumes
the independently rebuilt reference graph; surface and pressure consume the
current graph. Every output row is owner-gathered and uses no floating endpoint
atomics.

On the frozen active 100-sample combined cluster:

| Metric | Result | Gate |
| --- | ---: | ---: |
| CUDA/reference energy relative error | `1.59538558384e-6` | `<=1e-4` local correspondence |
| CUDA/reference gradient relative L2 | `1.04709669781e-6` | `<=1e-3` |
| CUDA/reference HVP relative L2 | `9.33383431823e-7` | `<=1e-3` |
| CUDA/reference HVP cosine loss | `2.14939177567e-13` | `<=1e-6` |
| active pressure centers | `100`, exact | exact |

The separately retained NCGA2 long-double dense assembly is linked only as an
independent oracle. A new independently written CPU matrix-free evaluator
matches it at:

- total energy relative error `1.28300750663e-16`;
- gradient relative L2 `8.07812376863e-17`;
- HVP relative L2 `3.8620648814e-16`.

An energy-only central derivative control at `epsilon=5e-5` gives first and
second directional errors `1.81394111829e-7` and `8.14572666058e-8`.
The preliminary `epsilon=1e-4` apparatus produced a first-derivative truncation
error `7.25576e-7`; it was not counted as a formula failure. The single
apparatus repair halves the step and is now immutable for this lineage.

## Negative identities and work

The common comparator rejects all exercised wrong identities:

- missing physical kernel chain `2/h`;
- owner-only pressure without neighbor-center contribution;
- complete HVP sign inversion;
- half viscosity force/curvature;
- wrong surface sign;
- current/reference viscosity graph swap.

The corrected evaluation work root is
`a7d23588e39ade50819491bdf8c2679ede53b320e8914461c8f4dd651568fda7`.
Graph strict-radius and capacity controls remain in the preceding checkpoint.

## Identity and repeatability

| Artifact | SHA-256 |
| --- | --- |
| public header | `15210868f84563fc2e10c63a63b08ee69b7442c65857819d1a7ea923091a786d` |
| CUDA graph/operator | `838ca48f747ef9020832e0b59fc6fb419b5dbcc16a3bae657a2e293d7e9d737f` |
| independent matrix-free reference | `9d40f99942f31905a5aa717786e28e187e038c01dad4bcb4e61d64ffecea9005` |
| harness | `b8dc9c8614839487a1a7b67822d5ec4fde0509497c329c3cf878aeca6ec5504d` |
| retained dense oracle source | `2da2e479abf14d083d916c7b7998930987807eb73516f723a9891da4f98e04ac` |
| clean Release binary A/B | `a9624619c0a6d36cb3edb375509a892994d89273d147039c1ec14b02fc14a92d` |
| exact graph stdout A/B | `9ea37466b2b5cd99203d6341094da73ab9b47ff6eaa38053525cd6f4215caf00` |
| exact operator stdout A/B | `bed9183e1314f4db8a9bfdf9193d6fe69b15158764027c53dfe5365b3fec5961` |

Fresh build directories were
`/tmp/nextengine-ncgp1-operator-a.haDNmO` and
`/tmp/nextengine-ncgp1-operator-b.Jsn3ju`. Both binaries and both semantic
reports are byte-identical. Raw build products remain outside Git.

## Ceiling and next action

This is bounded author correspondence for one tiny active combined state. It
does not prove trust-region convergence, boundary behavior, physical
trajectories, 50k cost or game-frame suitability. The next gate is the
matrix-free Steihaug--Toint step, starting unpreconditioned on the retained
tiny cases and only then using the frozen positive Jacobi profile.
