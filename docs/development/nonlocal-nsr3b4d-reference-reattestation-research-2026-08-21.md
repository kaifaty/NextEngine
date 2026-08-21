# NSR3-B4D external-reference re-attestation research -- 2026-08-21

Status: `FROZEN / LOCAL_IDENTITIES_ATTESTED / EXTERNAL_ARTIFACTS_MISSING`

## Finding

B4C4 packaging is complete and the current packaged-runner probe still emits
the selected B4C4C1 identity
`66e318cb69e0b0c0a3a40a2beafa2099ebf151287581b191a242e82dac6d6f3c`
and semantic result
`b4d5260012f4208026b814411891a221dda2d5823b242027d890d4066e69550c`.
The corrected formula, normalized objective, macro-publication, tiny accuracy,
complete packaging, W0I contract and W0I reader files also reproduce the exact
SHA-256 values frozen in the B4D contract.

The external state did change: none of the three required
`/tmp/cwref-*-hard-contact-final.bin` files is currently present. A search of
the workspace, other local NextEngine worktrees, Downloads and desktop/trash
locations found no surviving copy. This is an evidence-availability failure,
not evidence against the Nonlocal solver or the DFSPH curves.

## Decision

Freeze [B4D](../plans/nonlocal-nonlinear-solver-research/03b4d-reference-reattestation-contract.md)
as an executable fail-closed reader gate. The gate binds the exact formula,
solver, publication, packaging and W0I identities; validates complete external
files and their `CWREFV1` headers; and performs a one-byte mutation control in
memory before any trajectory can start.

Do not reconstruct plausible aggregate curves, substitute the old rejected
non-clearance files, accept a hash-only manifest as file availability, or run
B4E without the three exact payloads. The original comparator adaptation and
large trajectories were deliberately kept outside Git, so reproducing them
requires the original external generator lineage or an independently reviewed
new reference profile; an approximate regeneration cannot inherit the W0I
hashes.

## Next action

Implement the frozen reader and execute it against current external state. A
deterministic `MISSING_ARTIFACT` result preserves the boundary and provides a
ready re-entry point when the exact files are restored. Only a full B4D PASS
can authorize B4E design.

