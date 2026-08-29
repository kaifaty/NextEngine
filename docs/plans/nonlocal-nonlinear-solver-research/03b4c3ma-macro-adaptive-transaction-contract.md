# NSR3-B4C3MA -- adaptive macro transaction

Status: `EXECUTED / FAIL_PRESERVED / TOPOLOGY_RECLOSURE_REQUIRED`

Parent B4C3PE1 passes with JSON-without-final-LF SHA-256
`eb4d82300653d779baf00620cb83a2526d164347c1b97a65b487f1955a3b8d60`
and semantic SHA-256
`830c1613a65e92954744b104818a63f7be1f6e8f6fa3bf9580009a83265d5d48`.
B4C3TR and B4C3P remain exact negative controls.

## Policy identity

```text
sha256  4d36cdbc2e10e02a054156d5f6d1faaf5d0cffaa8364386098406eb8283de38e
text    nextengine.nonlocal.macro-adaptive-transaction|v1|private=binary64|levels=4|recovery=exact-reject-limit|selection=adjacent-fine|publication=accepted-macro-only|stability=mixed-v1
```

Reuse the B4C3P representation profile, macro-ledger policy, P1/P2 fixtures,
spectral estimator, four-level cap, embedded `smoke_gate`, KKT solver and
B4C3PE1 mixed admission exactly.

## One-frame transaction

For each P1/P2 fixture with `macro_frames=1`:

1. obtain the initial level count from the unchanged read-only start/forecast
   spectral path;
2. run up to four binary64 private levels from the exact same committed start;
3. refine only an exact typed `REJECT_LIMIT`; every other failure is fatal;
4. select only a passing fine level whose immediately preceding level passes
   the embedded gate; a failed level breaks adjacency;
5. publish the selected fine endpoint once through B4C3P balanced publication;
6. commit one decoded state, one frame step `1`, one legacy ledger entry and
   one macro-policy ledger entry atomically.

Count every attempted substep, outer/rejected trial and HVP. Require accepted
substeps `<=192`, attempted level `<=768`, no live workspace, no all-pair
candidate path and exact attempted/discarded accounting.

## Admission and physical gates

Apply B4C3PE1 with `D=RMS(coarse,fine)` and the unchanged binary64 floor to
the publication error `RMS(decoded,fine)`, independently for position and
velocity. Every field must select `TEMPORAL_BUDGET` or
`ABSOLUTE_REPRESENTATION_BUDGET`.

Require the private fine run, embedded gate, balanced aggregate bounds,
decoded finite aggregate, non-residual macro-ledger checks, KKT sum-scale
residual `<=1e-9`, exact residual correspondence, canonical geometry, root
recomputation and exact contact set/time preservation across publication.

## Failure controls

1. Exact recovery grammar accepts only
   `SUBSTEP_<completed>:KKT_SOLVE:REJECT_LIMIT`; wrong index, suffix, prefix,
   non-decimal or any other KKT failure is fatal.
2. Passing non-adjacent levels separated by failure cannot select.
3. Exhaustion without an adjacent passing gated pair commits nothing.
4. A forced failure after private selection but before publication leaves
   state, frame/ledger counts, roots and cumulative totals exact.
5. Full B4C3PE1 reproduces at its exact parent hash.

Two complete B4C3MA reports must be byte-identical.

## Decision boundary

PASS selects `ADAPTIVE_MACRO_TRANSACTION_CANDIDATE` and authorizes only design
of a complete P1/P2 adaptive macro recovery replay. FAIL preserves B4C3PE1 as
the last positive boundary. Adaptive-versus-fixed comparison, nominal corpus,
B4C4/B4D, CUDA, runtime/schema and production remain blocked.

## Executed outcome

The isolated transaction fails only raw binary64 boundary-membership equality:
P1 retains 40 of 64 exact-equality features after decode, with 24 lost, none
gained, zero penetration and maximum coordinate difference `2.78e-17 m`.
Every solver, selection, mixed-budget, ledger, root, work and rollback gate
passes. Preserve this FAIL; only a new canonical-integer topology contract is
authorized.
