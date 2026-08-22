# NSR3-B4E2D7R11 -- binary128 accepted-sign oracle contract

Status: `FROZEN / NOT_RUN / OFFLINE_DIAGNOSTIC_ONLY`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4e2d7r11-binary128-accepted-sign-oracle|v1|parent=43de2d5c58242a7bbf5a0d721e3ad17a51293e98:48b498487db1265d28123dc2ae2edca92a66bddc2cee7714b3f6ffb77216e77f:ee7b1d4eb0b5eb37334415fa38a1ee2c2a716fa9ab5b628985e7fed06c1c710d|accepted=eta1e-8:trial0:0fea41f73c2c7260bd0b79d2a03e6b8d2a5a0f658d738e39866a7bdc45866647;eta1e-9:trial0:aed7362e7135fa622044dec448a0797bacfbd0fcbc4625731c5153a88e4beebb;eta1e-10:trial0:29d636a21d2c2a71ed6d0114d7d1ea743b2acfbfd3ba6b42746f3118f9d9b5ff|profile=linux-x86-64;gcc-float128;sizeof-float12816;flt-radix2;flt128-mant-dig113;libquadmath|oracle=independent-binary128-radius,kernel,density,phr,inertia;exact-promoted-binary64-inputs-and-kernel-scale;fixed-order-and-compensated|resolved=agree-sign-and-abs-reduction>=4096-binary128-total-ulp|gate=3-resolved-positive;0-negative;0-unresolved;divided-relative-error<=0.05;pair-membership-exact|controls=d7r10-complete-bytes;accepted-pair-roots;work-acceptance;forced-rollback|routes=binary128-sign-contradiction;offline-sign-certificate;stronger-oracle-required|precedence=contradiction,certificate,stronger|runs=2-release-builds;2-processes;byte-exact;timing=none|runtime-float128=none;outer-updates=none;trajectory=none;public-commit=none;physics-mutation=none|credit=one-offline-accepted-sign-certificate-only
```

Identity SHA-256:
`7d7a65dc2523b353d53b3ceb21b56c461b30e0b3455dc9eabd7842a4b76da809`.

## Required command

Add `--nonlocal-al-binary128-accepted-sign-oracle`. It must:

1. reproduce complete D7R10 bytes and its three exact solve/accepted pairs;
2. fail before evaluation unless the frozen compiler/type/libquadmath profile
   is exact;
3. independently evaluate current/trial radius, kernel, density, PHR, inertia
   and total energy in `__float128` from exact-promoted binary64 inputs;
4. emit fixed-order and compensated reductions, binary128 total ULPs and the
   frozen 4096-ULP resolution result;
5. compare every binary64/binary128 pair-membership decision;
6. compare each D7R10 divided reduction sign and relative magnitude to the
   compensated oracle with maximum error `0.05`;
7. preserve D7R10 work/acceptance facts, force rollback and publish no state;
8. emit exactly one route under the frozen precedence.

## Routes

1. `BINARY128_SIGN_CONTRADICTION`: any accepted reduction resolves negative.
2. `OFFLINE_ACCEPTED_SIGN_CERTIFICATE`: no preceding route; all three signs
   resolve positive, pair membership agrees and all candidate errors pass.
3. `STRONGER_ORACLE_REQUIRED`: every control passes but the complete
   certificate gate does not.

Parent, identity, accepted-pair root, profile, formula independence,
nonfinite, pair-membership, work/acceptance, rollback or precedence mismatch
is hard FAIL. PASS grants one offline three-pair sign certificate only. It
does not authorize runtime binary128, outer integration, a general error
bound, cap/tolerance, pressure gate, `beta`, kernel, state precision,
trajectory, performance, GPU/runtime or production use.

