// Japanese lyrics, the same four lines twice. Left: the fonts as shaped (every law off). Right: the three CSS
// typography laws pressed — text-autospace (1/8 em between kanji and Latin/digits), text-spacing-trim: trim-start
// (opening brackets lose their left half at a line start), hanging-punctuation: first force-end last (brackets and
// stops hang outside the box). The thin rule marks each column's left edge; the box holds tools, the lines hold meaning.
comp({ width: 1920, height: 1080, fps: 30, seconds: 4, background: "#F4F1EA" });
const INK = "#161616", RULE = "#C9A24A";
const LINES = "「Motoliiを2026年に」\n夜のCity、Neonが3つ。\n『Sync』の画をVismで、\n「ここから」と、君は言った。";
const OFF = { "Text Autospace": "No Autospace", "Text Spacing Trim": "Space All", "Hanging Punctuation": "None" };
const ON = { "Text Autospace": "Normal", "Text Spacing Trim": "Trim Start", "Hanging Punctuation": "First Force End Last" };
const column = (name, x, laws) => {
  rectangle({ name: `${name} rule` }).fill(RULE).set("Scale", [2 / 270, 720 / 270]).set("Position", [x, 540]);
  const lines = text(LINES, { name }).fill(INK).font("Hiragino Sans").set("Alignment", "Left").set("Size", 64)
    .set("Anchor", [0, 540]).set("Position", [x, 540]);
  for (const [law, value] of Object.entries(laws)) lines.set(law, value);
  return lines;
};
column("Laws off", 140, OFF);
column("Laws on", 1040, ON);
