# NSR3-B4C3A1 balanced publication ledger research -- 2026-08-21

Status: `COMPLETE / LEDGER_CONTRACT_REQUIRED / B4C3T_BLOCKED`

## State transition being audited

The selected physical substep now has two consecutive state changes:

```text
(x_n, v_n) --KKT solve--> (x*, v*) --balanced publish/decode--> (x_q, v_q)
```

The existing KKT ledger closes only the first arrow. The second arrow is a
deterministic numerical representation event, not a pressure/contact/gravity
force. It must be visible in evidence rather than folded into the physical
reactions.

Define:

```text
delta P_q = m * sum_i(v_q_i - v*_i)
delta C_q = sum_i(x_q_i - x*_i) / N

L_raw = P(v_q) - P(v_n) - I_g + R_support + R_contact
L_compensated = L_raw - delta P_q
```

`L_compensated` must reproduce the solver ledger within a forward-error bound.
`L_raw` is expected to contain the admitted quantization impulse and must be
reported separately. Subtracting it without recording `delta P_q` would hide
the representation defect; treating it as support/contact reaction would be
physically wrong.

For mechanical energy, record rather than reinterpret:

```text
delta E_q = delta K_q + delta Phi_pressure_q + delta U_gravity_q
```

The kinetic term has the frozen norm inequality from B4C3Q. Balanced aggregate
position gives a direct gravitational bound. Pressure-energy change is finite
and exactly recomputed at decoded geometry, but B4C3A1 does not invent an
unjustified universal cap for it; full-horizon B4C3T must observe its cumulative
behavior.

## Transaction ownership

Each adaptive level owns both staged frames and staged ledger entries. The
embedded gate atomically commits the selected fine pair; all coarse ledger
entries are destroyed with their frames. A solver/publication/ledger failure
commits neither frames nor numerical impulses and leaves the prior root/state/
ledger totals exact.

## Decision

Freeze B4C3A1 under the B4C3Q selected profile. It revalidates P1 `21/42` and P2
`1/2`, fine-only commit, exact order/repeat, physical correspondence and the
quantization-aware momentum/energy ledger. B4C3T, B4C4 and nominal execution
remain blocked.
