import { LegacyHostBoundaryScreen } from "../legacy/index.js";
import { expect, userEvent, within } from "storybook/test";

const meta = {
  title: "M3 References/Current HTML/Host Boundary",
  component: LegacyHostBoundaryScreen,
  parameters: {
    docs: {
      description: {
        component:
          "現行HTMLを段階分解するための比較基準。JSXのpropsやDOM境界を製品APIへ転記しない。",
      },
    },
  },
};

export default meta;

export const AllSurfaces = {
  args: {
    fixture: "all-surfaces",
  },
  name: "#all-surfaces · integrated reference",
};

export const AssetExplorer = {
  args: {
    fixture: "asset-explorer",
  },
  name: "#asset-explorer · project files",
};

export const Settings = {
  args: {
    fixture: "settings",
  },
  name: "#settings · user settings",
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
