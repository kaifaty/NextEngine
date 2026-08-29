# Nonlocal external-reference reader

This is the standalone R1E research reader for the new external DFSPH
reference profile. It is not runtime code, a public ABI, a persistence format
or a solver plugin. It deliberately does not link SPlisHSPlasH and does not
reuse the R1D generator parser, serializer, manifests or q99 implementation.

Build out of tree:

```text
cmake -S crates/continuum-water/tools/nonlocal-reference-reader \
  -B <external-build> -G Ninja -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_CXX_COMPILER=/usr/bin/g++
cmake --build <external-build> --parallel
ctest --test-dir <external-build> --output-on-failure
```

The frozen profile can be checked without reading an artifact:

```text
LC_ALL=C nonlocal-reference-reader --profile-self-test
```

Positive attestation requires the explicit absolute external artifact root
that contains the frozen R1D profile directory:

```text
LC_ALL=C nonlocal-reference-reader --attest \
  /home/kaifaty/.cache/nextengine/reference-artifacts
```

Relative roots, `/tmp`, symlinked path components, non-regular payloads,
capacity/size/hash/manifest/layout/semantic/aggregate mismatches and non-finite
decoded values fail closed. The canonical report omits the host-specific root
string so equivalent explicit roots can produce the same attestation bytes.
