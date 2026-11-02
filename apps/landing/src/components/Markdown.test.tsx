import { render, screen } from "@testing-library/react";
import Markdown from "./Markdown";

describe("Markdown", () => {
  it("renders markdown content", () => {
    render(<Markdown content="# Hello" />);
    expect(screen.getByText("Hello")).toBeInTheDocument();
  });

  it("renders empty content", () => {
    render(<Markdown content="" />);
    expect(screen.getByTestId("markdown")).toBeInTheDocument();
  });
});
