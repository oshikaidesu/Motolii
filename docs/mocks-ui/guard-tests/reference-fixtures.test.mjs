import { readFile } from "node:fs/promises";
import path from "node:path";
import test from "node:test";
import assert from "node:assert/strict";
import { loadReferenceFixtures } from "../src/reference/loadReferenceFixtures.js";

const root = path.resolve(import.meta.dirname, "..");
const fixtures = {
  document: JSON.parse(await readFile(path.join(root, "fixtures/reference-document.json"))),
  scenes: JSON.parse(await readFile(path.join(root, "fixtures/reference-scenes.json"))),
  tokens: JSON.parse(await readFile(path.join(root, "fixtures/reference-candidate-tokens.json"))),
};

const SCREEN_IDS = [
  "empty-browser",
  "mixed-timeline",
  "parameter-easing",
  "stage-frame-tools",
  "shared-effect-relative",
];

test("all five screens project the same three fixture objects", () => {
  for (const screenId of SCREEN_IDS) {
    const projected = loadReferenceFixtures(screenId, fixtures);
    assert.equal(projected.screenId, screenId);
    assert.equal(projected.document, fixtures.document);
    assert.equal(projected.tokens, fixtures.tokens);
    assert.equal(projected.scene, fixtures.scenes.screens[screenId]);
    assert.equal(projected.timeline.bars.length, 6);
  }
});

test("three shared uses retain non-adjacent and different stack positions", () => {
  const projected = loadReferenceFixtures("shared-effect-relative", fixtures);
  const sharedBars = projected.timeline.bars
    .map((bar, index) => ({ bar, index }))
    .filter(({ bar }) => bar.flow);
  assert.deepEqual(sharedBars.map(({ index }) => index), [0, 2, 4]);
  const definitionId = fixtures.document.effect_definitions[0].id;
  assert.deepEqual(
    [0, 2, 4].map((index) =>
      fixtures.document.tracks[0].items[index].envelope.effects.findIndex(
        (effect) => effect.definition_id === definitionId,
      ),
    ),
    [0, 1, 0],
  );
});

test("transient mute, keyframe, fold, Camera, and Hand states come from scenes", () => {
  const changed = structuredClone(fixtures);
  changed.scenes.screens["mixed-timeline"].muted = [4];
  changed.scenes.screens["mixed-timeline"].keyframed = [3];
  changed.scenes.screens["shared-effect-relative"].folded = false;
  changed.scenes.screens["stage-frame-tools"].focus = "stage.hand";
  changed.scenes.screens["stage-frame-tools"].hover = "stage.camera";

  const mixed = loadReferenceFixtures("mixed-timeline", changed);
  assert.equal(mixed.timeline.bars[4].states.some(({ id }) => id === "mute"), true);
  assert.equal(mixed.timeline.bars[3].states.some(({ id }) => id === "keyframe"), true);
  const shared = loadReferenceFixtures("shared-effect-relative", changed);
  assert.equal(
    shared.timeline.bars.flatMap((bar) => bar.states).find(({ id }) => id === "fold-count").label,
    "Expanded · 3 uses",
  );
  const stage = loadReferenceFixtures("stage-frame-tools", changed);
  assert.equal(stage.scene.focus, "stage.hand");
  assert.equal(stage.scene.hover, "stage.camera");
});

test("unknown screens and malformed fixture owners fail closed", () => {
  assert.throws(() => loadReferenceFixtures("unknown", fixtures), /unknown reference screen/);
  assert.throws(
    () => loadReferenceFixtures("empty-browser", {
      ...fixtures,
      scenes: { ...fixtures.scenes, screens: {} },
    }),
    /empty-browser scene must be an object/,
  );
  assert.throws(
    () => loadReferenceFixtures("empty-browser", {
      ...fixtures,
      tokens: {
        ...fixtures.tokens,
        "candidate-space": { compact: { $value: { value: Number.NaN } } },
      },
    }),
    /candidate spacing must be finite/,
  );
});
