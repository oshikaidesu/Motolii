import { DiscoveryBrowserCandidate } from "@motolii/motolii-web";
import { LegacyHostBoundaryScreen } from "../legacy/index.js";
import { expect, userEvent, within } from "storybook/test";

const meta = {
  title: "M3 Archive/Legacy HTML/Host Boundary",
  component: LegacyHostBoundaryScreen,
  parameters: {
    docs: {
      description: {
        component:
          "アーカイブ済みHTMLのparity比較専用。新しいUI判断を追加せず、JSXのpropsやDOM境界を製品APIへ転記しない。",
      },
    },
  },
};

export default meta;

export const AllSurfaces = {
  args: {
    fixture: "all-surfaces",
  },
  name: "archive/#all-surfaces · integrated reference",
};

export const AssetExplorer = {
  args: {
    fixture: "asset-explorer",
  },
  name: "archive/#asset-explorer · project files",
};

export const Settings = {
  args: {
    fixture: "settings",
  },
  name: "archive/#settings · user settings",
};

export const PluginDiscoveryCandidate = {
  args: {
    fixture: "plugin-browser-candidate",
    BrowserComponent: DiscoveryBrowserCandidate,
  },
  name: "Plugin Browser · shared discovery shell candidate",
};

export const MissingPlugin = {
  args: {
    fixture: "all-surfaces",
  },
  name: "Missing plugin · interaction state",
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    const missingCard = await canvas.findByRole("button", {
      name: /Ribbon Array/,
    });
    await userEvent.click(missingCard);
    await expect(
      canvas.getByRole("button", { name: "Review recovery" }),
    ).toBeVisible();
  },
};

export const Recovery = {
  ...MissingPlugin,
  name: "Missing plugin · recovery dialog",
  play: async (context) => {
    await MissingPlugin.play(context);
    const canvas = within(context.canvasElement);
    await userEvent.click(
      canvas.getByRole("button", { name: "Review recovery" }),
    );
    await expect(
      canvas.getByRole("dialog", { name: "Plugin recovery candidate" }),
    ).toBeVisible();
  },
};
