import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";

import { PluginsPage } from "@/features/plugins/PluginsPage";
import { useDesktopStore } from "@/store/useDesktopStore";

describe("PluginsPage", () => {
  it("renders an empty state when there are no plugins", () => {
    useDesktopStore.setState({
      plugins: [],
      selectedPluginName: null,
      busy: {
        bootstrap: false,
        settings: false,
        plugins: false,
        doctor: false,
        activity: false,
        cache: false,
      },
    });

    render(<PluginsPage />);

    expect(screen.getByText("No managed plugins yet")).toBeInTheDocument();
  });
});
