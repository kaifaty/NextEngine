# NSR3-B4E2D7R19R18 shadow outer-6 resume evidence

Date: `2026-08-24`

Status: `PASS / SHADOW OUTER6 CONTINUATION CANDIDATE / NOT ADMISSIBLE`

## Outcome

The epoch-1 slice candidate and independent unsliced cumulative oracle execute
the same outer 6 from exact cloned R16/R17 inputs and produce identical
physics, nonlinear, precision and work roots.

Outer 6 completes safely, but is not yet admissible. This is a successful
resume-equivalence result, not convergence or production approval.

## Parent and chain closure

```text
R18 identity SHA-256  6f26279f6422ca0d9b4f76a9eb15ae10d8b961a2d807ba98ca6881055c80bcc0
R17 stdout SHA-256    d930b6d57b1bd11936949d7ef7ab2e7f2c607c048fd9b53db6fcfb94f41b2447
R17 semantic          c0c6d0285a724dd3615f5cfab077e4b8bc86a73abede691b60c847889a15a173
R17 owner state       d013821c2f5c862086bb56774b73b97a94f597aa5a6c05d87f9434d8f6b3ba54
```

The exact source envelope, epoch grant, R17 transition receipt and active
owner all revalidate before work. R16/R15/R14 and every transitive parent
remain exact.

## Candidate/oracle equivalence

```text
candidate budget base       slice 0 / 512
oracle budget base          cumulative 523 / 8704
outer update root           9dfe1a51ab952f83513437e6a2c1bb45eef749ffbc04b70d5fd8a65449cb2ed5
work root                   cd6418fc71ce5ffaad64303c6a299a044d4815cf05225bde25346da9537d9bda
candidate/oracle exact      true
```

Each independent execution performs exactly one outer update with:

```text
inner trials / accepted / rejected   2 / 2 / 0
HVP delta                           50
workspace delta                      4
precision-audit delta                2
all-pair candidate calls             0
```

Both results are finite and free of precision contradiction, sign
contradiction, oracle-required reduction or budget exhaustion.

## Physical result and disposition

```text
primal          2.5139003545504579e-08
stationarity    4.3586704721245672e-13
admissible      false
position root   494dc8d29030f06b79b7047a82ce83eb994e863541becc37f60d3343f08ff29f
dual root       9ec049a088270e211435a5cc16b4fea23826c261145209cb28748b8dec92c778
```

Stationarity is already below its limit, but primal remains above `1e-8`.
R18 therefore authorizes no public commit or next outer. A later owner/grant
stage must decide the outer-7 continuation boundary first.

## Canonical successor

The active owner changes from flags `1` to `3` and reproduces consumed root
`abbef5a38a947f21945d9a96e50df2597a4ac4647a2d733c911ff55cb74eaa74`.

```text
successor history  a36fa9c02821f1eac2785aa0ec35632f088810b3be625303271d2b9750b8a5b1
receipt bytes/root 420 / 5a029f24a82b60117492f1929a2d63489d2321b63286a9260282101b74ef8a64
state bytes/root   212 / 5ad2f99d356c9f5f7553d84189e1e279dcd360bdb9a588370a874b2c5fe707bc
```

The exact successor used ledger is:

```text
7,2,25,1,50,573,37,23,552,21,2,23,0
```

Epoch remains `1`; slice HVP becomes `50`; cumulative HVP becomes `573`.
The additive recurrence/direct ledger remains exact: `552 + 21 = 573`.

## Negative corpus and rollback

All `13/13` fixed routes pass with corpus root
`e780608d1046430944fd5f081eac91bc891f9f611dcd6e85303fc442e467bebb`.
Prework source/grant/owner/duplicate/stale/resource failures, candidate/oracle
failures, injected oracle mismatch and precommit abort preserve the exact R17
state. Duplicate replay preserves the exact R18 state and executes no work.

## Work and authority

R18 executes one control parent substep, one private candidate outer 6 and one
private oracle outer 6. It performs no outer 7, second substep, macro,
trajectory, public/world commit or timing. Live budget code and production
policy remain unchanged. The canonical objects are private research receipts,
not durable/concurrent CAS or a public save schema.

## Reproducibility

Contract/research commit:
`ae08436e` (`docs: freeze D7R19R18 shadow outer6 resume`).

Implementation commit:
`c68c098d` (`research: execute shadow outer6 resume oracle`).

Two clean Release builds:

- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r18-a.U6vdqD`;
- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r18-b.BI7uZP`.

Both binaries are `6,436,608` bytes, have SHA-256
`a07c604ee5ab580b36cb9cad619c4cc590e7729ee42aa19311042bd5e48bde62`
and GNU build ID `498cdb3c0e5e44d18d53ac66b8464f2e69cc6004`.

Fresh process outputs:

- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r18-a.2B8CGj`;
- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r18-b.CCbV03`.

Both exit `0`, emit empty stderr and reproduce the same `2,596` stdout bytes:

```text
stdout SHA-256  3047e85dc42f22afe287b47b0bc3aaa2e9de3bb767b01b6d4c516497c51841e8
semantic        baadf44a6ae0a2d83865dc41d5e9de1fef82e64eb52f36fc0b3a78a9c213e14d
route           SHADOW_OUTER6_CONTINUATION_CANDIDATE
```

## Next action

Research/freeze R19 as a zero-work, one-use outer-7 grant from the exact R18
state. It must remain in epoch 1, preserve slice/cumulative `50/573`, bind the
non-admissible outer-6 result and fail duplicate/stale use before work. Do not
execute outer 7 before that contract passes.
