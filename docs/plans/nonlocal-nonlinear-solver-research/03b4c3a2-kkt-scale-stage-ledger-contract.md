# NSR3-B4C3A2 -- KKT-scale balanced stage ledger

Status: `PASS / CANONICAL_KKT_SCALE_STAGE_LEDGER_CANDIDATE / REPLAY_DESIGN_ONLY`

Parent B4C3L selects `CANONICAL_KKT_SCALE_LEDGER_CANDIDATE`; its
JSON-without-final-LF SHA-256 is
`ba1684f09575662d10fe8646780195afc6b650c2f2bcbcfc62a7a44918a29540`
and semantic SHA-256 is
`c56c80ed7d1790d575147d4afc456102904b6264443e55febff71d8dcf8719ac`.

## Identity

```text
identity        joint-pressure-canonical-balanced-stage-r2-kkt-ledger
representation  f57d88c222a8334206a962a72a814bdd530696ed2767c94fa88910281265579c
ledger policy   b1136c2c3dc7970cbee0ce67b0d129b849ed8957268bb7281063af9e60200e2f
P1 scenario     d42eeb5753b654e5a13fe3d4db30010bc77118c6f216f9b9cb6ca404643124de
P2 scenario     5755be013a3704373b7cb2d21ea1608d6497dec980120af42d73fa7d9f802cd8
```

Run P1 at `21/42` and P2 at `1/2` from the same B4C3A1 fixtures. Solver,
quantizer, time step, stage roots, physical gates and energy formulae remain
unchanged.

## Ledger fields and admission

Every entry must serialize and policy-hash:

```text
raw published ledger/residual
direct quantization impulse
compensated ledger and source KKT ledger
strict max scale/residual       diagnostic, never physical gate
KKT sum scale/residual          physical gate <= 1e-9
source KKT residual
residual correspondence error/bound
impulse, center and energy closure/decomposition fields
```

The policy hash and representation profile are both bound into the policy
ledger root. Continue producing the legacy B4C3A1 ledger root from its original
field set. Require selected fine legacy roots exactly:

```text
P1  5feac29a07dbd03dc0aa4467056fed84bcec698de3c720ad16de854aeb559eea
P2  bb5562f1dc268a7dcc885092eaa45d1db2b21a76dc0088f9de79116d6fc7b312
```

Require canonical trajectory roots exactly:

```text
P1  ece583962cb07f7a15bb1b84a83719895ec5af2c2735b0904ea76f833180939d
P2  8138d5202b4a754c43b8ee7306200b29d6d1c75b959717f9a0f3c54c1f543ddd
```

## Transaction and physical gates

Commit only the fine frame and policy-ledger sequence. Coarse roots/entries,
including their diagnostic residuals, remain private. Global step/sample
identity, decoded continuation, aggregate-balanced bounds, contact equivalence,
binary/nearest correspondence and every B4C3A1 kinetic/pressure/gravity/
mechanical decomposition gate remain unchanged.

Repeat and reverse/affine order runs must reproduce frames and every new policy
field exactly. The candidate KKT residual must correspond to the source KKT
residual within the frozen B4C3L forward bound. Strict residual is mandatory
and finite but may exceed `1e-9` without failing candidate admission.

## Negative controls

1. Forced solver failure after two private fine substeps leaves committed
   frames and both legacy/policy ledgers empty.
2. Zero/invalid KKT scale, KKT residual above `1e-9`, corrupted compensated
   closure and nonfinite strict diagnostic each reject the policy entry.
3. Removing the strict diagnostic or policy identity changes/fails the policy
   ledger root; it cannot silently fall back to the legacy root.
4. B4C3L, B4C3A1, B4C3Q, B4C3A and B4C2T reports remain exact.

Two complete reports must be byte-identical.

## Decision boundary

PASS selects `CANONICAL_KKT_SCALE_STAGE_LEDGER_CANDIDATE` and authorizes only
design of a new complete adaptive recovery replay. FAIL preserves B4C3L and
B4C3A1. No complete controller, fixed reference, nominal, CUDA, runtime, schema
or production authority is granted.

## Executed outcome

B4C3A2 passes twice byte-identically; see the
[dated evidence](../../development/nonlocal-nsr3b4c3a2-kkt-stage-ledger-evidence-2026-08-21.md).
Both canonical trajectory and legacy ledger roots remain B4C3A1-exact, while
new deterministic policy roots bind the selected residual semantics. Atomic
commit, rollback, order/repeat, physical correspondence and negative controls
all pass. Only a new complete adaptive recovery design is authorized.
