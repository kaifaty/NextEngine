# NSR3-B4E2D7R19R14 soft-cap suspension evidence

Date: `2026-08-24`

Status: `PASS / SOFT_CAP_OUTER_BOUNDARY_SUSPENDED / SHADOW ONLY`

## Outcome

R14 proves a bounded policy for finishing already-admitted work without
turning the total-HVP limit into an unbounded cap increase. The target trial
starts below the soft boundary at total `498`, owns only its existing
34-HVP per-trial allowance and completes at total `523`, below its dynamic
ceiling `532`.

R13 already proves that the accepted trial then completes inner and outer 5
with zero new HVP. R14 therefore denies outer 6 and emits one deterministic,
versioned outer-boundary continuation-token candidate. It does not execute
resume or mutate the live budget implementation.

## Parent and accounting closure

```text
R14 identity SHA-256  0d8a39cfefd050febf87c9544c4a33b0cee9251a769bc3b73d6a2f126d4d839f
R13 stdout SHA-256    0f248c4506303055cbf7a967dd1328a36d8628341a04b6861aec210af17f57f9
R13 semantic          3a60f64ee09a85b4ea892f12341ff3b8c4e7dcb63bbe4eb4cbebd99c237e5e57
trial start total     498
soft limit            512
trial allowance        34
dynamic ceiling       532
recurrence HVP         24
model HVP               1
completed total       523
overshoot               11
unused allowance         9
```

R12/R11/R10/R9 remain transitively exact through the R13 parent. Admission
is tested only at trial start. The admitted trial remains within its original
hard allowance; no later solve or outer update inherits the unused reserve.

## Suspended boundary and token candidate

```text
outer state root   5dd9a07d60cbfe851bdc4c383944a1eb8042f178a821f4a546da33101dd0a19a
position root      58dadefd7426e9f80188b694557a02efcaf005dd67abfff5e337b90f214be5a8
dual root          f4279bde29358f65b17b15ad7456e8c5743b44ec99fd3612d78cb5f905b00aca
predicted root     36112dde1e0b274c5b9216f4818b82977a0c80dc257478c111b0f0a9390d2d7e
static identity    a2d97ab6f26383d826366eba2a3d4392f5ef9610dda87e89509e93bd9daf61e8
token root         c06dbfeeac346d4413d114082d18b4e4725edfac81896ed72cbabfacf3f188b5
```

The candidate token uses schema version `1`, records `next_outer=6`, previous
primal bits `0x3e5c40ff44000000`, previous admissibility `false`, provisional
index `-1`, budget epoch `0`, slice HVP `523` and cumulative HVP `523`.

The exact boundary is inner-complete and outer-complete. Outer 6 is denied at
the soft boundary. The token is private research evidence, not a public save
format or an authorization to resume.

## Work, rollback and scope

R14 performs one control-parent replay and then projects the policy over the
exact captured boundary. The projection consumes:

```text
new workspaces        0
new HVP               0
new model HVP         0
new trials            0
new precision audits  0
new outer updates     0
```

Parent and token rollback checks pass. No resume, outer 6, second solve,
second substep, macro, trajectory, timing or public commit executes. Physics,
live budget code and production policy remain unchanged.

## Reproducibility

Contract/research commit:
`6ed0d64c` (`research: freeze soft cap suspension projection`).

Implementation commit:
`8bb3c51d` (`research: project soft cap suspension`).

Two clean Release builds:

- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r14-a.uj2j6S`;
- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r14-b.FkgBCi`.

Both binaries are `6,166,768` bytes, have SHA-256
`61f3218d12dd19549d8140d382825ad240cb3b1fffb8aef8c05f20193b9ffbd3`
and GNU build ID `f3108ef82844b0c064b6c562431ac427e2542c09`.

Fresh process outputs:

- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r14-a.RQgRrc`;
- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r14-b.TFHkNs`.

Both exit `0`, emit empty stderr and reproduce byte-exact:

```text
stdout-with-LF bytes  2,123
stdout SHA-256        16b357b173d5c36f123dbbad31cca0c777229ff38772d3847a85357c9a0b4625
semantic result       34fdc84cb5bddcc89337f7b1cc961cbc18f5d5eda7cfe008ac6ae0af89d08751
route                 SOFT_CAP_OUTER_BOUNDARY_SUSPENDED
```

## Decision

Retain R14 as proof that this exact in-flight trial can finish atomically and
stop at a complete outer boundary under a soft admission limit. The policy
does not solve general mid-inner suspension and must fail closed if an
over-cap accepted trial needs another trust solve.

Research D7R19R15 as token validation and negative controls before any live
budget state-machine integration. It must validate the exact token and reject
at least schema, static identity, predicted state, parameter identity, budget
epoch/ledger and state-root mismatches before work. R15 may classify token
admission only; it may not execute resume, outer 6 or mutate live budget code.
