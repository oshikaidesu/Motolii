import test from "node:test";
import assert from "node:assert/strict";
import {
  createState,
  deleteRange,
  duplicateState,
  evaluateState,
  insertText,
  randomInPose,
  reconcileWholeText,
  replaceRange,
  stateText,
  validateState,
  withOverride,
} from "./reconcile.mjs";

const RUN_OVERRIDE = {
  offsetY: -36,
  visualScale: 1.4,
  timing: { mode: "pinned", value: 4 },
};

function preparedState() {
  const initial = createState("夜を走る");
  return withOverride(initial, "g3", RUN_OVERRIDE);
}

test("inserting 道 gives it a new Auto identity and preserves 走", () => {
  const before = preparedState();
  const after = insertText(before, 1, "道");
  const evaluated = evaluateState(after, { seed: 1842, interval: 2 });

  assert.equal(stateText(after), "夜道を走る");
  assert.deepEqual(
    after.units.map((unit) => [unit.grapheme, unit.id]),
    [
      ["夜", "g1"],
      ["道", "g5"],
      ["を", "g2"],
      ["走", "g3"],
      ["る", "g4"],
    ],
  );
  assert.equal(after.units.find((unit) => unit.grapheme === "道").override, null);
  assert.deepEqual(after.units.find((unit) => unit.id === "g3").override, RUN_OVERRIDE);
  assert.equal(evaluated.find((unit) => unit.grapheme === "道").start, 2);
  assert.equal(evaluated.find((unit) => unit.id === "g3").start, 4);
  assert.equal(validateState(after).uniqueIds, true);
});

test("Random In is deterministic per seed and identity", () => {
  assert.deepEqual(randomInPose("g5", 1842), randomInPose("g5", 1842));
  assert.notDeepEqual(randomInPose("g5", 1842), randomInPose("g5", 1843));
  assert.notDeepEqual(randomInPose("g5", 1842), randomInPose("g6", 1842));
});

test("replacement never transfers overrides silently", () => {
  const before = preparedState();
  const after = replaceRange(before, 2, 1, "飛");

  assert.equal(stateText(after), "夜を飛る");
  assert.equal(after.units.find((unit) => unit.grapheme === "飛").override, null);
  assert.deepEqual(after.needsReview, [
    {
      oldId: "g3",
      grapheme: "走",
      reason: "Replacement never inherits manual overrides automatically",
    },
  ]);
});

test("deletion removes the override with the unit and keeps the prior snapshot intact", () => {
  const before = preparedState();
  const after = deleteRange(before, 2, 1);

  assert.equal(stateText(after), "夜をる");
  assert.equal(stateText(before), "夜を走る");
  assert.deepEqual(before.units.find((unit) => unit.id === "g3").override, RUN_OVERRIDE);
  assert.equal(after.units.some((unit) => unit.id === "g3"), false);
});

test("whole-text reconcile preserves an unambiguous repeated suffix after insertion", () => {
  let state = createState("夜へ 夜へ");
  state = withOverride(state, "g4", {
    offsetX: 12,
    timing: { mode: "offset", value: -2 },
  });
  const after = reconcileWholeText(state, "夜へ 深い夜へ");

  assert.equal(stateText(after), "夜へ 深い夜へ");
  assert.equal(validateState(after).uniqueIds, true);
  assert.equal(after.needsReview.length, 0);
  assert.deepEqual(after.units.find((unit) => unit.id === "g4").override, {
    offsetX: 12,
    timing: { mode: "offset", value: -2 },
  });
});

test("whole-text reconcile reports overridden changed-middle units instead of guessing", () => {
  let state = createState("夜を走る");
  state = withOverride(state, "g3", RUN_OVERRIDE);
  const after = reconcileWholeText(state, "夜に飛ぶ");

  assert.equal(stateText(after), "夜に飛ぶ");
  assert.equal(validateState(after).uniqueIds, true);
  assert.deepEqual(after.needsReview, [
    {
      oldId: "g3",
      grapheme: "走",
      reason: "Whole-text edit could not preserve this overridden unit safely",
    },
  ]);
});

test("independent duplicate remints IDs and copies overrides by value", () => {
  const original = preparedState();
  const duplicate = duplicateState(original);

  assert.deepEqual(
    duplicate.state.units.map((unit) => unit.grapheme),
    original.units.map((unit) => unit.grapheme),
  );
  assert.equal(
    duplicate.state.units.some((unit) => original.units.some((source) => source.id === unit.id)),
    false,
  );
  duplicate.state.units.find((unit) => unit.grapheme === "走").override.visualScale = 2;
  assert.equal(original.units.find((unit) => unit.grapheme === "走").override.visualScale, 1.4);
});
