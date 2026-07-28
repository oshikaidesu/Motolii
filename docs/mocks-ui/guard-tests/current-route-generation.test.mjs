import test from "node:test";
import assert from "node:assert/strict";
import {
  CURRENT_ROUTE_SCREENS,
  CurrentRouteGenerationError,
  validateCurrentRouteManifest,
} from "../scripts/current-route-generation.mjs";

const VARIANTS = ["variant-a", "variant-b"];

const BASE = {
  schemaVersion: 2,
  generation: "fixture-generation",
  sourceManifestSha256: "a".repeat(64),
  transformVersion: "transform-v1",
  environment: {
    viewport: { width: 640, height: 480 },
    scale: 1,
    locale: "fixture-locale",
    timezone: "fixture-zone",
    theme: "fixture-theme",
    reducedMotion: "fixture-motion",
    browserVersion: "fixture-browser",
    browserRevision: "fixture-revision",
    fontFixture: {
      files: [
        {
          path: "fonts/inter.woff2",
          sha256: "a".repeat(64),
          weight: 400,
        },
      ],
      computedFamily: "fixture-family",
    },
  },
  screens: CURRENT_ROUTE_SCREENS.map((screen) => ({ screen, mode: "fixture-mode" })),
  captures: CURRENT_ROUTE_SCREENS.flatMap((screen) =>
    VARIANTS.map((variant) => ({
      path: `captures/${screen}.${variant}.png`,
      screen,
      variant,
      sha256: "b".repeat(64),
    })),
  ),
};

function manifest(overrides = {}) {
  return structuredClone({
    ...BASE,
    ...overrides,
    environment: {
      ...BASE.environment,
      ...(overrides.environment ?? {}),
      fontFixture: {
        ...BASE.environment.fontFixture,
        ...(overrides.environment?.fontFixture ?? {}),
      },
    },
  });
}

function expectCode(code, run) {
  assert.throws(run, (error) => {
    assert.ok(error instanceof CurrentRouteGenerationError);
    assert.equal(error.code, code);
    return true;
  });
}

test("returns a validated deep clone for valid manifest", () => {
  const input = manifest();
  const before = structuredClone(input);
  const output = validateCurrentRouteManifest(input, { expectedVariants: VARIANTS });
  assert.notStrictEqual(output, input);
  assert.deepEqual(input, before);
});

test("rejects top-level schema/renamed/missing fields", () => {
  expectCode("CR2-SCHEMA", () => {
    const bad = manifest();
    bad.unowned = true;
    validateCurrentRouteManifest(bad, { expectedVariants: VARIANTS });
  });
  expectCode("CR2-SCHEMA", () => {
    const bad = manifest();
    delete bad.transformVersion;
    validateCurrentRouteManifest(bad, { expectedVariants: VARIANTS });
  });
  expectCode("CR2-SCHEMA", () => {
    const bad = manifest();
    bad.extra = true;
    validateCurrentRouteManifest(bad, { expectedVariants: VARIANTS });
  });
});

test("rejects schemaVersion 1", () => {
  const bad = manifest();
  bad.schemaVersion = 1;
  expectCode("CR2-SCHEMA", () => {
    validateCurrentRouteManifest(bad, { expectedVariants: VARIANTS });
  });
});

test("rejects screens cardinality and order", () => {
  expectCode("CR2-SCHEMA", () => {
    const bad = manifest();
    bad.screens.push({ screen: "fixture-extra", mode: "fixture-mode" });
    validateCurrentRouteManifest(bad, { expectedVariants: VARIANTS });
  });
  expectCode("CR2-SCHEMA", () => {
    const bad = manifest();
    bad.screens = bad.screens.slice(0, 4);
    validateCurrentRouteManifest(bad, { expectedVariants: VARIANTS });
  });
  expectCode("CR2-SCHEMA", () => {
    const bad = manifest();
    bad.screens = [...bad.screens];
    bad.screens[4] = bad.screens[3];
    bad.screens[3] = { screen: "stage-frame-tools", mode: "fixture-mode" };
    validateCurrentRouteManifest(bad, { expectedVariants: VARIANTS });
  });
});

test("rejects capture order and capture path shape", () => {
  expectCode("CR2-VARIANT", () => {
    const bad = manifest();
    bad.captures = [...bad.captures].reverse();
    validateCurrentRouteManifest(bad, { expectedVariants: VARIANTS });
  });

  expectCode("CR2-SCHEMA", () => {
    const bad = manifest();
    bad.captures[1] = {
      ...bad.captures[1],
      path: bad.captures[0].path,
    };
    validateCurrentRouteManifest(bad, { expectedVariants: VARIANTS });
  });

  expectCode("CR2-SCHEMA", () => {
    const bad = manifest();
    bad.captures[0] = { ...bad.captures[0], path: "tmp.png" };
    validateCurrentRouteManifest(bad, { expectedVariants: VARIANTS });
  });

  expectCode("CR2-SCHEMA", () => {
    const bad = manifest();
    bad.captures[0] = { ...bad.captures[0], path: "captures/../evil.png" };
    validateCurrentRouteManifest(bad, { expectedVariants: VARIANTS });
  });

  expectCode("CR2-SCHEMA", () => {
    const bad = manifest();
    bad.captures[0] = { ...bad.captures[0], path: "/tmp/evil.png" };
    validateCurrentRouteManifest(bad, { expectedVariants: VARIANTS });
  });

  expectCode("CR2-SCHEMA", () => {
    const bad = manifest();
    bad.captures[0] = { ...bad.captures[0], path: "captures/foo\\bar.normal.png" };
    validateCurrentRouteManifest(bad, { expectedVariants: VARIANTS });
  });

  expectCode("CR2-SCHEMA", () => {
    const bad = manifest();
    bad.captures.push({
      path: "captures/extra.variant-a.png",
      screen: "extra",
      variant: "variant-a",
      sha256: "c".repeat(64),
    });
    validateCurrentRouteManifest(bad, { expectedVariants: VARIANTS });
  });

  expectCode("CR2-SCHEMA", () => {
    const bad = manifest();
    bad.captures[0] = { ...bad.captures[0], sha256: "bad" };
    validateCurrentRouteManifest(bad, { expectedVariants: VARIANTS });
  });

  expectCode("CR2-SCHEMA", () => {
    const bad = manifest();
    bad.captures[0] = {
      ...bad.captures[0],
      path: "captures/empty-browser.variant-b.png",
      variant: "variant-b",
      screen: "empty-browser",
    };
    validateCurrentRouteManifest(bad, { expectedVariants: VARIANTS });
  });
});

test("rejects environment shape and values", () => {
  expectCode("CR2-SCHEMA", () => {
    const bad = manifest();
    bad.environment.extra = true;
    validateCurrentRouteManifest(bad, { expectedVariants: VARIANTS });
  });

  expectCode("CR2-SCHEMA", () => {
    const bad = manifest();
    bad.environment.scale = 0;
    validateCurrentRouteManifest(bad, { expectedVariants: VARIANTS });
  });

  expectCode("CR2-SCHEMA", () => {
    const bad = manifest();
    bad.environment.viewport = { width: 0, height: 480 };
    validateCurrentRouteManifest(bad, { expectedVariants: VARIANTS });
  });

  expectCode("CR2-SCHEMA", () => {
    const bad = manifest();
    bad.environment.viewport = { width: "640", height: 480 };
    bad.environment = {
      ...bad.environment,
      viewport: bad.environment.viewport,
    };
    validateCurrentRouteManifest(bad, { expectedVariants: VARIANTS });
  });

  expectCode("CR2-SCHEMA", () => {
    const bad = manifest();
    bad.environment.viewport = { width: 640, height: 480, unit: "fixture" };
    validateCurrentRouteManifest(bad, { expectedVariants: VARIANTS });
  });

  expectCode("CR2-SCHEMA", () => {
    const bad = manifest();
    delete bad.environment.timezone;
    validateCurrentRouteManifest(bad, { expectedVariants: VARIANTS });
  });

  expectCode("CR2-SCHEMA", () => {
    const bad = manifest();
    bad.environment.fontFixture.computedFamily = "";
    validateCurrentRouteManifest(bad, { expectedVariants: VARIANTS });
  });

  expectCode("CR2-SCHEMA", () => {
    const bad = manifest();
    bad.environment.fontFixture.files = [];
    validateCurrentRouteManifest(bad, { expectedVariants: VARIANTS });
  });

  expectCode("CR2-SCHEMA", () => {
    const bad = manifest();
    bad.environment.fontFixture.extra = true;
    validateCurrentRouteManifest(bad, { expectedVariants: VARIANTS });
  });

  expectCode("CR2-SCHEMA", () => {
    const bad = manifest();
    bad.environment.fontFixture.files = [
      {
        path: "fonts/a.woff2",
        sha256: "a".repeat(64),
        weight: 400,
      },
      {
        path: "fonts/a.woff2",
        sha256: "b".repeat(64),
        weight: 500,
      },
    ];
    validateCurrentRouteManifest(bad, { expectedVariants: VARIANTS });
  });

  expectCode("CR2-SCHEMA", () => {
    const bad = manifest();
    bad.environment.fontFixture.files = [
      {
        path: "fonts/bad.txt",
        sha256: "not-a-hash",
        weight: 400,
      },
    ];
    validateCurrentRouteManifest(bad, { expectedVariants: VARIANTS });
  });

  expectCode("CR2-SCHEMA", () => {
    const bad = manifest();
    bad.environment.fontFixture.files = [
      {
        path: "../fonts/a.woff2",
        sha256: "a".repeat(64),
        weight: 400,
      },
    ];
    validateCurrentRouteManifest(bad, { expectedVariants: VARIANTS });
  });

  expectCode("CR2-SCHEMA", () => {
    const bad = manifest();
    bad.environment.fontFixture.files = [
      {
        path: "fonts/a.woff2",
        sha256: "a".repeat(64),
        weight: 400,
      },
      {
        path: "fonts/B.woff2",
        sha256: "b".repeat(64),
        weight: 400,
      },
    ];
    validateCurrentRouteManifest(bad, { expectedVariants: VARIANTS });
  });

  expectCode("CR2-SCHEMA", () => {
    const bad = manifest();
    bad.environment.fontFixture.files = [
      {
        path: "fonts/dup.woff2",
        sha256: "a".repeat(64),
        weight: 1.5,
      },
    ];
    validateCurrentRouteManifest(bad, { expectedVariants: VARIANTS });
  });

  for (const field of [
    "locale",
    "timezone",
    "theme",
    "reducedMotion",
    "browserVersion",
    "browserRevision",
  ]) {
    expectCode("CR2-SCHEMA", () => {
      const bad = manifest();
      bad.environment[field] = "";
      validateCurrentRouteManifest(bad, { expectedVariants: VARIANTS });
    });
  }
});

test("rejects mode empty and generation pattern", () => {
  expectCode("CR2-SCHEMA", () => {
    const bad = manifest();
    bad.screens[0] = { ...bad.screens[0], mode: "" };
    validateCurrentRouteManifest(bad, { expectedVariants: VARIANTS });
  });

  expectCode("CR2-GENERATION", () => {
    const bad = manifest();
    bad.generation = "bad generation";
    validateCurrentRouteManifest(bad, { expectedVariants: VARIANTS });
  });

  for (const value of [null, undefined, 123, ".", "..", "x".repeat(129)]) {
    expectCode("CR2-GENERATION", () => {
      const bad = manifest();
      bad.generation = value;
      validateCurrentRouteManifest(bad, { expectedVariants: VARIANTS });
    });
  }
});


test("rejects expectedVariants with invalid values", () => {
  expectCode("CR2-SCHEMA", () => {
    validateCurrentRouteManifest(BASE, { expectedVariants: [] });
  });
  expectCode("CR2-SCHEMA", () => {
    validateCurrentRouteManifest(BASE, { expectedVariants: ["variant-a", "variant-a"] });
  });
  expectCode("CR2-SCHEMA", () => {
    validateCurrentRouteManifest(BASE, { expectedVariants: ["variant-a", 123] });
  });
  expectCode("CR2-SCHEMA", () => {
    validateCurrentRouteManifest(BASE);
  });
});
