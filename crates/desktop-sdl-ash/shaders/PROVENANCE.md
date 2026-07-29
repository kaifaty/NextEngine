# B0 textured shader provenance

The `b0_textured` vertex and fragment modules are engine-owned shader assets for
the minimal indexed rendering path. They are distributed under the repository
license.

The checked-in SPIR-V modules are generated offline from the adjacent GLSL 4.50
sources. Runtime shader compilation is not permitted.

## Pinned compiler invocation

Compiler:

```text
glslangValidator
Glslang Version: 10:11.7.0
GLSL Version: 4.60 glslang Khronos. 11.7.0
SPIR-V Version 0x00010500, Revision 4
Khronos Tool ID 8
SPIR-V Generator Version 10
```

Commands, executed with this directory as the current directory:

```text
glslangValidator --quiet -V --target-env vulkan1.2 -S vert -e main -o b0_textured.vert.spv b0_textured.vert
glslangValidator --quiet -V --target-env vulkan1.2 -S frag -e main -o b0_textured.frag.spv b0_textured.frag
```

Vulkan 1.2 is the highest Vulkan target accepted by this pinned compiler and
emits SPIR-V 1.5. The modules are valid inputs to the engine's Vulkan 1.3
baseline. The exact source and module hashes are recorded in `manifest.json`.

The modules in this revision were produced by the copy of
`glslangValidator.exe` shipped with the locally installed Android Emulator.
The absolute installation path is deliberately excluded because it is not a
portable build input. That compiler executable is 7,572,296 bytes and has
SHA-256
`0b7b2ea92af3f4b74f4bea9b4389dd2bbda3bed345cc8fece8a8d22bcf7b9d7f`.
