const referenceViewport = {
  width: "1440px",
  minHeight: "900px",
  overflow: "auto",
};

export const decorators = [
  (Story) => (
    <div style={referenceViewport}>
      <Story />
    </div>
  ),
];

export const parameters = {
  layout: "fullscreen",
  controls: {
    disable: true,
  },
  options: {
    storySort: {
      order: ["M3 References", ["Current HTML", "Skeleton"]],
    },
  },
};
