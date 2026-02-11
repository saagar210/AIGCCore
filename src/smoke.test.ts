import { describe, expect, it } from "vitest";
import type { PackCommandStatus } from "./ui/packs/types";

describe("ui smoke", () => {
  it("runs test harness", () => {
    expect(true).toBe(true);
  });

  it("future pack status shape remains stable", () => {
    const status: PackCommandStatus = {
      status: "READY",
      message: "scaffold"
    };
    expect(status.status).toBe("READY");
  });
});
