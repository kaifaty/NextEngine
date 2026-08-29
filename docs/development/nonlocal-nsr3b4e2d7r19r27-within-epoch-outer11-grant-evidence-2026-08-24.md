# NSR3-B4E2D7R19R27 within-epoch outer-11 grant evidence

Date: `2026-08-24`

Status: `PASS / OUTER11_GRANT_CANDIDATE / SHADOW ONLY`

## Outcome

The exact non-admissible R26 successor issues one zero-work outer-11 grant
inside epoch 1. All canonical roots match the independently frozen targets.
The source owner is consumed once and the active owner remains unconsumed.

```text
source state     317f63c5a3f02df3cbb3e905fed9b0b1740ebcb363799c22d4febb3a837fe1fd
owner before     526ac852b7e52a48e2692b611f2e4ea8e8356fd52019fbeeb9c4935db404f281
owner after      94e0619fd8babb7398035bdfb06e17a569b18cd15957f09b1a32680502089eba
grant            3ded4fc34e99c45657fad87a0aeb4853e1d948817efad60b4106f659c4adefaa
receipt          53a938176504b9059504c511402b5b6f3e8b4dfb17bc6a2aa0f42204160cc330
active owner     431466b59941f862c97c75c5614be7d2dd525199bfbe9ea5e175753bc4cd431e
committed state  369cab15f75ee90237aa5cbe334aa0da5f857e218bec9812773e43faa9682226
```

Epoch remains `1`; next outer is `11`; slice/cumulative HVP remain `282/805`.
The complete ledger remains `11,2,26,1,282,805,54,32,775,30,2,32,0`.

## Atomicity and scope

All `14/14` valid, rejection, abort and duplicate-replay routes pass at corpus
root `2a50513e3b011a6fe03fa1131cf87602476f3c3cd93605649160f459a2435bb7`.
Every failure rolls back byte-exactly and duplicate replay is idempotent.

```text
new HVP                 0
new workspaces          0
new precision audits    0
new outer updates       0
outer 11 executed       false
public/world commit     false
timing admitted         false
runtime/production      false
```

The grant preserves R26's low stationarity and slow primal-progress
observation without interpreting either. This result authorizes only
research/freeze of one separate outer-11 candidate/oracle pair. It does not
authorize a cap, penalty, policy, public-state or production change.

## Reproducibility

Contract/research commit: `989f18c6`.

Implementation commit: `a8ecfb68`.

Two clean Release builds:

- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r27-a.Hkl9x8`;
- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r27-b.FDmjL0`.

Both binaries are `6,799,408` bytes, have SHA-256
`955b855b894be58cbac66ec901617ba9bb40c4c6b918a5f6fbd8a7233755f5c5`
and GNU build ID `f4bdf6461033c053b55ba4f54da8e1cc6f9bf693`.

Fresh sequential one-process outputs:

- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r27-a.g9Ybyl`;
- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r27-b.380xp4`.

Both exit `0`, emit empty stderr and reproduce `1,415` stdout bytes:

```text
stdout SHA-256  3760daf964f16b64d6c7685b8cdde255f31812d5aacee55cc7c18d2a079d65c2
semantic        7f08a85aae890fcf48d207cbdd7be8424675b98eef25506aad68b604a16e47dd
route           OUTER11_GRANT_CANDIDATE
```

These are correctness/reproducibility runs, not timing or performance
measurements.

## Next action

Research/freeze one outer-11 slice candidate at `282/512` and one independent
unsliced oracle at cumulative `805/8704`, with cap 34 unchanged. Use the new
observation to decide what discriminator can distinguish AL stagnation from a
discretization floor; do not execute outer 11 before its contract freezes.
