import { createHash } from "node:crypto";
import { mkdtemp, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";
import assert from "node:assert/strict";
import {
  ReferenceGuardError,
  inspectRegistry,
  verifyFixtureCausality,
  verifyReferenceManifest,
} from "../scripts/reference-guard.mjs";

const BASE_FILES = {
  "src/main.jsx": `
    import { ReferenceScreen } from "./reference/Screen.jsx";
    export const screenRegistry = {
      "candidate": { Component: Candidate, catalogKind: "candidate" },
      "diagnostic": { Component: Diagnostic, catalogKind: "diagnostic" },
      "archive/old": { Component: Archive, catalogKind: "archive" },
      "reference/screen-a": { Component: ReferenceScreen, catalogKind: "reference" },
    };
  `,
  "src/candidate/Candidate.jsx": `
    import { buildCandidateModel } from "./model.js";
    import "./candidate.css";
    export function Candidate({ fixture }) {
      return <section className="candidate">{buildCandidateModel(fixture)}</section>;
    }
  `,
  "src/candidate/model.js": `
    export function buildCandidateModel(fixture) { return fixture.label; }
  `,
  "src/candidate/candidate.css": `
    .candidate { color: var(--candidate-text); }
  `,
  "tests/candidate.test.js": `
    import { Candidate } from "../src/candidate/Candidate.jsx";
    export const candidateTestEvidence = Candidate;
  `,
  "tests/render-normal.mjs": `
    import { readFile } from "node:fs/promises";
    import { buildCandidateModel } from "../src/candidate/model.js";
    export async function renderNormal({ fixturePaths }) {
      const values = [];
      for (const layer of ["document", "scenes", "tokens"]) {
        const fixture = JSON.parse(await readFile(fixturePaths[layer], "utf8"));
        values.push(buildCandidateModel(fixture));
      }
      return new Map([["screen-a", Buffer.from(values.join("|"))]]);
    }
  `,
  "src/reference/load.js": `
    export function loadReferenceFixtures(value) { return value; }
  `,
  "src/reference/Screen.jsx": `
    import { Candidate } from "../candidate/Candidate.jsx";
    import { loadReferenceFixtures } from "./load.js";
    export function ReferenceScreen({ fixtures }) {
      const projected = loadReferenceFixtures(fixtures);
      return <Candidate fixture={projected} />;
    }
  `,
  "fixtures/reference-document.json": `{"label":"document"}\n`,
  "fixtures/reference-scenes.json": `{"label":"scenes"}\n`,
  "fixtures/reference-candidate-tokens.json": `{"label":"tokens"}\n`,
};

async function sha256(filename) {
  return createHash("sha256").update(await readFile(filename)).digest("hex");
}

async function writeTree(files = BASE_FILES) {
  const root = await mkdtemp(path.join(tmpdir(), "motolii-reference-guard-test-"));
  for (const [relative, contents] of Object.entries(files)) {
    const filename = path.join(root, relative);
    await mkdir(path.dirname(filename), { recursive: true });
    await writeFile(filename, contents);
  }
  return root;
}

async function manifestFor(root) {
  const asset = async (assetPath, role, exports = []) => ({
    path: assetPath,
    role,
    exports,
    sha256: await sha256(path.join(root, assetPath)),
  });
  return {
    schemaVersion: 1,
    registryModule: "src/main.jsx",
    referenceRoots: ["src/reference"],
    referenceFiles: ["src/reference/Screen.jsx", "src/reference/load.js"],
    assets: [
      await asset("src/candidate/Candidate.jsx", "runtime", ["Candidate"]),
      await asset("src/candidate/model.js", "runtime", [
        "buildCandidateModel",
      ]),
      await asset("src/candidate/candidate.css", "runtime"),
      await asset("tests/candidate.test.js", "test", [
        "candidateTestEvidence",
      ]),
      await asset("tests/render-normal.mjs", "test", ["renderNormal"]),
    ],
    causalRenderer: {
      path: "tests/render-normal.mjs",
      export: "renderNormal",
    },
    screens: [
      {
        id: "screen-a",
        route: "reference/screen-a",
        module: "src/reference/Screen.jsx",
        export: "ReferenceScreen",
        requiredImports: [
          {
            source: "src/candidate/Candidate.jsx",
            imported: "Candidate",
            fixtureProp: "fixture",
          },
        ],
      },
    ],
    fixtureLayers: [
      {
        id: "document",
        path: "fixtures/reference-document.json",
        probe: { pointer: "/label", value: "document-mutated" },
        changedScreens: ["screen-a"],
      },
      {
        id: "scenes",
        path: "fixtures/reference-scenes.json",
        probe: { pointer: "/label", value: "scenes-mutated" },
        changedScreens: ["screen-a"],
      },
      {
        id: "tokens",
        path: "fixtures/reference-candidate-tokens.json",
        probe: { pointer: "/label", value: "tokens-mutated" },
        changedScreens: ["screen-a"],
      },
    ],
  };
}

async function withFixture(run) {
  const root = await writeTree();
  try {
    await run(root, await manifestFor(root));
  } finally {
    await rm(root, { recursive: true, force: true });
  }
}

async function expectGuard(code, run) {
  await assert.rejects(run, (error) => {
    assert.ok(error instanceof ReferenceGuardError);
    assert.equal(error.code, code);
    return true;
  });
}

async function setRenderer(root, manifest, source) {
  const filename = path.join(root, manifest.causalRenderer.path);
  await writeFile(filename, source);
  const asset = manifest.assets.find(
    (entry) => entry.path === manifest.causalRenderer.path,
  );
  asset.sha256 = await sha256(filename);
}

test("accepts an exact imported source closure and all three causal layers", async () => {
  await withFixture(async (root, manifest) => {
    const result = await verifyReferenceManifest(root, manifest);
    assert.deepEqual([...result.screenIds], ["screen-a"]);
    assert.deepEqual([...result.runtimeAssets].sort(), [
      "src/candidate/Candidate.jsx",
      "src/candidate/candidate.css",
      "src/candidate/model.js",
    ]);
    await verifyFixtureCausality({ root, manifest });
  });
});

test("registry categories are static and disjoint", async () => {
  await withFixture(async (root) => {
    const routes = await inspectRegistry(path.join(root, "src/main.jsx"));
    assert.equal(routes.get("candidate"), "candidate");
    assert.equal(routes.get("diagnostic"), "diagnostic");
    assert.equal(routes.get("archive/old"), "archive");
    assert.equal(routes.get("reference/screen-a"), "reference");
  });
});

test("rejects reference routes that leak into the candidate catalog", async () => {
  await withFixture(async (root, manifest) => {
    const filename = path.join(root, "src/main.jsx");
    await writeFile(
      filename,
      BASE_FILES["src/main.jsx"].replace(
        'Component: ReferenceScreen, catalogKind: "reference"',
        'Component: ReferenceScreen, catalogKind: "candidate"',
      ),
    );
    await expectGuard("RG-ROUTE", () => verifyReferenceManifest(root, manifest));
  });
});

test("rejects a semantic-ID claim without a direct source import", async () => {
  await withFixture(async (root, manifest) => {
    manifest.screens[0].requiredImports = [];
    await expectGuard("RG-PROVENANCE", () =>
      verifyReferenceManifest(root, manifest),
    );
  });
});

test("rejects a required import that exists only in the manifest", async () => {
  await withFixture(async (root, manifest) => {
    await writeFile(
      path.join(root, "src/reference/Screen.jsx"),
      BASE_FILES["src/reference/Screen.jsx"].replace(
        'import { Candidate } from "../candidate/Candidate.jsx";',
        "",
      ),
    );
    await expectGuard("RG-PROVENANCE", () =>
      verifyReferenceManifest(root, manifest),
    );
  });
});

test("rejects an unused source import used only as provenance decoration", async () => {
  await withFixture(async (root, manifest) => {
    await writeFile(
      path.join(root, "src/reference/Screen.jsx"),
      `
        import { Candidate } from "../candidate/Candidate.jsx";
        import { loadReferenceFixtures } from "./load.js";
        export function ReferenceScreen({ fixtures }) {
          const projected = loadReferenceFixtures(fixtures);
          return <section>{projected.label}</section>;
        }
      `,
    );
    await expectGuard("RG-PROVENANCE", () =>
      verifyReferenceManifest(root, manifest),
    );
  });
});

test("rejects a required component rendered only behind dead control flow", async () => {
  await withFixture(async (root, manifest) => {
    await writeFile(
      path.join(root, "src/reference/Screen.jsx"),
      `
        import { Candidate } from "../candidate/Candidate.jsx";
        import { loadReferenceFixtures } from "./load.js";
        export function ReferenceScreen({ fixtures }) {
          const projected = loadReferenceFixtures(fixtures);
          if (false) return <Candidate fixture={projected} />;
          return <section>{projected.label}</section>;
        }
      `,
    );
    await expectGuard("RG-PROVENANCE", () =>
      verifyReferenceManifest(root, manifest),
    );
  });
});

test("rejects stale provenance hashes", async () => {
  await withFixture(async (root, manifest) => {
    manifest.assets[0].sha256 = "0".repeat(64);
    await expectGuard("RG-PROVENANCE", () =>
      verifyReferenceManifest(root, manifest),
    );
  });
});

test("rejects test evidence unrelated to the runtime source closure", async () => {
  await withFixture(async (root, manifest) => {
    const filename = path.join(root, "tests/candidate.test.js");
    await writeFile(filename, "export const unrelated = true;\n");
    const testAsset = manifest.assets.find(
      (asset) => asset.path === "tests/candidate.test.js",
    );
    testAsset.exports = ["unrelated"];
    testAsset.sha256 = await sha256(filename);
    await expectGuard("RG-PROVENANCE", () =>
      verifyReferenceManifest(root, manifest),
    );
  });
});

test("rejects undeclared runtime dependencies", async () => {
  await withFixture(async (root, manifest) => {
    manifest.assets = manifest.assets.filter(
      (asset) => asset.path !== "src/candidate/model.js",
    );
    await expectGuard("RG-CLOSURE", () =>
      verifyReferenceManifest(root, manifest),
    );
  });
});

test("rejects runtime imports from legacy or archive paths", async () => {
  const files = {
    ...BASE_FILES,
    "src/legacy/Legacy.jsx": `export function Legacy() { return null; }\n`,
    "src/reference/Screen.jsx": `
      import { Legacy } from "../legacy/Legacy.jsx";
      import { loadReferenceFixtures } from "./load.js";
      export function ReferenceScreen({ fixtures }) {
        const projected = loadReferenceFixtures(fixtures);
        return <Legacy fixture={projected} />;
      }
    `,
  };
  const root = await writeTree(files);
  try {
    const manifest = await manifestFor(root);
    manifest.assets.push({
      path: "src/legacy/Legacy.jsx",
      role: "runtime",
      exports: ["Legacy"],
      sha256: await sha256(path.join(root, "src/legacy/Legacy.jsx")),
    });
    manifest.screens[0].requiredImports = [
      {
        source: "src/legacy/Legacy.jsx",
        imported: "Legacy",
        fixtureProp: "fixture",
      },
    ];
    await expectGuard("RG-LEGACY", () =>
      verifyReferenceManifest(root, manifest),
    );
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test("rejects reference leaves that import their registry", async () => {
  await withFixture(async (root, manifest) => {
    await writeFile(
      path.join(root, "src/reference/Screen.jsx"),
      `${BASE_FILES["src/reference/Screen.jsx"]}\nimport { screenRegistry } from "../main.jsx";\n`,
    );
    await expectGuard("RG-SELF-REGISTER", () =>
      verifyReferenceManifest(root, manifest),
    );
  });
});

test("rejects dynamic imports even when their target is not executed", async () => {
  await withFixture(async (root, manifest) => {
    await writeFile(
      path.join(root, "src/reference/dynamic.js"),
      'export const dynamic = () => import("../legacy/Later.jsx");\n',
    );
    manifest.referenceFiles.push("src/reference/dynamic.js");
    await expectGuard("RG-IMPORT", () =>
      verifyReferenceManifest(root, manifest),
    );
  });
});

test("rejects a copied source file inside the reference tree", async () => {
  await withFixture(async (root, manifest) => {
    const copied = await readFile(
      path.join(root, "src/candidate/model.js"),
      "utf8",
    );
    await writeFile(path.join(root, "src/reference/copied-model.js"), copied);
    manifest.referenceFiles.push("src/reference/copied-model.js");
    await expectGuard("RG-COPY", () => verifyReferenceManifest(root, manifest));
  });
});

test("rejects a reference file omitted from the provenance closure", async () => {
  await withFixture(async (root, manifest) => {
    await writeFile(
      path.join(root, "src/reference/undeclared.js"),
      "export const undeclared = true;\n",
    );
    await expectGuard("RG-CLOSURE", () =>
      verifyReferenceManifest(root, manifest),
    );
  });
});

test("rejects an ignored loadReferenceFixtures return", async () => {
  await withFixture(async (root, manifest) => {
    await writeFile(
      path.join(root, "src/reference/Screen.jsx"),
      `
        import { Candidate } from "../candidate/Candidate.jsx";
        import { loadReferenceFixtures } from "./load.js";
        export function ReferenceScreen({ fixtures }) {
          loadReferenceFixtures(fixtures);
          return <Candidate fixture={fixtures} />;
        }
      `,
    );
    await expectGuard("RG-FIXTURE-LOAD", () =>
      verifyReferenceManifest(root, manifest),
    );
  });
});

test("rejects a bound fixture result that never reaches a source component", async () => {
  await withFixture(async (root, manifest) => {
    await writeFile(
      path.join(root, "src/reference/Screen.jsx"),
      `
        import { Candidate } from "../candidate/Candidate.jsx";
        import { loadReferenceFixtures } from "./load.js";
        export function ReferenceScreen({ fixtures }) {
          const projected = loadReferenceFixtures(fixtures);
          return <Candidate fixture={fixtures} data-projected={Boolean(projected)} />;
        }
      `,
    );
    await expectGuard("RG-FIXTURE-LOAD", () =>
      verifyReferenceManifest(root, manifest),
    );
  });
});

test("rejects a decorative loaded component beside a live raw-fixture component", async () => {
  await withFixture(async (root, manifest) => {
    await writeFile(
      path.join(root, "src/reference/Screen.jsx"),
      `
        import { Candidate } from "../candidate/Candidate.jsx";
        import { loadReferenceFixtures } from "./load.js";
        export function ReferenceScreen({ fixtures }) {
          const projected = loadReferenceFixtures(fixtures);
          const decorative = <Candidate fixture={projected} />;
          void decorative;
          return <Candidate fixture={fixtures} />;
        }
      `,
    );
    await expectGuard("RG-FIXTURE-LOAD", () =>
      verifyReferenceManifest(root, manifest),
    );
  });
});

test("rejects any live required-component sibling that bypasses fixture loading", async (suite) => {
  for (const [name, returned] of [
    [
      "fragment",
      `<><Candidate fixture={projected} /><Candidate fixture={fixtures} /></>`,
    ],
    [
      "array",
      `[<Candidate key="loaded" fixture={projected} />, <Candidate key="raw" fixture={fixtures} />]`,
    ],
  ]) {
    await suite.test(name, async () => {
      await withFixture(async (root, manifest) => {
        await writeFile(
          path.join(root, "src/reference/Screen.jsx"),
          `
            import { Candidate } from "../candidate/Candidate.jsx";
            import { loadReferenceFixtures } from "./load.js";
            export function ReferenceScreen({ fixtures }) {
              const projected = loadReferenceFixtures(fixtures);
              return ${returned};
            }
          `,
        );
        await expectGuard("RG-FIXTURE-LOAD", () =>
          verifyReferenceManifest(root, manifest),
        );
      });
    });
  }
});

test("rejects a loaded fixture binding that is reassigned before render", async () => {
  await withFixture(async (root, manifest) => {
    await writeFile(
      path.join(root, "src/reference/Screen.jsx"),
      `
        import { Candidate } from "../candidate/Candidate.jsx";
        import { loadReferenceFixtures } from "./load.js";
        export function ReferenceScreen({ fixtures }) {
          let projected = loadReferenceFixtures(fixtures);
          projected = fixtures;
          return <Candidate fixture={projected} />;
        }
      `,
    );
    await expectGuard("RG-FIXTURE-LOAD", () =>
      verifyReferenceManifest(root, manifest),
    );
  });
});

test("tracks an aliased named fixture-loader import", async () => {
  await withFixture(async (root, manifest) => {
    await writeFile(
      path.join(root, "src/reference/Screen.jsx"),
      `
        import { Candidate } from "../candidate/Candidate.jsx";
        import { loadReferenceFixtures as loadFixtures } from "./load.js";
        export function ReferenceScreen({ fixtures }) {
          const projected = loadFixtures(fixtures);
          return <Candidate fixture={projected} />;
        }
      `,
    );
    await verifyReferenceManifest(root, manifest);
  });
});

test("rejects fixture results used only in lossy prop expressions", async (suite) => {
  for (const [name, expression] of [
    ["comma", "(projected, fixtures)"],
    ["conditional", "false ? projected : fixtures"],
    ["logical fallback", "projected || fixtures"],
  ]) {
    await suite.test(name, async () => {
      await withFixture(async (root, manifest) => {
        await writeFile(
          path.join(root, "src/reference/Screen.jsx"),
          `
            import { Candidate } from "../candidate/Candidate.jsx";
            import { loadReferenceFixtures } from "./load.js";
            export function ReferenceScreen({ fixtures }) {
              const projected = loadReferenceFixtures(fixtures);
              return <Candidate fixture={${expression}} />;
            }
          `,
        );
        await expectGuard("RG-FIXTURE-LOAD", () =>
          verifyReferenceManifest(root, manifest),
        );
      });
    });
  }
});

test("accepts a direct member projection from the loaded fixture", async () => {
  await withFixture(async (root, manifest) => {
    await writeFile(
      path.join(root, "src/reference/Screen.jsx"),
      `
        import { Candidate } from "../candidate/Candidate.jsx";
        import { loadReferenceFixtures } from "./load.js";
        export function ReferenceScreen({ fixtures }) {
          const projected = loadReferenceFixtures(fixtures);
          return <Candidate fixture={projected.document} />;
        }
      `,
    );
    await verifyReferenceManifest(root, manifest);
  });
});

test("rejects fixture loading hidden in a runtime helper", async () => {
  await withFixture(async (root, manifest) => {
    const filename = path.join(root, "src/candidate/model.js");
    await writeFile(
      filename,
      `
        export function buildCandidateModel(fixture) {
          return loadReferenceFixtures(fixture).label;
        }
      `,
    );
    const modelAsset = manifest.assets.find(
      (asset) => asset.path === "src/candidate/model.js",
    );
    modelAsset.sha256 = await sha256(filename);
    await expectGuard("RG-FIXTURE-LOAD", () =>
      verifyReferenceManifest(root, manifest),
    );
  });
});

test("rejects raw JSX and CSS colors in the reference tree", async (suite) => {
  await suite.test("JSX raw hex", async () => {
    await withFixture(async (root, manifest) => {
      await writeFile(
        path.join(root, "src/reference/Screen.jsx"),
        BASE_FILES["src/reference/Screen.jsx"].replace(
          "<Candidate fixture={projected} />",
          '<Candidate fixture={projected} style={{ color: "#fff" }} />',
        ),
      );
      await expectGuard("RG-RAW-COLOR", () =>
        verifyReferenceManifest(root, manifest),
      );
    });
  });

  await suite.test("CSS rgb", async () => {
    await withFixture(async (root, manifest) => {
      await writeFile(
        path.join(root, "src/reference/reference.css"),
        ".reference { color: rgb(1 2 3); }\n",
      );
      manifest.referenceFiles.push("src/reference/reference.css");
      await expectGuard("RG-RAW-COLOR", () =>
        verifyReferenceManifest(root, manifest),
      );
    });
  });

  await suite.test("inline token color", async () => {
    await withFixture(async (root, manifest) => {
      await writeFile(
        path.join(root, "src/reference/Screen.jsx"),
        BASE_FILES["src/reference/Screen.jsx"].replace(
          "<Candidate fixture={projected} />",
          "<Candidate fixture={projected} style={{ color: projected.color }} />",
        ),
      );
      await expectGuard("RG-RAW-COLOR", () =>
        verifyReferenceManifest(root, manifest),
      );
    });
  });

  await suite.test("CSS hsl", async () => {
    await withFixture(async (root, manifest) => {
      await writeFile(
        path.join(root, "src/reference/reference.css"),
        ".reference { box-shadow: 0 0 1px hsl(0 0% 0%); }\n",
      );
      manifest.referenceFiles.push("src/reference/reference.css");
      await expectGuard("RG-RAW-COLOR", () =>
        verifyReferenceManifest(root, manifest),
      );
    });
  });

  await suite.test("computed inline style key", async () => {
    await withFixture(async (root, manifest) => {
      await writeFile(
        path.join(root, "src/reference/Screen.jsx"),
        BASE_FILES["src/reference/Screen.jsx"].replace(
          "<Candidate fixture={projected} />",
          "<Candidate fixture={projected} style={{ [projected.role]: projected.value }} />",
        ),
      );
      await expectGuard("RG-RAW-COLOR", () =>
        verifyReferenceManifest(root, manifest),
      );
    });
  });
});

test("rejects a fixture layer whose mutation does not affect its normal capture", async () => {
  await withFixture(async (root, manifest) => {
    await setRenderer(
      root,
      manifest,
      `
        import { readFile } from "node:fs/promises";
        export async function renderNormal({ fixturePaths }) {
          const document = JSON.parse(await readFile(fixturePaths.document, "utf8"));
          const scenes = JSON.parse(await readFile(fixturePaths.scenes, "utf8"));
          return { "screen-a": document.label + "|" + scenes.label };
        }
      `,
    );
    await expectGuard("RG-CAUSAL", () =>
      verifyFixtureCausality({ root, manifest }),
    );
  });
});

test("rejects mutation-hint and path-only causal cheats", async (suite) => {
  await suite.test("mutation hint", async () => {
    await withFixture(async (root, manifest) => {
      await setRenderer(
        root,
        manifest,
        `
          export async function renderNormal({ mutation }) {
            return { "screen-a": mutation?.layer ?? "baseline" };
          }
        `,
      );
      await expectGuard("RG-CAUSAL", () =>
        verifyFixtureCausality({ root, manifest }),
      );
    });
  });

  await suite.test("path identity", async () => {
    await withFixture(async (root, manifest) => {
      await setRenderer(
        root,
        manifest,
        `
          export async function renderNormal({ fixturePaths }) {
            return { "screen-a": Object.values(fixturePaths).join("|") };
          }
        `,
      );
      await expectGuard("RG-CAUSAL", () =>
        verifyFixtureCausality({ root, manifest }),
      );
    });
  });

  await suite.test("call count", async () => {
    await withFixture(async (root, manifest) => {
      await setRenderer(
        root,
        manifest,
        `
          let calls = 0;
          export async function renderNormal() {
            return { "screen-a": String(calls++) };
          }
        `,
      );
      await expectGuard("RG-CAUSAL", () =>
        verifyFixtureCausality({ root, manifest }),
      );
    });
  });
});

test("rejects missing, reordered, or renamed fixture layers", async () => {
  await withFixture(async (root, manifest) => {
    manifest.fixtureLayers.reverse();
    await expectGuard("RG-CAUSAL", () =>
      verifyFixtureCausality({ root, manifest }),
    );
  });
});

test("rejects renderers that mutate a source fixture", async () => {
  await withFixture(async (root, manifest) => {
    const sourceFixture = JSON.stringify(
      path.join(root, "fixtures/reference-document.json"),
    );
    await setRenderer(
      root,
      manifest,
      `
        import { writeFile } from "node:fs/promises";
        export async function renderNormal() {
          await writeFile(${sourceFixture}, '{"label":"corrupt"}\\n');
          return { "screen-a": "capture" };
        }
      `,
    );
    await expectGuard("RG-CAUSAL", () =>
      verifyFixtureCausality({ root, manifest }),
    );
  });
});

test("rejects unknown capture screens", async () => {
  await withFixture(async (root, manifest) => {
    await setRenderer(
      root,
      manifest,
      `
        export async function renderNormal() {
          return { "screen-a": "capture", "screen-b": "unexpected" };
        }
      `,
    );
    await expectGuard("RG-CAUSAL", () =>
      verifyFixtureCausality({ root, manifest }),
    );
  });
});
