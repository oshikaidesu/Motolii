import { createHash } from "node:crypto";
import { mkdir, mkdtemp, readFile, rm, symlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";
import assert from "node:assert/strict";
import {
  CurrentRouteProvenanceError,
  computeCurrentRouteSourceClosure,
  verifyCurrentRouteSourceClosure,
  validateCurrentRouteProvenance,
  currentRouteSourceManifestSha256,
} from "../scripts/current-route-provenance.mjs";

const BASE_FILES = {
  "package.json": `
{
  "name": "current-route-proof-fixture",
  "dependencies": {
    "@motolii/motolii-web": "file:ui/motolii-web"
  }
}
`,
  "src/main.jsx": `
import { marker } from "./dep.jsx";
import asset from "./asset.txt?raw";
import css from "./shared.css";
import { token } from "@motolii/motolii-web";
export { marker } from "./reexport.js";
export { token as workspaceToken };
export const captured = asset + marker + token;
`,
  "src/reexport.js": `
export { marker } from "./dep.jsx";
`,
  "src/dep.jsx": `
export const marker = "marker";
`,
  "src/asset.txt": "fixture-bytes",
  "src/shared.css": `
@import "./theme.css";
.shared { background: url("./assets/pixel.png"); }
`,
  "src/theme.css": `
:root { --route-probe: 1; }
`,
  "src/assets/pixel.png": Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]),
  "ui/motolii-web/package.json": `
{
  "name": "@motolii/motolii-web",
  "module": "src/index.js",
  "main": "src/index.js",
  "exports": {
    ".": "./src/index.js"
  }
}
`,
  "ui/motolii-web/src/index.js": `
export { token } from "./token.js";
`,
  "ui/motolii-web/src/token.js": `
export const token = "token";
`,
};

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function expectCode(code, run) {
  return assert.rejects(Promise.resolve().then(run), (error) => {
    assert.ok(error instanceof CurrentRouteProvenanceError);
    assert.equal(error.code, code);
    return true;
  });
}

async function writeTree(root, files = BASE_FILES) {
  for (const [relative, contents] of Object.entries(files)) {
    const filename = path.join(root, relative);
    await mkdir(path.dirname(filename), { recursive: true });
    if (Buffer.isBuffer(contents)) {
      await writeFile(filename, contents);
    } else {
      await writeFile(filename, contents.trimStart());
    }
  }
}

async function withFixture(run) {
  const root = await mkdtemp(path.join(tmpdir(), "motolii-current-route-provenance-"));
  await writeTree(root);
  try {
    await run(root);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
}

async function manifestFromEntries(root) {
  const assets = await computeCurrentRouteSourceClosure({
    repoRoot: root,
    entries: ["src/main.jsx"],
  });
  return { schemaVersion: 2, assets };
}

function entryPaths() {
  return [
    "src/asset.txt",
    "src/main.jsx",
    "src/reexport.js",
    "src/dep.jsx",
    "src/shared.css",
    "src/theme.css",
    "src/assets/pixel.png",
    "ui/motolii-web/src/index.js",
    "ui/motolii-web/src/token.js",
  ].sort();
}

test("accepts full transitive closure with export-from, ?raw, css and workspace package", async () => {
  await withFixture(async (root) => {
    const closure = await computeCurrentRouteSourceClosure({
      repoRoot: root,
      entries: ["src/main.jsx"],
    });
    assert.deepEqual(
      closure.map((entry) => entry.path),
      entryPaths(),
    );
    for (const { path: entryPath } of closure) {
      const expected = await sha256(await readFile(path.join(root, entryPath)));
      const actual = closure.find((entry) => entry.path === entryPath)?.sha256;
      assert.equal(actual, expected);
    }
    const provenance = await manifestFromEntries(root);
    await verifyCurrentRouteSourceClosure({
      repoRoot: root,
      entries: ["src/main.jsx"],
      provenance,
    });
    assert.equal(
      currentRouteSourceManifestSha256(
        Buffer.from(JSON.stringify(provenance)),
      ),
      sha256(Buffer.from(JSON.stringify(provenance))),
    );
  });
});

test("rejects dynamic import in closure traversal", async () => {
  await withFixture(async (root) => {
    await writeFile(path.join(root, "src/main.jsx"), 'export const probe = import("./dep.jsx");');
    await expectCode("CRP-IMPORT", () =>
      computeCurrentRouteSourceClosure({
        repoRoot: root,
        entries: ["src/main.jsx"],
      }),
    );
  });
});

test("rejects require and export-all traversal shortcuts", async () => {
  await withFixture(async (root) => {
    await writeFile(path.join(root, "src/main.jsx"), 'require("./dep.jsx");');
    await expectCode("CRP-IMPORT", () =>
      computeCurrentRouteSourceClosure({ repoRoot: root, entries: ["src/main.jsx"] }),
    );
    await writeFile(path.join(root, "src/main.jsx"), 'export * from "./dep.jsx";');
    const closure = await computeCurrentRouteSourceClosure({
      repoRoot: root,
      entries: ["src/main.jsx"],
    });
    assert.deepEqual(
      closure.map((entry) => entry.path),
      ["src/dep.jsx", "src/main.jsx"],
    );
  });
});

test("rejects unresolved import", async () => {
  await withFixture(async (root) => {
    await writeFile(path.join(root, "src/main.jsx"), 'import "./missing.js";');
    await expectCode("CRP-IMPORT", () =>
      computeCurrentRouteSourceClosure({
        repoRoot: root,
        entries: ["src/main.jsx"],
      }),
    );
  });
});

test("rejects a relative import that escapes the repository", async () => {
  await withFixture(async (root) => {
    await writeFile(path.join(root, "src/main.jsx"), 'import "../../../outside.js";');
    await expectCode("CRP-PATH", () =>
      computeCurrentRouteSourceClosure({
        repoRoot: root,
        entries: ["src/main.jsx"],
      }),
    );
  });
});

test("resolves an extensionless directory index", async () => {
  await withFixture(async (root) => {
    await mkdir(path.join(root, "src/folder"));
    await writeFile(path.join(root, "src/folder/index.js"), "export const indexed = true;");
    await writeFile(path.join(root, "src/main.jsx"), 'export { indexed } from "./folder";');
    const closure = await computeCurrentRouteSourceClosure({
      repoRoot: root,
      entries: ["src/main.jsx"],
    });
    assert.deepEqual(
      closure.map((entry) => entry.path),
      ["src/folder/index.js", "src/main.jsx"],
    );
  });
});

test("rejects symlink file in closure", async () => {
  await withFixture(async (root) => {
    await symlink(
      path.join(root, "src", "dep.jsx"),
      path.join(root, "src", "linked.jsx"),
    );
    await writeFile(
      path.join(root, "src/main.jsx"),
      `import { marker } from "./linked.jsx";\nexport { marker };\n`,
    );
    await expectCode("CRP-SYMLINK", () =>
      computeCurrentRouteSourceClosure({
        repoRoot: root,
        entries: ["src/main.jsx"],
      }),
    );
  });
});

test("rejects a regular file reached through a symlinked directory", async () => {
  await withFixture(async (root) => {
    await symlink(path.join(root, "src"), path.join(root, "linked-src"));
    await expectCode("CRP-SYMLINK", () =>
      computeCurrentRouteSourceClosure({
        repoRoot: root,
        entries: ["linked-src/dep.jsx"],
      }),
    );
  });
});

test("resolves file workspace dependencies from the importing package owner", async () => {
  await withFixture(async (root) => {
    await mkdir(path.join(root, "nested"));
    await writeFile(
      path.join(root, "nested/package.json"),
      JSON.stringify({
        dependencies: { "@motolii/motolii-web": "file:../ui/motolii-web" },
      }),
    );
    await writeFile(
      path.join(root, "nested/main.jsx"),
      'import { token } from "@motolii/motolii-web"; export { token };',
    );
    const closure = await computeCurrentRouteSourceClosure({
      repoRoot: root,
      entries: ["nested/main.jsx"],
    });
    assert.deepEqual(
      closure.map((entry) => entry.path),
      [
        "nested/main.jsx",
        "ui/motolii-web/src/index.js",
        "ui/motolii-web/src/token.js",
      ],
    );
  });
});

test("rejects unresolved workspace package subpaths", async () => {
  await withFixture(async (root) => {
    await writeFile(
      path.join(root, "src/main.jsx"),
      'import token from "@motolii/motolii-web/private"; export { token };',
    );
    await expectCode("CRP-IMPORT", () =>
      computeCurrentRouteSourceClosure({ repoRoot: root, entries: ["src/main.jsx"] }),
    );
  });
});

test("strips queries from CSS imports and URLs", async () => {
  await withFixture(async (root) => {
    await writeFile(
      path.join(root, "src/shared.css"),
      '@import url(./theme.css?inline); .shared { background: url("./assets/pixel.png?url#pixel"); }',
    );
    const closure = await computeCurrentRouteSourceClosure({
      repoRoot: root,
      entries: ["src/shared.css"],
    });
    assert.deepEqual(
      closure.map((entry) => entry.path),
      ["src/assets/pixel.png", "src/shared.css", "src/theme.css"],
    );
  });
});

test("rejects provenance schema mismatches", async () => {
  await withFixture(async (root) => {
    const provenance = await manifestFromEntries(root);
    const badVersion = { ...provenance, schemaVersion: 1 };
    await expectCode("CRP-SCHEMA", () =>
      validateCurrentRouteProvenance(badVersion),
    );
    const extra = { ...provenance, extra: true };
    await expectCode("CRP-SCHEMA", () => validateCurrentRouteProvenance(extra));
    const absolutePath = {
      ...provenance,
      assets: provenance.assets.map((entry, index) =>
        index === 0 ? { ...entry, path: "/src/main.jsx" } : entry,
      ),
    };
    await expectCode("CRP-SCHEMA", () =>
      validateCurrentRouteProvenance(absolutePath),
    );
  });
});

test("rejects declared path mismatch by omission and extra declarations", async () => {
  await withFixture(async (root) => {
    const provenance = await manifestFromEntries(root);
    const missingExport = {
      ...provenance,
      assets: provenance.assets.filter((entry) => entry.path !== "src/dep.jsx"),
    };
    await expectCode("CRP-MISSING", () =>
      verifyCurrentRouteSourceClosure({
        repoRoot: root,
        entries: ["src/main.jsx"],
        provenance: missingExport,
      }),
    );

    const missingRaw = {
      ...provenance,
      assets: provenance.assets.filter((entry) => entry.path !== "src/asset.txt"),
    };
    await expectCode("CRP-MISSING", () =>
      verifyCurrentRouteSourceClosure({
        repoRoot: root,
        entries: ["src/main.jsx"],
        provenance: missingRaw,
      }),
    );

    const missingCssAsset = {
      ...provenance,
      assets: provenance.assets.filter((entry) => entry.path !== "src/assets/pixel.png"),
    };
    await expectCode("CRP-MISSING", () =>
      verifyCurrentRouteSourceClosure({
        repoRoot: root,
        entries: ["src/main.jsx"],
        provenance: missingCssAsset,
      }),
    );

    const missingWorkspace = {
      ...provenance,
      assets: provenance.assets.filter(
        (entry) => entry.path !== "ui/motolii-web/src/index.js",
      ),
    };
    await expectCode("CRP-MISSING", () =>
      verifyCurrentRouteSourceClosure({
        repoRoot: root,
        entries: ["src/main.jsx"],
        provenance: missingWorkspace,
      }),
    );

    const extra = {
      ...provenance,
      assets: [
        ...provenance.assets,
        { path: "src/unrelated.js", sha256: "0".repeat(64) },
      ].sort((left, right) => (left.path < right.path ? -1 : left.path > right.path ? 1 : 0)),
    };
    await expectCode("CRP-EXTRA", () =>
      verifyCurrentRouteSourceClosure({
        repoRoot: root,
        entries: ["src/main.jsx"],
        provenance: extra,
      }),
    );
  });
});

test("uses one binary canonical order for mixed-case paths", async () => {
  await withFixture(async (root) => {
    await writeFile(path.join(root, "src/Z.js"), "export const upper = true;");
    await writeFile(path.join(root, "src/a.js"), "export const lower = true;");
    const assets = await computeCurrentRouteSourceClosure({
      repoRoot: root,
      entries: ["src/a.js", "src/Z.js"],
    });
    assert.deepEqual(
      assets.map((entry) => entry.path),
      ["src/Z.js", "src/a.js"],
    );
    assert.deepEqual(validateCurrentRouteProvenance({ schemaVersion: 2, assets }).assets, assets);
  });
});

test("rejects provenance duplicates, non-canonical order, and hash drift", async () => {
  await withFixture(async (root) => {
    const provenance = await manifestFromEntries(root);
    const duplicate = {
      ...provenance,
      assets: [...provenance.assets, provenance.assets[0]],
    };
    await expectCode("CRP-SCHEMA", () => validateCurrentRouteProvenance(duplicate));

    const nonCanonical = {
      ...provenance,
      assets: [...provenance.assets].reverse(),
    };
    await expectCode("CRP-ORDER", () => validateCurrentRouteProvenance(nonCanonical));

    const drifted = {
      ...provenance,
      assets: [...provenance.assets],
    };
    drifted.assets[0] = {
      ...drifted.assets[0],
      sha256: "0".repeat(64),
    };
    await expectCode("CRP-HASH", () =>
      verifyCurrentRouteSourceClosure({
        repoRoot: root,
        entries: ["src/main.jsx"],
        provenance: drifted,
      }),
    );
  });
});

test("rejects entry root escape in closure inputs", async () => {
  await withFixture(async (root) => {
    await expectCode("CRP-SCHEMA", () =>
      computeCurrentRouteSourceClosure({
        repoRoot: root,
        entries: ["../src/main.jsx"],
      }),
    );
  });
});

test("source manifest hashing accepts bytes only", () => {
  expectCode("CRP-SCHEMA", () => currentRouteSourceManifestSha256("{}"));
});
