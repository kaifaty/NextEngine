# NSR3-B4E2D7R2R -- signed-zero reclosure contract

Status: `FROZEN / NOT_RUN / REPLAY_ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r2r-signed-zero-reclosure|v1|parent=7b201ab9d281f8ffa3e866e88b067682d116ef3cf38a6507da4a3e4dea99561e:efd1f8d003a66a4c76544a17697c99c74232257ac2fde56caee520b6115904b3:f6b18542b1a25a548a11925552f014148152c817c8de557a86f9e97f4c628bcb|kernel-horizon=numeric-zero;bits=0,9223372036854775808,0;publish-signbits;no-canonicalization|trace=unchanged-d7r2-internal;parent-report-byte-exact;full-step-exact;sets=active8,fluid28,boundary800;full-delta=active0-0,fluid0-0,boundary72-120;alpha-half=topology-stable,predicted-positive,direct-positive,ratio>=0.1|route-precedence=unchanged-d7r2;expected-not-forced=trust-reject-policy-reclosure|runs=2-release-builds;2-processes;byte-exact;timing=none|trajectory=none;commit=none;physics-mutation=none|credit=selected-route-research-only
```

Identity SHA-256:
`034d39324962a5552d5a87d3cb5759e18961401fc87cd9b49e4581e4414b45d0`.

## Required command

Add `--nonlocal-al-topology-step-signed-zero-reclosure`. It must:

1. recompute the unchanged D7R2 diagnostic and require its failed report plus
   stdout-with-LF SHA
   `f6b18542b1a25a548a11925552f014148152c817c8de557a86f9e97f4c628bcb`;
2. publish horizon kernel values, raw bits and sign bits;
3. require numerical zero and exact bits
   `[0,9223372036854775808,0]` without altering kernel results;
4. require exact D7R1 full step, current set counts, full set deltas and the
   topology-stable half-step descent row;
5. apply unchanged D7R2 route precedence and emit exactly one route;
6. keep public state, commit count and all parent command bytes exact.

## Gates

- identity and parent report hash are exact;
- all D7R2 gates except its superseded positive-zero-bit predicate pass;
- horizon values compare equal to `0.0` and exact signed-zero bits match;
- current counts are active `8`, fluid `28`, boundary `800`;
- full delta is active `0/0`, fluid `0/0`, boundary `72/120`;
- exponent-one row moves coordinates, changes no set, has positive predicted
  and direct reduction and direct ratio at least `0.1`;
- route is exactly `TRUST_REJECT_POLICY_RECLOSURE_REQUIRED` under unchanged
  precedence;
- rollback is exact and no state/trajectory/physics/timing mutation occurs.

Any mismatch is hard FAIL. PASS authorizes only trust/globalization policy
research and a separately frozen discriminator. It does not authorize a
policy implementation, larger reject cap, fixed half-step, tighter inner
tolerance, solver PASS, trajectory, performance, runtime, GPU/PhysX or
production work.
