import { createServer, type Server } from "node:http";
import { mkdtempSync, mkdirSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import type { Case } from "../src/cases.js";
import { checkInventory } from "../src/inventory.js";
import { runCase } from "../src/run.js";

const spec = {
  servers: [{ url: "https://{host}/wiki/api/v2" }],
  components: { schemas: { Page: { type: "object", required: ["id", "title"], properties: { id: { type: "string" }, title: { type: "string" }, parentId: { type: "string", nullable: true } } } } },
  paths: { "/pages/{id}": { get: { operationId: "getPageById", responses: { "200": { content: { "application/json": { schema: { $ref: "#/components/schemas/Page" } } } }, "404": { content: {} } } } } },
};

let server: Server;
let base: string;
let reply: unknown = {};
beforeAll(async () => {
  server = createServer((req, res) => {
    if (req.url === "/wiki/api/v2/pages/missing") { res.statusCode = 404; return res.end(); }
    if (req.url === "/wiki/api/v2/pages/odd") { res.statusCode = 418; return res.end("{}"); }
    res.setHeader("content-type", "application/json");
    res.end(JSON.stringify(reply));
  }).listen(0);
  await new Promise((r) => server.once("listening", r));
  base = `http://127.0.0.1:${(server.address() as any).port}`;
});
afterAll(() => server.close());

const mk = (id: string, expect?: Case["expect"]): Case => ({
  id, ops: ["getPageById"], setup: [], expect,
  request: { method: "GET", path: `/wiki/api/v2/pages/${id}` },
});
const run = (c: Case) => runCase(c, "schema", { base, auth: "", spec: async () => spec });

describe("schema mode", () => {
  it("passes a conforming body", async () => {
    reply = { id: "1", title: "t", parentId: null };
    expect((await run(mk("1"))).status).toBe("pass");
  });
  it("fails a wrong field type", async () => {
    reply = { id: 1, title: "t" };
    const r = await run(mk("1"));
    expect(r.status).toBe("fail");
    expect(r.detail).toContain("/id must be string");
  });
  it("fails an undocumented status", async () => {
    const r = await run(mk("odd"));
    expect(r.status).toBe("fail");
    expect(r.detail).toContain("not documented");
  });
  it("passes a documented empty-body status and enforces expect.status", async () => {
    expect((await run(mk("missing", { status: 404 }))).status).toBe("pass");
    expect((await run(mk("missing", { status: 200 }))).status).toBe("fail");
  });
});

describe("check:inventory", () => {
  it("reports missing cases and unknown operationIds", () => {
    const dir = mkdtempSync(join(tmpdir(), "inv-"));
    mkdirSync(join(dir, "parity"));
    mkdirSync(join(dir, "cases/c1"), { recursive: true });
    const row = (op: string, c: string) => `| GET | \`/x\` | \`${op}\` | Sum | not started | ${c} |`;
    writeFileSync(join(dir, "parity/p-rest-v2.md"), [row("opA", "c1"), row("opB", "nope"), row("opC", "-")].join("\n"));
    writeFileSync(join(dir, "cases/c1/case.json"), JSON.stringify({ id: "c1", ops: ["opA", "ghost"], request: { method: "GET", path: "/x" } }));
    expect(checkInventory(join(dir, "parity"), join(dir, "cases"))).toEqual([
      "p-rest-v2.md: opB names missing case nope",
      "case c1: unknown operationId ghost",
    ]);
  });
});
