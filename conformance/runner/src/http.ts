import type { Req } from "./cases.js";
import type { Json } from "./schema.js";

export interface Res { status: number; headers: Record<string, string>; body: Json; raw: string }

export async function send(base: string, auth: string, r: Req): Promise<Res> {
  const url = new URL(base.replace(/\/$/, "") + r.path);
  for (const [k, v] of Object.entries(r.query ?? {})) url.searchParams.set(k, v);
  const hasBody = r.body !== undefined && r.body !== null;
  let form: FormData | undefined;
  if (r.multipart) {
    form = new FormData();
    for (const [k, v] of Object.entries(r.multipart.fields ?? {})) form.append(k, v);
    form.append("file", new Blob([r.multipart.content]), r.multipart.filename);
  }
  const res = await fetch(url, {
    method: r.method,
    headers: {
      accept: "application/json",
      ...(auth ? { authorization: "Basic " + Buffer.from(auth).toString("base64") } : {}),
      ...(hasBody && !form ? { "content-type": "application/json" } : {}),
      ...r.headers,
    },
    body: form ?? (hasBody ? JSON.stringify(r.body) : undefined),
  });
  const raw = await res.text();
  let body: Json = raw === "" ? null : raw;
  if ((res.headers.get("content-type") ?? "").includes("json") && raw) {
    try { body = JSON.parse(raw); } catch { /* left as raw text; schema mode reports it */ }
  }
  return { status: res.status, headers: Object.fromEntries(res.headers), body, raw };
}
