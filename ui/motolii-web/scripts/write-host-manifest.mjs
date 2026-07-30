import { createHash } from "node:crypto";
import { readFile, readdir, writeFile } from "node:fs/promises";
import path from "node:path";

const root = path.resolve(import.meta.dirname, "..");
const output = path.join(root, "generated-host");
const entries = [];

async function walk(directory) {
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const absolute = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      await walk(absolute);
    } else if (entry.isFile()) {
      const relative = path.relative(output, absolute).replaceAll(path.sep, "/");
      if (relative !== "asset-manifest.json") {
        const bytes = await readFile(absolute);
        entries.push({
          path: relative,
          bytes: bytes.byteLength,
          sha256: createHash("sha256").update(bytes).digest("hex"),
        });
      }
    } else {
      throw new Error(`unexpected host bundle entry ${absolute}`);
    }
  }
}

await walk(output);
entries.sort((left, right) => left.path.localeCompare(right.path));
await writeFile(
  path.join(output, "asset-manifest.json"),
  `${JSON.stringify({ version: 1, entries }, null, 2)}\n`,
);
