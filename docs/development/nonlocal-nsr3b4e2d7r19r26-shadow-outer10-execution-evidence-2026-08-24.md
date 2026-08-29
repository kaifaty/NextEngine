# NSR3-B4E2D7R19R26 shadow outer-10 execution evidence

Date: `2026-08-24`

Status: `PASS / SHADOW_OUTER10_EXECUTION_CANDIDATE / NOT ADMISSIBLE`

## Outcome

The slice candidate and independent unsliced oracle execute outer 10 with
complete bit/work equivalence. Candidate remains within epoch-1 capacity.
The result is finite and accepted but still non-admissible.

```text
update root       13794ca4a0901f38739666110f213026df6171dc0f80cac35e51a3b7b8eef7ec
work root         fbb9fc3f9aa7d2e546d965fa2c1e422abaeb608ed86547164c5bba31c5759023
trials            2 accepted / 0 rejected
HVP/workspace/precision delta   51 / 4 / 2
candidate/oracle exact          true
```

## Physical and mechanism result

```text
primal          1.9256488625885027e-08   bits 0x3e54ad2eec000000
stationarity    2.0818980800439604e-14   bits 0x3d1770a9a48ed9c8
admissible      false
position root   174cad44a3edbc7eab514986e6be3d1bdd3c219aa68fc3c9913b164ce4058562
dual root       63770158bc269b879c4e3b98ac12a176be334902f0c929d05d686a157ce40f87
```

Primal improves only about 1.0032% from outer 9, versus 4.454% at outer 9.
Both trials are accepted, use 51 HVP total, and the last uses 26/34. Slow
primal progress is therefore not reject churn or immediate cap pressure.

Stationarity returns from R24's `9.58664e-11` to `2.08190e-14`, close to the
outer-8 scale. Candidate/oracle exactness and two clean builds classify the
R24 spike as a reproducible local trajectory transient, not offset-dependent
nondeterminism or monotonic inner-solve degradation.

The remaining research question is external primal convergence: obtain at
least one more unchanged-policy outer observation before deciding whether to
freeze a discriminator for AL stagnation versus discretization floor.

## Canonical successor

```text
history         7aaf5e5077f937f03db754e407243e8c6b61b0f813931cc59d83f7f7dd866785
consumed owner  3e956008c7c633d5ab74e249af21f1a4503ae9f5709603cdc763b0047d45e28d
receipt         3b5c6dd3b8824eddc3aad566a8b663d344e1702f7958dced701c61710d688776
state           317f63c5a3f02df3cbb3e905fed9b0b1740ebcb363799c22d4febb3a837fe1fd
used            11,2,26,1,282,805,54,32,775,30,2,32,0
```

Epoch remains `1`; slice/cumulative HVP become `282/805`, leaving 230 slice
HVP. Recurrence/direct accounting closes at `775 + 30 = 805`.

All `13/13` routes pass at corpus root
`5159615db0345963ce91158a8af1dbfa98d40c16d2afd057f528c3370338bf8c`.
Rollback and duplicate replay are exact. No following outer, substep, macro,
trajectory, timing or public/world commit occurs.

## Reproducibility

Contract/research commit: `0ad40178`.

Implementation commit: `bcc30180`.

Two clean Release builds:

- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r26-a.8Tm55q`;
- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r26-b.HofVCI`.

Both binaries are `6,761,720` bytes, have SHA-256
`43fde512eb9bb476fa2391c3ff2417f201a0570bab1042ec05798ee45903db20`
and GNU build ID `d4b897a04b78012041930397632a7f3bca9c4c44`.

Fresh sequential one-process outputs:

- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r26-a.XJBHbt`;
- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r26-b.6ouuWJ`.

Both exit `0`, emit empty stderr and reproduce `1,887` stdout bytes:

```text
stdout SHA-256  67a4734dc549f19e57812813ec89e21ce23a90d5e1bb8415a41b7fcd1de78295
semantic        6f5d971e6c0dfe3e4f21c48130e3e82f861355a42691b836f5adcc4b78c97f22
route           SHADOW_OUTER10_EXECUTION_CANDIDATE
```

## Next action

Research/freeze a zero-work one-use outer-11 grant at exact state/receipt/
history and slice/cumulative `282/805`. Keep all caps unchanged; after the
grant, research one outer-11 candidate/oracle observation before designing a
primal-stagnation discriminator.
