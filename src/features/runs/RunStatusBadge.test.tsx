// story: e02s03
// scenario: SC-e02s03-P2-03
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { RunStatusBadge } from "./RunStatusBadge";
import type { RunEndStatus } from "./types";

describe("RunStatusBadge", () => {
  it("renders each state with its own label", () => {
    const seen = new Map<RunEndStatus | "notRunning", string>();

    const first = render(<RunStatusBadge state="notRunning" />);
    expect(screen.getByText("not running")).toBeInTheDocument();
    first.unmount();

    const cases: Array<[RunEndStatus, string]> = [
      ["exited", "exited"],
      ["failed", "failed"],
      ["stopped", "stopped"],
    ];

    for (const [state, label] of cases) {
      const view = render(<RunStatusBadge state={state} />);
      expect(screen.getByText(label)).toBeInTheDocument();
      seen.set(state, label);
      view.unmount();
    }

    expect(seen.size).toBe(3);
  });

  it("shows the exit code when a failed run has one", () => {
    render(<RunStatusBadge state="failed" exitCode={7} />);
    expect(screen.getByText(/failed/)).toHaveTextContent("7");
  });

  it("renders running without an end label", () => {
    render(<RunStatusBadge state="running" />);
    expect(screen.queryByText(/exited|failed|stopped|not running/)).toBeNull();
    expect(screen.getByText(/running/)).toBeInTheDocument();
  });
});
