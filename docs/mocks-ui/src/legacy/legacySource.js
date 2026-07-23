import sourceHtml from "../../../mocks/m3-vism-host-boundary.html?raw";

function requiredMatch(pattern, label) {
  const match = sourceHtml.match(pattern);
  if (!match) {
    throw new Error(`Legacy host boundary is missing its ${label}`);
  }
  return match[1];
}

// このbridgeが読むのはリポジトリ同梱の固定fixtureだけに限定する。
export const legacyStyle = requiredMatch(
  /<style>([\s\S]*?)<\/style>/i,
  "style element",
);

export const legacyBody = requiredMatch(
  /<body[^>]*>([\s\S]*?)<script>/i,
  "body",
);

export const legacyScript = requiredMatch(
  /<script>([\s\S]*?)<\/script>\s*<\/body>/i,
  "script",
);

