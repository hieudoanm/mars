import { getDocs, getDoc } from "./docs";

describe("docs", () => {
  it("has docs array", () => {
    const docs = getDocs();
    expect(Array.isArray(docs)).toBe(true);
    expect(docs.length).toBeGreaterThan(0);
  });

  it("gets doc by slug", () => {
    const doc = getDoc("getting-started");
    expect(doc).toBeDefined();
    expect(doc?.slug).toBe("getting-started");
  });

  it("returns undefined for unknown slug", () => {
    expect(getDoc("unknown")).toBeUndefined();
  });
});
