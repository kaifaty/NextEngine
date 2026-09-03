---
name: blender-scripting
description: >-
  Write and run Blender Python scripts for 3D automation and procedural
  modeling. Use when the user wants code-driven Blender scene changes, batch
  processing, geometry generation, modifiers, or import/export.
license: Apache-2.0
metadata:
  author: terminal-skills
  version: "1.1.0"
  category: automation
  tags: ["blender", "3d", "python", "automation", "procedural"]
  compatibility: >-
    Requires Blender 3.0+ installed and accessible from the command line.
---

# Blender Scripting

Produce the requested `.blend`, exported model or render through a small,
repeatable `bpy` script. The visual/model artifact is primary; a reusable
procedural framework is not implied.

## Workflow

1. Inspect the Blender version, source file and requested output. Preserve an
   existing source by saving to a new path unless overwrite was explicit.
2. Write the smallest script that performs the requested scene operation.
3. Run headlessly:

   `blender [source.blend] --background --python script.py -- [arguments]`

4. Verify the resulting scene/model with task-relevant facts such as object
   names, mesh counts, dimensions, modifiers and export existence.
5. When appearance matters, render a preview and inspect it before handoff.

## Useful invariants

- Blender is Z-up. Convert coordinate conventions explicitly at import/export
  boundaries.
- Operators depend on selection, active object, mode and view-layer context.
  Prefer direct `bpy.data` or `bmesh` operations when they avoid fragile
  operator context.
- Link created objects to a collection, call `mesh.update()` after
  `from_pydata()`, and release temporary `bmesh` objects with `bm.free()`.
- Apply modifiers only when the requested output needs baked geometry.
- Import/export operator names vary by Blender version; inspect the installed
  API instead of assuming an example matches.
- For batch work, make item failures explicit and do not silently overwrite all
  sources.

Do not install Blender, create add-ons, build a generic asset pipeline or add
rendering/material polish unless the user's requested artifact needs it.
