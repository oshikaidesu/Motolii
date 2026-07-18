import { AllSurfacesScreen } from "../screens/AllSurfacesScreen";

const meta = {
  title: "M3 References/Skeleton/All Surfaces",
  component: AllSurfacesScreen,
  parameters: {
    docs: {
      description: {
        component:
          "分解境界を確認する骨格版。現行HTMLの視覚正本や製品tokenではない。",
      },
    },
  },
};

export default meta;

export const ComponentSkeleton = {
  name: "Component skeleton (not current reference)",
};
