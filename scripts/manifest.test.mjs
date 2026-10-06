import { test } from "node:test";
import assert from "node:assert/strict";
import { createManifest } from "./manifest.mjs";

const valid = { version: "0.1.0", tag: "v0.1.0", signature: "signed-installer", publishedAt: "2026-10-06T18:00:00.000Z" };
test("versioned signed Windows artifact, stable download name", () => {
  const manifest = createManifest(valid);
  assert.equal(manifest.version, valid.version);
  assert.equal(manifest.platforms["windows-x86_64"].signature, valid.signature);
  assert.equal(manifest.platforms["windows-x86_64"].url, "https://github.com/andrefonn/revenueos-desktop/releases/download/v0.1.0/RevenueOS-Windows-x64-Setup.exe");
});
test("reject absent signature, mismatched tag, invalid version", () => {
  for (const overrides of [{ signature: "" }, { tag: "v0.2.0" }, { version: "../../evil" }, { version: "0.1.0-beta" }]) {
    assert.throws(() => createManifest({ ...valid, ...overrides }));
  }
});
