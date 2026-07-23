import document from "../../fixtures/reference-document.json";
import scenes from "../../fixtures/reference-scenes.json";
import tokens from "../../fixtures/reference-candidate-tokens.json";
import "../screens/all-surfaces.css";
import "./reference-font.css";

const injected = globalThis.__MOTOLII_REFERENCE_FIXTURES__;

export const referenceFixtures = Object.freeze(
  injected ?? { document, scenes, tokens },
);
