import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

export type Json = any;

const HERE = dirname(fileURLToPath(import.meta.url));
export const CONF = join(HERE, "..", "..");
export const REPO = join(CONF, "..");

const REF = /^#\/components\/schemas\//;

// OpenAPI 3.0 schema -> JSON Schema (2019-09 keywords ajv understands). $refs point at root $defs.
export function toJsonSchema(s: Json): Json {
  if (Array.isArray(s)) return s.map(toJsonSchema);
  if (s === null || typeof s !== "object") return s;
  const out: Json = {};
  for (const [k, v] of Object.entries(s)) {
    if (k === "$ref" && typeof v === "string") out.$ref = v.replace(REF, "#/$defs/");
    else if (k === "nullable") continue;
    else if (k === "exclusiveMinimum" || k === "exclusiveMaximum") {
      const bound = k === "exclusiveMinimum" ? "minimum" : "maximum";
      if (v === true) { out[k] = s[bound]; delete out[bound]; } else if (v !== false) out[k] = v;
    }
    else if (k === "properties") out[k] = Object.fromEntries(Object.entries(v as Json).map(([n, p]) => [n, toJsonSchema(p)]));
    else if (k === "example" || k === "examples" || k === "default" || k === "enum" || k === "const") out[k] = v;
    else out[k] = toJsonSchema(v);
  }
  if (s.nullable === true) {
    if (Array.isArray(out.enum) && !out.enum.includes(null)) out.enum = [...out.enum, null];
    return { anyOf: [out, { type: "null" }] };
  }
  return out;
}

export function sha256(buf: Buffer | string) {
  return createHash("sha256").update(buf).digest("hex");
}

export const PINS: Record<"v1" | "v2", { url: string; sha256: string }> =
  JSON.parse(readFileSync(join(CONF, "spec-pins.json"), "utf8"));

// specPaths: local files, matched to a pin by sha256; otherwise download into conformance/.cache.
export async function loadSpec(version: "v1" | "v2", specPaths: string[] = []): Promise<Json> {
  const pin = PINS[version];
  let file = specPaths.find((p) => sha256(readFileSync(p)) === pin.sha256);
  if (!file && specPaths.length) throw new Error(`no --spec file matches pinned ${version} sha256 ${pin.sha256}`);
  if (!file) {
    file = join(CONF, ".cache", `${version}.json`);
    if (!existsSync(file) || sha256(readFileSync(file)) !== pin.sha256) {
      const res = await fetch(pin.url);
      if (!res.ok) throw new Error(`spec download ${res.status}`);
      const buf = Buffer.from(await res.arrayBuffer());
      if (sha256(buf) !== pin.sha256) throw new Error(`downloaded ${version} spec sha256 ${sha256(buf)} != pinned ${pin.sha256}`);
      mkdirSync(dirname(file), { recursive: true });
      writeFileSync(file, buf);
    }
  }
  return JSON.parse(readFileSync(file, "utf8"));
}

export interface Op { operationId: string; method: string; template: string; responses: Json }

const METHODS = ["get", "post", "put", "delete", "patch"];

export function listOps(spec: Json): Op[] {
  return Object.entries<Json>(spec.paths).flatMap(([template, item]) =>
    METHODS.filter((m) => item[m]).map((m) => ({
      operationId: item[m].operationId, method: m.toUpperCase(), template, responses: item[m].responses ?? {},
    })));
}

export function matchOp(ops: Op[], method: string, path: string, base: string): Op | undefined {
  const bare = path.split("?")[0];
  const rel = bare.startsWith(base) ? bare.slice(base.length) : bare;
  return ops.find((o) => o.method === method &&
    new RegExp("^" + o.template.replace(/[.*+?^$()|[\]\\]/g, "\\$&").replace(/\{[^}]+\}/g, "[^/]+") + "$").test(rel));
}

export function serverBase(spec: Json): string {
  const u: string = spec.servers?.[0]?.url ?? "";
  return u.replace(/^https?:\/\/[^/]+/, "").replace(/\/$/, "");
}
