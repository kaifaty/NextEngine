# NSR3-B4E2D7R9 divided-difference evidence

Date: `2026-08-22`

Status: `FAIL / INVALID_REPLAY_ACCEPTANCE_CONTROL / CANDIDATE_UNSELECTED`

## Reproducibility

Two clean Release builds produce byte-identical 4,956,328-byte executables at
SHA `c7bd608faea47bbdc7eae53421e1c0f179d538cd781e1d825b7234c8bc8c67ed`
and Build ID `322c026e6ba08c9622922bec8e52aed53cb18b7c`.

Both D7R9 processes exit one with empty stderr and byte-identical 15,645-byte
stdout reports at SHA
`2c45e93d4153edf714b0f490be3f3d760b14a1ead1e8ed8ae9528d2555ec91f0`.
The semantic result is
`b5a0f921a65d82c07a019ee08be970468b80aeec6075c3da706f7623cd346ce9`.
Raw evidence is under
`/home/kaifaty/.cache/nextengine/external/run-nonlocal-b4e2d7r9.3sFIzK`.

D7R8 remains byte-exact at stdout SHA
`42ce10541b29dd03b589e1e8699436e65cb5a794a45b8644fc040990dd4648b1`
in both builds. The D7R9 oracle ledger is the frozen `11` resolved positive,
zero resolved negative and `12` unresolved trials. State roots, work
decisions, branch ledger and rollback are exact.

## Contract failure

The first failure is `REPLAY_ACCEPTANCE`. The contract required zero accepted
replay trials, but an exact D7R8 replay necessarily contains two historical
accepted intermediate trials inside inner solves that later fail:

```text
eta=1e-8  trial 7
eta=1e-9  trial 5
eta=1e-10 none
```

The discriminator does not introduce or commit either acceptance. They are
part of the parent trace and were already visible as `would_accept=true` in
D7R8. Requiring both exact parent work and a zero inherited count is therefore
an impossible control. Preserve D7R9 as FAIL; do not reinterpret it as a
candidate PASS.

## Numerical observation retained without selection

The classification fields are useful negative-run evidence, not authority:

- compensated absolute energy passes only `2/11` scored trials;
- propagated divided difference passes `11/11` scored trials;
- its relative error to the compensated extended oracle ranges from
  `0.0000714710` to `0.0399153`;
- it repairs cases both with and without compact-support crossings;
- every PHR branch count remains zero;
- no unresolved trial is scored or newly accepted.

The full candidate therefore strongly supports the precancellation mechanism,
while merely compensating independent absolute evaluations is insufficient.
It cannot be selected until the acceptance control is reclosed before a new
run.

## Decision

Freeze D7R9R1 with the exact inherited ledger above, `new_acceptance=0` and
otherwise unchanged candidate algebra, scoring, routes and controls. D7R9R1
must reproduce the complete D7R9 FAIL bytes before it can classify the
candidate.

