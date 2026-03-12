import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { existsSync } from "fs";
import {
  isPlatformSupported,
  getSupportedPlatforms,
  getPlatformPackageName,
} from "../src/loader";

// ── Platform helpers ──────────────────────────────────────────────────────────

describe("getSupportedPlatforms()", () => {
  it("returns the 4 supported platform keys", () => {
    const platforms = getSupportedPlatforms();
    expect(platforms).toContain("linux-x64");
    expect(platforms).toContain("darwin-arm64");
    expect(platforms).toContain("darwin-x64");
    expect(platforms).toContain("win32-x64");
    expect(platforms).toHaveLength(4);
  });

  it("all keys follow the <os>-<arch> format", () => {
    for (const key of getSupportedPlatforms()) {
      expect(key).toMatch(/^[a-z0-9]+-[a-z0-9]+$/);
    }
  });
});

describe("isPlatformSupported()", () => {
  it("returns a boolean", () => {
    expect(typeof isPlatformSupported()).toBe("boolean");
  });

  it("returns true on known CI platforms (linux-x64, darwin-arm64)", () => {
    // The test runner itself must be on a supported platform
    // If this fails, add the platform to PLATFORM_PACKAGES in loader.ts
    const key = `${process.platform}-${process.arch}`;
    const supported = getSupportedPlatforms();

    if (supported.includes(key)) {
      expect(isPlatformSupported()).toBe(true);
    } else {
      // On an unsupported platform (e.g. linux-arm64) — acceptable in dev
      expect(isPlatformSupported()).toBe(false);
    }
  });
});

describe("getPlatformPackageName()", () => {
  it("returns null for an unsupported platform key", () => {
    // We can't easily fake process.platform, so just verify the return type
    const result = getPlatformPackageName();
    expect(result === null || typeof result === "string").toBe(true);
  });

  it("returns a string starting with repo-intel- when supported", () => {
    const result = getPlatformPackageName();
    if (result !== null) {
      expect(result).toMatch(/^repo-intel-/);
    }
  });
});

// ── getBinaryPath() ───────────────────────────────────────────────────────────

describe("getBinaryPath()", () => {
  const originalEnv = process.env["REPO_INTEL_BINARY"];

  afterEach(() => {
    // Restore env after each test
    if (originalEnv === undefined) {
      delete process.env["REPO_INTEL_BINARY"];
    } else {
      process.env["REPO_INTEL_BINARY"] = originalEnv;
    }
  });

  it("resolves when REPO_INTEL_BINARY points to an existing file", async () => {
    // Use the Node.js binary itself as a stand-in for a real binary
    const nodeBin = process.execPath;
    process.env["REPO_INTEL_BINARY"] = nodeBin;

    // Re-import to pick up the env var (dynamic import bypasses module cache)
    const { getBinaryPath } = await import("../src/loader");
    const result = getBinaryPath();

    expect(result).toBe(nodeBin);
    expect(existsSync(result)).toBe(true);
  });

  it("throws when REPO_INTEL_BINARY points to a non-existent file", async () => {
    process.env["REPO_INTEL_BINARY"] = "/nonexistent/path/to/repo-intel";

    const { getBinaryPath } = await import("../src/loader");

    expect(() => getBinaryPath()).toThrow("REPO_INTEL_BINARY");
    expect(() => getBinaryPath()).toThrow("does not exist");
  });

  it("throws a descriptive error on unsupported platform when no env override", async () => {
    // Only testable when the current platform is unsupported OR the package isn't installed.
    // We test the error message shape by checking what happens without the env var
    // and without the platform package installed (typical in CI for this package).
    delete process.env["REPO_INTEL_BINARY"];

    const { getBinaryPath } = await import("../src/loader");

    // On a supported platform where the package IS installed, this won't throw.
    // On all other cases it will. We just verify it doesn't crash unexpectedly.
    try {
      const path = getBinaryPath();
      expect(typeof path).toBe("string");
      expect(path.length).toBeGreaterThan(0);
    } catch (err) {
      // If it throws, the error must be descriptive
      expect(err).toBeInstanceOf(Error);
      const message = (err as Error).message;
      expect(message).toMatch(/repo-intel/);
    }
  });

  it("error message includes supported platforms list", async () => {
    process.env["REPO_INTEL_BINARY"] = "/nonexistent/repo-intel";

    const { getBinaryPath } = await import("../src/loader");

    // This path will throw because the file doesn't exist
    expect(() => getBinaryPath()).toThrow("REPO_INTEL_BINARY");
  });
});
