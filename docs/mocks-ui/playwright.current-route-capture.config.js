import { defineConfig } from "@playwright/test";

const BASE_URL = "http://127.0.0.1:4174/";

export default defineConfig({
  testDir: "./tests",
  testMatch: /current-route-capture-v1etc[.]playwright[.]js$/,
  outputDir: "./test-results-current-route-capture",
  fullyParallel: false,
  forbidOnly: Boolean(process.env.CI),
  retries: process.env.CI ? 2 : 0,
  workers: 1,
  reporter: process.env.CI ? [["list"], ["html", { open: "never" }]] : "list",
  use: {
    baseURL: BASE_URL,
    browserName: "chromium",
    channel: "chrome",
    viewport: { width: 1440, height: 900 },
    deviceScaleFactor: 1,
    colorScheme: "dark",
    locale: "ja-JP",
    reducedMotion: "reduce",
    screenshot: "only-on-failure",
    trace: "retain-on-failure",
  },
  webServer: [
    {
      command:
        "npm run dev -- --mode current-route-capture --port 4174 --host 127.0.0.1 --strictPort",
      cwd: ".",
      url: BASE_URL,
      reuseExistingServer: !process.env.CI,
      timeout: 120_000,
    },
  ],
});
