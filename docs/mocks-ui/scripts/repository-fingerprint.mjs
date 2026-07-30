import { createHash } from "node:crypto";
import { execFile } from "node:child_process";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { promisify } from "node:util";

const execFileAsync = promisify(execFile);

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

export async function repositoryFingerprint(root) {
  const { stdout: names } = await execFileAsync(
    "git",
    ["ls-files", "-co", "--exclude-standard", "-z"],
    { cwd: root, encoding: "buffer", maxBuffer: 32 * 1024 * 1024 },
  );
  const files = names.toString("utf8").split("\0").filter(Boolean).sort();
  const digest = createHash("sha256");
  for (const filename of files) {
    digest.update(filename).update("\0").update(await readFile(path.join(root, filename)));
  }
  const { stdout: status } = await execFileAsync(
    "git",
    ["status", "--porcelain=v1", "-z", "--untracked-files=all"],
    { cwd: root, encoding: "buffer", maxBuffer: 32 * 1024 * 1024 },
  );
  return sha256(Buffer.concat([Buffer.from(digest.digest("hex")), status]));
}
