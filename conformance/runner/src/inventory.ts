import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { loadCases } from "./cases.js";
import { CONF, REPO } from "./schema.js";

const ROW = /^\| [A-Z]+ \| `[^`]+` \| `([^`]+)` \| .*? \| ([^|]+?) \| ([^|]*?) \|$/;

// Inventories are generated from the pinned spec, so their operationIds stand in for it offline.
export function checkInventory(parityDir = join(REPO, "docs/parity"), casesDir = join(CONF, "cases")): string[] {
  const cases = new Map(loadCases(casesDir).map((c) => [c.id, c]));
  const known = new Set<string>();
  const errors: string[] = [];
  for (const f of readdirSync(parityDir).filter((n) => /-rest-v\d\.md$/.test(n))) {
    for (const line of readFileSync(join(parityDir, f), "utf8").split("\n")) {
      const m = ROW.exec(line);
      if (!m) continue;
      known.add(m[1]);
      for (const id of m[3].split(/[,\s]+/).map((s) => s.replace(/`/g, "")).filter((s) => s && s !== "-")) {
        const c = cases.get(id);
        if (!c) errors.push(`${f}: ${m[1]} names missing case ${id}`);
        else if (!c.ops.includes(m[1])) errors.push(`${f}: case ${id} does not list ${m[1]} in ops`);
      }
    }
  }
  for (const c of cases.values()) for (const op of c.ops) if (!known.has(op)) errors.push(`case ${c.id}: unknown operationId ${op}`);
  return errors;
}
