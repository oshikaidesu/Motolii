/** @type { import('@storybook/react-vite').StorybookConfig } */
const config = {
  stories: ["../src/stories/**/*.stories.@(js|jsx)"],
  framework: {
    name: "@storybook/react-vite",
    options: {},
  },
};

export default config;
