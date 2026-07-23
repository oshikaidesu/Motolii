import { chromium } from "@playwright/test";

const OUT = "/private/tmp/claude-501/-Users-member-ottoto-rust-ae-Motolii/a2f37b37-9ccf-45cd-8277-bb60ac85d829/scratchpad";
const TYPES = ["Bounce", "Elastic", "Cyclic", "Random", "Steps", "Elastic Steps"];

const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1280, height: 900 } });
await page.goto("http://127.0.0.1:5173/#plugin-browser-candidate", { waitUntil: "domcontentloaded" });
await page.locator('.app[data-parity-ready="true"]').waitFor({ state: "visible" });
await page.locator("#interval-easing").click();
for (const type of TYPES) {
  await page.locator(`[data-advanced-curve="${type}"]`).click();
  await page.waitForTimeout(120);
  await page.locator("#easing-panel").screenshot({ path: `${OUT}/mock-${type.replace(" ", "-").toLowerCase()}.png` });
}
await browser.close();
console.log("captured");
