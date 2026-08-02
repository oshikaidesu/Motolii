import "../screens/all-surfaces.css";
import { GlobalTitlebar } from "../screens/GlobalTitlebar";

const meta = {
  title: "M3 References/Skeleton/Global titlebar",
  component: GlobalTitlebar,
  parameters: {
    docs: {
      description: {
        component:
          "固定AllSurfaces sourceから抽出したtitlebar。product接続とSettings/Export意味は含めない。",
      },
    },
  },
};

export default meta;

export const FixedSource = {
  args: { project: "night_drive.mv" },
};
