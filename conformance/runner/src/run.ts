import { randomBytes } from "node:crypto";
import { isDeepStrictEqual } from "node:util";
import AjvModule from "ajv";
import { subst, type Case } from "./cases.js";
import { send, type Res } from "./http.js";
import { normalize } from "./normalize.js";
import { listOps, matchOp, serverBase, toJsonSchema, type Json, type Op } from "./schema.js";

const Ajv = (AjvModule as any).default ?? AjvModule;

export type Mode = "schema" | "diff" | "record";
export interface Result { id: string; status: "pass" | "fail" | "skip"; detail: string }
export interface Ctx { base: string; auth: string; spec: (path: string) => Promise<Json> }

export async function execute(c: Case, ctx: Ctx): Promise<{ res: Res; vars: Json }> {
  const vars: Json = { run: randomBytes(4).toString("hex") };
  for (const s of c.setup) {
    const r = await send(ctx.base, ctx.auth, subst(s, vars));
    if (r.status >= 300) throw new Error(`setup ${s.as}: ${s.method} ${s.path} -> ${r.status} ${r.raw.slice(0, 200)}`);
    vars[s.as] = r.body;
  }
  return { res: await send(ctx.base, ctx.auth, subst(c.request, vars)), vars };
}

function responseSchema(spec: Json, op: Op, status: number): Json | null | undefined {
  let r = op.responses[String(status)] ?? op.responses.default;
  if (!r) return undefined;
  if (r.$ref) r = r.$ref.split("/").reduce((o: Json, k: string) => (k === "#" ? spec : o[k]), spec);
  return r.content?.["application/json"]?.schema ?? null;
}

export function validateSchema(spec: Json, op: Op, res: Res): string | null {
  const schema = responseSchema(spec, op, res.status);
  if (schema === undefined) return `status ${res.status} not documented for ${op.operationId} (${Object.keys(op.responses).join(",")})`;
  if (schema === null) return null;
  const defs = Object.fromEntries(Object.entries<Json>(spec.components?.schemas ?? {}).map(([k, v]) => [k, toJsonSchema(v)]));
  const validate = new Ajv({ strict: false, validateFormats: false }).compile({ ...toJsonSchema(schema), $defs: defs });
  if (typeof res.body !== "object" || res.body === null) return `body is not JSON: ${res.raw.slice(0, 80)}`;
  return validate(res.body) ? null : validate.errors!.slice(0, 3).map((e: Json) => `${e.instancePath || "/"} ${e.message}`).join("; ");
}

function firstDiff(a: Json, b: Json, at = "$"): string {
  if (a && b && typeof a === "object" && typeof b === "object") {
    for (const k of new Set([...Object.keys(a), ...Object.keys(b)])) {
      if (!isDeepStrictEqual(a[k], b[k])) return firstDiff(a[k], b[k], `${at}.${k}`);
    }
  }
  return `${at}: expected ${JSON.stringify(a)} got ${JSON.stringify(b)}`;
}

export async function runCase(c: Case, mode: Mode, ctx: Ctx): Promise<Result> {
  const fail = (detail: string): Result => ({ id: c.id, status: "fail", detail });
  if (mode === "diff" && c.expect?.body === undefined) return { id: c.id, status: "skip", detail: "no recorded expect.body" };
  let out;
  try { out = await execute(c, ctx); } catch (e) { return fail(String((e as Error).message)); }
  const { res } = out;
  if (mode === "record") {
    const expect = { status: res.status, headers: { "content-type": (res.headers["content-type"] ?? "").split(";")[0] }, body: normalize(res.body, c.normalize) };
    return { id: c.id, status: "pass", detail: JSON.stringify({ ...c, expect }, null, 2) };
  }
  if (c.expect?.status !== undefined && c.expect.status !== res.status) return fail(`status ${res.status}, expected ${c.expect.status}`);
  if (mode === "diff") {
    for (const [h, v] of Object.entries(c.expect?.headers ?? {})) {
      if ((res.headers[h.toLowerCase()] ?? "").split(";")[0] !== v) return fail(`header ${h}: expected ${v} got ${res.headers[h.toLowerCase()]}`);
    }
    const got = normalize(res.body, c.normalize);
    return isDeepStrictEqual(got, c.expect!.body) ? { id: c.id, status: "pass", detail: "" } : fail(firstDiff(c.expect!.body, got));
  }
  const spec = await ctx.spec(c.request.path);
  const op = matchOp(listOps(spec), c.request.method, c.request.path, serverBase(spec));
  if (!op) return fail(`${c.request.method} ${c.request.path} matches no operation in the spec`);
  if (!c.ops.includes(op.operationId)) return fail(`request is ${op.operationId}, not listed in ops [${c.ops}]`);
  const err = validateSchema(spec, op, res);
  return err ? fail(err) : { id: c.id, status: "pass", detail: `${op.operationId} ${res.status}` };
}
