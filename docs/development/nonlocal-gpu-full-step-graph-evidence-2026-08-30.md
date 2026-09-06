# NCGP1 scalable graph baseline evidence — 2026-08-30

| Field | Value |
| --- | --- |
| Research ID | `NCGP1` revision 1, graph checkpoint |
| Result | `AUTHOR_SUPPORTED_BOUNDED / GRAPH_BASELINE_PASS` |
| Product status | `REPORT_ONLY`; no formula, solver, trajectory or performance claim |
| Host | Linux x86-64, RTX 3080 `sm_86`, CUDA compiler/runtime 13.3 |

## Outcome

The persistent workspace admits the frozen 16k/50k dynamic states plus the
unique three-layer analytical-basin ghost shell. It constructs current CSR on
device from quantized micrometre positions, retains canonical ascending IDs
under a fixed input permutation and rejects inclusive-radius and capacity
wrong identities through the common graph path.

| Profile | Dynamic | Ghost | Directed IDs | Maximum degree | Graph root |
| --- | ---: | ---: | ---: | ---: | --- |
| support edge | 4 | 0 | bounded tiny | bounded tiny | `51472ea7e0e3d677b2386ee68a8ad8da196c9e1557489608646096bb5ff761e6` |
| lattice 4x4x4 | 64 | 0 | bounded tiny | bounded tiny | `5b442c487cdf6d4c5eb823620b0612c210583724fc6a9f9b22a95208efb817a7` |
| water 16k | 16,000 | 43,056 | 1,755,688 | 123 | payload not copied at this checkpoint |
| water 50k | 50,000 | 43,056 | 5,711,868 | 123 | `9f1ef440f7ca1c760c8f4586b1445d87908d8c5d05ef07ecf5372c96a344aed3` |

The coherent and fixed-permuted 50k inputs produce the same complete graph
root. The deliberate strict `<` support variant differs on the exact-radius
fixture. A 257-sample coincident row returns `CapacityExceeded` instead of
truncating or writing beyond the fixed 256-neighbor row.

The workspace reserves `63,049,697` device bytes, including the complete
`50,000 * 256` neighbor capacity and sort/scan scratch. A single exploratory
hot graph call measured approximately `3.49 ms`; it is not a percentile or
full-step result and has no performance status.

## Identity and repeatability

| Artifact | SHA-256 |
| --- | --- |
| NCGP1 contract | `334418de01d0d8a1964331693e7d6846daef2cfb78fc93cc02d7c1bcf3ef32d7` |
| public header | `15210868f84563fc2e10c63a63b08ee69b7442c65857819d1a7ea923091a786d` |
| CUDA workspace/graph | `a358d9bf828608b9c242ddcb0710f81152fd23eae07b9d6bbbe51e82df8d3bca` |
| independent graph/profile reference | `227572c614471322be9efe07dbe6b8237bf22ed3868b410b20e771ebd4dc7139` |
| graph harness | `4618e41c318403a718805aad194cd9d679190bf850bb4fde4e3bcda0be48a911` |
| clean Release binary A/B | `927d76e7357113f26b71176fb02528e5308547d9f43b4680cb2ba5a041f53f68` |
| exact stdout A/B | `e585fe337f47b18a680cd5d7c91389e9632d7263e9bce2338ae8a428f817baf1` |

Fresh build directories were
`/tmp/nextengine-ncgp1-graph2-a.rOoJjd` and
`/tmp/nextengine-ncgp1-graph2-b.pYfOcW`. Both used

```sh
cmake -S crates/continuum-water/tools/nonlocal-feasibility \
  -B <build> -G Ninja -DCMAKE_BUILD_TYPE=Release
cmake --build <build> --target nonlocal-corrected-cuda-full-step -j 4
<build>/nonlocal-corrected-cuda-full-step --graph-self-test
```

The binaries and stdout streams are byte-identical. Raw build products remain
outside Git.

## Ceiling and next action

This checkpoint establishes only bounded author evidence for scalable graph,
stable ID, basin-shell capacity and admission behavior. Compute Sanitizer and
independent review remain deferred until the complete physical candidate.
The next gate is an independently compared matrix-free corrected
energy/gradient/HVP; no trajectory or full timing is authorized yet.
