# NSR3-B4DR1C5 trajectory evidence -- 2026-08-21

Status: `PASS / R1D_EXECUTION_AUTHORIZED / NO_R1E_B4E_RUNTIME`

## Result

All three corrected short scenarios pass twice in fresh processes. Reports and
complete `CWREFV2` payloads are byte-identical within every same-scenario pair,
all six stderr streams are empty and all six processes exit zero.

The ownership correction is physically exercised rather than accepted only by
manifest inspection. Orifice runs against analytical `x_max=2.0` while its
source-support boundary remains one metre wide, completes all 24 steps and
ends with 28 samples in the receiver region. No boundary root, sample count,
mass, volume, timestep, cold-start policy, convergence tolerance or contact
rule changed from R1C4.

## Build and profile identity

| Input | Attested value |
|---|---|
| implementation commit | `95f0eee4f66cb2275c0c4732acd2159eeba2bef1` |
| R1C5 identity | `3f5a73693f05daf487898fd692160357c1775652a1c11e65205356243c57386f` |
| executable | 1,568,808 bytes; `66c64f12cb82d420ad2b701bcfd70b516761cdbe113922d659b20bdc9536e78e` |
| ELF build ID | `b0ad6b2023aa588bbcb08256ab01391a9caf99d3` |
| focused tests | `12/12 PASS` |

The verified ordinary full upstream clone remains pinned at
`eccce86155776f6ac52d5080b1f720a52bf29450`. Its complete-object
`git fsck --full --no-dangling` closure and the eight reproduced upstream
library hashes are unchanged from R1A. The tracked cold-start/diagnostic patch
remains the only upstream source patch. R1B report SHA-256
`c6a4950dc5fa1db6f2269009111e8f711582327857a87110187518714213ea8a`
and R1C1 manifest report SHA-256
`6d293328ce6f4dab5065558b54e7f6eec9a0927a53b2d6d2c4791065343988f3`
remain exact.

## Paired trajectory results

| Scenario | Payload bytes | Payload SHA-256 | Report SHA-256 | Max P/V iterations | Contact hits | Final receiver | Wall pair | Peak RSS pair |
|---|---:|---|---|---|---:|---:|---:|---:|
| `CW-HYDRO-001` | 7,804,813 | `9c58556d8ab93ef346181c1fe96befb1d6533e181ac6c2acf689f23bf3d73c63` | `cb5c19fad36bd9390dff4b09dd259ea1b2b2a4270bdf68a53b60b8beec747835` | `220 / 12` | 19,056 | 0 | 2.37 / 2.41 s | 34,268 / 34,472 KiB |
| `CW-DAMBREAK-001` | 7,804,810 | `f5a6045dbb5478d112d540bbea3f7814d000982fcb2ff2fb1d234777a80f9c7d` | `b24ccf21ac3be3321ad7eb44e5c5782f01b9645c4ac9f9b9f7b86cf69ab2b3c0` | `203 / 9` | 18,537 | 0 | 1.90 / 1.91 s | 37,152 / 37,456 KiB |
| `CW-ORIFICE-001` | 7,804,912 | `af31b4fd3af4b8ea3cd83b07f01eb4e7dd3f6807b993bb3377b7c3afa876b18a` | `5431e532ae0fc579489e26b81240bc61b0bfb26fd87017bd0b98f2126e2d4619` | `205 / 13` | 20,008 | 28 | 2.46 / 2.48 s | 34,556 / 34,424 KiB |

Each row describes two independently created output directories. The payload
and report sizes and SHA-256 values match across its pair. The suite executes
in normative Hydro, Dam, Orifice order and encounters no fail-stop boundary.

## Decision

R1C5 passes and supersedes no negative evidence from R1C through R1C4. It
authorizes implementation and execution of R1D full external generation only,
under pressure cap 300 and the corrected R1C5 geometry/profile lineage.

It grants no R1E attestation, B4E nominal-corpus design, old W0I/W1 credit,
runtime authority, public schema or production claim. Full schedules must run
twice from fresh processes, publish into explicit content-addressed external
artifact roots and remain byte-identical per scenario before R1E can be
designed.
