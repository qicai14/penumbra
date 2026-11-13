import { defineConfig } from "@playwright/test";

// BASE_URL set (CI after deploy) → test the deployed site; unset → build preview on :4173.
const baseURL = process.env.BASE_URL;

export default defineConfig({
  testDir: "tests",
  use: { baseURL: baseURL ?? "http://localhost:4173" },
  webServer: baseURL ? undefined : { command: "bun run preview --port 4173 --strictPort", port: 4173 },
});
