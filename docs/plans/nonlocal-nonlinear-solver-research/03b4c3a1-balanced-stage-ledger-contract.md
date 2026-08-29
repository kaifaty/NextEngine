# NSR3-B4C3A1 -- balanced canonical stage and publication ledger

Status: `PASS / CANONICAL_BALANCED_STAGE_LEDGER_CANDIDATE / B4C3T_DESIGN_AUTHORIZED`

Parent B4C3Q selects `CANONICAL_AGGREGATE_BALANCED_CANDIDATE`; semantic SHA-256
is `ca5f49f3179e05f01fed58a690919b1b78877cc7d6ad2d20611aeb836dd6468f`
and JSON-without-final-LF SHA-256 must equal
`35def7a51dcce516d3f65d763595a4e3dc52fe771b16368773d3b4493c30182c`.

## Identity and roots

```text
identity  joint-pressure-canonical-balanced-stage-r1
profile   f57d88c222a8334206a962a72a814bdd530696ed2767c94fa88910281265579c
P1        d42eeb5753b654e5a13fe3d4db30010bc77118c6f216f9b9cb6ca404643124de
P2        5755be013a3704373b7cb2d21ea1608d6497dec980120af42d73fa7d9f802cd8
```

Use only B4C3Q balanced publication and existing frame/trajectory byte
encoding. Do not reuse B4C3A nearest-even roots or profile.

## One-frame atomic transaction

Run P1 at `21/42` and P2 at `1/2`. Every level starts from the same decoded
committed fixture. Every passing substep stages one balanced frame plus one
publication-ledger entry and continues from its exact decode.

The unchanged embedded gate commits the fine frames and their ledger entries
as one batch. Require exact sample IDs/steps, final decode, repeat/reverse/
coprime-affine frames and ledger values. No coarse frame or ledger term may
appear in committed totals. A forced failure after two private stages, plus all
B4C3Q typed publication failures, commits zero frames and zero ledger entries.

Preserve the B4C3Q physical bounds against binary64 and nearest-even fine runs:
`100e-6 m`, `1e-3 m/s`, exact contact features and one coarse-step contact-time
allowance.

## Momentum publication ledger

For each fine and coarse staged substep compute both direct vector expressions:

```text
I_q = momentum(v_q) - momentum(v*)

L_solver = momentum(v*) - momentum(v_n)
           - I_gravity + R_support + R_contact

L_raw = momentum(v_q) - momentum(v_n)
        - I_gravity + R_support + R_contact

L_compensated = L_raw - I_q
```

Require:

- direct `I_q` equals the B4C3Q exact aggregate velocity report within a
  componentwise binary64 forward-error allowance;
- `L_compensated` equals `L_solver` within its forward-error allowance;
- compensated residual keeps the original `<=1e-9` KKT gate;
- per-publication `norm(I_q) <= m*sqrt(3)*0.5e-6 + allowance`;
- committed cumulative impulse is the exact ordered sum of fine entries and
  its norm is no greater than `fine_substeps` times that per-step bound;
- raw residual is reported and is not mislabeled as a physical solver failure.

## Position and energy publication ledger

For every entry compute direct center shift and aggregate report closure. Record
decoded and solved kinetic, pressure, gravitational and total mechanical
energy, then require:

```text
delta E_q = delta K_q + delta Phi_pressure_q + delta U_gravity_q
```

within a binary64 forward-error allowance. The pressure energy is recomputed at
decoded geometry through the selected joint neighborhood with audit disabled.
Require all terms finite, the B4C3Q kinetic inequality, and

```text
abs(delta U_gravity_q) <= m * abs(g_y) * 0.5e-6 + allowance.
```

The contract records maximum and committed cumulative absolute pressure and
mechanical publication deltas but deliberately freezes no empirical cap for
them. B4C3T must use this evidence to derive a separately frozen long-horizon
gate.

## Exit

Two reports must be byte-identical. B4C3Q, B4C3A, NPR1-A and B4C2T historical
reports remain exact.

PASS selects `CANONICAL_BALANCED_STAGE_LEDGER_CANDIDATE` and authorizes only
B4C3T full-controller/long-horizon physical-bound design. FAIL preserves B4C3Q
as a quantization-policy result but blocks canonical continuation. No nominal,
CUDA, runtime, schema or production authority is granted.
