import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { spawnSync } from "node:child_process";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";
import assert from "node:assert/strict";
import { repositoryFingerprint } from "../scripts/repository-fingerprint.mjs";

function git(root, ...args) {
  const result = spawnSync("git", args, {
    cwd: root,
    encoding: "utf8",
  });
  if (result.status !== 0) {
    throw new Error(result.stderr || `git ${args[0]} failed`);
  }
  return result.stdout;
}

function gitStatus(root) {
  return git(root, "status", "--porcelain=v1", "-z", "--untracked-files=all");
}

async function temporaryRoot(run) {
  const root = await mkdtemp(path.join(tmpdir(), "motolii-repository-fingerprint-"));
  try {
    await run(root);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
}

function expectReadOnly(root, run) {
  const before = gitStatus(root);
  return run().then((value) => {
    const after = gitStatus(root);
    assert.equal(before, after);
    return value;
  });
}

function commitWithOneFile(root) {
  git(root, "init");
  git(root, "add", "tracked.txt");
  git(
    root,
    "-c",
    "user.name=motolii-test",
    "-c",
    "user.email=test@motolii.local",
    "commit",
    "-m",
    "seed",
  );
}

test("is deterministic for unchanged repository bytes and status", async () => {
  await temporaryRoot(async (root) => {
    await writeFile(path.join(root, "tracked.txt"), "initial");
    commitWithOneFile(root);

    const first = await expectReadOnly(root, () => repositoryFingerprint(root));
    const second = await expectReadOnly(root, () => repositoryFingerprint(root));
    assert.equal(first, second);
  });
});

test("changes when a tracked file changes and when an untracked file is added", async () => {
  await temporaryRoot(async (root) => {
    await writeFile(path.join(root, "tracked.txt"), "initial");
    commitWithOneFile(root);
    const baseline = await expectReadOnly(root, () => repositoryFingerprint(root));

    await writeFile(path.join(root, "tracked.txt"), "mutated");
    const trackedChanged = await expectReadOnly(root, () => repositoryFingerprint(root));
    assert.notEqual(trackedChanged, baseline);

    await writeFile(path.join(root, "new-untracked.txt"), "untracked");
    const untrackedAdded = await expectReadOnly(root, () => repositoryFingerprint(root));
    assert.notEqual(untrackedAdded, trackedChanged);
  });
});
