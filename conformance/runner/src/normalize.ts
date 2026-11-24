import type { Json } from "./schema.js";

export interface Rules {
  ignore?: string[];
  placeholders?: Record<string, string>;
  sort?: string[];
}

const UUID = /\b[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\b/g;

// shortcut: supports `$.a.b[*].c` only (no filters or recursive descent).
function select(root: Json, path: string): Array<[Json, string]> {
  const keys = path.replace(/^\$\.?/, "").split(/\.|(?=\[)/).filter(Boolean).map((k) => k.replace(/^\[|\]$/g, ""));
  let cur: Array<[Json, string]> = [[{ $: root }, "$"]];
  for (const k of keys) {
    cur = cur.flatMap(([o, key]) => {
      const v = o[key];
      if (v === null || typeof v !== "object") return [];
      return k === "*" ? Object.keys(v).map((i): [Json, string] => [v, i]) : k in v ? [[v, k] as [Json, string]] : [];
    });
  }
  return cur;
}

function strip(v: Json, parent?: string): Json {
  if (Array.isArray(v)) return v.map((x) => strip(x));
  if (v === null || typeof v !== "object") return v;
  return Object.fromEntries(Object.entries(v)
    .filter(([k]) => k !== "createdAt" && !(parent === "_links" && k === "base"))
    .map(([k, x]) => [k, strip(x, k)]));
}

function mapStrings(v: Json, f: (s: string) => string): Json {
  if (typeof v === "string") return f(v);
  if (Array.isArray(v)) return v.map((x) => mapStrings(x, f));
  if (v && typeof v === "object") return Object.fromEntries(Object.entries(v).map(([k, x]) => [k, mapStrings(x, f)]));
  return v;
}

export function normalize(body: Json, rules: Rules = {}): Json {
  const out = strip(structuredClone(body));
  for (const p of rules.ignore ?? []) for (const [o, k] of select(out, p).reverse()) Array.isArray(o) ? o.splice(Number(k), 1) : delete o[k];
  for (const p of rules.sort ?? []) for (const [o, k] of select(out, p)) {
    if (Array.isArray(o[k])) o[k].sort((a: Json, b: Json) => (JSON.stringify(a) < JSON.stringify(b) ? -1 : 1));
  }
  const seen = new Map<string, string>();
  const counts: Record<string, number> = {};
  const assign = (name: string, value: string) => {
    if (!seen.has(value)) seen.set(value, `<${name}:${(counts[name] = (counts[name] ?? 0) + 1)}>`);
  };
  for (const [name, p] of Object.entries(rules.placeholders ?? {})) {
    for (const [o, k] of select(out, p)) if (o[k] !== null && typeof o[k] !== "object") assign(name, String(o[k]));
  }
  mapStrings(out, (s) => { for (const u of s.match(UUID) ?? []) assign("uuid", u); return s; });
  return mapStrings(out, (s) => {
    if (seen.has(s)) return seen.get(s)!;
    let r = s.replace(UUID, (u) => seen.get(u)!);
    for (const [v, ph] of seen) r = r.replace(new RegExp(`(?<=/)${v.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}(?=/|\\?|$)`, "g"), ph);
    return r;
  });
}
