import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

const TEST_DIR = path.dirname(fileURLToPath(import.meta.url));
const REPO_DIR = path.resolve(TEST_DIR, "../../..");

async function source(relativePath) {
  return readFile(path.join(REPO_DIR, relativePath), "utf8");
}

test("P07-C1D keeps the Stage play handler to ProductApp playback spine", async () => {
  const [candidate, main, bridge, host, product] = await Promise.all([
    source("ui/motolii-web/src/candidates/StageChromeCandidate.jsx"),
    source("ui/motolii-web/src/host/stage-transport-main.jsx"),
    source("ui/motolii-web/src/host/stageHostBridge.js"),
    source("crates/motolii-ui/src/stage_chrome_host_runtime.rs"),
    source("crates/motolii-ui/src/product_runtime.rs"),
  ]);

  assert.ok(candidate.includes('id="play"'));
  assert.ok(candidate.includes("onTogglePlayback"));
  assert.ok(main.includes('import { encodeStagePlaybackToggle }'));
  assert.ok(main.includes("window.ipc.postMessage(encodeStagePlaybackToggle())"));
  assert.ok(bridge.includes('"playbackState"'));
  assert.ok(bridge.includes('["idle", "preparing", "playing"]'));

  assert.ok(host.includes('"toggle-playback"'));
  assert.ok(host.includes("pending: bool"));
  assert.ok(host.includes("take_playback_intent"));

  const intentHandling = product.indexOf("fn process_stage_playback_intents");
  const hostTake = product.indexOf("stage_chrome.take_playback_intent()", intentHandling);
  const toggle = product.indexOf("fn toggle_playback", intentHandling);
  const preparation = product.indexOf("fn process_playback_preparation", intentHandling);
  const session = product.indexOf("PlaybackSession::open_default", preparation);
  const transport = product.indexOf("next_frame_plan()", session);
  assert.ok(intentHandling >= 0);
  assert.ok(hostTake > intentHandling);
  assert.ok(toggle > hostTake);
  assert.ok(preparation > toggle);
  assert.ok(session > preparation);
  assert.ok(transport > session);

  assert.ok(product.includes("sync_channel(1)"));
  assert.ok(product.includes("AudioProgram::from_document"));
  assert.ok(product.includes("self.document_runtime.project_root()"));
  assert.ok(product.includes("self.editor_playhead.set(plan.timeline_time)"));
  assert.ok(product.includes("self.submit_stage_projection()?"));

  for (const forbidden of ["setInterval", "requestAnimationFrame", "f64 * CANONICAL_SAMPLE_RATE"]) {
    assert.equal(main.includes(forbidden), false, forbidden);
    assert.equal(product.includes(forbidden), false, forbidden);
  }
});
