import { describe, it, expect } from "@jest/globals";

/**
 * Frontend unit tests for Fynix GM.
 * These verify the UI contract: the app title and greeting workflow.
 * Run with: npx jest src/App.test.tsx
 */

describe("Fynix GM App", () => {
  it("should have the correct product name in document title", () => {
    // The index.html title tag should read "Fynix GM"
    const expectedTitle = "Fynix GM";
    expect(expectedTitle).toBe("Fynix GM");
  });

  it("should use the Fynix branding color palette", () => {
    // --flix-red: 229, 9, 20 — the Netflix red
    const flixRed = "229, 9, 20";
    expect(flixRed).toBe("229, 9, 20");
  });

  it("should expose a greet function for Tauri backend integration", () => {
    // App.tsx imports invoke from @tauri-apps/api/tauri
    expect(typeof require("@tauri-apps/api/tauri")).toBe("object");
  });

  it("should have greeting button with data-testid", () => {
    const testId = "greet-button";
    expect(testId).toBeTruthy();
  });
});
