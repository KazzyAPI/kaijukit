import { describe, expect, it } from "vitest";
import { matchOp, toJsonSchema } from "../src/schema.js";

describe("toJsonSchema", () => {
  it("turns nullable into anyOf with null", () => {
    expect(toJsonSchema({ type: "string", nullable: true })).toEqual({ anyOf: [{ type: "string" }, { type: "null" }] });
  });
  it("adds null to nullable enums", () => {
    expect(toJsonSchema({ type: "string", enum: ["a"], nullable: true })).toEqual({
      anyOf: [{ type: "string", enum: ["a", null] }, { type: "null" }],
    });
  });
  it("rewrites component refs to $defs, including nested", () => {
    expect(toJsonSchema({ type: "object", properties: { p: { $ref: "#/components/schemas/Page" } }, allOf: [{ $ref: "#/components/schemas/Base" }] }))
      .toEqual({ type: "object", properties: { p: { $ref: "#/$defs/Page" } }, allOf: [{ $ref: "#/$defs/Base" }] });
  });
  it("converts boolean exclusiveMinimum", () => {
    expect(toJsonSchema({ type: "integer", minimum: 0, exclusiveMinimum: true })).toEqual({ type: "integer", exclusiveMinimum: 0 });
  });
  it("does not treat a property named nullable as the keyword", () => {
    expect(toJsonSchema({ properties: { nullable: { type: "string" } } })).toEqual({ properties: { nullable: { type: "string" } } });
  });
});

describe("matchOp", () => {
  const ops = [
    { operationId: "getPageById", method: "GET", template: "/pages/{id}", responses: {} },
    { operationId: "getChildPages", method: "GET", template: "/pages/{id}/children", responses: {} },
  ];
  it("matches templates under the base path", () => {
    expect(matchOp(ops, "GET", "/wiki/api/v2/pages/7/children", "/wiki/api/v2")?.operationId).toBe("getChildPages");
    expect(matchOp(ops, "GET", "/wiki/api/v2/pages/7", "/wiki/api/v2")?.operationId).toBe("getPageById");
    expect(matchOp(ops, "POST", "/wiki/api/v2/pages/7", "/wiki/api/v2")).toBeUndefined();
  });
  it("ignores the query string", () => {
    expect(matchOp(ops, "GET", "/wiki/api/v2/pages/7?limit=1", "/wiki/api/v2")?.operationId).toBe("getPageById");
  });
});
