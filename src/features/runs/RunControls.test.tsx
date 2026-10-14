// story: e02s02
// scenario: SC-e02s02-P2-04
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { RunControls } from "./RunControls";

describe("RunControls", () => {
  it("offers Stop while the run is live", async () => {
    const onStop = vi.fn();
    render(<RunControls running onStop={onStop} />);

    const button = screen.getByRole("button", { name: /stop/i });
    expect(button).toBeEnabled();

    await userEvent.click(button);
    expect(onStop).toHaveBeenCalledTimes(1);
  });

  it("disables Stop once the run has ended", () => {
    render(<RunControls running={false} onStop={vi.fn()} />);

    expect(screen.getByRole("button", { name: /stop/i })).toBeDisabled();
  });
});
