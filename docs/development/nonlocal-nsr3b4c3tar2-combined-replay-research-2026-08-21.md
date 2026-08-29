# B4C3TAR2 combined adaptive recovery replay design

Status: `COMPLETE / CONTRACT FROZEN / IMPLEMENTATION NEXT`

Date: `2026-08-21`

## Composition boundary

B4C3TAR2 combines exactly two selected, independently tested policies:

```text
B4C3TAR  exact KKT_SOLVE:REJECT_LIMIT -> continue refinement
B4C3A2   compensated ledger -> KKT sum-scale physical admission
```

It does not add a third repair. The spectral estimator, four-level sequence,
adjacent embedded gate, quantizer, long-horizon binary/energy/contact bounds,
P2 onset schedule and physical coefficients stay as frozen in B4C3TA/B4C3TAR.

## Root ownership

Canonical frames still use the B4C3Q representation profile. Each committed
entry now has two evidence roots:

```text
legacy ledger root  proves old physical/energy field identity
policy ledger root  binds both residual normalizers and B4C3L policy
```

Only the selected fine level contributes either root. Failed/coarse levels
remain private but contribute attempted-work diagnostics. The trajectory root
continues to bind canonical frames, not ledger policy.

## Decision

Freeze a complete eight-frame P1 / sixteen-frame P2 replay under a new r2
identity. PASS may reopen only B4C3TR fixed canonical reference design. The old
r0/r1 failures remain negative evidence and are not renamed or reinterpreted.
