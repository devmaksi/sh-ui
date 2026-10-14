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
          { id: "a1", path: "/home/user/build.sh", config: { args: [], env: {}, workingDir: null } },
          { id: "b2", path: "/home/user/deploy.sh", config: { args: [], env: {}, workingDir: null } },
        ]}
      />,
    );

    expect(screen.getByText("/home/user/build.sh")).toBeInTheDocument();
    expect(screen.getByText("/home/user/deploy.sh")).toBeInTheDocument();
  });

  // scenario: SC-e01s03-P0-M3
  it("asks to start the entry the user selected", async () => {
    const onStart = vi.fn();

    render(
      <ScriptsList
        entries={[{ id: "a1", path: "/home/user/build.sh", config: { args: [], env: {}, workingDir: null } }]}
        onStart={onStart}
      />,
    );

    await userEvent.click(screen.getByRole("button", { name: /^start/i }));

    expect(onStart).toHaveBeenCalledWith("a1");
  });

  // story: e02s01
  // scenario: SC-e02s01-P2-05
  it("asks to edit the entry the user selected", async () => {
    const onEdit = vi.fn();

    render(
      <ScriptsList
        entries={[{ id: "a1", path: "/home/user/build.sh", config: { args: [], env: {}, workingDir: null } }]}
        onEdit={onEdit}
      />,
    );

    await userEvent.click(screen.getByRole("button", { name: /edit/i }));

    expect(onEdit).toHaveBeenCalledWith("a1");
  });

  // scenario: SC-e01s02-P1-08
  it("asks to remove the entry the user selected", async () => {
    const onRemove = vi.fn();

    render(
      <ScriptsList
        entries={[{ id: "a1", path: "/home/user/build.sh", config: { args: [], env: {}, workingDir: null } }]}
        onRemove={onRemove}
      />,
    );

    await userEvent.click(screen.getByRole("button", { name: /remove/i }));

    expect(onRemove).toHaveBeenCalledWith("a1");
  });
});
