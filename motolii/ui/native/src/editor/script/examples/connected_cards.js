// Boxes joined by lines (first-class connectors): a row of specimen cards on red, each with its name on top.
// Dashed ropes hang between neighbours; as the cards change width, the ropes stretch, slacken and swing a little late.
comp({ width: 1920, height: 1080, fps: 30, seconds: 6, background: "#D63A3A" });

const PAPER = "#F2EDE4", INK = "#161616";
const names = ["Bud", "Tepal", "Stigma", "Stamen", "Receptacle"];
const tones = [INK, "#8E1B1B", INK, "#5A0F0F", INK];
const cards = names.map((name, i) => {
  const r = rectangle({ name }).fill(tones[i]);
  r.set("Position", [0, 0]);
  return r;
});
const row = group(...cards).name("Row");
row.set("Display", "Flex").set("Justify Content", "Center").set("Align Items", "Center").set("Gap", 80)
  .set("Horizontal Sizing", "Fixed").set("Width", 1920).set("Position", [0, 470]);
const widths = [[0, 170], [1.5, 300], [3, 140], [4.5, 240], [6, 170]];
cards.forEach((card, i) => {
  card.set("Horizontal Sizing", "Fixed").set("Vertical Sizing", "Fixed").set("Height", 200 + (i % 2) * 60)
    .set("Transition Duration", 0.4).set("Transition Easing", "Ease In Out");
  card.keys("Width", widths.map(([s, w], k) => [s + i * 0.12, w + ((i + k) % 3) * 40, "Bezier"]));
});

// Names that belong to the cards.
cards.forEach((card, i) => {
  const t = text(names[i], { name: `${names[i]} label` }).fill(PAPER).font("Helvetica Neue");
  t.set("Scale", [0.3, 0.3]).set("Position Anchor", card).set("Position Area", "Top").set("Margin", 10);
});

// Ropes between neighbours: dashed, hanging, swinging late.
for (let i = 0; i + 1 < cards.length; i++) {
  const rope = line({ name: `Rope ${i + 1}` }).fill(PAPER);
  rope.set("Connect From", cards[i]).set("Connect To", cards[i + 1])
    .set("From Side", "Bottom").set("To Side", "Bottom").set("Line Path", "Hang").set("Slack", 45).set("Margin", 8)
    .set("Dash", 10).set("Dash Gap", 8).set("Transition Duration", 0.7).set("Transition Easing", "Ease Out");
}
// One long arc over the top, from the first card to the last.
const arc = line({ name: "Arc" }).fill(PAPER);
arc.set("Connect From", cards[0]).set("Connect To", cards[4])
  .set("From Side", "Top").set("To Side", "Top").set("Line Path", "Curved").set("Margin", 60).set("Dash", 4).set("Dash Gap", 10);
arc.key("Dash Offset", 0, 0, "Linear").key("Dash Offset", 6, -240);
