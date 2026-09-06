# NCGP15 unified constrained Nonlocal evidence

Date: `2026-09-01`

Final result: `APPARATUS_INCONCLUSIVE / MUTATION_CORPUS_REDESIGN_REQUIRED`

Claim ceiling: `NO PHYSICAL OR CUDA CLAIM`

## Question and bounded answer

NCGP15 implemented one CPU long-double position-space minimization of inertia,
corrected normal viscosity and corrected surface energy subject to explicit
unilateral density and analytic box constraints. It used a CSR candidate, a
separately written all-pairs oracle, physical permutations, exact work
receipts and transactional publication.

The actual corrected term corpus is encouraging but not yet admissible as a
physics result. After one permitted apparatus repair, every retained formula,
normal/tangential viscosity, surface pair, translation, tetrahedron gradient,
zero-coefficient, energy and rollback control passes. Phase B and Phase C are
still `NOT_RUN_BY_PRECEDENCE`, however, because three mandatory mutation
controls do not have valid discriminating fixtures or rejection semantics.
The result is therefore apparatus-inconclusive; it neither supports nor
refutes the unified constrained water formulation.

## Frozen identity

- Revision-2 contract commit: `593801cd863aac9a335a21b56bdf24c315320a49`;
- contract SHA-256/root:
  `edce0d46d8ea1772e7c5ee6d5af21a46b27d5ebc623471cbfaf530032c5b6cf6`;
- implementation commit: `a5e16ffa5f5ee3f8f14166ecde2dcb4d1a91ed49`;
- one-line apparatus repair commit:
  `9aaf9a3a3712569f836825c157fba8a9bb236913`;
- repaired tree: `ee74b17cc44da8f38dbb6d166f1abe73ea79c85f`;
- repaired source root:
  `cbe76b698852d617cedb1a25c8b2c9c0cac4cc5cb3db266dcbb6e5799ffdfdca`;
- repaired Release binary SHA-256:
  `73614eb969b007b3fbc749c21d46a4dd16c3846140bb7eb4d76d660660dc3923`;
- generated compile-command root:
  `5dbacb5779c09eaeab44a1c4379d56d7182d34e5409449766b341d7634172a28`.

Author build directories outside Git:

- initial: `/tmp/nextengine-ncgp15-release-a.iAooKJ`;
- repaired: `/tmp/nextengine-ncgp15-repair-a.sTJDPN`.

## Exact command

```text
cmake -S crates/continuum-water/tools/nonlocal-feasibility \
  -B <fresh> -DCMAKE_BUILD_TYPE=Release
cmake --build <fresh> \
  --target nonlocal-corrected-cpu-unified-constrained -j2
<fresh>/nonlocal-corrected-cpu-unified-constrained \
  --unified-constrained-surface-viscosity
```

## First run and permitted repair

The initial committed candidate built successfully and exited `2` with empty
stderr. Its stdout was `553193291` bytes with SHA-256
`bb16a72c285103160a7b077d276485e172759c5616e9ef576da1cb1e4404096f`,
result root
`b6f2f8cde51ca563e52032a5255a5f7c859faddd5df92e3e54dbc2ed837cf89f`
and identical expected/actual total-work root
`68035c5d24810f136bb5f5eb10e78ce239efff115db65ae2a84d92525f553c6d`.

The first failure was purely apparatus-side. A valid unbounded final KKT trace
contains `MULTIPLIER_UPDATE -> PROJECTION -> FINAL_GATE -> TOPOLOGY`, while
the trace grammar omitted `PROJECTION -> FINAL_GATE`. This made 49 otherwise
valid traces fail and prevented the rollback control's normal commit. The one
allowed repair changed exactly one grammar alternative; it did not change an
equation, fixture, coefficient, tolerance, solver schedule or work cap.

## Repaired run

The repaired candidate again built cleanly and exited `2` with empty stderr.
Its stdout was `553794136` bytes with SHA-256
`6632e5d1dc9a4ce6d21b9ac38f247668898c37a366e4b7723f9806b60fa77ac1`,
result root
`61951c003012b660601f458db34ad0fd4985b0ac5c74235f6f32df07fd73928b`
and identical expected/actual total-work root
`4529b808739913dbb41486979d6a2861e7962c3b47d88883acf3b2b383916634`.
Stderr SHA-256 is the empty-file root
`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.

The following corrected controls pass:

- retained FCR0/FCR1 and NCGP14 identity;
- analytical normal-viscosity decay and exact tangential `mu=0` behavior;
- repulsive and attractive surface pairs at base, `0.75 m` and `100000 m`;
- both translation-consistency receipts;
- combined-tetrahedron term/gradient correspondence;
- exact `lambda_v=0` and `gamma=0` comparators;
- 32-step surface-energy and 16+16 reversible-energy controls;
- half-viscosity, wrong-surface-sign and missing-surface-factor mutations;
- nonfinite, duplicate-ID and capacity rejection;
- normal commit, forced rollback and final-work-verifier rollback.

Three mandatory mutation controls remain load-bearing apparatus failures:

| Mutation | Observation | Why it is not admissible |
| --- | --- | --- |
| omitted kernel `2/h` | corrected all-term TIGHT baseline itself reaches the frozen work ceiling; mutated route ends `LINE_SEARCH_EXHAUSTED`; state difference `0.0578368341 m` | no passing corrected baseline, so the mutation is not isolated |
| current/reference viscosity graph swap | corrected candidate and oracle pass; mutated route reaches a typed work ceiling; state difference `1.971918e-6 m` | the control incorrectly excludes a typed cap from expected rejection |
| finite pressure penalty | corrected all-term TIGHT baseline itself reaches the frozen work ceiling; mutated route also reaches the ceiling; state difference `0.0312953349 m` | no passing corrected pressure baseline, so penalty causality is not isolated |

The repair allowance is exhausted. Treating these as PASS after observing them
would change the frozen apparatus post hoc. Phase B `P/PV/PS/PVS`, Phase C,
the detached NCGP14 replay, the second clean build/run, sanitizers and final
independent runtime review are therefore `NOT_RUN_BY_PRECEDENCE`.

## Decision and next action

Preserve NCGP15 as an honest apparatus result. Freeze a new apparatus revision
before code with the same physics and solver:

1. use a converged pressure-only confined baseline for the omitted-kernel and
   finite-penalty mutations;
2. retain the existing graph-swap fixture, but define its already observed
   typed work ceiling as an expected rejection when the corrected candidate
   and independent oracle pass;
3. retain the repaired trace grammar as a regression control;
4. leave every physical coefficient, tolerance, cap, Phase-B mask and Phase-C
   byte unchanged.

Only a reviewed successor may reach the physical masks. CUDA correspondence,
4k/16k/50k correctness and timing remain blocked. CPU DFSPH remains the
product fallback; SPEC-38 and ADR-076 remain Proposed; the roadmap is
unchanged.
