import { writeFileSync } from "node:fs";
import { join } from "node:path";
import { parseArgs } from "node:util";
import { loadCases } from "./cases.js";
import { checkInventory } from "./inventory.js";
import { runCase, type Mode, type Result } from "./run.js";
import { CONF, loadSpec } from "./schema.js";

const { values: a } = parseArgs({ options: {
  base: { type: "string", default: "http://localhost:8080" },
  mode: { type: "string", default: "schema" },
  case: { type: "string", multiple: true },
  spec: { type: "string", multiple: true },
  cases: { type: "string", default: join(CONF, "cases") },
} });

if (a.mode === "inventory") {
  const errors = checkInventory(undefined, a.cases);
  errors.forEach((e) => console.error(e));
  console.log(errors.length ? `${errors.length} inventory error(s)` : "inventory ok");
  process.exit(errors.length ? 1 : 0);
}
if (!["schema", "diff", "record"].includes(a.mode!)) { console.error(`unknown mode ${a.mode}`); process.exit(2); }
const mode = a.mode as Mode;

// Record talks to a reference instance: credentials come only from the environment and are never written.
const auth = mode === "record" ? process.env.REF_AUTH ?? "" : process.env.KK_AUTH ?? "admin@kaijukit.local:kk-dev-token";
if (mode === "record" && !auth) { console.error("record needs REF_AUTH=user:token for the reference instance"); process.exit(2); }

const specs: Record<string, Promise<unknown>> = {};
const spec = (path: string) => (specs[path.startsWith("/wiki/rest/api") ? "v1" : "v2"] ??=
  loadSpec(path.startsWith("/wiki/rest/api") ? "v1" : "v2", a.spec)) as Promise<any>;

let cases = loadCases(a.cases!);
if (a.case?.length) {
  const missing = a.case.filter((id) => !cases.some((c) => c.id === id));
  if (missing.length) { console.error(`no such case: ${missing}`); process.exit(2); }
  cases = cases.filter((c) => a.case!.includes(c.id));
}

const results: Result[] = [];
for (const c of cases) {
  const r = await runCase(c, mode, { base: a.base!, auth, spec });
  if (mode === "record" && r.status === "pass") {
    writeFileSync(join(a.cases!, c.id, "case.json"), r.detail + "\n");
    r.detail = "recorded";
  }
  results.push(r);
}

const w = Math.max(4, ...results.map((r) => r.id.length));
console.log(`${"CASE".padEnd(w)}  RESULT  DETAIL`);
for (const r of results) console.log(`${r.id.padEnd(w)}  ${r.status.toUpperCase().padEnd(6)}  ${r.detail}`);
const count = (s: string) => results.filter((r) => r.status === s).length;
console.log(`\n${mode}: ${count("pass")} passed, ${count("fail")} failed, ${count("skip")} skipped`);
process.exit(count("fail") ? 1 : 0);
