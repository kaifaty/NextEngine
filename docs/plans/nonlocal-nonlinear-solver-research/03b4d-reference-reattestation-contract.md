# NSR3-B4D -- external reference re-attestation contract

Status: `FROZEN / EXECUTION_AUTHORIZED / EXTERNAL_ARTIFACTS_REQUIRED`

Identity projection:

```text
nextengine.nonlocal.nsr3b4d-reference-reattestation|v1|formula=nuv-variational-fcr2|boundary=split-static-boundary-r0|solver=nuv-newton-krylov-r0+outer-state-hessian-tape-v1|canonical=balanced-macro-publication|packaging=complete-lane-flat-adjacency-candidate|w0i=186e1e31c0aa2636525bbc54e4fe4335b8432e7e99eddf3221d08e0380b65c90|file_count=3|max_bytes=33554432|mutation=memory-byte-flip-before-trajectory
```

Identity SHA-256:
`47c78bdb115c0e5d7ed7132a6de3e62e3ba9533396a7f346b510b354dc5222fe`.

## Purpose

B4D is a fail-closed provenance and input-integrity gate between the packaged
B4C4 pressure runner and any nominal B4E trajectory. It does not compare a
Nonlocal trajectory with DFSPH and cannot issue physical, performance, CUDA,
runtime or production credit.

## Frozen parent inputs

The design preflight at commit `c9ff7aaef6393af0b3059b0390f8064c23fd5f5b`
records these unchanged inputs:

| Input | SHA-256 or identity |
|---|---|
| corrected formula contract | `8d693724d6b32d4aa899f57551b45248d645a1ecdcec3ec20cd66572b4a1c5ab` |
| normalized-objective contract | `97d74e24452df09fa56ed64b0f8f7e1c07c93f85ca152a75896f797b5094f3bb` |
| macro-publication contract | `eecac76c8a902dfa5b10979dbc4afa3e8fbccd473c0cba5e0a807fa3e4764617` |
| tiny adaptive-accuracy contract | `4eccdd0e1f78cb507fcbafd843d9926dc1a6822e4d1bc541b75ed39278f11299` |
| complete flat-adjacency contract | `fa45981154617c0f22474d485f3fa7030f7a498c263905c4c09ba87d3853e606` |
| W0I reference contract | `21a36b1eb17f5a710d5fa5848a7a2555992cade4ddd717bf310b558250345204` |
| W0I reader source | `58c15b58c9720072fc8cbe177540e9d5ddb7b35a9dbdb538e5ba10c07b7683df` |
| B4C4C1 semantic result | `b4d5260012f4208026b814411891a221dda2d5823b242027d890d4066e69550c` |
| B4C4C1 identity | `66e318cb69e0b0c0a3a40a2beafa2099ebf151287581b191a242e82dac6d6f3c` |

Any mismatch is `PARENT_IDENTITY_MISMATCH`; it requires a new B4D contract and
must not be hidden by regenerating an expected value.

## Required external files

All files remain untracked external evidence. B4D reads them but never copies
them into the repository.

| Scenario | Path | Outputs | Bytes | Complete-file SHA-256 | W0H scenario root |
|---|---|---:|---:|---|---|
| `CW-HYDRO-001` | `/tmp/cwref-hydro-hard-contact-final.bin` | 51 | 7,344,252 | `84ae867f5b336cd0bd51be6f29a6a2a1f27f702c424f1dbd0a8f735b9f4bb435` | `030357596535f052469e1cc2729859106ba58baadaf49ec2d84eceff46521e63` |
| `CW-DAMBREAK-001` | `/tmp/cwref-dam-hard-contact-final.bin` | 181 | 26,064,772 | `853d965489a40082a024aeee5a19f98aef054417014af8556fa212687d88d12c` | `fb779545fd4429e74158d773b1aa51bd12af55c5d1b821251c9016bfae97f94c` |
| `CW-ORIFICE-001` | `/tmp/cwref-orifice-hard-contact-final.bin` | 181 | 26,064,772 | `60e9b3538d621ef1a3f1ae77569640740df471fbe4e5ef8eaa813d3751930849` | `13ab66fbbe7b167f3e69f8ddd0e3887120a9f586b57ff426cb240a8e66f991e1` |

The common encoded sample count is exactly `6,000`. A file must not exceed
`33,554,432` bytes.

## Reader and failure gates

For each file, in the table order, the B4D reader must:

1. fail closed on missing, non-regular, unreadable or oversized input before
   allocating the payload;
2. read the exact preflight byte count and reject a size change during read;
3. require magic `CWREFV1\0`, the frozen 32-byte scenario root, exact output
   count, exact sample count, exact step cadence and exact total byte length;
4. require the frozen complete-file SHA-256;
5. flip the final byte in a private in-memory copy and prove the mutation is
   rejected by the same complete-file hash gate;
6. retain no payload after the report is constructed.

The fixed output steps are `0..1200` by 24 for hydro and `0..720` by 4 for
dam-break and orifice. Integer fields are little-endian, matching the frozen
W0I `CWREFV1` reader.

The first failing file and reason are deterministic. Missing input is
`MISSING_ARTIFACT`, not a solver or physics failure. No candidate trajectory
may start unless all three positive readers and all three mutation controls
pass in one process.

## Execution and exit gate

The research binary exposes `--reference-attestation-self-test`. Two complete
executions must be byte-identical and report:

- the B4D identity and frozen W0I attestation root;
- expected and observed file size/hash for all three files;
- format/profile and mutation-rejection booleans;
- `trajectory_started=false`;
- `external_reference_candidate_selected=true` only when every gate passes.

A PASS selects only `EXTERNAL_REFERENCE_REATTESTED_CANDIDATE` and authorizes a
separately frozen B4E nominal pressure-water corpus design. A missing or
mismatched artifact leaves B4E, CUDA, runtime schemas and production blocked.

