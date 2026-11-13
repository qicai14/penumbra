// Prints one line per plugin enabled in .claude/settings.json that is not installed for this clone.
// Used by .claude/hooks/check-prereqs.sh (report) and scripts/bootstrap.sh (--names: ids to install).
import { homedir } from "node:os";
import { join } from "node:path";

const root = join(import.meta.dir, "..");
const settings = await Bun.file(join(root, ".claude/settings.json")).json();
const configDir = process.env.CLAUDE_CONFIG_DIR ?? join(homedir(), ".claude");
const installed: Record<string, { scope: string; projectPath?: string }[]> = await Bun.file(
  join(configDir, "plugins/installed_plugins.json"),
)
  .json()
  .then((j) => j.plugins ?? {})
  .catch(() => ({}));

const norm = (p: string) => p.replaceAll("\\", "/").replace(/\/$/, "").toLowerCase();
const here = norm(root);
const namesOnly = process.argv.includes("--names");

for (const [id, on] of Object.entries(settings.enabledPlugins ?? {})) {
  if (!on) continue;
  const ok = (installed[id] ?? []).some((e) => e.scope === "user" || (e.projectPath && norm(e.projectPath) === here));
  if (ok) continue;
  console.log(namesOnly ? id : `${id.split("@")[0]} plugin not installed in this clone; scripts/bootstrap.sh installs it.`);
}
