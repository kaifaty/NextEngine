import { existsSync } from "node:fs";
import { delimiter, join } from "node:path";

export function resolveQmdCommand() {
  if (process.platform !== "win32") {
    return { command: "qmd", prefixArgs: [] };
  }

  const searchPath = process.env.Path ?? process.env.PATH ?? "";
  for (const directory of searchPath.split(delimiter)) {
    if (!directory) {
      continue;
    }

    const launcher = join(
      directory,
      "node_modules",
      "@tobilu",
      "qmd",
      "bin",
      "qmd",
    );
    if (existsSync(launcher)) {
      return { command: process.execPath, prefixArgs: [launcher] };
    }
  }

  throw new Error(
    "QMD global package launcher was not found on PATH. " +
      "Run: npm install --global @tobilu/qmd@2.5.3",
  );
}
