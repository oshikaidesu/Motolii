import { createHash } from "node:crypto";
import { readFile, readdir } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

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
const expected = `${JSON.stringify({ version: 1, entries }, null, 2)}\n`;
const actual = await readFile(path.join(output, "asset-manifest.json"), "utf8");
if (actual !== expected) {
  process.stderr.write("generated Host asset manifest is stale\n");
  process.exitCode = 1;
}
