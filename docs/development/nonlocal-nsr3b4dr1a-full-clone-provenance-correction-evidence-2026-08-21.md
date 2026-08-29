# NSR3-B4DR1A full-clone provenance correction evidence -- 2026-08-21

Status: `PASS_RETAINED / PROVENANCE_WORDING_CORRECTED / R1C_UNCHANGED`

## Correction

The original R1A evidence described both retained upstream source trees as
ordinary full Git clones. A later object-database audit found that this was
too strong: one tree was a valid blobless partial clone with promisor
metadata, while the copied `fresh-clone` tree had lost that metadata and
reported missing historical blobs under `git fsck --full`. Its checked-out
`eccce86155776f6ac52d5080b1f720a52bf29450` working files and all compiled
inputs were present, but it was not a complete Git object database.

This correction preserves that negative provenance fact. It does not relabel
the incomplete tree as full and does not use promisor-aware `git fsck` success
as proof of completeness.

## True-full-clone revalidation

A new source tree was cloned with both local hardlink optimization and object
filtering disabled from a network-populated complete repository. Before
configure it had the exact pinned `HEAD`, no partial-clone/promisor
configuration, a clean status and passed `git fsck --full --no-dangling`.
After configure, its only status entry was the expected generated untracked
`Utilities/Version.h`.

The strict R1A profile reproduced the generated header and all eight static
artifacts exactly:

| Artifact | SHA-256 |
|---|---|
| `Utilities/Version.h` | `72be70658e43ed1f6e310de446ad5153ccb3f505f853b2040ffcf99a1ed8c900` |
| `libSPlisHSPlasH.a` | `172e6777027564566d6282f8679f4d93cbd4cdc193fc236eb9fdaa210ea07d20` |
| `libUtilities.a` | `93c6dd16ae500aae89b370b8a3bca902da3ba5a32e91a99630f66dfa89a1d0c4` |
| `libtinyexpr.a` | `63e8e8bda564e8738dfbb085a4032ba893207b4dffb2ba0b3acb25f723126cbc` |
| `libMD5.a` | `e983a90676a7200508467fc2a8e1ae3c0453515cdc77d9ca3c0e620ab0d11239` |
| `libpartio.a` | `a0f9ae32d8661ea47e848fbed167ea2ed9d1f652293736b3700eb6d360892494` |
| `libzlib.a` | `70f93ec9b3400961560606548e672b036106d82e6ba8b37aa2d870a00cbf7d97` |
| `libCompactNSearch.a` | `5d3df5a05d8148b2fae02a7cd9a81a2fe09d06af82c5a98a041255f772c87e61` |
| `libDiscregrid.a` | `8f6fa4d0f0992e0154e02554a836673ad676e3b52bc68ef773bc9c2c3885c19a` |

The historical manifest implementation at commit
`d3c22e88d4ad2b0e7e0653f91ef4f1ab54fa95f8` was then rebuilt against this
true-full-clone library closure. It reproduced:

- executable: 1,510,480 bytes,
  `c8933e011f842b0e9d5bf145555dd01a74b9da103cabc3bb6d5c0598a027b6ca`;
- R1C1 report: 1,744 bytes,
  `6d2933283281591cc1dd259053de2b68b1f27188e945c7e753ef21323eb558f3`;
- focused tests: `6/6 PASS`.

These are exactly the artifact and report identities recorded by the original
R1C1 evidence.

## Decision

R1A and R1C1 retain `PASS`: their compiled inputs, binary artifacts and
manifest outputs reproduce from a verified complete Git clone. The narrower
claim that both originally retained source trees were complete is withdrawn.
Every subsequent external reference build must prove no promisor/partial
configuration and `git fsck --full --no-dangling` success before configure;
checked-out-file equality alone is insufficient provenance evidence.

This correction grants no trajectory, R1D, B4E, runtime or production
authority beyond the authority already established by R1C1.
