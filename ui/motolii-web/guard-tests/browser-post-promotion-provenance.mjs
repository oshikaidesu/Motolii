import assert from "node:assert/strict";

const FIXED_BROWSER_COMPONENT_SHA256 =
  "4edb3dfc49726aa700e77a14197571a43de2d80d9838a824c22cb68e0ac3d5b8";
const POST_PROMOTION_TASK = "G0-6H-V1ETB";
const POST_PROMOTION_FILE = "ui/motolii-web/src/candidates/DiscoveryBrowserCandidate.jsx";
const POST_PROMOTION_REASON = ["development-only", "Starter", "Media projection"].join(" ");
const POST_PROMOTION_ENTRY_KEYS = [
  "task",
  "file",
  "reason",
  "fixedSourceSha256",
  "currentSha256",
];
const POST_PROMOTION_INDEX0_CURRENT_SHA256 =
  "866124a69caaa168fa19c67e6c723db97fec67a61071bdbe66973576266c42f4";

export function validatePostPromotionChanges(provenance, currentComponentSha256) {
  const changes = provenance.postPromotionChanges;
  if (changes === undefined) {
    if (currentComponentSha256 !== FIXED_BROWSER_COMPONENT_SHA256) {
      throw new Error("component bytes differ from fixed commit without postPromotionChanges");
    }
    return;
  }
  if (!Array.isArray(changes)) {
    throw new Error("postPromotionChanges must be an array");
  }
  if (changes.length < 1) {
    throw new Error("postPromotionChanges must contain at least one entry");
  }
  for (const entry of changes) {
    if (entry === null || typeof entry !== "object" || Array.isArray(entry)) {
      throw new Error("postPromotionChanges entry must be an object");
    }
    const keys = Object.keys(entry);
    if (keys.length !== POST_PROMOTION_ENTRY_KEYS.length) {
      throw new Error("postPromotionChanges entry has wrong key count");
    }
    for (const key of POST_PROMOTION_ENTRY_KEYS) {
      if (!Object.hasOwn(entry, key)) {
        throw new Error(`postPromotionChanges entry missing key ${key}`);
      }
    }
    for (const key of keys) {
      if (!POST_PROMOTION_ENTRY_KEYS.includes(key)) {
        throw new Error(`postPromotionChanges entry has extra key ${key}`);
      }
    }
  }
  const index0 = changes[0];
  if (index0.task !== POST_PROMOTION_TASK) {
    throw new Error("postPromotionChanges task literal mismatch");
  }
  if (index0.file !== POST_PROMOTION_FILE) {
    throw new Error("postPromotionChanges file literal mismatch");
  }
  if (index0.reason !== POST_PROMOTION_REASON) {
    throw new Error("postPromotionChanges reason literal mismatch");
  }
  if (index0.fixedSourceSha256 !== FIXED_BROWSER_COMPONENT_SHA256) {
    throw new Error("postPromotionChanges fixedSourceSha256 mismatch");
  }
  if (index0.currentSha256 !== POST_PROMOTION_INDEX0_CURRENT_SHA256) {
    throw new Error("postPromotionChanges index 0 currentSha256 mismatch");
  }
  const index0File = index0.file;
  for (const entry of changes) {
    if (entry.file !== index0File) {
      throw new Error("postPromotionChanges file must match index 0");
    }
  }
  for (let i = 1; i < changes.length; i += 1) {
    const entry = changes[i];
    if (typeof entry.task !== "string" || entry.task.length === 0) {
      throw new Error("postPromotionChanges task must be non-empty string");
    }
    if (typeof entry.reason !== "string" || entry.reason.length === 0) {
      throw new Error("postPromotionChanges reason must be non-empty string");
    }
  }
  const seenTasks = new Set();
  for (const entry of changes) {
    if (seenTasks.has(entry.task)) {
      throw new Error("postPromotionChanges task must be unique");
    }
    seenTasks.add(entry.task);
  }
  for (let i = 1; i < changes.length; i += 1) {
    if (changes[i].fixedSourceSha256 !== changes[i - 1].currentSha256) {
      throw new Error("postPromotionChanges hash chain break");
    }
  }
  const lastEntry = changes[changes.length - 1];
  if (lastEntry.currentSha256 !== currentComponentSha256) {
    throw new Error("postPromotionChanges currentSha256 mismatch");
  }
}
