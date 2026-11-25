import { readdirSync, readFileSync, existsSync } from "node:fs";
import { join } from "node:path";
import type { Json } from "./schema.js";
import type { Rules } from "./normalize.js";

export interface Req { method: string; path: string; query?: Record<string, string>; headers?: Record<string, string>; body?: Json; multipart?: Multipart; noFollow?: boolean }
export interface Multipart { filename: string; content: string; fields?: Record<string, string> }
export interface Case {
  id: string; ops: string[]; setup: Array<Req & { as: string }>; request: Req;
  expect?: { status?: number; body?: Json; headers?: Record<string, string> };
  normalize?: Rules;
}

export function loadCases(dir: string): Case[] {
  if (!existsSync(dir)) return [];
  return readdirSync(dir, { withFileTypes: true }).filter((d) => d.isDirectory()).map((d) => {
    const c: Case = JSON.parse(readFileSync(join(dir, d.name, "case.json"), "utf8"));
    if (c.id !== d.name) throw new Error(`case dir ${d.name} has id ${c.id}`);
    return { ...c, setup: c.setup ?? [] };
  });
}

const TPL = /\{\{\s*([\w.-]+)\s*\}\}/g;

function lookup(vars: Json, name: string): Json {
  const v = name.split(".").reduce((o, k) => o?.[k], vars);
  if (v === undefined) throw new Error(`unresolved variable {{${name}}}`);
  return v;
}

// A string that is exactly one template keeps the variable's type (numbers stay numbers in bodies).
export function subst(v: Json, vars: Json): Json {
  if (typeof v === "string") {
    const whole = v.match(/^\{\{\s*([\w.-]+)\s*\}\}$/);
    return whole ? lookup(vars, whole[1]) : v.replace(TPL, (_, n) => String(lookup(vars, n)));
  }
  if (Array.isArray(v)) return v.map((x) => subst(x, vars));
  if (v && typeof v === "object") return Object.fromEntries(Object.entries(v).map(([k, x]) => [k, subst(x, vars)]));
  return v;
}
