# NSR3-B4DR0 -- external-reference recovery audit contract

Status: `FROZEN / READ_ONLY_AUDIT_AUTHORIZED`

Identity projection:

```text
nextengine.nonlocal.nsr3b4dr0-reference-recovery-audit|v1|w0i=186e1e31c0aa2636525bbc54e4fe4335b8432e7e99eddf3221d08e0380b65c90|payloads=3|source_hashes=3|locations=tmp,home,worktrees,trash,git-object-store|public-hash-lookup=diagnostic-only|decision=exact-payload,exact-generator,new-root
```

Identity SHA-256:
`30fff8ee1aa85e5dab803931daf500253f884d9277a3e852abb616194f5c4675`.

## Question

Can B4D be re-entered under its existing W0I identity from an exact retained
payload or exact retained generator lineage, or must the external comparator
receive a new independently frozen identity?

This is a read-only recovery audit. It cannot create substitute payloads,
download/build code, change W0I or authorize B4E.

## Frozen search inventory

The audit checks:

1. the three exact B4D paths and all readable user/worktree/download/trash
   locations for filenames, exact sizes and complete SHA-256 values;
2. reachable and unreachable shared Git objects for the two exact payload
   sizes and the recorded adaptation-diff, comparator-source and binary
   SHA-256 values;
3. existing NextEngine documents/history for an actual adapter source, patch,
   build recipe or durable payload location rather than a hash-only statement;
4. exact-hash public lookup as a diagnostic only; absence from search cannot
   prove global absence;
5. local availability of the pinned upstream commit and the recorded GCC/CMake
   build environment.

The three payload hashes, three generator-lineage hashes and W0I root remain
exactly those in the B4D contract.

## Classification

Exactly one result is selected:

- `EXACT_PAYLOAD_RESTORE_AVAILABLE`: all three complete payloads exist and
  match; copy only to the frozen external paths and rerun B4D twice.
- `EXACT_GENERATOR_RESTORE_AVAILABLE`: the exact adaptation diff, comparator
  source and binary are available under their recorded hashes; regenerate
  outside Git and require all three historical payload hashes before B4D.
- `NEW_REFERENCE_PROFILE_REQUIRED`: neither exact path is available. Preserve
  historical W0I/W1 evidence and freeze a new external generator/profile with
  new roots before any download, implementation or generation.

A source description, compiler-version match, source hash without source
bytes, or algorithmically similar reconstruction is insufficient for either
exact-restoration classification.

## Exit gate

The report must list evidence for and against both exact-restoration paths,
the first missing irreducible input, rejected shortcuts and the smallest next
action. B4D remains FAIL and B4E remains blocked for every result except a
later successful B4D rerun over exact historical payloads.

