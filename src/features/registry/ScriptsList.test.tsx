// story: e01s01
// scenario: SC-e01s01-P0-02
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
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
});
