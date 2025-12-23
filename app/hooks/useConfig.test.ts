import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { useConfig } from "./useConfig";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";

const mockConfig = {
  sphinx: {
    source_dir: "docs",
    build_dir: "_build/html",
    server: { port: 0 },
  },
  python: { interpreter: "python" },
  editor: { command: "nvim" },
};

describe("useConfig", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should load config on mount", async () => {
    vi.mocked(invoke).mockResolvedValue(mockConfig);

    const { result } = renderHook(() => useConfig());

    expect(result.current.loading).toBe(true);

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.config).toEqual(mockConfig);
    expect(result.current.error).toBeNull();
    expect(invoke).toHaveBeenCalledWith("load_config");
  });

  it("should handle error when loading config fails", async () => {
    vi.mocked(invoke).mockRejectedValue(new Error("Config not found"));

    const { result } = renderHook(() => useConfig());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.config).toBeNull();
    expect(result.current.error).toBe("Error: Config not found");
  });

  it("should reload config when reload() is called", async () => {
    vi.mocked(invoke).mockResolvedValue(mockConfig);

    const { result } = renderHook(() => useConfig());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(invoke).toHaveBeenCalledTimes(1);

    act(() => {
      result.current.reload();
    });

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledTimes(2);
    });
  });
});
