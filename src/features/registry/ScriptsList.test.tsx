// story: e01s01
// scenario: SC-e01s01-P0-02
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { ScriptsList } from "./ScriptsList";

describe("ScriptsList", () => {
  it("shows an explicit empty state when the registry has no entries", () => {
    render(<ScriptsList entries={[]} />);

    expect(
      screen.getByText("No scripts are registered yet."),
    ).toBeInTheDocument();
  });

  it("lists the path of every registered script", () => {
    render(
      <ScriptsList
        entries={[
          { id: "a1", path: "/home/user/build.sh" },
          { id: "b2", path: "/home/user/deploy.sh" },
        ]}
      />,
    );

    expect(screen.getByText("/home/user/build.sh")).toBeInTheDocument();
    expect(screen.getByText("/home/user/deploy.sh")).toBeInTheDocument();
  });

  // scenario: SC-e01s02-P1-08
  it("asks to remove the entry the user selected", async () => {
    const onRemove = vi.fn();

    render(
      <ScriptsList
        entries={[{ id: "a1", path: "/home/user/build.sh" }]}
        onRemove={onRemove}
      />,
    );

    await userEvent.click(screen.getByRole("button", { name: /remove/i }));

    expect(onRemove).toHaveBeenCalledWith("a1");
  });
});
