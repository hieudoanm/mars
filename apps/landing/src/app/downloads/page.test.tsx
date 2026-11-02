import { render, screen } from "@testing-library/react";
import DownloadsPage from "./page";

describe("DownloadsPage", () => {
  it("renders download page title", () => {
    render(<DownloadsPage />);
    expect(screen.getByText(/Get Mars for your platform/i)).toBeInTheDocument();
  });

  it("renders platform options", () => {
    render(<DownloadsPage />);
    expect(screen.getByText("macOS")).toBeInTheDocument();
    expect(screen.getByText("Windows")).toBeInTheDocument();
    expect(screen.getByText("Linux")).toBeInTheDocument();
  });
});
