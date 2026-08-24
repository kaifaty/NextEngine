# NSR3-B4E2D7R19R20 shadow outer-7 execution evidence

Date: `2026-08-24`

Status: `PASS / SHADOW_OUTER7_EXECUTION_CANDIDATE / NOT ADMISSIBLE`

## Outcome

The epoch-1 slice candidate and independently initialized unsliced cumulative
oracle execute the same outer 7 and produce bit/work-identical results. The
candidate stays inside its remaining slice capacity.

Outer 7 is finite and improves the primal violation, but remains
non-admissible. This closes continuation equivalence at a nonzero slice
offset; it is not a public commit or production result.

## Candidate/oracle equivalence

```text
candidate budget base       slice 50 / 512
oracle budget base          cumulative 573 / 8704
outer update root           f9cf69b189cf92b715ac8602420ff6126c08d0f30d31686e4d50bfe7ec204eb5
work root                   e5f47a3842efab2c609aad259e5676d72e294270f5d6b13805e13e9d9e3e7c37
candidate/oracle exact      true
```

Each lane performs exactly one outer update with:

```text
inner trials / accepted / rejected   2 / 2 / 0
HVP delta                           52
workspace delta                      4
precision-audit delta                2
all-pair candidate calls             0
```

Both lanes are finite and have no precision/sign contradiction, oracle-bound
reduction or budget exhaustion.

## Physical result

```text
primal          2.14040711821184e-08       bits 0x3e56fb819c000000
stationarity    1.4979505686051011e-13     bits 0x3d4514ef89e8a7f5
admissible      false
position root   bda0ac4fa2be5c29484a17c520a968046849a49aa892a7c60b04f808a6199afd
dual root       dc5bcc6bb918c3336ebe4bd99e03d7b8e1c92a61ca0db73418303ed0a59484eb
```

Primal decreases from R18's `2.5139003545504579e-8` by about 14.9%, while
stationarity remains far below its limit. The observed bottleneck continues
to be outer AL constraint closure rather than rejected inner trust solves.

## Canonical successor

```text
successor history  7916f59936efde08993e6d94733e5e7254afa8d6d5b7ab0b26bc723d91978b9d
consumed owner     94df43022a9b8fd28e00f926db066e065bd8fa17ff65598b01dfe48e11e47f8a
receipt bytes/root 420 / ec7a61ea0123865de888304c77ba2b90df17337dc8196e208745c5ae970c6bfd
state bytes/root   212 / 1da2e0f54f4f15409ee29f8eb1be10798940635212c64448d12cc7e92982b84f
```

The exact successor ledger is:

```text
8,2,27,1,102,625,41,25,602,23,2,25,0
```

Epoch remains `1`; slice/cumulative HVP become `102/625`, leaving 410 slice
HVP. The additive HVP ledger closes exactly at `602 + 23 = 625`.

## Atomicity and scope

All `13/13` valid/negative routes pass with corpus root
`930cfe9a996d4f9e883748311a0d6f851c222fa6d71e31915bbadc03fe07f08d`.
Every failure preserves complete transaction bytes; duplicate replay is
prework and idempotent. R18 and R19 parent stdout remain exact.

R20 executes one inherited parent substep, one private candidate outer 7 and
one private oracle outer 7. It executes no second substep, macro, trajectory,
timing or public/world commit. The receipt is private research evidence, not
durable/concurrent CAS or runtime authority.

## Reproducibility

Contract/research commit:
`fa24ffe7` (`docs: freeze D7R19R20 shadow outer7 execution`).

Implementation commit:
`eaa59dde` (`research(nonlocal): execute shadow outer7 oracle`).

Two clean Release builds:

- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r20-a.EdFjBw`;
- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r20-b.hbNltb`.

Both binaries are `6,532,472` bytes, have SHA-256
`03f83e9047f1b010ced80699859ba325869586f5b5a45bfabe2978866e63e17d`
and GNU build ID `c5dd74257b65ddcb33b821fe8c2cb1410e371956`.

Fresh one-process outputs:

- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r20-a.PjB01X`;
- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r20-b.zr9Ods`.

Both exit `0`, emit empty stderr and reproduce the same `1,882` stdout bytes:

```text
stdout SHA-256  df06ac4125082f715d200829b5afef203f0a9ab046f6e9018f97bd8066b1d113
semantic        a375f2e5aaabe89694ed2f2d4383f3cf1d51212c9a648d8094e5b17ec97d1412
route           SHADOW_OUTER7_EXECUTION_CANDIDATE
```

## Next action

Research/freeze a zero-work, one-use outer-8 grant inside epoch 1. Bind the
exact R20 state/receipt/history and preserve slice/cumulative `102/625`. Do not
execute outer 8, reset epoch, publish state or run timing before that contract
passes.
