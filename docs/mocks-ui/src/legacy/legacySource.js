import sourceHtml from "../../../mocks/m3-vism-host-boundary.html?raw";

function requiredMatch(pattern, label) {
  const match = sourceHtml.match(pattern);
  if (!match) {
    throw new Error(`Legacy host boundary is missing its ${label}`);
  }
  return match[1];
}

const ACCEPTED_ROUTE_LEGACY_REMOVALS = [
  `      --bg:#141414; --panel:#1a1a1a; --raised:#222222; --hover:#2c2c2c;
      --line:#3b3b3b; --line2:#686868; --ink:#f0f0f0; --sub:#c6c6c6; --muted:#929292;
`,
  `      --active:#d8b574; --data:#78b5b0; --shape:#aaa0d0; --warning:#e18a6d;
      --ok:#90b287; `,
  `      --way-project:#6eb3ae; --way-files:#83a8cf; --way-plugins:#9f9fcf;
      --way-stage:#bca072; --way-inspector:#8eb086; --way-timeline:#cc9587;
`,
];

function removeRequiredSpanOnce(source, span) {
  const firstIndex = source.indexOf(span);
  if (firstIndex === -1) {
    throw new Error("Legacy host boundary palette span drifted");
  }
  if (source.indexOf(span, firstIndex + span.length) !== -1) {
    throw new Error("Legacy host boundary palette span is duplicated");
  }
  return source.replace(span, "");
}

// このbridgeが読むのはリポジトリ同梱の固定fixtureだけに限定する。
export const legacyStyle = requiredMatch(
  /<style>([\s\S]*?)<\/style>/i,
  "style element",
);
export const acceptedRouteLegacyStyle = ACCEPTED_ROUTE_LEGACY_REMOVALS.reduce(
  removeRequiredSpanOnce,
  legacyStyle,
);

export const legacyBody = requiredMatch(
  /<body[^>]*>([\s\S]*?)<script>/i,
  "body",
);

export const legacyScript = requiredMatch(
  /<script>([\s\S]*?)<\/script>\s*<\/body>/i,
  "script",
);
