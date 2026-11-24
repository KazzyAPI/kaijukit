import { describe, expect, it } from "vitest";
import { normalize } from "../src/normalize.js";

describe("normalize", () => {
  it("drops volatile fields", () => {
    expect(normalize({ id: "1", createdAt: "x", version: { number: 1, createdAt: "y" }, _links: { base: "h", self: "s" } }))
      .toEqual({ id: "1", version: { number: 1 }, _links: { self: "s" } });
  });
  it("applies ignore paths with wildcards", () => {
    expect(normalize({ results: [{ a: 1, b: 2 }, { a: 3, b: 4 }] }, { ignore: ["$.results[*].b"] }))
      .toEqual({ results: [{ a: 1 }, { a: 3 }] });
  });
  it("assigns placeholders by first occurrence and rewrites references", () => {
    const body = { results: [{ id: "900", parentId: null, _links: { webui: "/pages/900/x" } }, { id: "512", parentId: "900" }] };
    expect(normalize(body, { placeholders: { id: "$.results[*].id" } })).toEqual({
      results: [{ id: "<id:1>", parentId: null, _links: { webui: "/pages/<id:1>/x" } }, { id: "<id:2>", parentId: "<id:1>" }],
    });
  });
  it("replaces uuids", () => {
    const u = "123e4567-e89b-12d3-a456-426614174000";
    expect(normalize({ a: u, b: `x/${u}` })).toEqual({ a: "<uuid:1>", b: "x/<uuid:1>" });
  });
  it("sorts declared unordered arrays", () => {
    expect(normalize({ r: [{ k: "b" }, { k: "a" }] }, { sort: ["$.r"] })).toEqual({ r: [{ k: "a" }, { k: "b" }] });
  });
  it("does not mutate input", () => {
    const i = { createdAt: "x" };
    normalize(i);
    expect(i).toEqual({ createdAt: "x" });
  });
});
